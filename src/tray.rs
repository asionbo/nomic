use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::app::{DeliveryMode, Shared, UiCommand};
use crate::icon;

pub struct Tray {
    pub icon: TrayIcon,
    status: MenuItem,
    paste: CheckMenuItem,
    clipboard_only: CheckMenuItem,
    network: Submenu,
    pub show_qr_id: String,
    pub copy_link_id: String,
    pub paste_id: String,
    pub clipboard_only_id: String,
    pub rotate_id: String,
    pub quit_id: String,
    pub network_ids: Vec<String>,
    network_items: Vec<CheckMenuItem>,
}

impl Tray {
    pub fn new(shared: &Shared, auto_paste: bool) -> anyhow::Result<Self> {
        let menu = Menu::new();
        let status = MenuItem::new("空闲 · 等待手机连接", false, None);
        let show_qr = MenuItem::new("显示二维码（打开浏览器）", true, None);
        let copy_link = MenuItem::new("复制手机链接", true, None);
        let paste = CheckMenuItem::new("粘贴到光标处", true, auto_paste, None);
        let clipboard_only = CheckMenuItem::new("仅复制到剪贴板", true, !auto_paste, None);
        let rotate = MenuItem::new("重新生成令牌", true, None);
        let network = Submenu::new("网络", true);
        let mut network_items = Vec::new();
        for ip in shared.candidates.iter() {
            let item = CheckMenuItem::new(ip.to_string(), true, false, None);
            network.append(&item)?;
            network_items.push(item);
        }
        let quit = MenuItem::new("退出 nomic", true, None);

        menu.append(&status)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&show_qr)?;
        menu.append(&copy_link)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&paste)?;
        menu.append(&clipboard_only)?;
        menu.append(&network)?;
        menu.append(&rotate)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit)?;

        let show_qr_id = show_qr.id().as_ref().to_string();
        let copy_link_id = copy_link.id().as_ref().to_string();
        let paste_id = paste.id().as_ref().to_string();
        let clipboard_only_id = clipboard_only.id().as_ref().to_string();
        let rotate_id = rotate.id().as_ref().to_string();
        let quit_id = quit.id().as_ref().to_string();
        let network_ids = network_items
            .iter()
            .map(|item| item.id().as_ref().to_string())
            .collect();

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("nomic · 手机文字输入")
            .with_icon(icon::icon(false, false))
            .build()?;

        // macOS 使用模板图标，跟随系统明暗。
        #[cfg(target_os = "macos")]
        icon.set_icon_as_template(true);

        let mut tray = Self {
            icon,
            status,
            paste,
            clipboard_only,
            network,
            show_qr_id,
            copy_link_id,
            paste_id,
            clipboard_only_id,
            rotate_id,
            quit_id,
            network_ids,
            network_items,
        };
        let _ = &tray.network;
        tray.set_selected_ip(shared.lan_ip().to_string());
        Ok(tray)
    }

    pub fn set_status(&self, text: &str) {
        self.status.set_text(text);
    }

    pub fn set_state(&self, connected: bool, flash: bool) {
        let _ = self.icon.set_icon(Some(icon::icon(connected, flash)));
    }

    pub fn set_selected_ip(&mut self, selected: String) {
        for item in &self.network_items {
            item.set_checked(item.text() == selected);
        }
    }

    /// 返回 true 表示应退出事件循环。
    pub fn handle_menu(&mut self, id: &str, shared: &Shared) -> bool {
        if id == self.show_qr_id {
            let _ = open::that(shared.pair_url());
            return false;
        }
        if id == self.copy_link_id {
            // 复用常驻剪贴板实例由 main 处理，这里单独复制不影响待注入文本。
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(shared.mobile_url());
            }
            self.status.set_text("已复制手机链接到剪贴板");
            return false;
        }
        if id == self.paste_id || id == self.clipboard_only_id {
            let paste = id == self.paste_id;
            *shared.mode.write().unwrap() = if paste {
                DeliveryMode::Paste
            } else {
                DeliveryMode::ClipboardOnly
            };
            self.paste.set_checked(paste);
            self.clipboard_only.set_checked(!paste);
            self.status.set_text(if paste {
                "投递：粘贴到光标处"
            } else {
                "投递：仅复制到剪贴板"
            });
            return false;
        }
        if id == self.rotate_id {
            *shared.token.write().unwrap() = crate::app::new_token();
            shared.token_generation.send_modify(|g| *g += 1);
            let _ = shared.kick.send(());
            let _ = shared.ui_tx.send(UiCommand::RotateToken);
            self.status.set_text("令牌已更新，请重新扫码");
            return false;
        }
        if id == self.quit_id {
            let _ = shared.ui_tx.send(UiCommand::Shutdown);
            let _ = shared.kick.send(());
            return true;
        }
        if self.network_ids.contains(&id.to_string()) {
            let idx = self.network_ids.iter().position(|x| x == id).unwrap();
            let ip = shared.candidates[idx];
            *shared.lan_ip.write().unwrap() = ip;
            self.set_selected_ip(ip.to_string());
            self.status
                .set_text(format!("网络已切换为 {ip}，请重新扫码"));
        }
        false
    }
}
