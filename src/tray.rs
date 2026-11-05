use muda::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::app::{DeliveryMode, Shared, UiCommand};
use crate::i18n::Lang;
use crate::icon;

pub struct Tray {
    pub icon: TrayIcon,
    status: MenuItem,
    paste: CheckMenuItem,
    clipboard_only: CheckMenuItem,
    auto_enter: CheckMenuItem,
    network: Submenu,
    language: Submenu,
    show_qr: MenuItem,
    copy_link: MenuItem,
    rotate: MenuItem,
    quit: MenuItem,
    pub show_qr_id: String,
    pub copy_link_id: String,
    pub paste_id: String,
    pub clipboard_only_id: String,
    pub auto_enter_id: String,
    pub rotate_id: String,
    pub quit_id: String,
    pub network_ids: Vec<String>,
    pub lang_ids: Vec<String>,
    network_items: Vec<CheckMenuItem>,
    lang_items: Vec<CheckMenuItem>,
}

impl Tray {
    pub fn new(shared: &Shared, auto_paste: bool) -> anyhow::Result<Self> {
        let s = shared.lang().strings();
        let menu = Menu::new();
        let status = MenuItem::new(s.status_idle, false, None);
        let show_qr = MenuItem::new(s.menu_show_qr, true, None);
        let copy_link = MenuItem::new(s.menu_copy_link, true, None);
        let paste = CheckMenuItem::new(s.menu_paste, true, auto_paste, None);
        let clipboard_only = CheckMenuItem::new(s.menu_clipboard_only, true, !auto_paste, None);
        let auto_enter = CheckMenuItem::new(s.menu_auto_enter, true, false, None);
        let rotate = MenuItem::new(s.menu_rotate, true, None);
        let network = Submenu::new(s.menu_network, true);
        let mut network_items = Vec::new();
        for ip in shared.candidates.iter() {
            let item = CheckMenuItem::new(ip.to_string(), true, false, None);
            network.append(&item)?;
            network_items.push(item);
        }
        // 语言菜单的标签固定用各自语言的名称（English / 中文），不随当前语言变。
        let language = Submenu::new(s.menu_language, true);
        let mut lang_items = Vec::new();
        for lang in [Lang::En, Lang::Zh] {
            let item = CheckMenuItem::new(lang.menu_label(), true, lang == shared.lang(), None);
            language.append(&item)?;
            lang_items.push(item);
        }
        let quit = MenuItem::new(s.menu_quit, true, None);

        menu.append(&status)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&show_qr)?;
        menu.append(&copy_link)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&paste)?;
        menu.append(&clipboard_only)?;
        menu.append(&auto_enter)?;
        menu.append(&network)?;
        menu.append(&language)?;
        menu.append(&rotate)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit)?;

        let show_qr_id = show_qr.id().as_ref().to_string();
        let copy_link_id = copy_link.id().as_ref().to_string();
        let paste_id = paste.id().as_ref().to_string();
        let clipboard_only_id = clipboard_only.id().as_ref().to_string();
        let auto_enter_id = auto_enter.id().as_ref().to_string();
        let rotate_id = rotate.id().as_ref().to_string();
        let quit_id = quit.id().as_ref().to_string();
        let network_ids = network_items
            .iter()
            .map(|item| item.id().as_ref().to_string())
            .collect();
        let lang_ids = lang_items
            .iter()
            .map(|item| item.id().as_ref().to_string())
            .collect();

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(s.tray_tooltip)
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
            auto_enter,
            network,
            language,
            show_qr,
            copy_link,
            rotate,
            quit,
            show_qr_id,
            copy_link_id,
            paste_id,
            clipboard_only_id,
            auto_enter_id,
            rotate_id,
            quit_id,
            network_ids,
            lang_ids,
            network_items,
            lang_items,
        };
        let _ = &tray.network;
        let _ = &tray.language;
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

    fn set_selected_lang(&mut self, selected: Lang) {
        for (i, item) in self.lang_items.iter().enumerate() {
            item.set_checked(i == selected as usize);
        }
    }

    /// 切换语言后重写所有随语言变化的菜单文字（含 tooltip）。
    fn retranslate(&mut self, lang: Lang) {
        let s = lang.strings();
        self.paste.set_text(s.menu_paste);
        self.clipboard_only.set_text(s.menu_clipboard_only);
        self.auto_enter.set_text(s.menu_auto_enter);
        self.network.set_text(s.menu_network);
        self.language.set_text(s.menu_language);
        // status / show_qr / copy_link / rotate / quit 是 MenuItem，按 id 重建文本。
        self.status.set_text(s.status_idle);
        self.show_qr.set_text(s.menu_show_qr);
        self.copy_link.set_text(s.menu_copy_link);
        self.rotate.set_text(s.menu_rotate);
        self.quit.set_text(s.menu_quit);
        self.icon.set_tooltip(Some(s.tray_tooltip)).ok();
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
            self.status
                .set_text(shared.lang().strings().status_link_copied);
            return false;
        }
        if id == self.paste_id || id == self.clipboard_only_id {
            let paste = id == self.paste_id;
            let s = shared.lang().strings();
            *shared.mode.write().unwrap() = if paste {
                DeliveryMode::Paste
            } else {
                DeliveryMode::ClipboardOnly
            };
            self.paste.set_checked(paste);
            self.clipboard_only.set_checked(!paste);
            self.status.set_text(if paste {
                s.mode_paste
            } else {
                s.mode_clipboard_only
            });
            return false;
        }
        if id == self.auto_enter_id {
            // muda 在 macOS 上点击后已自动翻转勾选态，这里以菜单当前态为准写回共享状态。
            let enabled = self.auto_enter.is_checked();
            *shared.auto_enter.write().unwrap() = enabled;
            let s = shared.lang().strings();
            self.status.set_text(if enabled {
                s.status_auto_enter_on
            } else {
                s.status_auto_enter_off
            });
            return false;
        }
        if id == self.rotate_id {
            *shared.token.write().unwrap() = crate::app::new_token();
            shared.token_generation.send_modify(|g| *g += 1);
            let _ = shared.kick.send(());
            let _ = shared.ui_tx.send(UiCommand::RotateToken);
            self.status
                .set_text(shared.lang().strings().status_token_rotated);
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
            self.status.set_text(
                shared
                    .lang()
                    .strings()
                    .fmt_network_switched
                    .replace("{ip}", &ip.to_string()),
            );
            return false;
        }
        if let Some(idx) = self.lang_ids.iter().position(|x| x == id) {
            let lang = [Lang::En, Lang::Zh][idx];
            *shared.lang.write().unwrap() = lang;
            self.set_selected_lang(lang);
            self.retranslate(lang);
            self.status.set_text(
                lang.strings()
                    .fmt_lang_switched
                    .replace("{lang}", lang.display_name()),
            );
        }
        false
    }
}
