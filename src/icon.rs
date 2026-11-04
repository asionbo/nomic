/// 程序化生成托盘图标，避免资源文件与图像解码依赖。
/// macOS 使用模板图标（系统按明暗自动着色），其他平台用彩色圆点。
pub fn icon(connected: bool, flash: bool) -> tray_icon::Icon {
    const S: u32 = 22;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    let center = (S - 1) as f32 / 2.0;
    let radius = 8.0;
    let (r, g, b) = if flash {
        (255u8, 214u8, 10u8)
    } else if connected {
        (44u8, 190u8, 110u8)
    } else {
        (140u8, 146u8, 156u8)
    };
    for y in 0..S {
        for x in 0..S {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let i = ((y * S + x) * 4) as usize;
            if dx * dx + dy * dy <= radius * radius {
                // 中间挖空，做成环形「信号」标记。
                if dx * dx + dy * dy >= (radius - 3.0).powi(2) {
                    rgba[i] = r;
                    rgba[i + 1] = g;
                    rgba[i + 2] = b;
                    rgba[i + 3] = 255;
                }
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, S, S).expect("valid RGBA icon")
}
