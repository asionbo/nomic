# nomic

Use your phone as a wireless keyboard for your computer. Type on your phone or use its built-in voice dictation, then send the text straight to your desktop cursor. It's especially useful when your computer has no microphone but you still want to dictate.

Built with native Rust for a tiny memory footprint.

[中文说明](README.zh.md) · [项目主页](https://asionbo.github.io/nomic/) · [站点源码](site/SITE.md)

## How it works

1. Start nomic. It opens a page with a QR code in your browser.
2. Scan the code with your phone. The input page opens in your phone's browser. Both devices must be on the same Wi-Fi network.
3. Click the target input field on your computer, then type on your phone and tap Send.

Text is copied to the clipboard and automatically pasted at your cursor (using ⌘V / Ctrl+V). If auto-paste isn't available, you can paste the text manually from the clipboard. Optionally, enable **"Press Enter after paste"** in the tray menu to have nomic also press Enter after each paste — handy for chat apps and search boxes.

## Features

- **Phone dictation** — use the microphone built into your phone's keyboard; the page itself never requests microphone access.
- **Instant pairing** — connect by scanning a QR code. No account or cloud service required; all traffic stays on your local network.
- **Multiple devices** — connect several phones at once; text is sent to your computer one message at a time.
- **Tray controls** — choose how text is delivered (auto-paste or clipboard only), optionally press Enter automatically after each paste, select a network interface, rotate the token, and change the interface language.
- **Bilingual interface** — English by default; switch to Chinese from the tray menu or with `--lang zh`.
- **Lightweight** — a single ~1.5 MB binary, with no frontend framework, image libraries, or external assets.

## Installation

Download the latest release:

| Platform | File | Notes |
|---|---|---|
| macOS | `nomic-macos-universal.zip` | Universal build for Apple silicon and Intel; verified |
| Windows | `nomic-windows-x86_64.zip` | 64-bit; no console window |
| Linux | `nomic-linux-x86_64.tar.gz` | Requires GTK3 and `libayatana-appindicator3` |

Or build from source:

```sh
cargo build --release
```

## First run

- **macOS**: Auto-paste requires Accessibility permission. Go to System Settings → Privacy & Security → Accessibility and add nomic. Without permission, nomic falls back to clipboard-only; paste manually with ⌘V.
- **Windows**: Allow the firewall prompt when you first launch nomic, and make sure your Wi-Fi network is set to *Private*.
- **Linux**: Requires `libgtk-3-dev` and `libayatana-appindicator3-1` (or your distribution's `libappindicator3-1`). GNOME requires the AppIndicator extension. Auto-paste works on X11; on Wayland, nomic falls back to clipboard-only.

## Building from source

```sh
cargo run --release                  # Start the tray app and open the QR page
cargo run --release -- --port 8765   # Use a custom port (default: 8765)
cargo run --release -- --clipboard-only
cargo run --release -- --lang zh     # Chinese interface (English by default)
```

Requires Rust (edition 2024). On Linux, install `libgtk-3-dev libayatana-appindicator3-dev libxdo-dev` first.

## Security model

- The pairing link contains a 128-bit token that is regenerated every time the app starts. Anyone on your local network with the link can send text to your computer, so treat the QR code like a password.
- Rotating the token from the tray menu or restarting the app disconnects all phones and invalidates old links.
- Every HTTP route checks the token and returns 404 if it is invalid. The `Host` header is checked to prevent DNS rebinding, and WebSocket handshakes validate the `Origin` header.
- Limits: 64 KiB per message, up to 8 simultaneous clients, and one message per connection every 100 ms.

## Troubleshooting

- **Can't connect from your phone** — both devices must be on the same Wi-Fi network, without guest network or AP isolation. Corporate VLANs may also block connections.
- **Wrong network interface selected** — if you're using a VPN or Docker, choose the correct address from the tray's Network menu.
- **Auto-paste doesn't work** — on macOS, check Accessibility permission. On Windows, the target window may be running as administrator.
- The QR code must contain a local network IP address, not `127.0.0.1`.

## License

See [LICENSE](LICENSE).
