//! Pure-Rust QR matrix encoding and SVG generation.

use crate::error::RenderError;
use qrcode::Color;
use qrcode::EcLevel;
use qrcode::QrCode;

/// Two-dimensional binary matrix representing an encoded QR code symbol.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    width: usize,
    modules: Vec<bool>,
}

impl QrMatrix {
    /// Encodes a text payload into a QR matrix using error correction level M.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::EmptyPayload`] if the trimmed payload is empty, or
    /// [`RenderError::QrEncoding`] if the payload exceeds QR capacity.
    pub fn encode(payload: &str) -> Result<Self, RenderError> {
        if payload.trim().is_empty() {
            return Err(RenderError::EmptyPayload);
        }

        let code =
            QrCode::with_error_correction_level(payload.as_bytes(), EcLevel::M).map_err(|err| {
                RenderError::QrEncoding {
                    message: err.to_string(),
                }
            })?;

        let width = code.width();
        let colors = code.to_colors();
        let mut modules = Vec::with_capacity(colors.len());
        for color in colors {
            modules.push(color == Color::Dark);
        }

        Ok(Self { width, modules })
    }

    /// Returns the module width (number of modules per side) of the QR matrix.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns whether the module at coordinate `(x, y)` is dark (black).
    ///
    /// Returns `false` if `(x, y)` is outside matrix bounds.
    #[must_use]
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.width {
            let idx = y * self.width + x;
            self.modules.get(idx).copied().unwrap_or(false)
        } else {
            false
        }
    }

    /// Renders the QR matrix as an SVG string with the specified pixel size.
    #[must_use]
    pub fn to_svg_string(&self, size_px: u32) -> String {
        let mut svg = String::with_capacity(self.width * self.width * 16 + 256);
        svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 "#);
        svg.push_str(&self.width.to_string());
        svg.push(' ');
        svg.push_str(&self.width.to_string());
        svg.push_str(r#"" width=""#);
        svg.push_str(&size_px.to_string());
        svg.push_str(r#"" height=""#);
        svg.push_str(&size_px.to_string());
        svg.push_str(r#"" shape-rendering="crispEdges">"#);
        svg.push_str(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##);

        for y in 0..self.width {
            for x in 0..self.width {
                if self.is_dark(x, y) {
                    svg.push_str(r#"<rect x=""#);
                    svg.push_str(&x.to_string());
                    svg.push_str(r#"" y=""#);
                    svg.push_str(&y.to_string());
                    svg.push_str(r##"" width="1" height="1" fill="#000000"/>"##);
                }
            }
        }

        svg.push_str("</svg>");
        svg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty_payload() {
        assert_eq!(QrMatrix::encode(""), Err(RenderError::EmptyPayload));
        assert_eq!(QrMatrix::encode("   \t\n"), Err(RenderError::EmptyPayload));
    }

    #[test]
    fn test_encode_valid_payload() {
        let matrix = QrMatrix::encode("BOX-042").expect("valid QR encode");
        assert!(matrix.width() > 0);
        // Finders at top-left (0,0) must be dark
        assert!(matrix.is_dark(0, 0));
        // Out of bounds must return false
        assert!(!matrix.is_dark(matrix.width() + 10, 0));
        assert!(!matrix.is_dark(0, matrix.width() + 10));
    }

    #[test]
    fn test_to_svg_string() {
        let matrix =
            QrMatrix::encode("https://snipeit.local/hardware/42").expect("valid QR encode");
        let svg = matrix.to_svg_string(210);
        assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg""#));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains(r#"width="210""#));
        assert!(svg.contains(r#"height="210""#));
        assert!(svg.contains(r##"fill="#000000""##));
    }
}
