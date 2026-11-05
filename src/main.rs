#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod i18n;
mod icon;
mod inject;
mod qr;
mod server;
mod tray;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use i18n::Lang;
use muda::MenuEvent;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
#[cfg(target_os = "macos")]
use tao::platform::macos::EventLoopExtMacOS;
use tokio::sync::{broadcast, mpsc, watch};
use tray_icon::TrayIconEvent;

use app::{DeliveryMode, Shared, UiCommand, UserEvent};
use inject::{InjectOutcome, Injector};
use server::HostToClient;
use tray::Tray;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let preferred_port: u16 = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(8765);
    let clipboard_only = args.iter().any(|a| a == "--clipboard-only");
    let lang = args
        .windows(2)
        .find(|w| w[0] == "--lang")
        .and_then(|w| Lang::from_str(&w[1]))
        .unwrap_or_default();

    // 先绑定再生成二维码，保证 URL 里的端口是真实端口。
    let listener = app::bind(preferred_port)?;
    let port = listener.local_addr()?.port();
    let candidates = app::lan_candidates();
    let lan_ip = candidates
        .first()
        .copied()
        .unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1));
    if candidates.is_empty() {
        tracing::warn!(
            "no LAN IPv4 found; the QR code will point to 127.0.0.1 and phones cannot connect"
        );
    }

    let token = app::new_token();
    tracing::info!("pairing page: http://127.0.0.1:{port}/pair?t={token}");
    let (inject_tx, inject_rx) = mpsc::unbounded_channel();
    let (ui_tx, ui_rx) = mpsc::unbounded_channel();
    let (kick, _) = broadcast::channel(16);
    let (generation_tx, _) = watch::channel(0u64);

    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    // tao 默认在 run() 时强制 Regular 策略，会覆盖 Info.plist 的 LSUIElement，
    // 导致应用出现在 Dock。这里显式设为 Accessory：菜单栏常驻、不进 Dock。
    #[cfg(target_os = "macos")]
    event_loop.set_activation_policy(tao::platform::macos::ActivationPolicy::Accessory);
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let shared = Shared {
        port,
        lang: Arc::new(std::sync::RwLock::new(lang)),
        token: Arc::new(std::sync::RwLock::new(token)),
        lan_ip: Arc::new(std::sync::RwLock::new(lan_ip)),
        candidates: Arc::new(candidates),
        mode: Arc::new(std::sync::RwLock::new(if clipboard_only {
            DeliveryMode::ClipboardOnly
        } else {
            DeliveryMode::Paste
        })),
        auto_paste_available: Arc::new(AtomicBool::new(true)),
        connected: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        kick,
        token_generation: Arc::new(generation_tx),
        inject_tx,
        inject_rx: Arc::new(tokio::sync::Mutex::new(inject_rx)),
        ui_tx,
        ui_rx: Arc::new(tokio::sync::Mutex::new(ui_rx)),
        wake: proxy.clone(),
    };

    let injector = Injector::new();
    let auto_paste = injector.auto_paste_available() && !clipboard_only;
    shared
        .auto_paste_available
        .store(auto_paste, Ordering::SeqCst);
    let tray = Tray::new(&shared, auto_paste)?;

    // 服务运行在独立 tokio 线程；监听 socket 从 std 转成 tokio。
    let server_shared = shared.clone();
    let ui_rx = server_shared.ui_rx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            listener.set_nonblocking(true).ok();
            let tokio_listener =
                tokio::net::TcpListener::from_std(listener).expect("tokio listener");
            let app = server::router(server_shared.clone());
            let shutdown = async move {
                loop {
                    match ui_rx.lock().await.recv().await {
                        Some(UiCommand::Shutdown) => break,
                        Some(UiCommand::RotateToken) => {}
                        None => break,
                    }
                }
            };
            axum::serve(tokio_listener, app)
                .with_graceful_shutdown(shutdown)
                .await
                .expect("axum serve");
        });
    });

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let mut injector = injector;
    let mut tray = tray;
    let mut connected = false;
    let mut flash_until: Option<std::time::Instant> = None;

    // 托盘菜单事件转发到 tao 事件循环，避免跨线程操作 UI。
    let menu_proxy = proxy.clone();
    std::thread::spawn(move || {
        for event in menu_channel.iter() {
            let _ = menu_proxy.send_event(UserEvent::Status(format!("menu:{}", event.id.as_ref())));
        }
    });

    // tao 的 run() 返回 `!`；进程退出时 OS 回收 server 线程。
    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            let _ = proxy.send_event(UserEvent::ShowQr);
        }
        Event::UserEvent(UserEvent::ShowQr) => {
            let _ = open::that(shared.pair_url());
            tray.set_status(shared.lang().strings().status_qr_opened);
        }
        Event::UserEvent(UserEvent::Status(msg)) => {
            if let Some(id) = msg.strip_prefix("menu:") {
                let quit = tray.handle_menu(id, &shared);
                if quit {
                    *control_flow = ControlFlow::Exit;
                }
            }
        }
        Event::UserEvent(UserEvent::InjectionReady) => {
            while let Ok(job) = shared.inject_rx.blocking_lock().try_recv() {
                tracing::info!("injected id={}", job.id);
                let outcome = injector.deliver(&shared, &job.text);
                let ack = match outcome {
                    InjectOutcome::Pasted => HostToClient::Ack {
                        id: job.id,
                        status: "pasted".into(),
                        message: None,
                    },
                    InjectOutcome::ClipboardOnly => HostToClient::Ack {
                        id: job.id,
                        status: "clipboard-only".into(),
                        message: None,
                    },
                    InjectOutcome::Failed(e) => HostToClient::Ack {
                        id: job.id,
                        status: "error".into(),
                        message: Some(e),
                    },
                };
                let _ = job.reply.send(ack);
            }
            flash_until = Some(std::time::Instant::now() + Duration::from_millis(600));
            tray.set_state(connected, true);
            let s = shared.lang().strings();
            tray.set_status(&format!(
                "{} · {}",
                s.status_received,
                shared.mode().label(shared.lang())
            ));
        }
        Event::UserEvent(UserEvent::Connected(n)) => {
            connected = n > 0;
            tray.set_state(connected, false);
            let s = shared.lang().strings();
            tray.set_status(&if connected {
                s.fmt_connected.replace("{n}", &n.to_string())
            } else {
                s.status_idle.into()
            });
        }
        Event::MainEventsCleared => {
            if let Some(until) = flash_until
                && std::time::Instant::now() >= until
            {
                flash_until = None;
                tray.set_state(connected, false);
            }
            let _ = tray_channel.try_recv();
            *control_flow = if flash_until.is_some() {
                ControlFlow::WaitUntil(std::time::Instant::now() + Duration::from_millis(100))
            } else {
                ControlFlow::Wait
            };
        }
        _ => {}
    });
    #[allow(unreachable_code)]
    Ok(())
}
