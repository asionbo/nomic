# nomic

把手机当作电脑的无线麦克风 / 键盘：手机输入文字（可用系统键盘的语音听写），一键回传到桌面，写入剪贴板并自动粘贴到当前光标位置。

纯 Rust 原生实现，**不含 Electron / Tauri / WebView**。桌面端只有一个常驻托盘进程；浏览器只用于显示二维码的临时标签页和手机自带浏览器。

## 状态

✅ 已在 macOS（arm64）完成编译与端到端验证：WS 发送中文/emoji → 剪贴板 → 模拟 ⌘V 粘贴 → ack 回传手机页；安全校验（错误 token / 伪造 Host 返回 404）与降级路径（无辅助功能权限时自动转仅剪贴板）均实测通过。中英文双语界面（默认英文）也已实测两种语言的页面、托盘状态与 ack 消息。release 二进制约 1.5 MB，RSS 约 50 MB。Windows / Linux 平台尚未实测。

工具链位置（若 cargo 不在 PATH）：

```sh
export RUSTUP_HOME=/Volumes/Samsung-SSD/rust/.rustup
export CARGO_HOME=/Volumes/Samsung-SSD/rust/.cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

## 运行

```sh
cargo run --release
# 可选：
cargo run --release -- --port 8765
cargo run --release -- --clipboard-only
cargo run --release -- --lang zh    # 界面切换中文，默认英文
```

**多语言**：默认英文（`en`），`--lang zh` 切中文。覆盖托盘菜单与状态、二维码页、手机录入页、ack 提示与错误消息；日志固定英文。目前支持 `en` / `zh` 两个语言，字符串集中在 `src/i18n.rs`，加语言只需新增一份 `Strings` 常量并在 `Lang::from_str` / `Lang::strings` 里登记。

启动后托盘出现，并自动用系统默认浏览器打开二维码页。以下菜单名以中文（`--lang zh`）为例：

| 菜单项 | 作用 |
|---|---|
| 显示二维码（打开浏览器） / Show QR code | 打开本地 `/pair` 页，内含二维码与明文链接 |
| 复制手机链接 / Copy phone link | 复制 `http://<LAN_IP>:<port>/m?t=<token>` |
| 粘贴到光标处 / 仅复制到剪贴板 | 切换投递方式 |
| 网络 / Network | 多网卡时切换二维码使用的 LAN IP |
| 重新生成令牌 / Regenerate token | 作废旧链接、断开已连接的手机 |
| 退出 nomic / Quit nomic | 优雅关闭服务与托盘 |

托盘图标：灰色 = 空闲，绿色 = 有设备连接，黄色闪一下 = 刚收到文本。

## 使用方法

**首次使用（每台电脑一次）：**

1. 启动 nomic（`cargo run --release` 或直接运行二进制）。
2. **macOS**：模拟 ⌘V 需要「辅助功能」授权 —— 打开「系统设置 → 隐私与安全性 → 辅助功能」，把 nomic 加进去并打开开关。不授权也能用（自动降级为仅复制，需手动 ⌘V）。
3. **Windows**：首次启动弹防火墙提示时点「允许」，并把当前 Wi-Fi 设为「专用网络」。

**日常使用（每次三步）：**

1. **点托盘「显示二维码」** —— 默认浏览器打开二维码页。
2. **手机扫码** —— 相机或扫码 App 扫码，在手机浏览器打开输入页。要求手机与电脑连**同一个 Wi-Fi**。页面顶部圆点变绿即已连接。
3. **在电脑上点一下目标输入框**（让光标落在你要输入的地方），然后在手机输入文字 —— 支持手机键盘自带的语音听写，页面不申请麦克风权限 —— 点「发送到电脑」。

文字随即出现在电脑光标处，同时留在剪贴板里（可再 ⌘/Ctrl+V 一次）。手机页收到 ack 后才清空输入框，发送失败不会丢字。

**小提示：**

- 手机页可「添加到主屏幕」，之后像 App 一样打开；连接断开会自动重连（指数退避，回到前台立即重试）。
- 若点了「发送」电脑没反应：先把电脑上的目标窗口点成前台焦点，再按一次发送；剪贴板里始终有刚发的内容，可手动粘贴。
- 多台手机可同时连接；发来的文本按先后顺序串行注入。
- 重启 nomic 或点「重新生成令牌」后二维码失效，需重新扫码（旧页面会显示「令牌已失效」）。
- 只想复制不自动粘贴：托盘菜单切到「仅复制到剪贴板」。

## 平台要求

**macOS**
- 模拟 ⌘V 需要在「系统设置 → 隐私与安全性 → 辅助功能」授权本程序。未授权时 enigo 报错并自动降级为「仅剪贴板」，手机端会提示手动粘贴。
- 分发时打成 `.app`，`Info.plist` 设 `LSUIElement=true` 隐藏 Dock 图标；arm64 与 x86_64 分别构建后 `lipo` 合成通用二进制。

**Windows**
- 使用 `windows_subsystem = "windows"`，无控制台窗口。
- 首次监听会弹防火墙提示，请选择允许并把当前 Wi-Fi 标记为「专用网络」。
- 目标窗口若以管理员权限运行，普通进程无法向其注入按键，会自动降级为仅剪贴板。

**Linux**
- 运行需 GTK3 与 `libayatana-appindicator3-1`（或发行版的 `libappindicator3-1`）；GNOME 需启用 AppIndicator / KStatusNotifierItem 扩展。
- X11（XTEST）下自动粘贴正常；Wayland 下 `arboard` 依赖 `wlroots` 系 data-control 协议，按键注入大概率不可用，会自动降级为仅剪贴板。

## 网络与排障

- 手机与电脑必须在同一 Wi-Fi，且该网络未开启「访客隔离 / AP 隔离」；公司网 VLAN 隔离同样会失败。
- 挂 VPN 或 Docker 时可能选错网卡，在托盘「网络」子菜单切换，再重新打开二维码页。
- 二维码内容是 **LAN IP**，不是 `127.0.0.1`（否则手机会打开自己）。
- 端口默认 8765；被占用时自动改用系统随机端口，二维码使用真实端口。
- 链接含每次启动重新生成的 128bit 令牌；重启程序或点「重新生成令牌」后需重新扫码。
- 同网段能拿到链接的人即可发送文本，这是明确的威胁模型；不要长期暴露二维码截图。

## 安全设计

所有路由校验 token（错误返回 404）；校验 `Host` 防 DNS rebinding；WS 握手校验 `Origin` 防 CSWSH；消息上限 64 KiB、最多 8 个并发连接、每连接最小 100ms 间隔；手机页只在收到 ack 后清空输入框，断线指数退避重连；多设备文本串行注入，避免剪贴板竞争。

为什么用 HTTP 而不是 HTTPS：手机系统的键盘听写由 OS 完成，网页不调用 `getUserMedia` 或剪贴板 API，因此不需要安全上下文。**若以后要在网页内录音或调用浏览器敏感 API，必须升级为 HTTPS**（自签证书 + 手机信任）。

## 本地自测（无需手机）

```sh
# 从 /pair 页面或启动时日志拿到 token
curl "http://127.0.0.1:8765/health?t=<token>"
curl "http://127.0.0.1:8765/pair?t=<token>"      # 看内联 SVG 与明文 URL
curl "http://127.0.0.1:8765/m?t=<token>"        # 手机页
websocat "ws://127.0.0.1:8765/ws?t=<token>"     # 手发 {"type":"text","id":"1","text":"中文测试"}
curl -i "http://127.0.0.1:8765/m?t=wrong"       # 应为 404
curl -i -H "Host: evil.example" "http://127.0.0.1:8765/m?t=<token>"  # 应为 404
```

桌面浏览器开手机页 + DevTools 设备模式也能模拟。真实跨设备路径请用另一台同网段设备或手机热点验证。

## 结构

```
src/main.rs      入口、tao 事件循环、托盘/server 线程编排、注入队列消费
src/app.rs       共享状态、LAN IP 选择、预绑定 listener、令牌
src/server.rs    axum 路由、token/Host/Origin 校验、WebSocket 协议
src/inject.rs    arboard 常驻剪贴板 + enigo 粘贴，失败降级
src/tray.rs      muda 菜单与 tray-icon
src/qr.rs        qrcode 矩阵 -> SVG
src/icon.rs      程序化 RGBA 托盘图标
assets/pair.html     桌面二维码页（服务端注入 SVG）
assets/mobile.html   手机录入页（内联 CSS/JS，零外链）
```
