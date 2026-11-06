/// 程序化生成托盘图标，避免资源文件与图像解码依赖。
/// 颜色固定中性（macOS 上作模板图标由系统按明暗自动着色），状态全部用形状表达：
/// 空闲 = 圆环，已连接 = 圆环 + 右下圆点（与 app 图标呼应），注入中 = 实心圆。
///
/// 渲染质量说明：tray-icon 在 macOS 上把图标缩放到 18pt（Retina 下 36px），
/// 供它缩放的源图必须明显大于 18pt 且边缘解析式抗锯齿，否则状态栏锯齿明显。
/// 这里以 128px 渲染（SDF + 1px 解析式羽化），再交给系统高质量下采样。
pub fn icon(connected: bool, flash: bool) -> tray_icon::Icon {
    const S: u32 = 128;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    let center = (S - 1) as f64 / 2.0;
    // 几何尺寸以旧版 22px 画布为基准（u = S/22），保持与 app 图标一致的比例。
    let u = S as f64 / 22.0;
    let radius = 8.0 * u;
    let thickness = 3.0 * u;
    let dot_r = 2.6 * u;
    let dot_d = radius * std::f64::consts::FRAC_1_SQRT_2; // 圆点在环上，右下 45°
    let gap = dot_r + 1.1 * u; // 圆点周围留一圈空隙，让它从环上"浮"出来
    // macOS 模板图标的颜色会被系统覆盖，纯黑即可；其他平台用中性灰，明暗任务栏都可读。
    #[cfg(target_os = "macos")]
    let (r, g, b) = (0u8, 0u8, 0u8);
    #[cfg(not(target_os = "macos"))]
    let (r, g, b) = (142u8, 142u8, 147u8);
    for y in 0..S {
        for x in 0..S {
            let px = x as f64 + 0.5 - center;
            let py = y as f64 + 0.5 - center;
            // 圆盘覆盖度（1px 解析式羽化）。
            let cov = |edge: f64, d: f64| (edge - d + 0.5).clamp(0.0, 1.0);
            let alpha = if flash {
                cov(radius, px.hypot(py))
            } else {
                let d_ring = px.hypot(py);
                // 环 = 外圈覆盖度 - 内圈覆盖度。
                let mut ring = cov(radius, d_ring) - cov(radius - thickness, d_ring);
                let d_dot = (px - dot_d).hypot(py - dot_d);
                if connected {
                    // 从环上挖出圆点周围的空隙，再叠上实心圆点。
                    ring *= 1.0 - cov(gap, d_dot);
                    ring = ring.max(cov(dot_r, d_dot));
                }
                ring
            };
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
