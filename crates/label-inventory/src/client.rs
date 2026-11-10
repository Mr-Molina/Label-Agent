//! Snipe-IT REST API client and repository abstraction.

use crate::error::InventoryError;
use crate::models::AssetDto;
use crate::models::LocationDto;
use crate::models::PaginatedResponse;
use label_core::LocationNode;
use label_core::LocationPath;
use label_core::TaxonomyLevel;
use serde::Deserialize;
use std::time::Duration;

/// Canonical default inventory backend URL for Snipe-IT.
pub const DEFAULT_INVENTORY_URL: &str = "https://inventory.molinadeveloping.com";

/// Default HTTP request timeout for Snipe-IT API operations (10 seconds).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Internal helper for deserializing Snipe-IT direct or wrapped payloads.
#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Wrapped { payload: T },
    Direct(T),
}

/// Configuration options for connecting to the Snipe-IT inventory service.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryConfig {
    base_url: String,
    api_token: String,
    timeout: Duration,
}

impl InventoryConfig {
    /// Constructs a validated inventory client configuration.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError::EmptyBaseUrl`] if `base_url` is whitespace.
    /// Returns [`InventoryError::EmptyApiToken`] if `api_token` is whitespace.
    pub fn new(base_url: &str, api_token: &str, timeout: Duration) -> Result<Self, InventoryError> {
        let trimmed_url = base_url.trim().trim_end_matches('/');
        if trimmed_url.is_empty() {
            return Err(InventoryError::EmptyBaseUrl);
        }

        let trimmed_token = api_token.trim();
        if trimmed_token.is_empty() {
            return Err(InventoryError::EmptyApiToken);
        }

        Ok(Self {
            base_url: trimmed_url.to_owned(),
            api_token: trimmed_token.to_owned(),
            timeout,
        })
    }

    /// Constructs a configuration pointing to the canonical inventory backend.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError::EmptyApiToken`] if `api_token` is whitespace.
    pub fn with_default_url(api_token: impl Into<String>) -> Result<Self, InventoryError> {
        let token = api_token.into();
        Self::new(DEFAULT_INVENTORY_URL, &token, DEFAULT_TIMEOUT)
    }

    /// Returns the normalized base URL without a trailing slash.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the API bearer token.
    #[must_use]
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    /// Returns the configured request timeout duration.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Formats the standard HTTP `Authorization: Bearer <token>` header value.
    #[must_use]
    pub fn bearer_header(&self) -> String {
        format!("Bearer {}", self.api_token)
    }

    /// Resolves an API endpoint subpath against the normalized base URL.
    #[must_use]
    pub fn endpoint_url(&self, path: &str) -> String {
        let trimmed = path.trim().strip_prefix('/').unwrap_or(path.trim());
        if trimmed.is_empty() {
            self.base_url.clone()
        } else if self.base_url.ends_with("/api/v1") {
            let api_trimmed = trimmed.strip_prefix("api/v1/").unwrap_or(trimmed);
            format!("{}/{}", self.base_url, api_trimmed)
        } else if trimmed.starts_with("api/v1/") || trimmed == "api/v1" {
            format!("{}/{}", self.base_url, trimmed)
        } else {
            format!("{}/api/v1/{}", self.base_url, trimmed)
        }
    }
}

/// Abstract contract for location lookup, lineage resolution, and asset ops.
pub trait InventoryRepository {
    /// Finds an inventory location by its exact name or code identifier.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] on transport, parsing, or not-found errors.
    fn find_location_by_code(&self, code: &str) -> Result<LocationDto, InventoryError>;

    /// Resolves the 6-level taxonomy path from Building down to the container.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] if the hierarchy is invalid or missing.
    fn resolve_container_lineage(
        &self,
        container_code: &str,
    ) -> Result<LocationPath, InventoryError>;

    /// Lists all assets currently stored within the designated location.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] on network or deserialization failures.
    fn list_assets_in_location(&self, location_id: u64) -> Result<Vec<AssetDto>, InventoryError>;

    /// Checks out an asset identified by its tag to the target location.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] if the checkout operation fails.
    fn checkout_asset_to_location(
        &self,
        asset_tag: &str,
        location_id: u64,
    ) -> Result<(), InventoryError>;

    /// Relocates a container to a new parent location within the hierarchy.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] if the relocation operation fails.
    fn relocate_container(
        &self,
        container_id: u64,
        new_parent_id: u64,
    ) -> Result<(), InventoryError>;
}

/// Concrete REST API client communicating with a Snipe-IT inventory backend.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SnipeItClient {
    config: InventoryConfig,
    agent: ureq::Agent,
}

impl SnipeItClient {
    /// Creates a new Snipe-IT REST client with the specified configuration.
    #[must_use]
    pub fn new(config: InventoryConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(config.timeout())
            .build();
        Self { config, agent }
    }

    /// Returns a reference to the active inventory client configuration.
    #[must_use]
    pub fn config(&self) -> &InventoryConfig {
        &self.config
    }

    /// Maps HTTP response status codes to strongly typed [`InventoryError`]s.
    #[must_use]
    pub fn map_http_status(status: u16, resource: &str) -> InventoryError {
        match status {
            401 | 403 => InventoryError::Unauthorized,
            404 => InventoryError::NotFound {
                resource: resource.to_owned(),
            },
            429 => InventoryError::RateLimited,
            code => InventoryError::HttpTransport {
                message: format!("HTTP status {code} for {resource}"),
            },
        }
    }

    /// Fetches a location record by its primary numeric identifier.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] on network, HTTP error, or parsing failures.
    pub fn find_location_by_id(&self, id: u64) -> Result<LocationDto, InventoryError> {
        let url = self.config.endpoint_url(&format!("locations/{id}"));
        let res = self
            .agent
            .get(&url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .call();

        match res {
            Ok(resp) => {
                resp.into_json::<LocationDto>()
                    .map_err(|err| InventoryError::Deserialization {
                        message: err.to_string(),
                    })
            }
            Err(ureq::Error::Status(status, _)) => {
                Err(Self::map_http_status(status, &format!("locations/{id}")))
            }
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }

    /// Relocates a location record to a new parent location in the hierarchy.
    ///
    /// # Errors
    ///
    /// Returns [`InventoryError`] on transport, status, or parsing failures.
    pub fn relocate_location(
        &self,
        location_id: u64,
        new_parent_id: u64,
    ) -> Result<LocationDto, InventoryError> {
        let url = self
            .config
            .endpoint_url(&format!("api/v1/locations/{location_id}"));
        let payload = serde_json::json!({
            "parent_id": new_parent_id,
        });

        let res = self
            .agent
            .patch(&url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .set("Content-Type", "application/json")
            .send_json(payload);

        match res {
            Ok(resp) => {
                let parsed: ApiResponse<LocationDto> =
                    resp.into_json()
                        .map_err(|err| InventoryError::Deserialization {
                            message: err.to_string(),
                        })?;
                Ok(match parsed {
                    ApiResponse::Wrapped { payload } => payload,
                    ApiResponse::Direct(dto) => dto,
                })
            }
            Err(ureq::Error::Status(status, _)) => Err(Self::map_http_status(
                status,
                &format!("locations/{location_id}"),
            )),
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }
}

impl InventoryRepository for SnipeItClient {
    fn find_location_by_code(&self, code: &str) -> Result<LocationDto, InventoryError> {
        let url = self
            .config
            .endpoint_url(&format!("locations?search={code}"));
        let res = self
            .agent
            .get(&url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .call();

        match res {
            Ok(resp) => {
                let paginated: PaginatedResponse<LocationDto> =
                    resp.into_json()
                        .map_err(|err| InventoryError::Deserialization {
                            message: err.to_string(),
                        })?;
                paginated
                    .rows()
                    .iter()
                    .find(|loc| loc.name() == code)
                    .cloned()
                    .or_else(|| paginated.rows().first().cloned())
                    .ok_or_else(|| InventoryError::NotFound {
                        resource: format!("Location with code '{code}'"),
                    })
            }
            Err(ureq::Error::Status(status, _)) => Err(Self::map_http_status(
                status,
                &format!("locations search '{code}'"),
            )),
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }

    fn resolve_container_lineage(
        &self,
        container_code: &str,
    ) -> Result<LocationPath, InventoryError> {
        let start = self.find_location_by_code(container_code)?;
        let mut chain: Vec<LocationDto> = Vec::new();
        chain.push(start);

        while let Some(parent_id) = chain.last().and_then(LocationDto::parent_id) {
            if chain.len() >= 10 {
                break;
            }
            let parent = self.find_location_by_id(parent_id)?;
            chain.push(parent);
        }

        chain.reverse();

        let mut nodes = Vec::with_capacity(chain.len());
        for (index, loc) in chain.into_iter().enumerate() {
            let depth = (index + 1) as u8;
            let level = TaxonomyLevel::from_depth(depth)?;
            nodes.push(LocationNode::new(
                loc.id(),
                loc.name(),
                loc.parent_id(),
                level,
            ));
        }

        let path = LocationPath::from_container_lineage(nodes)?;
        Ok(path)
    }

    fn list_assets_in_location(&self, location_id: u64) -> Result<Vec<AssetDto>, InventoryError> {
        let url = self
            .config
            .endpoint_url(&format!("hardware?location_id={location_id}"));
        let res = self
            .agent
            .get(&url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .call();

        match res {
            Ok(resp) => {
                let paginated: PaginatedResponse<AssetDto> =
                    resp.into_json()
                        .map_err(|err| InventoryError::Deserialization {
                            message: err.to_string(),
                        })?;
                Ok(paginated.rows().to_vec())
            }
            Err(ureq::Error::Status(status, _)) => Err(Self::map_http_status(
                status,
                &format!("hardware?location_id={location_id}"),
            )),
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }

    fn checkout_asset_to_location(
        &self,
        asset_tag: &str,
        location_id: u64,
    ) -> Result<(), InventoryError> {
        let search_url = self
            .config
            .endpoint_url(&format!("hardware/bytag/{asset_tag}"));
        let asset_id = match self
            .agent
            .get(&search_url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .call()
        {
            Ok(resp) => {
                let asset: AssetDto =
                    resp.into_json()
                        .map_err(|err| InventoryError::Deserialization {
                            message: err.to_string(),
                        })?;
                asset.id()
            }
            Err(ureq::Error::Status(status, _)) => {
                return Err(Self::map_http_status(
                    status,
                    &format!("hardware/bytag/{asset_tag}"),
                ));
            }
            Err(ureq::Error::Transport(err)) => {
                return Err(InventoryError::HttpTransport {
                    message: err.to_string(),
                });
            }
        };

        let checkout_url = self
            .config
            .endpoint_url(&format!("hardware/{asset_id}/checkout"));
        let payload = serde_json::json!({
            "checkout_to_type": "location",
            "assigned_location": location_id,
        });

        let res = self
            .agent
            .post(&checkout_url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .set("Content-Type", "application/json")
            .send_json(payload);

        match res {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(status, _)) => Err(Self::map_http_status(
                status,
                &format!("hardware/{asset_id}/checkout"),
            )),
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }

    fn relocate_container(
        &self,
        container_id: u64,
        new_parent_id: u64,
    ) -> Result<(), InventoryError> {
        let url = self
            .config
            .endpoint_url(&format!("locations/{container_id}"));
        let payload = serde_json::json!({
            "parent_id": new_parent_id,
        });

        let res = self
            .agent
            .patch(&url)
            .set("Authorization", &self.config.bearer_header())
            .set("Accept", "application/json")
            .set("Content-Type", "application/json")
            .send_json(payload);

        match res {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(status, _)) => Err(Self::map_http_status(
                status,
                &format!("locations/{container_id}"),
            )),
            Err(ureq::Error::Transport(err)) => Err(InventoryError::HttpTransport {
                message: err.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_config_valid() {
        let config = InventoryConfig::new(
            "https://snipeit.example.com/api/v1/",
            "secret-token-123",
            Duration::from_secs(10),
        )
        .expect("valid configuration must succeed");

        assert_eq!(config.base_url(), "https://snipeit.example.com/api/v1");
        assert_eq!(config.api_token(), "secret-token-123");
        assert_eq!(config.timeout(), Duration::from_secs(10));
        assert_eq!(config.bearer_header(), "Bearer secret-token-123");
        assert_eq!(
            config.endpoint_url("locations"),
            "https://snipeit.example.com/api/v1/locations"
        );
        assert_eq!(
            config.endpoint_url("/hardware?limit=10"),
            "https://snipeit.example.com/api/v1/hardware?limit=10"
        );
        assert_eq!(
            config.endpoint_url(""),
            "https://snipeit.example.com/api/v1"
        );
    }

    #[test]
    fn test_inventory_config_empty_base_url() {
        assert_eq!(
            InventoryConfig::new("", "token", Duration::from_secs(5)),
            Err(InventoryError::EmptyBaseUrl)
        );
        assert_eq!(
            InventoryConfig::new("   ", "token", Duration::from_secs(5)),
            Err(InventoryError::EmptyBaseUrl)
        );
    }

    #[test]
    fn test_inventory_config_empty_api_token() {
        assert_eq!(
            InventoryConfig::new("https://example.com", "", Duration::from_secs(5)),
            Err(InventoryError::EmptyApiToken)
        );
        assert_eq!(
            InventoryConfig::new("https://example.com", "   ", Duration::from_secs(5)),
            Err(InventoryError::EmptyApiToken)
        );
    }

    #[test]
    fn test_map_http_status() {
        assert_eq!(
            SnipeItClient::map_http_status(401, "auth"),
            InventoryError::Unauthorized
        );
        assert_eq!(
            SnipeItClient::map_http_status(403, "auth"),
            InventoryError::Unauthorized
        );
        assert_eq!(
            SnipeItClient::map_http_status(404, "location-1"),
            InventoryError::NotFound {
                resource: "location-1".to_owned()
            }
        );
        assert_eq!(
            SnipeItClient::map_http_status(429, "rate"),
            InventoryError::RateLimited
        );
        assert_eq!(
            SnipeItClient::map_http_status(500, "server"),
            InventoryError::HttpTransport {
                message: "HTTP status 500 for server".to_owned()
            }
        );
    }

    #[test]
    fn test_snipe_it_client_construction() {
        let config = InventoryConfig::new(
            "http://localhost:8000/api/v1",
            "dummy-token",
            Duration::from_secs(3),
        )
        .expect("valid config");
        let client = SnipeItClient::new(config.clone());
        assert_eq!(client.config(), &config);
    }
}
