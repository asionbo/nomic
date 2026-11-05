# nomic

Use your phone as a wireless keyboard for your computer, with your phone keyboard's built-in voice dictation as an input option. Type on a phone web page and send text straight to the cursor on your desktop. It's especially useful when your computer has no microphone but you still want to dictate.

Pure native Rust. **No Electron, Tauri, or WebView** — the desktop app is a single tray-only process; the browser is used only for the QR page and the phone's own browser.

[中文说明](README.zh.md)

## How it works

1. Start nomic. A QR code opens in your browser.
2. Scan it with your phone — it opens the input page in the phone's browser (same Wi-Fi network required).
3. Click into any input field on your computer, then type on your phone and hit send.

Text is written to the clipboard and pasted at the cursor automatically (⌘V / Ctrl+V is simulated). If auto-paste is unavailable, the text stays in the clipboard and you can paste manually. The phone page only clears its input after the desktop acknowledges delivery, so typed text is never lost.

## Features

- **Voice dictation** — uses your phone keyboard's built-in mic; the page never requests microphone access.
- **Instant pairing** — scan a QR code; no accounts, no cloud. All traffic stays on your LAN.
- **Multi-device** — several phones can connect at once; text is injected serially.
- **Tray control** — delivery mode (auto-paste / clipboard-only), network interface, token rotation, UI language — all from the menu bar / system tray.
- **Bilingual UI** — English by default, 中文 via the tray menu or `--lang zh`.
- **Tiny** — a single ~1.5 MB binary. No frontend framework, no image libraries, no external assets.

## Installation

Download the latest build from [Releases](../../releases):

| Platform | File | Notes |
|---|---|---|
| macOS | `nomic-macos-universal.zip` | Apple silicon + Intel, signed & notarized |
| Windows | `nomic-windows-x86_64.zip` | 64-bit, no console window |
| Linux | `nomic-linux-x86_64.tar.gz` | Requires GTK3 + `libayatana-appindicator3` |

Or build from source:

```sh
cargo build --release
```

## First run

- **macOS**: auto-paste needs Accessibility permission — System Settings → Privacy & Security → Accessibility → enable nomic. Without it, nomic falls back to clipboard-only (paste manually with ⌘V).
- **Windows**: allow the firewall prompt on first launch and mark your Wi-Fi as a *private* network.
- **Linux**: needs `libgtk-3-dev`, `libayatana-appindicator3-1` (or your distro's `libappindicator3-1`); GNOME requires the AppIndicator extension. Auto-paste works on X11; on Wayland it falls back to clipboard-only.

## Building from source

```sh
cargo run --release                  # start the tray app and open the QR page
cargo run --release -- --port 8765   # custom port (default 8765)
cargo run --release -- --clipboard-only
cargo run --release -- --lang zh     # Chinese UI (default: English)
```

Requirements: Rust (edition 2024). On Linux, install `libgtk-3-dev libayatana-appindicator3-dev libxdo-dev` first.

## Security model

- The pairing URL contains a fresh 128-bit token, regenerated on every launch; anyone on the LAN with the link can type into your desktop — treat the QR code like a password.
- Rotating the token (tray menu or app restart) disconnects all phones and invalidates old links.
- Every HTTP route validates the token (404 otherwise); `Host` is checked against DNS rebinding, WebSocket handshakes against `Origin`.
- Limits: 64 KiB per message, 8 concurrent clients, 100 ms per-connection rate limit.
- HTTP (not HTTPS) is a deliberate trade-off: dictation is done by the phone OS keyboard, and the page uses no browser APIs that require a secure context.

## Troubleshooting

- **Phone can't connect** — both devices must be on the same Wi-Fi without guest/AP isolation; corporate VLANs usually block this too.
- **Wrong interface picked** — with a VPN or Docker running, switch the advertised address in the tray *Network* menu.
- **Auto-paste fails** — macOS: check Accessibility permission. Windows: the target window may run elevated (as admin). Either way, the text is still in your clipboard.
- The QR address must be a LAN IP, never `127.0.0.1` (or the phone would open the page on itself).

## License

See [LICENSE](LICENSE).
