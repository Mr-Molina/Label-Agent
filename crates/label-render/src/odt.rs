//! `OpenDocument` Text (`.odt`) compiler for BoxLabels-compatible archives.
//!
//! Assembles an uncompressed ODF 1.2 zip container with embedded styles,
//! XML content, SVG QR matrix, and ODF manifest.

use crate::error::RenderError;
use crate::qr::QrMatrix;
use crate::zip_store::ZipStoreBuilder;
use label_core::BoxLabelSpec;

/// Escapes XML special characters in text strings.
///
/// Replaces `&`, `<`, `>`, `"`, and `'` with their respective XML entity
/// references (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).
#[must_use]
pub fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

fn generate_styles_xml() -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<office:document-styles\n");
    s.push_str(" xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\"\n");
    s.push_str(" xmlns:style=\"urn:oasis:names:tc:opendocument:xmlns:style:1.0\"\n");
    s.push_str(" xmlns:fo=\"urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0\"\n");
    s.push_str(" office:version=\"1.2\">\n");
    s.push_str("  <office:master-styles>\n");
    s.push_str(
        "    <style:master-page style:name=\"Standard\" style:page-layout-name=\"Mpm1\"/>\n",
    );
    s.push_str("  </office:master-styles>\n");
    s.push_str("  <office:automatic-styles>\n");
    s.push_str("    <style:page-layout style:name=\"Mpm1\">\n");
    s.push_str("      <style:page-layout-properties\n");
    s.push_str("       fo:page-width=\"4in\"\n");
    s.push_str("       fo:page-height=\"2.3098in\"\n");
    s.push_str("       fo:margin-top=\"0.1701in\"\n");
    s.push_str("       fo:margin-bottom=\"0.1701in\"\n");
    s.push_str("       fo:margin-left=\"0.1701in\"\n");
    s.push_str("       fo:margin-right=\"0.1701in\"\n");
    s.push_str("       style:print-orientation=\"landscape\"/>\n");
    s.push_str("    </style:page-layout>\n");
    s.push_str("  </office:automatic-styles>\n");
    s.push_str("</office:document-styles>");
    s
}

fn generate_content_xml(spec: &BoxLabelSpec) -> String {
    let escaped_label_id = escape_xml(spec.label_id());
    let escaped_breadcrumb = escape_xml(&spec.location_path().breadcrumb());
    let escaped_icon = escape_xml(spec.category_icon());

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<office:document-content\n");
    s.push_str(" xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\"\n");
    s.push_str(" xmlns:style=\"urn:oasis:names:tc:opendocument:xmlns:style:1.0\"\n");
    s.push_str(" xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\"\n");
    s.push_str(" xmlns:table=\"urn:oasis:names:tc:opendocument:xmlns:table:1.0\"\n");
    s.push_str(" xmlns:draw=\"urn:oasis:names:tc:opendocument:xmlns:drawing:1.0\"\n");
    s.push_str(" xmlns:fo=\"urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0\"\n");
    s.push_str(" xmlns:xlink=\"http://www.w3.org/1999/xlink\"\n");
    s.push_str(" xmlns:svg=\"urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0\"\n");
    s.push_str(" office:version=\"1.2\">\n");
    s.push_str("  <office:body>\n");
    s.push_str("    <office:text>\n");
    s.push_str("      <text:p text:style-name=\"Title\">\n");
    if !escaped_icon.is_empty() {
        s.push_str("        <text:span text:style-name=\"CategoryIcon\">");
        s.push_str(&escaped_icon);
        s.push_str("</text:span> ");
    }
    s.push_str("        <text:span text:style-name=\"LabelId\">");
    s.push_str(&escaped_label_id);
    s.push_str("</text:span>\n");
    s.push_str("      </text:p>\n");
    s.push_str("      <text:p text:style-name=\"Breadcrumb\">");
    s.push_str(&escaped_breadcrumb);
    s.push_str("</text:p>\n");
    s.push_str("      <text:p text:style-name=\"QrCode\">\n");
    s.push_str("        <draw:frame draw:name=\"QR\" ");
    s.push_str("svg:width=\"0.7in\" svg:height=\"0.7in\" ");
    s.push_str("text:anchor-type=\"paragraph\">\n");
    s.push_str("          <draw:image xlink:href=\"Pictures/qr.svg\" ");
    s.push_str("xlink:type=\"simple\" xlink:show=\"embed\" xlink:actuate=\"onLoad\"/>\n");
    s.push_str("        </draw:frame>\n");
    s.push_str("      </text:p>\n");

    for item in spec.items() {
        let escaped_item = escape_xml(item);
        s.push_str("      <text:p text:style-name=\"Item\">\u{2022} ");
        s.push_str(&escaped_item);
        s.push_str("</text:p>\n");
    }

    s.push_str("    </office:text>\n");
    s.push_str("  </office:body>\n");
    s.push_str("</office:document-content>");
    s
}

fn generate_manifest_xml() -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str("<manifest:manifest\n");
    s.push_str(" xmlns:manifest=\"urn:oasis:names:tc:opendocument:xmlns:manifest:1.0\"\n");
    s.push_str(" manifest:version=\"1.2\">\n");
    s.push_str("  <manifest:file-entry manifest:full-path=\"/\"\n");
    s.push_str("   manifest:media-type=\"application/vnd.oasis.opendocument.text\"/>\n");
    s.push_str("  <manifest:file-entry manifest:full-path=\"styles.xml\"\n");
    s.push_str("   manifest:media-type=\"text/xml\"/>\n");
    s.push_str("  <manifest:file-entry manifest:full-path=\"content.xml\"\n");
    s.push_str("   manifest:media-type=\"text/xml\"/>\n");
    s.push_str("  <manifest:file-entry manifest:full-path=\"Pictures/qr.svg\"\n");
    s.push_str("   manifest:media-type=\"image/svg+xml\"/>\n");
    s.push_str("</manifest:manifest>");
    s
}

/// Compiles a container box label specification into a BoxLabels-compatible
/// `OpenDocument` Text (`.odt`) uncompressed ZIP archive.
///
/// # Errors
///
/// Returns [`RenderError`] if encoding the QR payload into an SVG fails.
pub fn compile_odt_label(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError> {
    let qr_svg = QrMatrix::encode(&spec.qr_payload_url())?.to_svg_string(210);

    let styles_xml = generate_styles_xml();
    let content_xml = generate_content_xml(spec);
    let manifest_xml = generate_manifest_xml();

    let mut zip = ZipStoreBuilder::new();
    zip.add_file("mimetype", b"application/vnd.oasis.opendocument.text");
    zip.add_file("styles.xml", styles_xml.as_bytes());
    zip.add_file("content.xml", content_xml.as_bytes());
    zip.add_file("Pictures/qr.svg", qr_svg.as_bytes());
    zip.add_file("META-INF/manifest.xml", manifest_xml.as_bytes());

    Ok(zip.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use label_core::LocationNode;
    use label_core::LocationPath;
    use label_core::TaxonomyLevel;

    fn sample_spec() -> BoxLabelSpec {
        let nodes = vec![
            LocationNode::new(1, "Main Building", None, TaxonomyLevel::Building),
            LocationNode::new(2, "Floor 2", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Hardware Lab", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack Alpha", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Bin Array 4", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "BOX-042", Some(5), TaxonomyLevel::ContainerBox),
        ];
        let path =
            LocationPath::from_container_lineage(nodes).expect("sample lineage must be valid");
        let items = vec![
            "AT&T <Cat6> & 'Fiber'".to_string(),
            "Fluke Tester".to_string(),
        ];
        BoxLabelSpec::new(
            "BOX-042",
            "https://inventory.example.com",
            "[BOX]",
            path,
            items,
        )
        .expect("box label spec must be valid")
    }

    #[test]
    fn test_escape_xml_entities() {
        let raw = "AT&T <Cat6> & 'Fiber' \"Quote\"";
        let escaped = escape_xml(raw);
        assert_eq!(
            escaped,
            "AT&amp;T &lt;Cat6&gt; &amp; &apos;Fiber&apos; &quot;Quote&quot;"
        );
    }

    #[test]
    fn test_compile_odt_label_mimetype_offset() {
        let spec = sample_spec();
        let archive = compile_odt_label(&spec).expect("ODT label compilation must succeed");

        // Per ODF 1.2, mimetype is stored uncompressed at offset 30
        assert_eq!(&archive[30..38], b"mimetype");
        assert_eq!(&archive[38..77], b"application/vnd.oasis.opendocument.text");
        assert!(archive[30..].starts_with(b"mimetypeapplication/vnd.oasis.opendocument.text"));
    }

    #[test]
    fn test_compile_odt_label_xml_escaping() {
        let spec = sample_spec();
        let archive = compile_odt_label(&spec).expect("ODT label compilation must succeed");

        let content_str = String::from_utf8_lossy(&archive);
        assert!(content_str.contains("AT&amp;T &lt;Cat6&gt; &amp; &apos;Fiber&apos;"));
        assert!(content_str.contains("BOX-042"));
        assert!(content_str.contains("Pictures/qr.svg"));
        assert!(content_str.contains("fo:page-width=\"4in\""));
        assert!(content_str.contains("fo:page-height=\"2.3098in\""));
        assert!(content_str.contains("fo:margin-top=\"0.1701in\""));
        assert!(content_str.contains("fo:margin-bottom=\"0.1701in\""));
        assert!(content_str.contains("fo:margin-left=\"0.1701in\""));
        assert!(content_str.contains("fo:margin-right=\"0.1701in\""));
        assert!(content_str.contains("\u{2022}"));
    }
}
