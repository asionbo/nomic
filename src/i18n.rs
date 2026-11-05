/// 应用内多语言：默认英文，`--lang zh` 切换中文。
/// 只覆盖用户可见的 UI 字符串（托盘、手机页、二维码页、ack 消息）；
/// tracing 日志统一用英文，不参与本地化。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Zh,
}

impl Lang {
    pub fn from_str(s: &str) -> Option<Lang> {
        match s.to_ascii_lowercase().as_str() {
            "en" | "en-us" | "english" => Some(Lang::En),
            "zh" | "zh-cn" | "zh-hans" | "chinese" => Some(Lang::Zh),
            _ => None,
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Zh => &ZH,
        }
    }
}

/// 所有用户可见字符串。动态内容用 `{n}` / `{ip}` / `{e}` 占位，调用方 replace。
pub struct Strings {
    pub html_lang: &'static str,
    // 托盘
    pub tray_tooltip: &'static str,
    pub menu_show_qr: &'static str,
    pub menu_copy_link: &'static str,
    pub menu_paste: &'static str,
    pub menu_clipboard_only: &'static str,
    pub menu_auto_enter: &'static str,
    pub menu_network: &'static str,
    pub menu_rotate: &'static str,
    pub menu_language: &'static str,
    pub menu_quit: &'static str,
    pub status_idle: &'static str,
    pub status_qr_opened: &'static str,
    pub status_link_copied: &'static str,
    pub status_token_rotated: &'static str,
    pub status_received: &'static str,
    pub status_auto_enter_on: &'static str,
    pub status_auto_enter_off: &'static str,
    pub fmt_connected: &'static str,        // {n}
    pub fmt_network_switched: &'static str, // {ip}
    pub fmt_lang_switched: &'static str,    // {lang}
    // 投递模式
    pub mode_paste: &'static str,
    pub mode_clipboard_only: &'static str,
    // 注入错误（ack message）
    pub err_clipboard_unavailable: &'static str,
    pub fmt_clipboard_write_failed: &'static str, // {e}
    // WS ack 错误
    pub err_desktop_exited: &'static str,
    pub err_bad_message: &'static str,
    pub fmt_qr_failed: &'static str, // {e}
    // 二维码页
    pub pair_title: &'static str,
    pub pair_h1: &'static str,
    pub pair_sub: &'static str,
    pub pair_aria: &'static str,
    pub pair_copy: &'static str,
    pub pair_copied: &'static str,
    pub pair_note_title: &'static str,
    pub pair_note1: &'static str,
    pub pair_note2: &'static str,
    pub pair_note3: &'static str,
    pub pair_note4: &'static str, // {addr}
    pub pair_note5: &'static str,
    // 手机页（静态部分）
    pub mobile_title: &'static str,
    pub mobile_send: &'static str,
    pub mobile_clear: &'static str,
    pub mobile_placeholder: &'static str,
    pub mobile_hint: &'static str,
    // 手机页（JS 部分，经 I18N JSON 注入）
    pub js_connecting: &'static str,
    pub js_connected: &'static str,
    pub js_disconnected: &'static str,
    pub js_reconnecting: &'static str,
    pub js_token_expired_status: &'static str,
    pub js_token_expired_toast: &'static str,
    pub js_toast_empty: &'static str,
    pub js_toast_not_connected: &'static str,
    pub js_toast_busy: &'static str,
    pub js_toast_pasted: &'static str,
    pub js_toast_copied: &'static str,
    pub js_toast_failed: &'static str, // {msg}
    pub js_toast_no_response: &'static str,
}

impl Strings {
    /// pair_note4 带地址占位。
    pub fn fmt_note4(&self, addr: &str) -> String {
        self.pair_note4.replace("{addr}", addr)
    }
}

impl Lang {
    /// 两种语言都能读懂的自身名称，语言菜单固定显示它，不随当前语言变。
    pub fn menu_label(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }
}

const EN: Strings = Strings {
    html_lang: "en",
    tray_tooltip: "nomic · type from your phone",
    menu_show_qr: "Show QR code (open browser)",
    menu_copy_link: "Copy phone link",
    menu_paste: "Paste at cursor",
    menu_clipboard_only: "Copy to clipboard only",
    menu_auto_enter: "Press Enter after paste",
    menu_network: "Network",
    menu_rotate: "Regenerate token",
    menu_language: "Language",
    menu_quit: "Quit nomic",
    status_idle: "Idle · waiting for a phone",
    status_qr_opened: "QR page opened, waiting for a phone",
    status_link_copied: "Phone link copied to clipboard",
    status_token_rotated: "Token updated, scan the QR again",
    status_received: "Just received text",
    status_auto_enter_on: "Auto-Enter on: Enter is pressed after each paste",
    status_auto_enter_off: "Auto-Enter off",
    fmt_connected: "Connected: {n} device(s)",
    fmt_network_switched: "Network switched to {ip}, scan the QR again",
    fmt_lang_switched: "Language switched to English; refresh the phone page for the new language",
    mode_paste: "Paste at cursor",
    mode_clipboard_only: "Copy to clipboard only",
    err_clipboard_unavailable: "Clipboard unavailable",
    fmt_clipboard_write_failed: "Failed to write clipboard: {e}",
    err_desktop_exited: "Desktop app is shutting down",
    err_bad_message: "Malformed message or empty text",
    fmt_qr_failed: "QR generation failed: {e}",
    pair_title: "nomic · scan to connect",
    pair_h1: "nomic · connect your phone",
    pair_sub: "Scan with your phone camera or a QR app",
    pair_aria: "QR code for pairing",
    pair_copy: "Copy link",
    pair_copied: "Copied",
    pair_note_title: "Can't connect?",
    pair_note1: "1. Phone and computer must be on the same Wi-Fi, without guest/AP isolation.",
    pair_note2: "2. The first listen may trigger a firewall prompt — allow it and mark the Wi-Fi as a private network.",
    pair_note3: "3. With multiple NICs or a VPN, pick another address in the tray \"Network\" menu, then reopen this page.",
    pair_note4: "4. The QR address {addr} must not be 127.0.0.1, or the phone would open itself.",
    pair_note5: "5. The link carries a one-time token; rescan after restarting the app or regenerating the token.",
    mobile_title: "nomic · type",
    mobile_send: "Send to computer",
    mobile_clear: "Clear",
    mobile_placeholder: "Type here, or use your keyboard's mic button for voice dictation…",
    mobile_hint: "Click into the target input field on the computer before sending. Text goes to the clipboard and is pasted automatically; if auto-paste is unavailable you'll be asked to press ⌘/Ctrl+V manually. Voice dictation comes from your phone keyboard's built-in mic — this page never requests microphone access.",
    js_connecting: "Connecting…",
    js_connected: "Connected",
    js_disconnected: "Disconnected",
    js_reconnecting: "Reconnecting…",
    js_token_expired_status: "Token expired",
    js_token_expired_toast: "Link expired — show the QR on the computer again",
    js_toast_empty: "Type something first",
    js_toast_not_connected: "Not connected yet, reconnecting…",
    js_toast_busy: "Previous message still pending",
    js_toast_pasted: "Pasted on the computer",
    js_toast_copied: "Copied to clipboard — press ⌘/Ctrl+V on the computer",
    js_toast_failed: "Send failed: {msg}",
    js_toast_no_response: "No response from the computer, try again (text kept)",
};

const ZH: Strings = Strings {
    html_lang: "zh-CN",
    tray_tooltip: "nomic · 手机文字输入",
    menu_show_qr: "显示二维码（打开浏览器）",
    menu_copy_link: "复制手机链接",
    menu_paste: "粘贴到光标处",
    menu_clipboard_only: "仅复制到剪贴板",
    menu_auto_enter: "粘贴后自动回车",
    menu_network: "网络",
    menu_rotate: "重新生成令牌",
    menu_language: "语言 / Language",
    menu_quit: "退出 nomic",
    status_idle: "空闲 · 等待手机连接",
    status_qr_opened: "已打开二维码，等待手机连接",
    status_link_copied: "已复制手机链接到剪贴板",
    status_token_rotated: "令牌已更新，请重新扫码",
    status_received: "刚收到文本",
    status_auto_enter_on: "自动回车已开启：粘贴后会追加回车键",
    status_auto_enter_off: "自动回车已关闭",
    fmt_connected: "已连接 {n} 台设备",
    fmt_network_switched: "网络已切换为 {ip}，请重新扫码",
    fmt_lang_switched: "语言已切换为中文，手机页刷新后生效",
    mode_paste: "粘贴到光标处",
    mode_clipboard_only: "仅复制到剪贴板",
    err_clipboard_unavailable: "剪贴板不可用",
    fmt_clipboard_write_failed: "写剪贴板失败：{e}",
    err_desktop_exited: "桌面端已退出",
    err_bad_message: "消息格式错误或内容为空",
    fmt_qr_failed: "二维码生成失败：{e}",
    pair_title: "nomic · 扫码连接",
    pair_h1: "nomic · 手机连接",
    pair_sub: "用手机相机或扫码 App 扫描",
    pair_aria: "扫码连接二维码",
    pair_copy: "复制链接",
    pair_copied: "已复制",
    pair_note_title: "连不上？",
    pair_note1: "1. 手机与电脑必须连同一个 Wi-Fi，且网络未开启「访客隔离 / AP 隔离」。",
    pair_note2: "2. 电脑防火墙首次可能弹窗，请允许本程序监听，并把当前 Wi-Fi 设为「专用网络」。",
    pair_note3: "3. 若电脑有多个网卡 / 挂了 VPN，可在托盘菜单的「网络」里换一个地址，再重新打开本页。",
    pair_note4: "4. 二维码里的地址 {addr} 不能写成 127.0.0.1，否则手机会打开自己。",
    pair_note5: "5. 链接含一次性令牌；重启程序或点托盘「重新生成令牌」后需重新扫码。",
    mobile_title: "nomic · 输入",
    mobile_send: "发送到电脑",
    mobile_clear: "清空",
    mobile_placeholder: "在这里输入，或用手机键盘上的麦克风按钮语音听写…",
    mobile_hint: "发送前请先点回电脑上要输入的输入框。文字会写入剪贴板并自动粘贴；若自动粘贴不可用，会提示你手动按 ⌘/Ctrl+V。语音听写请用手机系统键盘自带的麦克风，本页面不申请麦克风权限。",
    js_connecting: "连接中…",
    js_connected: "已连接",
    js_disconnected: "已断开",
    js_reconnecting: "重连中…",
    js_token_expired_status: "令牌已失效",
    js_token_expired_toast: "链接已失效，请在电脑上重新显示二维码",
    js_toast_empty: "请先输入内容",
    js_toast_not_connected: "尚未连接，正在重连…",
    js_toast_busy: "上一条还在处理",
    js_toast_pasted: "已粘贴到电脑",
    js_toast_copied: "已复制到剪贴板，请在电脑上按 ⌘/Ctrl+V",
    js_toast_failed: "发送失败：{msg}",
    js_toast_no_response: "电脑未响应，请重试（内容已保留）",
};
