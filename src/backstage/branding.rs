//! Static branding defaults for Backstage scaffold.
//!
//! Provides default branding configuration and assets. Users can edit
//! generated files manually after scaffold to customize branding.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Default branding configuration for Backstage app-config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingDefaults {
    /// Portal title displayed in header
    pub title: String,
    /// Subtitle shown below title
    pub subtitle: String,
    /// Primary brand color (hex)
    pub primary_color: String,
    /// Secondary brand color (hex)
    pub secondary_color: String,
    /// Optional logo path relative to branding directory
    pub logo_path: Option<String>,
    /// Optional favicon path relative to branding directory
    pub favicon_path: Option<String>,
}

impl Default for BrandingDefaults {
    fn default() -> Self {
        Self {
            title: "Developer Portal".to_string(),
            subtitle: "Powered by Backstage".to_string(),
            primary_color: "#0052CC".to_string(), // Backstage blue
            secondary_color: "#172B4D".to_string(),
            logo_path: Some("logo.svg".to_string()),
            favicon_path: None,
        }
    }
}

impl BrandingDefaults {
    /// Create branding with a custom portal name.
    pub fn with_name(name: &str) -> Self {
        Self {
            title: name.to_string(),
            ..Default::default()
        }
    }

    /// Generate the app section YAML for branding.
    pub fn to_app_config_yaml(&self) -> String {
        format!(
            r#"app:
  title: {}
  branding:
    theme:
      light:
        primaryColor: "{}"
        secondaryColor: "{}"
      dark:
        primaryColor: "{}"
        secondaryColor: "{}""#,
            self.title,
            self.primary_color,
            self.secondary_color,
            self.primary_color,
            self.secondary_color
        )
    }
}

/// Static branding assets embedded in the binary.
pub struct BrandingAssets;

impl BrandingAssets {
    /// Default logo SVG content - a simple developer portal icon.
    pub fn default_logo_svg() -> &'static str {
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
  <rect width="100" height="100" rx="10" fill="#0052CC"/>
  <text x="50" y="65" font-family="Arial, sans-serif" font-size="40" font-weight="bold" fill="white" text-anchor="middle">DP</text>
</svg>"##
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branding_defaults() {
        let branding = BrandingDefaults::default();
        assert_eq!(branding.title, "Developer Portal");
        assert!(branding.primary_color.starts_with('#'));
        assert!(branding.secondary_color.starts_with('#'));
    }

    #[test]
    fn test_branding_with_name() {
        let branding = BrandingDefaults::with_name("GBQR Portal");
        assert_eq!(branding.title, "GBQR Portal");
        assert_eq!(branding.subtitle, "Powered by Backstage");
    }

    #[test]
    fn test_app_config_yaml() {
        let branding = BrandingDefaults::default();
        let yaml = branding.to_app_config_yaml();
        assert!(yaml.contains("Developer Portal"));
        assert!(yaml.contains("#0052CC"));
        assert!(yaml.contains("primaryColor"));
    }

    #[test]
    fn test_default_logo_svg() {
        let svg = BrandingAssets::default_logo_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("#0052CC"));
    }
}
