//! Vertical integration catalog endpoint.
//!
//! Serves [`crate::integrations::catalog`] — the single source of truth for
//! advertised integrations and their support status — to the docs site and any
//! future entitlement layer. Static (config-independent), so it needs no state.

use axum::Json;

use crate::rest::dto::{integration_catalog, IntegrationCatalogEntryDto};

/// GET `/api/v1/integrations`
///
/// Returns the catalog of advertised integrations across every vertical, each
/// with its docs link and official support status (`proto` | `alpha` | `beta` |
/// `ga`).
#[utoipa::path(
    get,
    path = "/api/v1/integrations",
    tag = "Status",
    operation_id = "integrations_catalog",
    responses(
        (status = 200, description = "Vertical integration catalog with support status", body = Vec<IntegrationCatalogEntryDto>)
    )
)]
pub async fn catalog() -> Json<Vec<IntegrationCatalogEntryDto>> {
    Json(integration_catalog())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integrations_catalog_returns_entries() {
        let resp = catalog().await;
        assert!(!resp.0.is_empty());
        assert!(resp.0.iter().any(|e| e.slug == "claude"));
    }
}
