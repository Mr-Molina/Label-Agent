//! Pure-Rust TSPL2 (`BT320 Printer`) thermal label compiler (`59 mm x 102 mm`).
//!
//! Compiles a [`BoxLabelSpec`] into a deterministic 203-DPI (`8 dots/mm`)
//! TSPL2 command stream sized exclusively for `59 mm x 102 mm` (`472 x 816`
//! dots) die-cut thermal labels on the attached BT320 Printer (`0fe6:811e`).

use std::fmt::Write as _;

use label_core::BoxLabelSpec;

use crate::error::RenderError;
use crate::qr::QrMatrix;

/// Printable canvas width in dots at 203 DPI (`59 mm * 8 dots/mm = 472`).
pub const BT320_WIDTH_DOTS: u32 = 472;

/// Printable canvas length in dots at 203 DPI (`102 mm * 8 dots/mm = 816`).
pub const BT320_HEIGHT_DOTS: u32 = 816;

/// Escapes double quotes and control characters for TSPL2 string literals.
fn escape_tspl_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\r' | '\n' | '\t' => out.push(' '),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            _ => out.push('-'),
        }
    }
    out
}

/// Truncates a string to at most `max_chars` ASCII characters for label fit.
fn truncate_str(input: &str, max_chars: usize) -> String {
    let escaped = escape_tspl_str(input);
    if escaped.len() <= max_chars {
        escaped
    } else {
        let mut s: String = escaped.chars().take(max_chars.saturating_sub(3)).collect();
        s.push_str("...");
        s
    }
}

/// Compiles a [`BoxLabelSpec`] into a `59 mm x 102 mm` TSPL2 command buffer.
///
/// # Errors
///
/// Returns [`RenderError`] if the QR payload fails validation.
pub fn compile_bt320_tspl(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError> {
    let qr_url = spec.qr_payload_url();
    let _ = QrMatrix::encode(&qr_url)?;

    let mut tspl = String::with_capacity(1024);
    tspl.push_str("SIZE 59 mm, 102 mm\r\n");
    tspl.push_str("GAP 3 mm, 0 mm\r\n");
    tspl.push_str("DIRECTION 1,0\r\n");
    tspl.push_str("REFERENCE 0,0\r\n");
    tspl.push_str("CLS\r\n");

    // Outer border and section divider bars within 472 x 816 dot canvas.
    tspl.push_str("BOX 16,16,456,796,3\r\n");
    tspl.push_str("BAR 16,184,440,3\r\n");
    tspl.push_str("BAR 16,248,440,2\r\n");

    // Hardware QR code at top-left (28, 28), ECC level M, cell width 5.
    let qr_payload = escape_tspl_str(&qr_url);
    let _ = write!(tspl, "QRCODE 28,28,M,5,A,0,M2,S7,\"{qr_payload}\"\r\n");

    // Box label ID, category icon badge, and container code to right of QR.
    let label_id = truncate_str(spec.label_id(), 14);
    let category = truncate_str(&format!("[{}]", spec.category_icon()), 18);
    let container_name = truncate_str(spec.location_path().container_code(), 22);
    let _ = write!(tspl, "TEXT 180,32,\"4\",0,1,1,\"{label_id}\"\r\n");
    let _ = write!(tspl, "TEXT 180,84,\"3\",0,1,1,\"{category}\"\r\n");
    let _ = write!(tspl, "TEXT 180,128,\"2\",0,1,1,\"{container_name}\"\r\n");

    // 6-level location breadcrumb band (y = 198..238).
    let breadcrumb = truncate_str(&spec.location_path().breadcrumb(), 34);
    let _ = write!(tspl, "TEXT 28,204,\"2\",0,1,1,\"{breadcrumb}\"\r\n");

    // Bulleted contents list along the 102 mm vertical body (y = 268..760).
    let mut y_dots: u32 = 268;
    for item in spec.items().iter().take(15) {
        if y_dots > 760 {
            break;
        }
        let line = truncate_str(&format!("* {item}"), 34);
        let _ = write!(tspl, "TEXT 28,{y_dots},\"2\",0,1,1,\"{line}\"\r\n");
        y_dots = y_dots.saturating_add(32);
    }

    tspl.push_str("PRINT 1,1\r\n");
    Ok(tspl.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use label_core::{LocationNode, LocationPath, TaxonomyLevel};

    fn sample_spec() -> BoxLabelSpec {
        let nodes = vec![
            LocationNode::new(1, "HQ", None, TaxonomyLevel::Building),
            LocationNode::new(2, "F1", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Lab", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "RackA", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Shelf2", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "BOX-042", Some(5), TaxonomyLevel::ContainerBox),
        ];
        let path = LocationPath::from_container_lineage(nodes).expect("valid path");
        BoxLabelSpec::new(
            "BOX-042",
            "https://inventory.molinadeveloping.com/boxes",
            "ELEC",
            path,
            vec![
                "SMA Male-Male x10".to_owned(),
                "BNC Tee \"50 Ohm\"".to_owned(),
            ],
        )
        .expect("valid spec")
    }

    #[test]
    fn test_compile_bt320_tspl_header_and_geometry() {
        let spec = sample_spec();
        let bytes = compile_bt320_tspl(&spec).expect("TSPL compile must succeed");
        let text = String::from_utf8(bytes).expect("valid ASCII/UTF-8 TSPL");

        assert!(text.starts_with("SIZE 59 mm, 102 mm\r\nGAP 3 mm, 0 mm\r\n"));
        assert!(text.contains("QRCODE 28,28,M,5,A,0,M2,S7,\""));
        assert!(text.contains("TEXT 180,32,\"4\",0,1,1,\"BOX-042\"\r\n"));
        assert!(text.contains("TEXT 180,84,\"3\",0,1,1,\"[ELEC]\"\r\n"));
        assert!(text.contains("BNC Tee \\\"50 Ohm\\\""));
        assert!(text.ends_with("PRINT 1,1\r\n"));
        assert_eq!(BT320_WIDTH_DOTS, 472);
        assert_eq!(BT320_HEIGHT_DOTS, 816);
    }
}
