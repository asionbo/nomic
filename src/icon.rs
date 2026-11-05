/// 程序化生成托盘图标，避免资源文件与图像解码依赖。
/// macOS 使用模板图标（系统按明暗自动着色），其他平台用彩色圆点。
///
/// 渲染质量说明：tray-icon 在 macOS 上把图标缩放到 18pt（Retina 下 36px），
/// 供它缩放的源图必须明显大于 22px 且边缘抗锯齿，否则状态栏锯齿明显。
/// 这里以 44px 渲染，圆环边缘用 1px 解析式羽化，再交给系统高质量下采样。
pub fn icon(connected: bool, flash: bool) -> tray_icon::Icon {
    const S: u32 = 44;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    let center = (S - 1) as f64 / 2.0;
    let radius = S as f64 * 8.0 / 22.0; // 与旧版 22px 下 radius=8 的比例一致
    let thickness = S as f64 * 3.0 / 22.0;
    let (r, g, b) = if flash {
        (255u8, 214u8, 10u8)
    } else if connected {
        (44u8, 190u8, 110u8)
    } else {
        (140u8, 146u8, 156u8)
    };
    for y in 0..S {
        for x in 0..S {
            let px = x as f64 + 0.5 - center;
            let py = y as f64 + 0.5 - center;
            let d = (px * px + py * py).sqrt();
            // 圆盘覆盖度（1px 羽化），外圈减内圈得到环形。
            let cov = |edge: f64| (edge - d + 0.5).clamp(0.0, 1.0);
            let alpha = cov(radius) - cov(radius - thickness);
            if alpha <= 0.0 {
                continue;
            }
            let i = ((y * S + x) * 4) as usize;
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = (alpha * 255.0).round() as u8;
        }
    }
    tray_icon::Icon::from_rgba(rgba, S, S).expect("valid RGBA icon")
}
