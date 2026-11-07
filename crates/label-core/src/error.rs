//! Error types and result definitions for domain and taxonomy operations.

use thiserror::Error;

/// Core domain error enumeration for taxonomy, geometry, and labeling
/// operations.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// The specified taxonomy depth value is outside the valid range (1..=6).
    #[error("invalid taxonomy depth: {depth} (expected 1..=6)")]
    InvalidTaxonomyDepth {
        /// The depth value that was supplied.
        depth: u8,
    },

    /// The container node path depth does not match expected container depth.
    #[error("invalid container depth: expected {expected}, got {actual}")]
    InvalidContainerDepth {
        /// The expected container taxonomy depth.
        expected: usize,
        /// The actual taxonomy depth encountered.
        actual: usize,
    },

    /// The label identifier string was empty or contained only whitespace.
    #[error("label identifier cannot be empty")]
    EmptyLabelIdentifier,

    /// The inventory server base URL was empty or contained only whitespace.
    #[error("base URL cannot be empty")]
    EmptyBaseUrl,

    /// A geometric dimension was invalid or non-positive.
    #[error("invalid dimension for field: {field}")]
    InvalidDimension {
        /// Name of the geometric field that failed validation.
        field: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_display() {
        let err_depth = CoreError::InvalidTaxonomyDepth { depth: 7 };
        assert_eq!(
            err_depth.to_string(),
            "invalid taxonomy depth: 7 (expected 1..=6)"
        );

        let err_container = CoreError::InvalidContainerDepth {
            expected: 6,
            actual: 3,
        };
        assert_eq!(
            err_container.to_string(),
            "invalid container depth: expected 6, got 3"
        );

        let err_id = CoreError::EmptyLabelIdentifier;
        assert_eq!(err_id.to_string(), "label identifier cannot be empty");

        let err_url = CoreError::EmptyBaseUrl;
        assert_eq!(err_url.to_string(), "base URL cannot be empty");

        let err_dim = CoreError::InvalidDimension { field: "width_in" };
        assert_eq!(err_dim.to_string(), "invalid dimension for field: width_in");
    }

    #[test]
    fn test_core_error_clone_and_eq() {
        let err1 = CoreError::InvalidTaxonomyDepth { depth: 0 };
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
