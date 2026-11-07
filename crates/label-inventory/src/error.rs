//! Error types and result definitions for inventory repository and Snipe-IT
//! client operations.

use thiserror::Error;

/// Inventory error enumeration for Snipe-IT REST client, location resolution,
/// and asset checkout operations.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InventoryError {
    /// The base URL was empty or contained only whitespace.
    #[error("inventory base URL cannot be empty")]
    EmptyBaseUrl,

    /// The API token was empty or contained only whitespace.
    #[error("inventory API token cannot be empty")]
    EmptyApiToken,

    /// The remote server rejected the request with HTTP 401 Unauthorized or
    /// HTTP 403 Forbidden.
    #[error("unauthorized access to inventory system")]
    Unauthorized,

    /// The remote server rejected the request due to HTTP 429 rate limiting.
    #[error("inventory API rate limit exceeded")]
    RateLimited,

    /// The requested inventory resource could not be found.
    #[error("inventory resource not found: {resource}")]
    NotFound {
        /// The identifier or description of the missing resource.
        resource: String,
    },

    /// An HTTP transport error occurred while contacting the inventory system.
    #[error("HTTP transport failure: {message}")]
    HttpTransport {
        /// Description of the HTTP transport failure.
        message: String,
    },

    /// Failed to deserialize a response from the inventory system.
    #[error("inventory payload deserialization error: {message}")]
    Deserialization {
        /// Description of the deserialization failure.
        message: String,
    },

    /// An underlying core domain error occurred.
    #[error(transparent)]
    Core(#[from] label_core::CoreError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_error_display() {
        let err_url = InventoryError::EmptyBaseUrl;
        assert_eq!(err_url.to_string(), "inventory base URL cannot be empty");

        let err_token = InventoryError::EmptyApiToken;
        assert_eq!(err_token.to_string(), "inventory API token cannot be empty");

        let err_unauth = InventoryError::Unauthorized;
        assert_eq!(
            err_unauth.to_string(),
            "unauthorized access to inventory system"
        );

        let err_rate = InventoryError::RateLimited;
        assert_eq!(err_rate.to_string(), "inventory API rate limit exceeded");

        let err_not_found = InventoryError::NotFound {
            resource: "location 42".to_owned(),
        };
        assert_eq!(
            err_not_found.to_string(),
            "inventory resource not found: location 42"
        );

        let err_transport = InventoryError::HttpTransport {
            message: "connection timeout".to_owned(),
        };
        assert_eq!(
            err_transport.to_string(),
            "HTTP transport failure: connection timeout"
        );

        let err_deser = InventoryError::Deserialization {
            message: "invalid JSON syntax".to_owned(),
        };
        assert_eq!(
            err_deser.to_string(),
            "inventory payload deserialization error: invalid JSON syntax"
        );

        let core_err = label_core::CoreError::EmptyLabelIdentifier;
        let err_core = InventoryError::Core(core_err);
        assert_eq!(err_core.to_string(), "label identifier cannot be empty");
    }

    #[test]
    fn test_inventory_error_from_core() {
        let core_err = label_core::CoreError::InvalidTaxonomyDepth { depth: 8 };
        let inv_err: InventoryError = core_err.into();
        assert_eq!(
            inv_err,
            InventoryError::Core(label_core::CoreError::InvalidTaxonomyDepth { depth: 8 })
        );
    }

    #[test]
    fn test_inventory_error_clone_and_eq() {
        let err1 = InventoryError::NotFound {
            resource: "asset ASSET-109".to_owned(),
        };
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
