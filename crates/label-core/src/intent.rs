//! Voice command intent definitions and target label formats.
//!
//! Models parsed user voice intents, operational print execution modes, and
//! supported target physical label formats across Dymo, ODF/ODT, and Brother.

use serde::{Deserialize, Serialize};

/// Target output physical format and document layout for label generation.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelFormat {
    /// Dymo `LabelWriter` 450 thermal label (30256, 4.0" x 2.3098")
    /// rendered as PDF.
    Dymo30256Pdf,
    /// `BoxLabels` `OpenDocument` Text format (.odt) for LibreOffice/ODF
    /// printing.
    BoxLabelsOdt,
    /// Brother P-Touch template format (.lbx) with 1-bit monochrome raster.
    BrotherPtouchLbx,
}

/// Print execution mode governing physical output dispatch.
///
/// Distinguishes live printer queue spooling from dry-run preview simulation,
/// avoiding raw boolean parameters per Google Rust Style guidelines.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrintMode {
    /// Dispatch rendered document directly to the CUPS spool or raw socket.
    ExecutePrint,
    /// Perform full pipeline resolution and rendering without sending to
    /// printer.
    DryRun,
}

/// Strongly-typed structured domain intent parsed from voice input or API
/// requests.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceCommandIntent {
    /// Render and optionally print a physical label for a container box.
    PrintBoxLabel {
        /// Unique container box location code (e.g., "BOX-001-Wood").
        box_code: String,
        /// Target layout and format to render.
        format: LabelFormat,
    },
    /// Query the inventory server for items and assets currently stored in a
    /// box.
    QueryBoxContents {
        /// Unique container box location code.
        box_code: String,
    },
    /// Associate and check out an asset tag into a designated container box.
    CheckOutAssetToBox {
        /// Asset tracking identifier (e.g., "AST-0042").
        asset_tag: String,
        /// Target container box code receiving the asset.
        box_code: String,
    },
    /// Move a container box to a new parent location in the taxonomy hierarchy.
    RelocateBox {
        /// Code of the container box being relocated.
        box_code: String,
        /// Code of the target parent subdivision or furniture location.
        target_parent_code: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_format_derives_and_variants() {
        let fmt = LabelFormat::Dymo30256Pdf;
        let cloned = fmt;
        assert_eq!(fmt, cloned);
        assert_ne!(fmt, LabelFormat::BoxLabelsOdt);
        assert_ne!(fmt, LabelFormat::BrotherPtouchLbx);
    }

    #[test]
    fn test_print_mode_derives_and_variants() {
        let mode = PrintMode::ExecutePrint;
        let copied = mode;
        assert_eq!(mode, copied);
        assert_ne!(mode, PrintMode::DryRun);
    }

    #[test]
    fn test_voice_command_intent_pattern_matching() {
        let intent = VoiceCommandIntent::PrintBoxLabel {
            box_code: "BOX-101".to_string(),
            format: LabelFormat::Dymo30256Pdf,
        };

        match intent {
            VoiceCommandIntent::PrintBoxLabel { box_code, format } => {
                assert_eq!(box_code, "BOX-101");
                assert_eq!(format, LabelFormat::Dymo30256Pdf);
            }
            _ => panic!("unexpected intent variant matched"),
        }

        let checkout = VoiceCommandIntent::CheckOutAssetToBox {
            asset_tag: "AST-99".to_string(),
            box_code: "BOX-101".to_string(),
        };

        if let VoiceCommandIntent::CheckOutAssetToBox {
            asset_tag,
            box_code,
        } = checkout
        {
            assert_eq!(asset_tag, "AST-99");
            assert_eq!(box_code, "BOX-101");
        } else {
            panic!("unexpected intent variant");
        }
    }

    #[test]
    fn test_voice_command_intent_query_and_relocate() {
        let query = VoiceCommandIntent::QueryBoxContents {
            box_code: "BOX-202".to_string(),
        };
        if let VoiceCommandIntent::QueryBoxContents { box_code } = query {
            assert_eq!(box_code, "BOX-202");
        } else {
            panic!("unexpected intent variant");
        }

        let relocate = VoiceCommandIntent::RelocateBox {
            box_code: "BOX-202".to_string(),
            target_parent_code: "SHELF-3".to_string(),
        };
        if let VoiceCommandIntent::RelocateBox {
            box_code,
            target_parent_code,
        } = relocate
        {
            assert_eq!(box_code, "BOX-202");
            assert_eq!(target_parent_code, "SHELF-3");
        } else {
            panic!("unexpected intent variant");
        }
    }
}
