use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use tokio::sync::{broadcast, mpsc, watch};

use crate::i18n::Lang;
use crate::server::{ClientToHost, HostToClient};

pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const MAX_CLIENTS: usize = 8;

/// 投递方式。仅剪贴板是所有平台的保底路径。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryMode {
    Paste,
    ClipboardOnly,
}

impl DeliveryMode {
    pub fn label(self, lang: Lang) -> &'static str {
        let s = lang.strings();
        match self {
            DeliveryMode::Paste => s.mode_paste,
            DeliveryMode::ClipboardOnly => s.mode_clipboard_only,
        }
    }
}

/// 主线程（托盘/注入）与 tokio 线程（HTTP/WS）之间共享的状态。
#[derive(Clone)]
pub struct Shared {
    pub port: u16,
    /// 运行时可通过托盘菜单切换；手机页/二维码页按请求时值渲染。
    pub lang: Arc<std::sync::RwLock<Lang>>,
    pub token: Arc<std::sync::RwLock<String>>,
    pub lan_ip: Arc<std::sync::RwLock<Ipv4Addr>>,
    pub candidates: Arc<Vec<Ipv4Addr>>,
    pub mode: Arc<std::sync::RwLock<DeliveryMode>>,
    pub auto_paste_available: Arc<std::sync::atomic::AtomicBool>,
    pub connected: Arc<AtomicUsize>,
    /// 令牌轮换 / 退出时向所有 WS 连接广播。
    pub kick: broadcast::Sender<()>,
    pub token_generation: Arc<watch::Sender<u64>>,
    /// WS 收到的待注入文本；主线程串行消费。
    pub inject_tx: mpsc::UnboundedSender<Injection>,
    pub inject_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Injection>>>,
    /// 托盘命令 -> tokio 线程。
    pub ui_tx: mpsc::UnboundedSender<UiCommand>,
    pub ui_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<UiCommand>>>,
    /// 服务端通知主线程「队列里有待注入文本」。
    pub wake: tao::event_loop::EventLoopProxy<UserEvent>,
}

/// 一条待注入的文本，携带来源连接与消息 id，便于回 ack。
#[derive(Debug)]
pub struct Injection {
    pub id: String,
    pub text: String,
    pub reply: mpsc::UnboundedSender<HostToClient>,
}

#[derive(Debug)]
pub enum UiCommand {
    RotateToken,
    Shutdown,
}

#[derive(Debug)]
pub enum UserEvent {
    /// 待注入队列里有新任务，主线程取出来执行。
    InjectionReady,
    Connected(usize),
    ShowQr,
    Status(String),
}

/// 手机 -> 桌面 的文本消息（JSON 由 ws.rs 定义）。
pub type _ClientMessage = ClientToHost;

pub fn new_token() -> String {
    use base64::Engine;
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// 预绑定 listener：先拿到真实端口再生成二维码，端口被占用时交给 OS 分配。
pub fn bind(preferred: u16) -> Result<TcpListener> {
    let addrs = [
        SocketAddr::from(([0, 0, 0, 0], preferred)),
        SocketAddr::from(([0, 0, 0, 0], 0)),
    ];
    for addr in addrs {
        match TcpListener::bind(addr) {
            Ok(l) => {
                l.set_nonblocking(true)
                    .context("set listener nonblocking")?;
                return Ok(l);
            }
            Err(e) if addr.port() != 0 => {
                tracing::warn!("端口 {preferred} 不可用（{e}），改用随机端口");
            }
            Err(e) => return Err(e).context("绑定监听端口"),
        }
    }
    unreachable!()
}

/// 按可信度打分选择局域网 IPv4；跳过回环、链路本地、虚拟/VPN 网卡。
pub fn lan_candidates() -> Vec<Ipv4Addr> {
    let mut scored: Vec<(u8, Ipv4Addr)> = Vec::new();
    let Ok(ifaces) = if_addrs::get_if_addrs() else {
        return vec![];
    };
    for iface in ifaces {
        let IpAddr::V4(ip) = iface.ip() else { continue };
        let name = iface.name.to_ascii_lowercase();
        if ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() {
            continue;
        }
        if [
            "docker",
            "veth",
            "br-",
            "virbr",
            "tun",
            "tap",
            "utun",
            "vmnet",
            "vboxnet",
            "tailscale",
            "wg",
            "zt",
            "ham",
            "bridge",
            "lo",
        ]
        .iter()
        .any(|p| name.starts_with(p))
        {
            continue;
        }
        let o = ip.octets();
        let score = if o[0] == 192 && o[1] == 168 {
            100
        } else if o[0] == 10 {
            80
        } else if o[0] == 172 && (16..=31).contains(&o[1]) {
            70
        } else {
            10
        };
        if !scored.iter().any(|(_, existing)| *existing == ip) {
            scored.push((score, ip));
        }
    }
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    scored.into_iter().map(|(_, ip)| ip).collect()
}

impl Shared {
    pub fn token(&self) -> String {
        self.token.read().unwrap().clone()
    }
    pub fn lang(&self) -> Lang {
        *self.lang.read().unwrap()
    }
    pub fn lan_ip(&self) -> Ipv4Addr {
        *self.lan_ip.read().unwrap()
    }
    pub fn mode(&self) -> DeliveryMode {
        *self.mode.read().unwrap()
    }
    pub fn bump_connected(&self, delta: isize) -> usize {
        if delta > 0 {
            self.connected.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            self.connected
                .fetch_sub(1, Ordering::SeqCst)
                .saturating_sub(1)
        }
    }
    pub fn pair_url(&self) -> String {
        format!("http://127.0.0.1:{}/pair?t={}", self.port, self.token())
    }
    pub fn mobile_url(&self) -> String {
        format!(
            "http://{}:{}/m?t={}",
            self.lan_ip(),
            self.port,
            self.token()
        )
    }
}
