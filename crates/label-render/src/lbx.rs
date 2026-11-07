//! Brother P-Touch `.lbx` label template compiler.
//!
//! Produces a Brother P-Touch compatible `.lbx` archive containing
//! `prop.xml`, `label.xml`, and the 1-bit monochrome raster `Object0.bmp`.

use crate::bmp::encode_monochrome_bmp_192;
use crate::error::RenderError;
use crate::odt::escape_xml;
use crate::zip_store::ZipStoreBuilder;
use label_core::BoxLabelSpec;

/// Compiles a [`BoxLabelSpec`] into a Brother P-Touch `.lbx` zip archive.
///
/// The resulting archive contains:
/// - `prop.xml`: Brother P-Touch printer and label layout metadata.
/// - `label.xml`: XML structure with label identifier, location hierarchy
///   breadcrumb, items list, and reference to embedded bitmap object.
/// - `Object0.bmp`: 192x192 1-bit monochrome uncompressed Windows BMP.
///
/// # Errors
///
/// Returns [`RenderError`] if QR encoding or BMP generation fails.
pub fn compile_lbx_label(spec: &BoxLabelSpec) -> Result<Vec<u8>, RenderError> {
    let bmp_bytes = encode_monochrome_bmp_192(&spec.qr_payload_url())?;

    let prop_xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
        <Property>\n  \
          <LBXVersion>1.0</LBXVersion>\n  \
          <DeviceName>Brother P-Touch</DeviceName>\n  \
          <FormatVersion>1.0</FormatVersion>\n  \
          <LabelSize width=\"102\" height=\"58\" unit=\"mm\" />\n\
        </Property>\n";

    let escaped_id = escape_xml(spec.label_id());
    let escaped_breadcrumb = escape_xml(&spec.location_path().breadcrumb());

    let mut items_xml = String::new();
    for item in spec.items() {
        items_xml.push_str(&format!("    <Item>{}</Item>\n", escape_xml(item)));
    }

    let label_xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
        <Label>\n  \
          <Header>\n    \
            <Title>{escaped_id}</Title>\n    \
            <Location>{escaped_breadcrumb}</Location>\n  \
          </Header>\n  \
          <Objects>\n    \
            <Object name=\"Object0.bmp\" type=\"Bitmap\" width=\"192\" height=\"192\" />\n  \
          </Objects>\n  \
          <Items>\n{items_xml}  </Items>\n\
        </Label>\n"
    );

    let mut zip = ZipStoreBuilder::new();
    zip.add_file("prop.xml", prop_xml.as_bytes());
    zip.add_file("label.xml", label_xml.as_bytes());
    zip.add_file("Object0.bmp", &bmp_bytes);

    Ok(zip.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use label_core::{LocationNode, LocationPath, TaxonomyLevel};

    fn make_test_spec(label_id: &str, items: Vec<String>) -> BoxLabelSpec {
        let nodes = vec![
            LocationNode::new(1, "HQ & Main", None, TaxonomyLevel::Building),
            LocationNode::new(2, "Floor 2 <East>", Some(1), TaxonomyLevel::Floor),
            LocationNode::new(3, "Server \"Room\"", Some(2), TaxonomyLevel::Room),
            LocationNode::new(4, "Rack 'A'", Some(3), TaxonomyLevel::Furniture),
            LocationNode::new(5, "Shelf 3", Some(4), TaxonomyLevel::SubDivision),
            LocationNode::new(6, "Box 101", Some(5), TaxonomyLevel::ContainerBox),
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
    fn test_compile_lbx_label_header_and_entries() {
        let spec = make_test_spec(
            "BOX-042",
            vec!["Console Cable".into(), "SFP+ 10G Transceiver".into()],
        );
        let lbx_bytes = compile_lbx_label(&spec).expect("LBX compilation must succeed");

        // Must start with PK\x03\x04 zip header
        assert!(lbx_bytes.len() > 30);
        assert_eq!(&lbx_bytes[0..4], b"PK\x03\x04");

        // Must contain entries
        let archive_str = String::from_utf8_lossy(&lbx_bytes);
        assert!(archive_str.contains("prop.xml"));
        assert!(archive_str.contains("label.xml"));
        assert!(archive_str.contains("Object0.bmp"));
    }

    #[test]
    fn test_compile_lbx_label_xml_escaping() {
        let spec = make_test_spec(
            "BOX-<SPECIAL>&'042'\"",
            vec!["AT&T <Cat6> \"Patch\" 'Cable'".into()],
        );
        let lbx_bytes = compile_lbx_label(&spec).expect("LBX compilation must succeed");
        let archive_str = String::from_utf8_lossy(&lbx_bytes);

        // Raw unescaped dangerous tags must not appear in XML content
        assert!(archive_str.contains("BOX-&lt;SPECIAL&gt;&amp;&apos;042&apos;&quot;"));
        assert!(archive_str.contains("AT&amp;T &lt;Cat6&gt; &quot;Patch&quot; &apos;Cable&apos;"));
        assert!(archive_str.contains("HQ &amp; Main &gt; Floor 2 &lt;East&gt;"));
    }
}
