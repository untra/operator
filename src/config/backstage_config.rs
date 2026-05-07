use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Backstage integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BackstageConfig {
    /// Whether Backstage integration is enabled
    #[serde(default = "default_backstage_enabled")]
    pub enabled: bool,
    /// Whether to show Backstage in the Connections status section
    #[serde(default)]
    pub display: bool,
    /// Port for the Backstage server
    #[serde(default = "default_backstage_port")]
    pub port: u16,
    /// Auto-start Backstage server when TUI launches
    #[serde(default)]
    pub auto_start: bool,
    /// Subdirectory within `state_path` for Backstage installation
    #[serde(default = "default_backstage_subpath")]
    pub subpath: String,
    /// Subdirectory within backstage path for branding customization
    #[serde(default = "default_branding_subpath")]
    pub branding_subpath: String,
    /// Base URL for downloading backstage-server binary
    #[serde(default = "default_backstage_release_url")]
    pub release_url: String,
    /// Optional local path to backstage-server binary
    /// If set, this is used instead of downloading from `release_url`
    #[serde(default)]
    pub local_binary_path: Option<String>,
    /// Branding and theming configuration
    #[serde(default)]
    pub branding: BrandingConfig,
}

/// Branding configuration for Backstage portal
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BrandingConfig {
    /// App title shown in header
    #[serde(default = "default_app_title")]
    pub app_title: String,
    /// Organization name
    #[serde(default = "default_org_name")]
    pub org_name: String,
    /// Path to logo SVG (relative to branding path)
    #[serde(default)]
    pub logo_path: Option<String>,
    /// Theme colors (uses Operator defaults if not set)
    #[serde(default)]
    pub colors: ThemeColors,
}

fn default_app_title() -> String {
    "Operator Portal".to_string()
}

fn default_org_name() -> String {
    "Operator".to_string()
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            app_title: default_app_title(),
            org_name: default_org_name(),
            logo_path: Some("logo.svg".to_string()),
            colors: ThemeColors::default(),
        }
    }
}

/// Theme color configuration for Backstage
/// Default colors match Operator's tmux theme
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ThemeColors {
    /// Primary/accent color (default: salmon #cc6c55)
    #[serde(default = "default_color_primary")]
    pub primary: String,
    /// Secondary color (default: dark teal #114145)
    #[serde(default = "default_color_secondary")]
    pub secondary: String,
    /// Accent/highlight color (default: cream #f4dbb7)
    #[serde(default = "default_color_accent")]
    pub accent: String,
    /// Warning/error color (default: coral #d46048)
    #[serde(default = "default_color_warning")]
    pub warning: String,
    /// Muted text color (default: darker salmon #8a4a3a)
    #[serde(default = "default_color_muted")]
    pub muted: String,
}

fn default_color_primary() -> String {
    "#cc6c55".to_string() // salmon
}

fn default_color_secondary() -> String {
    "#114145".to_string() // dark teal
}

fn default_color_accent() -> String {
    "#f4dbb7".to_string() // cream
}

fn default_color_warning() -> String {
    "#d46048".to_string() // coral
}

fn default_color_muted() -> String {
    "#8a4a3a".to_string() // darker salmon
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: default_color_primary(),
            secondary: default_color_secondary(),
            accent: default_color_accent(),
            warning: default_color_warning(),
            muted: default_color_muted(),
        }
    }
}

fn default_backstage_enabled() -> bool {
    true
}

fn default_backstage_port() -> u16 {
    7007
}

fn default_backstage_subpath() -> String {
    "backstage".to_string()
}

fn default_branding_subpath() -> String {
    "branding".to_string()
}

fn default_backstage_release_url() -> String {
    "https://github.com/untra/operator/releases/latest/download".to_string()
}

impl Default for BackstageConfig {
    fn default() -> Self {
        Self {
            enabled: default_backstage_enabled(),
            display: false,
            port: default_backstage_port(),
            auto_start: false,
            subpath: default_backstage_subpath(),
            branding_subpath: default_branding_subpath(),
            release_url: default_backstage_release_url(),
            local_binary_path: None,
            branding: BrandingConfig::default(),
        }
    }
}
