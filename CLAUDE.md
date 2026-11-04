# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository state (read before changing anything)

The project compiles, passes `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`, and has been verified end-to-end on macOS (WS text → clipboard → ⌘V paste, ack returned to the client). `Cargo.lock` is committed and CI's `--locked` build works against it. Do not bump crate versions casually — the versions in `Cargo.toml` were pinned against the real APIs at first successful build (tray-icon 0.21, muda 0.17, tao 0.34, axum 0.8, enigo 0.6, rand 0.9).

The Rust toolchain is not on the default PATH. `scripts/build-check.sh` hardcodes its location:

```sh
export RUSTUP_HOME=/Volumes/Samsung-SSD/rust/.rustup
export CARGO_HOME=/Volumes/Samsung-SSD/rust/.cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

## Commands

```sh
sh scripts/build-check.sh          # cargo fetch + cargo check, logs to /tmp/nomic-fetch.log and /tmp/nomic-check.log
cargo check                        # after exporting the env vars above
cargo build --release              # single binary; release profile is opt-level=z + lto + panic=abort + strip
cargo run --release                # starts tray app and opens the QR page in the default browser
cargo run --release -- --port 8765
cargo run --release -- --clipboard-only
cargo test                         # no test suite exists yet
cargo test <test_name>             # run a single test once tests are added
cargo fmt --check && cargo clippy --all-targets -- -D warnings
RUST_LOG=info cargo run --release  # defaults to warn-level tracing
```

On Linux, system deps are `libgtk-3-dev libayatana-appindicator3-dev libxdo-dev`; `.github/workflows/build.yml` builds a 4-target matrix (macOS arm64/x86_64, Windows MSVC, Ubuntu 22.04).

Local end-to-end checks without a phone (token is only visible in the `/pair` page URL or startup logs):

```sh
curl "http://127.0.0.1:8765/health?t=<token>"
curl "http://127.0.0.1:8765/pair?t=<token>"     # inline QR SVG + plaintext URL
curl -i "http://127.0.0.1:8765/m?t=wrong"      # expect 404
curl -i -H "Host: evil.example" "http://127.0.0.1:8765/m?t=<token>"  # expect 404
websocat "ws://127.0.0.1:8765/ws?t=<token>"    # send {"type":"text","id":"1","text":"中文测试"}
```

## Architecture

A phone types into a LAN web page; the desktop writes the text to the clipboard and simulates ⌘/Ctrl+V. **No WebView, Electron, or Tauri** — the desktop binary is a native tray-only process. Browsers are used only for the desktop QR page (`/pair`) and the phone's own browser (`/m`).

The central constraint is **thread affinity**: macOS requires `NSApplication`, `tray-icon`/`muda`, `NSPasteboard`, and CGEvent injection on the main thread. So there are exactly two execution contexts:

- **Main thread** (`src/main.rs`): owns the `tao::EventLoop`, `Tray`, and the `Injector` (which holds the long-lived `arboard::Clipboard` and `enigo::Enigo`). Runs `EventLoop::run` and serially consumes the injection queue on `UserEvent::InjectionReady`.
- **One spawned tokio thread**: owns a 2-worker multi-thread runtime and `axum::serve` on a pre-bound `std::net::TcpListener` converted via `TcpListener::from_std`.

`Shared` (`src/app.rs`) is the cloneable state bridging the two. Key channels: `inject_tx`/`inject_rx` (phone text → main thread), each `Injection` carrying a per-connection `reply` sender for the ack; `wake: EventLoopProxy<UserEvent>` (server → main thread, for injection and connection counts — **without this the queue is never drained**); `ui_tx`/`ui_rx` (tray → server shutdown); `kick` broadcast + `token_generation` watch (invalidate all WS connections when the token rotates). `muda` menu events arrive on their own global receiver and are forwarded into the event loop by a third small thread.

The listener is bound **before** URLs are built (`app::bind`), so the QR always contains the real port; port 8765 falls back to an OS-assigned port. `lan_candidates()` scores private IPv4s and skips loopback/link-local/VPN/Docker interfaces; the QR must encode `http://<LAN_IP>:<port>/m?t=<token>`, never 127.0.0.1.

Routes in `src/server.rs`: `/pair` (server-generated QR SVG injected into `assets/pair.html`), `/m` (`assets/mobile.html`), `/ws`, `/health`. Every route validates the per-launch 128-bit token (returning 404) and `Host`; WS additionally validates `Origin`, caps message size at 64 KiB, caps clients at 8, and rate-limits at 100 ms/connection. Pages are `include_str!`-embedded with a restrictive CSP; there are no external assets or frontend dependencies.

WS protocol: phone sends `{type:"text", id, text}`; desktop replies `{type:"ack", id, status:"pasted"|"clipboard-only"|"error"}` or `{type:"token-expired"}`. `assets/mobile.html` keeps exactly one `pending` message, only clears the textarea on ack, resends on reconnect, and backs off exponentially. Any ack/queuing change must preserve this "never lose typed text" property.

`Injector::deliver` (`src/inject.rs`) writes the clipboard, waits 50 ms, then presses Meta/Control + V. **The `Clipboard` instance must never be dropped** — on X11/Wayland it owns the clipboard selection, so dropping it invalidates the content. Any failure (missing macOS Accessibility permission, elevated Windows target window, Wayland) sets `auto_paste_available = false` and degrades to clipboard-only, which is the universal fallback. Chinese/emoji go through the clipboard deliberately; never use per-character key typing.

`src/qr.rs` renders the QR matrix to SVG directly and `src/icon.rs` generates tray icons as raw RGBA, both to avoid image-decoding dependencies — keep new dependencies out unless there is no equivalent.

## Design invariants

- Keep the binary small: no WebView, no frontend framework, no image crate, no config file (CLI flags only), no CDN links in HTML.
- HTTP (not HTTPS) is intentional: speech input is the phone OS keyboard's dictation, so the page never calls `getUserMedia` or the clipboard API and does not need a secure context. Adding in-page recording or browser-sensitive APIs requires HTTPS.
- The token is per-launch, never persisted, and any QR/URL leak is equivalent to letting that LAN peer type into the desktop.
- See `README.md` for the full platform-permission and network troubleshooting matrix (macOS Accessibility, Windows firewall/UAC, Linux AppIndicator/Wayland, AP isolation).
