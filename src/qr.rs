use qrcode::QrCode;
use qrcode::types::{Color, QrError};

/// 把二维码矩阵拼成 SVG，避免引入 image / PNG 编码或前端 JS 库。
pub fn svg(data: &str, aria_label: &str) -> Result<String, QrError> {
    let code = QrCode::new(data.as_bytes())?;
    let width = code.width();
    let quiet = 4;
    let size = width + quiet * 2;
    let mut out = format!(
        "<svg class=\"qr\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" shape-rendering=\"crispEdges\" role=\"img\" aria-label=\"{}\"><rect width=\"{size}\" height=\"{size}\" fill=\"#fff\"/>",
        html_escape(aria_label)
    );
    let colors = code.to_colors();
    for (y, row) in colors.chunks(width).enumerate() {
        for (x, c) in row.iter().enumerate() {
            if *c == Color::Dark {
                out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"1\" height=\"1\" fill=\"#000\"/>",
                    x + quiet,
                    y + quiet
                ));
            }
        }
    }
    out.push_str("</svg>");
    Ok(out)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
