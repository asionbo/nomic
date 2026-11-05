# nomic

把手机当作电脑的无线键盘：在手机网页上打字或用语音输入法转文字，一键发送到电脑光标所在位置。目的是解决电脑没有mic，但是又想语音输入的场景。

纯 Rust 原生实现，占用内存极少。

[English](README.md) · [项目主页](https://asionbo.github.io/nomic/) · [站点源码](site/SITE.md)

## 工作原理

1. 启动 nomic，浏览器自动打开二维码页。
2. 手机扫码（需与电脑连**同一 Wi-Fi**），在手机浏览器打开输入页。
3. 在电脑上点一下目标输入框，然后在手机上打字、点发送。

文字会写入剪贴板并自动粘贴到光标处（模拟 ⌘V / Ctrl+V）。若自动粘贴不可用，文字仍保留在剪贴板中，可手动粘贴。在托盘菜单开启「粘贴后自动回车」后，每次粘贴成功还会自动模拟一次回车键，适合聊天输入框、搜索框等场景。

## 特性

- **手机听写** —— 用手机键盘自带的麦克输入，页面不申请麦克风权限。
- **即扫即用** —— 扫码配对；无账号、无云端。所有流量只走局域网。
- **多设备** —— 支持多台手机同时连接，文本按顺序串行注入。
- **托盘控制** —— 投递方式（自动粘贴 / 仅剪贴板）、粘贴后自动回车、网卡选择、令牌轮换、界面语言。
- **双语界面** —— 默认英文，托盘菜单或 `--lang zh` 切换中文。
- **轻量** —— 单个约 1.5 MB 的二进制；无前端框架、无图像库、无外部资源。

## 安装

从 [Releases](../../releases) 下载最新版本：

| 平台 | 文件 | 说明 |
|---|---|---|
| macOS | `nomic-macos-universal.zip` | Apple silicon + Intel 通用 已验证✅|
| Windows | `nomic-windows-x86_64.zip` | 64 位，无控制台窗口|
| Linux | `nomic-linux-x86_64.tar.gz` | 需 GTK3 + `libayatana-appindicator3`|

或从源码构建：

```sh
cargo build --release
```

## 首次运行

- **macOS**：自动粘贴需要「辅助功能」授权 —— 系统设置 → 隐私与安全性 → 辅助功能 → 添加 nomic。未授权时自动降级为仅剪贴板（手动 ⌘V）。
- **Windows**：首次启动允许防火墙提示，并把当前 Wi-Fi 设为「专用网络」。
- **Linux**：需要 `libgtk-3-dev`、`libayatana-appindicator3-1`（或发行版的 `libappindicator3-1`）；GNOME 需启用 AppIndicator 扩展。X11 下自动粘贴正常，Wayland 下降级为仅剪贴板。

## 从源码构建

```sh
cargo run --release                  # 启动托盘应用并打开二维码页
cargo run --release -- --port 8765   # 自定义端口（默认 8765）
cargo run --release -- --clipboard-only
cargo run --release -- --lang zh     # 中文界面（默认英文）
```

要求：Rust（edition 2024）。Linux 需先安装 `libgtk-3-dev libayatana-appindicator3-dev libxdo-dev`。

## 安全模型

- 配对链接含每次启动重新生成的 128-bit 令牌；同网段拿到链接的人即可向电脑输入文字 —— 请像保管密码一样保管二维码。
- 轮换令牌（托盘菜单或重启程序）会断开所有手机并使旧链接失效。
- 所有 HTTP 路由校验令牌（否则返回 404）；校验 `Host` 防 DNS rebinding，WebSocket 握手校验 `Origin`。
- 限制：单消息 64 KiB、最多 8 个并发客户端、每连接 100 ms 限速。

## 常见问题

- **手机连不上** —— 两台设备必须在同一 Wi-Fi 且未开启访客/AP 隔离；公司网 VLAN 通常也会拦截。
- **网卡选错** —— 挂 VPN 或 Docker 时，在托盘「网络」菜单里切换地址。
- **自动粘贴失败** —— macOS：检查辅助功能授权；Windows：目标窗口可能以管理员权限运行。
- 二维码地址必须是局域网 IP，不能是 `127.0.0.1`。

## 许可证

见 [LICENSE](LICENSE)。
