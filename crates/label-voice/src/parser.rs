//! Deterministic natural-language voice command intent parser.
//!
//! Maps spoken text commands into strongly-typed [`VoiceCommandIntent`]
//! instances for container label printing, inventory queries, asset checkout,
//! and relocation.

use label_core::LabelFormat;
use label_core::VoiceCommandIntent;

use crate::error::VoiceError;

/// Strips leading and trailing punctuation while preserving dashes and
/// underscores.
fn clean_token(token: &str) -> &str {
    token.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-' && c != '_')
}

/// Tokenizes an input string into cleaned non-empty word tokens.
fn tokenize(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    for raw in input.split_whitespace() {
        let cleaned = clean_token(raw);
        if !cleaned.is_empty() {
            tokens.push(cleaned);
        }
    }
    tokens
}

/// Splits token slice on the first occurrence of a delimiter word
/// (case-insensitive).
fn split_on_delimiter<'a>(
    tokens: &'a [&'a str],
    delimiter: &str,
) -> Option<(&'a [&'a str], &'a [&'a str])> {
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case(delimiter) {
            return Some((&tokens[..idx], &tokens[idx + 1..]));
        }
    }
    None
}

/// Extracts a box identifier token from a slice of tokens.
fn extract_box_code(tokens: &[&str]) -> Option<String> {
    // 1. Look for explicit BOX-* prefix
    for &tok in tokens {
        if tok.to_ascii_uppercase().starts_with("BOX-") {
            return Some(tok.to_string());
        }
    }

    // 2. Look for token immediately following "for"
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case("for") {
            if let Some(&next) = tokens.get(idx + 1) {
                if next.eq_ignore_ascii_case("box") {
                    if let Some(&after_box) = tokens.get(idx + 2) {
                        return Some(after_box.to_string());
                    }
                } else if !next.eq_ignore_ascii_case("the") && !next.eq_ignore_ascii_case("a") {
                    return Some(next.to_string());
                }
            }
        }
    }

    // 3. Look for token immediately following "box" (if next is not "label")
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case("box") {
            if let Some(&next) = tokens.get(idx + 1) {
                if !next.eq_ignore_ascii_case("label")
                    && !next.eq_ignore_ascii_case("labels")
                    && !next.eq_ignore_ascii_case("the")
                {
                    return Some(next.to_string());
                }
            }
        }
    }

    // 4. Fallback: filter stop words and take first candidate
    for &tok in tokens {
        let lower = tok.to_ascii_lowercase();
        if !matches!(
            lower.as_str(),
            "print"
                | "box"
                | "label"
                | "labels"
                | "for"
                | "on"
                | "brother"
                | "ptouch"
                | "lbx"
                | "odt"
                | "boxlabels"
                | "dymo"
                | "pdf"
                | "the"
                | "a"
                | "an"
                | "what"
                | "is"
                | "inside"
                | "in"
                | "list"
                | "contents"
                | "of"
                | "query"
                | "show"
                | "move"
                | "relocate"
                | "check"
                | "out"
                | "checkout"
                | "assign"
                | "asset"
                | "to"
                | "into"
        ) {
            return Some(tok.to_string());
        }
    }

    None
}

/// Extracts an asset tag token from checkout command tokens.
fn extract_asset_tag(tokens: &[&str]) -> Option<String> {
    // 1. Look for ASSET-* or AST-* prefix
    for &tok in tokens {
        let upper = tok.to_ascii_uppercase();
        if upper.starts_with("ASSET-") || upper.starts_with("AST-") {
            return Some(tok.to_string());
        }
    }

    // 2. Look for token immediately following "asset"
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case("asset") {
            if let Some(&next) = tokens.get(idx + 1) {
                return Some(next.to_string());
            }
        }
    }

    // 3. Look for token immediately following "assign"
    for (idx, &tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case("assign") {
            if let Some(&next) = tokens.get(idx + 1) {
                if !next.eq_ignore_ascii_case("asset") {
                    return Some(next.to_string());
                }
            }
        }
    }

    // 4. Fallback: filter out common checkout keywords
    for &tok in tokens {
        let lower = tok.to_ascii_lowercase();
        if !matches!(
            lower.as_str(),
            "check" | "out" | "checkout" | "assign" | "asset" | "the" | "a" | "an" | "item"
        ) {
            return Some(tok.to_string());
        }
    }

    None
}

/// Extracts a target parent location code (e.g., "S2", "RACK-1").
fn extract_target_parent_code(tokens: &[&str]) -> Option<String> {
    for &tok in tokens {
        let lower = tok.to_ascii_lowercase();
        if !matches!(
            lower.as_str(),
            "to" | "into" | "shelf" | "rack" | "location" | "parent" | "box" | "the" | "a" | "an"
        ) {
            return Some(tok.to_string());
        }
    }
    None
}

/// Parses a spoken natural language command into a [`VoiceCommandIntent`].
///
/// Supports 4 core operational command types:
/// - Print Box Label (`print box label for BOX-042 [on brother/odt]`)
/// - Query Box Contents (`what is inside box BOX-042`,
///   `list contents of box BOX-042`)
/// - Check Out Asset (`check out asset ASSET-109 to box BOX-042`,
///   `assign ASSET-109 to BOX-042`)
/// - Relocate Box (`move box BOX-042 to S2`, `relocate box BOX-042 to RACK-1`)
///
/// # Errors
///
/// - [`VoiceError::EmptyUtterance`] if the provided text is blank.
/// - [`VoiceError::MissingIdentifier`] if required box code or asset tag is
///   missing.
/// - [`VoiceError::UnrecognizedIntent`] if no supported command pattern
///   matches.
pub fn parse_spoken_command(utterance: &str) -> Result<VoiceCommandIntent, VoiceError> {
    let trimmed = utterance.trim();
    if trimmed.is_empty() {
        return Err(VoiceError::EmptyUtterance);
    }

    let tokens = tokenize(trimmed);
    if tokens.is_empty() {
        return Err(VoiceError::EmptyUtterance);
    }

    let lower = trimmed.to_ascii_lowercase();

    // 1. Relocate Box: "move ... to ...", "relocate ... to ..."
    let is_relocate = (lower.starts_with("move")
        || lower.starts_with("relocate")
        || lower.contains(" move ")
        || lower.contains(" relocate "))
        && (lower.contains(" to ") || lower.contains(" into "));

    if is_relocate {
        let (before, after) = split_on_delimiter(&tokens, "to")
            .or_else(|| split_on_delimiter(&tokens, "into"))
            .ok_or(VoiceError::MissingIdentifier {
                context: "relocate command requires 'to' separator",
            })?;

        let box_code = extract_box_code(before).ok_or(VoiceError::MissingIdentifier {
            context: "relocate command requires source box code",
        })?;

        let target_parent_code =
            extract_target_parent_code(after).ok_or(VoiceError::MissingIdentifier {
                context: "relocate command requires target parent location code",
            })?;

        return Ok(VoiceCommandIntent::RelocateBox {
            box_code,
            target_parent_code,
        });
    }

    // 2. Checkout Asset: "check out ... to ...", "assign ... to ..."
    let is_checkout =
        (lower.contains("check out") || lower.contains("checkout") || lower.contains("assign"))
            && (lower.contains(" to ") || lower.contains(" into "));

    if is_checkout {
        let (before, after) = split_on_delimiter(&tokens, "to")
            .or_else(|| split_on_delimiter(&tokens, "into"))
            .ok_or(VoiceError::MissingIdentifier {
                context: "checkout command requires 'to' separator",
            })?;

        let asset_tag = extract_asset_tag(before).ok_or(VoiceError::MissingIdentifier {
            context: "checkout command requires asset tag identifier",
        })?;

        let box_code = extract_box_code(after).ok_or(VoiceError::MissingIdentifier {
            context: "checkout command requires destination box code",
        })?;

        return Ok(VoiceCommandIntent::CheckOutAssetToBox {
            asset_tag,
            box_code,
        });
    }

    // 3. Query Box Contents: "what is inside...", "contents of...", "query box..."
    let is_query = lower.contains("inside")
        || lower.contains("contents")
        || lower.contains("query")
        || lower.contains("what is in")
        || lower.contains("what's in");

    if is_query {
        let box_code = extract_box_code(&tokens).ok_or(VoiceError::MissingIdentifier {
            context: "query box contents command requires box code",
        })?;

        return Ok(VoiceCommandIntent::QueryBoxContents { box_code });
    }

    // 4. Print Box Label: "print ..."
    let is_print = lower.starts_with("print") || lower.contains(" print ");

    if is_print {
        let format =
            if lower.contains("brother") || lower.contains("ptouch") || lower.contains("lbx") {
                LabelFormat::BrotherPtouchLbx
            } else if lower.contains("odt") || lower.contains("boxlabels") {
                LabelFormat::BoxLabelsOdt
            } else {
                LabelFormat::Dymo30256Pdf
            };

        let box_code = extract_box_code(&tokens).ok_or(VoiceError::MissingIdentifier {
            context: "print box label command requires box code",
        })?;

        return Ok(VoiceCommandIntent::PrintBoxLabel { box_code, format });
    }

    Err(VoiceError::UnrecognizedIntent {
        utterance: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_whitespace_utterance() {
        assert_eq!(parse_spoken_command(""), Err(VoiceError::EmptyUtterance));
        assert_eq!(
            parse_spoken_command("   \t\n "),
            Err(VoiceError::EmptyUtterance)
        );
    }

    #[test]
    fn test_unrecognized_utterance() {
        let err = parse_spoken_command("turn on living room lights")
            .expect_err("should fail with unrecognized intent");
        assert_eq!(
            err,
            VoiceError::UnrecognizedIntent {
                utterance: "turn on living room lights".to_string()
            }
        );
    }

    #[test]
    fn test_print_box_label_formats() {
        // Default format: Dymo30256Pdf
        let dymo = parse_spoken_command("print box label for BOX-042")
            .expect("parsing dymo command should succeed");
        assert_eq!(
            dymo,
            VoiceCommandIntent::PrintBoxLabel {
                box_code: "BOX-042".to_string(),
                format: LabelFormat::Dymo30256Pdf,
            }
        );

        // Brother format
        let brother = parse_spoken_command("print label for BOX-042 on brother")
            .expect("parsing brother command should succeed");
        assert_eq!(
            brother,
            VoiceCommandIntent::PrintBoxLabel {
                box_code: "BOX-042".to_string(),
                format: LabelFormat::BrotherPtouchLbx,
            }
        );

        // ODT format
        let odt = parse_spoken_command("print odt label for BOX-042")
            .expect("parsing odt command should succeed");
        assert_eq!(
            odt,
            VoiceCommandIntent::PrintBoxLabel {
                box_code: "BOX-042".to_string(),
                format: LabelFormat::BoxLabelsOdt,
            }
        );
    }

    #[test]
    fn test_query_box_contents_intents() {
        let q1 = parse_spoken_command("what is inside box BOX-042")
            .expect("parsing query should succeed");
        assert_eq!(
            q1,
            VoiceCommandIntent::QueryBoxContents {
                box_code: "BOX-042".to_string()
            }
        );

        let q2 = parse_spoken_command("list contents of box BOX-042")
            .expect("parsing list contents should succeed");
        assert_eq!(
            q2,
            VoiceCommandIntent::QueryBoxContents {
                box_code: "BOX-042".to_string()
            }
        );

        let q3 =
            parse_spoken_command("query box BOX-042").expect("parsing query box should succeed");
        assert_eq!(
            q3,
            VoiceCommandIntent::QueryBoxContents {
                box_code: "BOX-042".to_string()
            }
        );
    }

    #[test]
    fn test_checkout_asset_intents() {
        let c1 = parse_spoken_command("check out asset ASSET-109 to box BOX-042")
            .expect("parsing checkout should succeed");
        assert_eq!(
            c1,
            VoiceCommandIntent::CheckOutAssetToBox {
                asset_tag: "ASSET-109".to_string(),
                box_code: "BOX-042".to_string(),
            }
        );

        let c2 = parse_spoken_command("assign ASSET-109 to BOX-042")
            .expect("parsing assign should succeed");
        assert_eq!(
            c2,
            VoiceCommandIntent::CheckOutAssetToBox {
                asset_tag: "ASSET-109".to_string(),
                box_code: "BOX-042".to_string(),
            }
        );
    }

    #[test]
    fn test_relocate_box_intents() {
        let r1 = parse_spoken_command("move box BOX-042 to S2")
            .expect("parsing move box should succeed");
        assert_eq!(
            r1,
            VoiceCommandIntent::RelocateBox {
                box_code: "BOX-042".to_string(),
                target_parent_code: "S2".to_string(),
            }
        );

        let r2 = parse_spoken_command("relocate box BOX-042 to RACK-1")
            .expect("parsing relocate should succeed");
        assert_eq!(
            r2,
            VoiceCommandIntent::RelocateBox {
                box_code: "BOX-042".to_string(),
                target_parent_code: "RACK-1".to_string(),
            }
        );
    }

    #[test]
    fn test_missing_identifier_errors() {
        let err_print = parse_spoken_command("print box label")
            .expect_err("print without box code should fail");
        assert!(matches!(err_print, VoiceError::MissingIdentifier { .. }));

        let err_relocate =
            parse_spoken_command("move to S2").expect_err("move without box code should fail");
        assert!(matches!(err_relocate, VoiceError::MissingIdentifier { .. }));
    }
}
