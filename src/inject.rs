use std::sync::atomic::Ordering;

use anyhow::Result;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::app::{DeliveryMode, Shared};

pub enum InjectOutcome {
    Pasted,
    ClipboardOnly,
    Failed(String),
}

/// 剪贴板实例必须常驻：X11/Wayland 下由它持有剪贴板所有权，drop 掉内容即失效。
pub struct Injector {
    clipboard: Option<Clipboard>,
    enigo: Option<Enigo>,
}

impl Injector {
    pub fn new() -> Self {
        let clipboard = match Clipboard::new() {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::error!("剪贴板不可用：{e}");
                None
            }
        };
        let enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!("按键模拟不可用（{e}），自动降级为仅剪贴板");
                None
            }
        };
        Self { clipboard, enigo }
    }

    pub fn auto_paste_available(&self) -> bool {
        self.clipboard.is_some() && self.enigo.is_some()
    }

    pub fn deliver(&mut self, shared: &Shared, text: &str) -> InjectOutcome {
        let Some(clipboard) = self.clipboard.as_mut() else {
            return InjectOutcome::Failed("剪贴板不可用".into());
        };
        if let Err(e) = clipboard.set_text(text.to_owned()) {
            tracing::error!("写剪贴板失败：{e}");
            return InjectOutcome::Failed(format!("写剪贴板失败：{e}"));
        }

        let wants_paste = shared.mode() == DeliveryMode::Paste
            && shared.auto_paste_available.load(Ordering::SeqCst);
        if !wants_paste {
            return InjectOutcome::ClipboardOnly;
        }

        // 让剪贴板所有权在目标应用读取前生效。
        std::thread::sleep(std::time::Duration::from_millis(50));

        let Some(enigo) = self.enigo.as_mut() else {
            return InjectOutcome::ClipboardOnly;
        };
        let modifier = if cfg!(target_os = "macos") {
            Key::Meta
        } else {
            Key::Control
        };
        let result = (|| -> Result<(), enigo::InputError> {
            enigo.key(modifier, Direction::Press)?;
            enigo.key(Key::Unicode('v'), Direction::Click)?;
            enigo.key(modifier, Direction::Release)?;
            Ok(())
        })();

        match result {
            Ok(()) => InjectOutcome::Pasted,
            Err(e) => {
                tracing::warn!("模拟粘贴失败（{e}），降级为仅剪贴板；macOS 请检查辅助功能权限");
                shared.auto_paste_available.store(false, Ordering::SeqCst);
                InjectOutcome::ClipboardOnly
            }
        }
    }
}
