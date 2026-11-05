# 快速开始

## 1. 安装

从 [GitHub Releases](https://github.com/asionbo/nomic/releases/latest) 下载对应平台的最新版本：

| 平台 | 文件 | 说明 |
| --- | --- | --- |
| macOS | `nomic-macos-universal.zip` | Apple silicon + Intel 通用包，已验证 |
| Windows | `nomic-windows-x86_64.zip` | 64 位，无控制台窗口 |
| Linux | `nomic-linux-x86_64.tar.gz` | 需要 GTK3 与 AppIndicator |

也可以从源码构建（需要 Rust edition 2024）：

```sh
cargo build --release
```

## 2. 扫码配对

1. 启动 nomic，电脑浏览器会自动打开二维码页面。
2. 用手机相机扫码，在手机浏览器里打开输入页。**两台设备必须连同一个 Wi-Fi。**
3. 在电脑上点一下目标输入框，然后在手机上输入文字并点「发送到电脑」。

文字会写入剪贴板，并自动模拟 ⌘V / Ctrl+V 粘贴到光标处。

::: tip 粘贴后自动回车
聊天输入框、搜索框等场景往往需要发送。在托盘菜单勾选「粘贴后自动回车」后，每次粘贴成功，nomic 会再模拟一次回车键。该开关默认关闭；仅在「粘贴到光标处」模式下生效，粘贴失败或仅剪贴板模式下不会发送回车。
:::

::: warning 二维码等同于密码
链接里包含每次启动重新生成的 128-bit 令牌。同网段任何人拿到链接，都可以向这台电脑输入文字。不要公开分享二维码；怀疑泄露时，从托盘菜单选择「重新生成令牌」或重启应用。
:::

## 3. 首次运行的系统权限

- **macOS**：自动粘贴需要「辅助功能」授权。系统设置 → 隐私与安全性 → 辅助功能 → 添加 nomic。未授权时自动降级为仅剪贴板。
- **Windows**：首次启动允许防火墙提示，并把当前 Wi-Fi 设为「专用网络」。目标窗口若以管理员权限运行，可能无法模拟按键。
- **Linux**：需要 `libgtk-3-dev` 与 `libayatana-appindicator3-1`；GNOME 需要 AppIndicator 扩展。X11 下自动粘贴正常，Wayland 下降级为仅剪贴板。

## 命令行参数

```sh
cargo run --release                  # 启动托盘应用并打开二维码页
cargo run --release -- --port 8765   # 指定端口（默认 8765，被占用时自动换端口）
cargo run --release -- --clipboard-only  # 只写剪贴板，不模拟粘贴
cargo run --release -- --lang zh     # 中文界面（默认英文）
```

## 常见问题

**手机连不上？** 两台设备必须在同一 Wi-Fi，且没有开启访客网络 / AP 隔离；公司网 VLAN 通常也会拦截。二维码里的地址应该是局域网 IP，而不是 `127.0.0.1`。

**网卡选错了？** 如果电脑挂着 VPN 或 Docker，在托盘「网络」菜单里切换到正确的局域网地址，然后重新打开二维码页。

**自动粘贴没生效？** 先检查上面的系统权限。文字仍保留在剪贴板里，手动按 ⌘/Ctrl+V 即可。

更多细节见项目 [README](https://github.com/asionbo/nomic#readme)。
