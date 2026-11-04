use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, watch};

use crate::app::{MAX_CLIENTS, MAX_MESSAGE_BYTES, Shared};
use crate::i18n::Strings;
use crate::qr;

#[derive(Deserialize)]
pub struct WithToken {
    t: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientToHost {
    pub r#type: String,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum HostToClient {
    #[serde(rename = "ack")]
    Ack {
        id: String,
        status: String,
        message: Option<String>,
    },
    #[serde(rename = "token-expired")]
    TokenExpired,
    #[serde(rename = "status")]
    Status { connected: usize },
}

pub fn router(shared: Shared) -> Router {
    Router::new()
        .route("/pair", get(pair_page))
        .route("/m", get(mobile_page))
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .with_state(shared)
}

/// 令牌错误统一返回 404，减少信息泄露。返回 Some 表示校验失败。
fn check(state: &Shared, headers: &HeaderMap, t: Option<&str>) -> Option<Response> {
    let expected = state.token();
    if t != Some(expected.as_str()) {
        return Some(StatusCode::NOT_FOUND.into_response());
    }
    // DNS rebinding 防护：Host 必须是本机回环或已选 LAN 地址。
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    let lan = state.lan_ip().to_string();
    if host != "127.0.0.1" && host != "localhost" && host != lan && host != "[::1]" {
        return Some(StatusCode::NOT_FOUND.into_response());
    }
    None
}

fn page(body: &str) -> Response {
    let mut res = Html(body.to_owned()).into_response();
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate".parse().unwrap(),
    );
    // 本页无外链、无脚本文件；CSP 仅放行内联脚本。
    res.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; connect-src 'self' ws: wss:"
            .parse()
            .unwrap(),
    );
    res
}

async fn health(
    State(state): State<Shared>,
    Query(q): Query<WithToken>,
    headers: HeaderMap,
) -> Response {
    if let Some(r) = check(&state, &headers, q.t.as_deref()) {
        return r;
    }
    Json(serde_json::json!({
        "ok": true,
        "port": state.port,
        "lan_ip": state.lan_ip().to_string(),
        "mode": state.mode().label(state.lang),
        "connected": state.connected.load(std::sync::atomic::Ordering::SeqCst),
    }))
    .into_response()
}

async fn mobile_page(
    State(state): State<Shared>,
    Query(q): Query<WithToken>,
    headers: HeaderMap,
) -> Response {
    if let Some(r) = check(&state, &headers, q.t.as_deref()) {
        return r;
    }
    let s = state.lang.strings();
    let body = include_str!("../assets/mobile.html")
        .replace("{{HTML_LANG}}", s.html_lang)
        .replace("{{TITLE}}", s.mobile_title)
        .replace("{{PLACEHOLDER}}", s.mobile_placeholder)
        .replace("{{SEND}}", s.mobile_send)
        .replace("{{CLEAR}}", s.mobile_clear)
        .replace("{{HINT}}", s.mobile_hint)
        .replace("{{I18N}}", &js_i18n_json(s));
    page(&body)
}

async fn pair_page(
    State(state): State<Shared>,
    Query(q): Query<WithToken>,
    headers: HeaderMap,
) -> Response {
    if let Some(r) = check(&state, &headers, q.t.as_deref()) {
        return r;
    }
    let url = state.mobile_url();
    let s = state.lang.strings();
    let svg = match qr::svg(&url, s.pair_aria) {
        Ok(svg) => svg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                s.fmt_qr_failed.replace("{e}", &e.to_string()),
            )
                .into_response();
        }
    };
    let lan_address = format!("{}:{}", state.lan_ip(), state.port);
    let body = include_str!("../assets/pair.html")
        .replace("{{QR_SVG}}", &svg)
        .replace("{{MOBILE_URL}}", &html_escape(&url))
        .replace("{{HTML_LANG}}", s.html_lang)
        .replace("{{TITLE}}", s.pair_title)
        .replace("{{H1}}", s.pair_h1)
        .replace("{{SUB}}", s.pair_sub)
        .replace("{{QR_ARIA}}", s.pair_aria)
        .replace("{{COPY}}", s.pair_copy)
        .replace(
            "{{COPY_JSON}}",
            &serde_json::to_string(s.pair_copy).unwrap(),
        )
        .replace(
            "{{COPIED_JSON}}",
            &serde_json::to_string(s.pair_copied).unwrap(),
        )
        .replace("{{NOTE_TITLE}}", s.pair_note_title)
        .replace("{{NOTE1}}", s.pair_note1)
        .replace("{{NOTE2}}", s.pair_note2)
        .replace("{{NOTE3}}", s.pair_note3)
        .replace("{{NOTE4}}", &s.fmt_note4(&html_escape(&lan_address)))
        .replace("{{NOTE5}}", s.pair_note5);
    page(&body)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 手机页 JS 用的字符串，序列化成 JSON 常量内联，避免 JS 里写死中文。
fn js_i18n_json(s: &Strings) -> String {
    serde_json::json!({
        "connecting": s.js_connecting,
        "connected": s.js_connected,
        "disconnected": s.js_disconnected,
        "reconnecting": s.js_reconnecting,
        "tokenExpiredStatus": s.js_token_expired_status,
        "tokenExpiredToast": s.js_token_expired_toast,
        "toastEmpty": s.js_toast_empty,
        "toastNotConnected": s.js_toast_not_connected,
        "toastBusy": s.js_toast_busy,
        "toastPasted": s.js_toast_pasted,
        "toastCopied": s.js_toast_copied,
        "toastFailed": s.js_toast_failed,
        "toastNoResponse": s.js_toast_no_response,
    })
    .to_string()
}

async fn ws_handler(
    State(state): State<Shared>,
    Query(q): Query<WithToken>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if let Some(r) = check(&state, &headers, q.t.as_deref()) {
        return r;
    }
    // CSWSH：Origin 必须等于页面自身的 origin。
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if let Some(o) = origin
        && o != format!("http://{host}")
        && o != format!("https://{host}")
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    if state.connected.load(std::sync::atomic::Ordering::SeqCst) >= MAX_CLIENTS {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let kick = state.kick.subscribe();
    let generation = state.token_generation.subscribe();
    let inject_tx = state.inject_tx.clone();
    let state2 = state.clone();
    ws.max_message_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| handle_socket(socket, state2, kick, generation, inject_tx))
        .into_response()
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Shared,
    mut kick: broadcast::Receiver<()>,
    mut generation: watch::Receiver<u64>,
    inject_tx: mpsc::UnboundedSender<crate::app::Injection>,
) {
    let n = state.bump_connected(1);
    tracing::info!("phone connected ({n} active)");
    let _ = state.wake.send_event(crate::app::UserEvent::Connected(n));
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&HostToClient::Status { connected: n })
                .unwrap()
                .into(),
        ))
        .await;

    let (reply_tx, mut reply_rx) = mpsc::unbounded_channel::<HostToClient>();
    // 初始化为过去，让连接后的首条消息不被限流丢弃。
    let mut last_send = tokio::time::Instant::now() - Duration::from_millis(200);
    let mut ping = tokio::time::interval(Duration::from_secs(25));
    ping.tick().await; // 跳过第一次立即触发

    loop {
        tokio::select! {
            _ = generation.changed() => {
                let _ = socket.send(Message::Text(serde_json::to_string(&HostToClient::TokenExpired).unwrap().into())).await;
                break;
            }
            _ = kick.recv() => break,
            reply = reply_rx.recv() => {
                match reply {
                    Some(msg) => {
                        if socket.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await.is_err() { break; }
                    }
                    None => break,
                }
            }
            _ = ping.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() { break; }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(raw))) => {
                        // 每连接最小 100ms 间隔，超频直接丢弃。
                        if last_send.elapsed() < Duration::from_millis(100) {
                            continue;
                        }
                        last_send = tokio::time::Instant::now();
                        let parsed: Result<ClientToHost, _> = serde_json::from_str(&raw);
                        let reply = match parsed {
                            Ok(m) if m.r#type == "text" && !m.text.trim().is_empty() && m.text.len() <= MAX_MESSAGE_BYTES => {
                                let injection = crate::app::Injection {
                                    id: m.id,
                                    text: m.text,
                                    reply: reply_tx.clone(),
                                };
                                match inject_tx.send(injection) {
                                    Ok(()) => {
                                        let _ = state.wake.send_event(crate::app::UserEvent::InjectionReady);
                                        None
                                    }
                                    Err(_) => Some(HostToClient::Ack { id: "".into(), status: "error".into(), message: Some(state.lang.strings().err_desktop_exited.into()) }),
                                }
                            }
                            _ => Some(HostToClient::Ack { id: "".into(), status: "error".into(), message: Some(state.lang.strings().err_bad_message.into()) }),
                        };
                        if let Some(msg) = reply
                            && socket
                                .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                                .await
                                .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => {} // ping/pong/binary 忽略
                }
            }
        }
    }

    let n = state.bump_connected(-1);
    tracing::info!("phone disconnected ({n} active)");
    let _ = state.wake.send_event(crate::app::UserEvent::Connected(n));
    let _ = socket.send(Message::Close(None)).await;
}
