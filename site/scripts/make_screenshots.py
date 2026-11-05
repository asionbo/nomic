#!/usr/bin/env python3
"""Generate two WebP screenshots from nomic's real HTML templates.

Run from the site folder: python3 scripts/make_screenshots.py
Reads ../assets/*.html and writes the rendered HTML to a temporary directory;
the application assets are never modified. A fake QR matrix and example token
are used, so no real pairing secret is embedded in the images.

Requires headless Google Chrome and cwebp on macOS.
"""

import html
import json
import random
import subprocess
import tempfile
from pathlib import Path

CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "docs" / "public" / "images"


def mock_qr_svg() -> str:
    """A deterministic, non-scannable QR-shaped matrix (same look as src/qr.rs)."""
    rng = random.Random(20261105)
    size = 29
    quiet = 4
    total = size + 2 * quiet
    parts = [
        f'<svg class="qr" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total} {total}" '
        'shape-rendering="crispEdges" role="img" aria-label="示例二维码">'
        f'<rect width="{total}" height="{total}" fill="#fff"/>'
    ]

    def finder(cx: int, cy: int) -> None:
        for dy in range(7):
            for dx in range(7):
                edge = dx in (0, 6) or dy in (0, 6)
                inner = 2 <= dx <= 4 and 2 <= dy <= 4
                if edge or inner:
                    parts.append(
                        f'<rect x="{cx + dx + quiet}" y="{cy + dy + quiet}" '
                        'width="1" height="1" fill="#000"/>'
                    )

    dark: set[tuple[int, int]] = set()
    for corner in ((0, 0), (size - 7, 0), (0, size - 7)):
        finder(*corner)
        for dy in range(-1, 8):
            for dx in range(-1, 8):
                dark.add((corner[0] + dx, corner[1] + dy))
    for y in range(size):
        for x in range(size):
            if (x, y) not in dark and rng.random() < 0.48:
                parts.append(
                    f'<rect x="{x + quiet}" y="{y + quiet}" width="1" height="1" fill="#000"/>'
                )
    parts.append("</svg>")
    return "".join(parts)


def render(name: str, body: str, width: int, height: int, work: Path) -> None:
    source = work / f"{name}.html"
    target = OUT / f"{name}.png"
    source.write_text(body, encoding="utf-8")
    OUT.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            CHROME,
            "--headless=new",
            "--disable-gpu",
            "--hide-scrollbars",
            "--no-first-run",
            "--no-default-browser-check",
            "--force-device-scale-factor=2",
            f"--window-size={width},{height}",
            f"--screenshot={target}",
            source.as_uri(),
        ],
        check=True,
        capture_output=True,
        timeout=60,
    )
    print(f"wrote {target} ({target.stat().st_size:,} bytes)")


def main() -> None:
    repo = HERE.parent.parent
    if not Path(CHROME).exists():
        raise SystemExit(f"Chrome not found: {CHROME}")
    pair = (repo / "assets" / "pair.html").read_text(encoding="utf-8")
    mobile = (repo / "assets" / "mobile.html").read_text(encoding="utf-8")

    url = "http://192.168.1.23:8765/m?t=example-token"
    lang = {
        "pair_title": "nomic · 扫码连接",
        "h1": "nomic · 手机连接",
        "sub": "用手机相机或扫码 App 扫描",
        "copy": "复制链接",
        "note_title": "连不上？",
        "note1": "1. 手机与电脑必须连同一个 Wi-Fi，且网络未开启「访客隔离 / AP 隔离」。",
        "note2": "2. 电脑防火墙首次可能弹窗，请允许本程序监听，并把当前 Wi-Fi 设为「专用网络」。",
        "note3": "3. 若电脑有多个网卡 / 挂了 VPN，可在托盘菜单的「网络」里换一个地址，再重新打开本页。",
        "note4": "4. 二维码里的地址 192.168.1.23:8765 不能写成 127.0.0.1，否则手机会打开自己。",
        "note5": "5. 链接含一次性令牌；重启程序或点托盘「重新生成令牌」后需重新扫码。",
        "mobile_title": "nomic · 输入",
        "send": "发送到电脑",
        "clear": "清空",
        "placeholder": "在这里输入，或用手机键盘上的麦克风按钮语音听写…",
        "hint": "发送前请先点回电脑上要输入的输入框。文字会写入剪贴板并自动粘贴；若自动粘贴不可用，会提示你手动按 ⌘/Ctrl+V。语音听写请用手机系统键盘自带的麦克风，本页面不申请麦克风权限。",
    }
    i18n = {
        "connecting": "连接中…",
        "connected": "已连接",
        "disconnected": "已断开",
        "reconnecting": "重连中…",
        "tokenExpiredStatus": "令牌已失效",
        "tokenExpiredToast": "链接已失效，请在电脑上重新显示二维码",
        "toastEmpty": "请先输入内容",
        "toastNotConnected": "尚未连接，正在重连…",
        "toastBusy": "上一条还在处理",
        "toastPasted": "已粘贴到电脑",
        "toastCopied": "已复制到剪贴板，请在电脑上按 ⌘/Ctrl+V",
        "toastFailed": "发送失败：{msg}",
        "toastNoResponse": "电脑未响应，请重试（内容已保留）",
    }

    substitutions = {
        "{{QR_SVG}}": mock_qr_svg(),
        "{{MOBILE_URL}}": html.escape(url, quote=True),
        "{{HTML_LANG}}": "zh-CN",
        "{{TITLE}}": lang["pair_title"],
        "{{H1}}": lang["h1"],
        "{{SUB}}": lang["sub"],
        "{{COPY}}": lang["copy"],
        "{{COPIED_JSON}}": '"已复制"',
        "{{COPY_JSON}}": f'"{lang["copy"]}"',
        "{{NOTE_TITLE}}": lang["note_title"],
        "{{NOTE1}}": lang["note1"],
        "{{NOTE2}}": lang["note2"],
        "{{NOTE3}}": lang["note3"],
        "{{NOTE4}}": lang["note4"],
        "{{NOTE5}}": lang["note5"],
    }
    for key, value in substitutions.items():
        pair = pair.replace(key, value)
    if "{{" in pair:
        raise SystemExit("unreplaced placeholder left in pair.html")

    # The mobile page normally connects to the running desktop app. For the
    # screenshot, replace its JS with a static preview state (no WebSocket).
    mobile = mobile.replace("{{HTML_LANG}}", "zh-CN")
    mobile = mobile.replace("{{TITLE}}", lang["mobile_title"])
    mobile = mobile.replace("{{SEND}}", lang["send"])
    mobile = mobile.replace("{{CLEAR}}", lang["clear"])
    mobile = mobile.replace("{{PLACEHOLDER}}", html.escape(lang["placeholder"], quote=True))
    mobile = mobile.replace("{{HINT}}", lang["hint"])
    start = mobile.index("<script>")
    end = mobile.index("</script>", start) + len("</script>")
    preview = f"""<script>
  const I18N = {json.dumps(i18n, ensure_ascii=False)};
  document.getElementById('dot').className = 'dot on';
  document.getElementById('status').textContent = I18N.connected;
  document.getElementById('text').value = '今天下午三点开会，记得带上项目演示材料。';
  document.getElementById('send').disabled = false;
</script>"""
    mobile = mobile[:start] + preview + mobile[end:]
    if "{{" in mobile:
        raise SystemExit("unreplaced placeholder left in mobile.html")

    with tempfile.TemporaryDirectory(prefix="nomic-screenshots-") as tmp:
        work = Path(tmp)
        render("pair-page", pair, 900, 980, work)
        render("mobile-page", mobile, 430, 800, work)
        # Compress; Chrome writes large PNGs at 2x scale.
        for name in ("pair-page", "mobile-page"):
            subprocess.run(
                ["cwebp", "-q", "86", "-m", "6", str(OUT / f"{name}.png"), "-o", str(OUT / f"{name}.webp")],
                check=True,
                capture_output=True,
            )
            (OUT / f"{name}.png").unlink()
            print(f"wrote {OUT / f'{name}.webp'} ({(OUT / f'{name}.webp').stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
