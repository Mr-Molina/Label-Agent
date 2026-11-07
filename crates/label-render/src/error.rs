//! Error types for QR code generation, label compilation, and spooling.

use std::io;
use thiserror::Error;

/// Errors that can occur during label rendering, document compilation,
/// or spool operations.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// The supplied payload string was empty or contained only whitespace.
    #[error("payload cannot be empty")]
    EmptyPayload,

    /// An error occurred during QR code matrix encoding.
    #[error("QR code encoding failed: {message}")]
    QrEncoding {
        /// Descriptive details of the QR encoding failure.
        message: String,
    },

    /// The label identifier or file stem is invalid for spool file creation.
    #[error("invalid spool identifier")]
    InvalidSpoolIdentifier,

    /// An I/O error occurred while writing or reading spool artifacts.
    #[error("I/O error during render/spool operation: {message}")]
    Io {
        /// Descriptive details of the I/O failure.
        message: String,
    },
}

impl From<io::Error> for RenderError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_error_display() {
        let err_empty = RenderError::EmptyPayload;
        assert_eq!(err_empty.to_string(), "payload cannot be empty");

        let err_qr = RenderError::QrEncoding {
            message: "data too large".to_string(),
        };
        assert_eq!(
            err_qr.to_string(),
            "QR code encoding failed: data too large"
        );

        let err_spool = RenderError::InvalidSpoolIdentifier;
        assert_eq!(err_spool.to_string(), "invalid spool identifier");

        let err_io = RenderError::Io {
            message: "disk full".to_string(),
        };
        assert_eq!(
            err_io.to_string(),
            "I/O error during render/spool operation: disk full"
        );
    }

    #[test]
    fn test_render_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let render_err = RenderError::from(io_err);
        assert_eq!(
            render_err,
            RenderError::Io {
                message: "file not found".to_string(),
            }
        );
    }

    #[test]
    fn test_render_error_clone_and_eq() {
        let err1 = RenderError::EmptyPayload;
        let err2 = err1.clone();
        assert_eq!(err1, err2);

        let err3 = RenderError::QrEncoding {
            message: "overflow".to_string(),
        };
        assert_ne!(err1, err3);
    }
}
