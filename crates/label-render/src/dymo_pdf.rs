//! Pure-Rust PDF-1.4 compiler for Dymo 30256 Large Address labels.
//!
//! Generates vector PDF output matching standard 4.0" x 2.3098" thermal
//! geometry (`288.00` pt x `166.31` pt) with zero external C dependencies or
//! interpreters.

use crate::error::RenderError;
use crate::qr::QrMatrix;
use label_core::BoxLabelSpec;

/// Escapes special characters in PDF literal string tokens `(...)`.
#[must_use]
pub fn escape_pdf_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\r' | '\n' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

struct PdfBuilder {
    buffer: Vec<u8>,
    offsets: Vec<usize>,
}

impl PdfBuilder {
    fn new() -> Self {
        let mut buffer = Vec::with_capacity(4096);
        buffer.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");
        Self {
            buffer,
            offsets: Vec::new(),
        }
    }

    fn add_object(&mut self, content: &str) -> usize {
        let obj_num = self.offsets.len() + 1;
        self.offsets.push(self.buffer.len());
        self.buffer
            .extend_from_slice(format!("{obj_num} 0 obj\n{content}\nendobj\n").as_bytes());
        obj_num
    }

    fn finish(mut self) -> Vec<u8> {
        let num_objects = self.offsets.len();
        let xref_offset = self.buffer.len();

        self.buffer
            .extend_from_slice(format!("xref\n0 {}\n", num_objects + 1).as_bytes());
        self.buffer.extend_from_slice(b"0000000000 65535 f \n");

        for &offset in &self.offsets {
            self.buffer
                .extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }

        self.buffer.extend_from_slice(
            format!(
                "trailer\n<<\n  /Size {}\n  /Root 1 0 R\n>>\nstartxref\n{xref_offset}\n%%EOF\n",
                num_objects + 1
            )
            .as_bytes(),
        );

        self.buffer
    }
}

/// Compiles a [`BoxLabelSpec`] into a Dymo 30256 thermal label PDF-1.4
/// document.
///
/// Output dimensions are strictly `/MediaBox [0 0 288.00 166.31]`
/// (4.0" x 2.3098").
///
/// # Errors
///
/// Returns [`RenderError`] if QR code matrix encoding fails.
pub fn compile_dymo_pdf(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError> {
    let qr = QrMatrix::encode(&spec.qr_payload_url())?;

    let qr_size = 50.4_f64; // 0.70 inches * 72 pt/in
    let qr_x = 14.0_f64;
    let qr_y = 166.31 - 14.0 - qr_size; // ~101.91
    let width = qr.width();
    let module_size = if width > 0 {
        qr_size / width as f64
    } else {
        1.0
    };

    let mut stream = String::with_capacity(4096);

    // 1. Vector QR code modules
    stream.push_str("0 0 0 rg\n");
    for y in 0..width {
        for x in 0..width {
            if qr.is_dark(x, y) {
                let px = qr_x + (x as f64) * module_size;
                let py = qr_y + ((width - 1 - y) as f64) * module_size;
                stream.push_str(&format!(
                    "{:.2} {:.2} {:.2} {:.2} re f\n",
                    px, py, module_size, module_size
                ));
            }
        }
    }

    // 2. Header text: Label Identifier
    stream.push_str("BT\n");
    stream.push_str("/F1 16 Tf\n");
    stream.push_str("72.00 138.00 Td\n");
    stream.push_str(&format!("({}) Tj\n", escape_pdf_str(spec.label_id())));
    stream.push_str("ET\n");

    // 3. Location hierarchy breadcrumb
    stream.push_str("BT\n");
    stream.push_str("/F2 9 Tf\n");
    stream.push_str("72.00 120.00 Td\n");
    stream.push_str(&format!(
        "({}) Tj\n",
        escape_pdf_str(&spec.location_path().breadcrumb())
    ));
    stream.push_str("ET\n");

    // 4. Bulleted inventory items
    let mut item_y = 92.0_f64;
    let line_height = 12.0_f64;
    for item in spec.items() {
        if item_y < 12.0 {
            break;
        }
        stream.push_str("BT\n");
        stream.push_str("/F2 9 Tf\n");
        stream.push_str(&format!("14.00 {:.2} Td\n", item_y));
        let bullet_line = format!("• {}", item);
        stream.push_str(&format!("({}) Tj\n", escape_pdf_str(&bullet_line)));
        stream.push_str("ET\n");
        item_y -= line_height;
    }

    let mut doc = PdfBuilder::new();

    // 1: Catalog
    doc.add_object("<<\n  /Type /Catalog\n  /Pages 2 0 R\n>>");

    // 2: Pages
    doc.add_object("<<\n  /Type /Pages\n  /Kids [3 0 R]\n  /Count 1\n>>");

    // 3: Page (MediaBox [0 0 288.00 166.31])
    doc.add_object(concat!(
        "<<\n",
        "  /Type /Page\n",
        "  /Parent 2 0 R\n",
        "  /MediaBox [0 0 288.00 166.31]\n",
        "  /Contents 4 0 R\n",
        "  /Resources <<\n",
        "    /Font <<\n",
        "      /F1 5 0 R\n",
        "      /F2 6 0 R\n",
        "    >>\n",
        "  >>\n",
        ">>",
    ));

    // 4: Contents stream
    doc.add_object(&format!(
        "<<\n  /Length {}\n>>\nstream\n{}\nendstream",
        stream.len(),
        stream
    ));

    // 5: Font F1 (Helvetica-Bold)
    doc.add_object("<<\n  /Type /Font\n  /Subtype /Type1\n  /BaseFont /Helvetica-Bold\n>>");

    // 6: Font F2 (Helvetica)
    doc.add_object("<<\n  /Type /Font\n  /Subtype /Type1\n  /BaseFont /Helvetica\n>>");

    Ok(doc.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use label_core::{LocationNode, LocationPath, TaxonomyLevel};

    fn make_test_spec(label_id: &str, items: Vec<String>) -> BoxLabelSpec {
        let nodes = vec![
            LocationNode::new(1, "HQ", None, TaxonomyLevel::Building),
            LocationNode::new(2, "Floor 2", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Lab", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack A", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Shelf 1", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "Box 42", Some(5), TaxonomyLevel::ContainerBox),
        ];
        let path = LocationPath::from_container_lineage(nodes).expect("valid lineage in test");
        BoxLabelSpec::new(
            label_id,
            "https://inventory.local/boxes",
            "box-icon",
            path,
            items,
        )
        .expect("valid spec in test")
    }

    #[test]
    fn test_compile_dymo_pdf_structure() {
        let spec = make_test_spec("BOX-001", vec!["Cat6 Cable".into(), "Switch".into()]);
        let pdf_bytes = compile_dymo_pdf(&spec).expect("PDF compilation must succeed");

        let pdf_str = String::from_utf8_lossy(&pdf_bytes);

        // Required headers and geometry assertions
        assert!(pdf_str.starts_with("%PDF-1.4"));
        assert!(pdf_str.contains("/MediaBox [0 0 288.00 166.31]"));
        assert!(pdf_str.trim_end().ends_with("%%EOF"));

        // Content assertions
        assert!(pdf_str.contains("BOX-001"));
        assert!(pdf_str.contains("HQ > Floor 2 > Lab > Rack A > Shelf 1 > Box 42"));
        assert!(pdf_str.contains("• Cat6 Cable"));
        assert!(pdf_str.contains("• Switch"));
    }

    #[test]
    fn test_escape_pdf_str() {
        assert_eq!(escape_pdf_str("Normal Text"), "Normal Text");
        assert_eq!(
            escape_pdf_str("Text (with) parens \\ backslash"),
            "Text \\(with\\) parens \\\\ backslash"
        );
        assert_eq!(escape_pdf_str("Line1\nLine2\r"), "Line1 Line2 ");
    }
}
