#[macro_use]
extern crate impl_ops;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitfield;

use serde::{Deserialize, Serialize};

pub mod channels;
pub mod media;
pub mod permissions;
pub mod servers;
pub mod types;
pub mod users;
pub mod util;
pub mod ws;

/// # hCaptcha Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaptchaFeature {
    /// Whether captcha is enabled
    pub enabled: bool,
    /// Client key used for solving captcha
    pub key: String,
}

/// # Generic Service Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Feature {
    /// Whether the service is enabled
    pub enabled: bool,
    /// URL pointing to the service
    pub url: String,
}

/// # Voice Server Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceFeature {
    /// Whether voice is enabled
    pub enabled: bool,
    /// URL pointing to the voice API
    pub url: String,
    /// URL pointing to the voice WebSocket server
    pub ws: String,
}

/// # Feature Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevoltFeatures {
    /// hCaptcha configuration
    pub captcha: CaptchaFeature,
    /// Whether email verification is enabled
    pub email: bool,
    /// Whether this server is invite only
    pub invite_only: bool,
    /// File server service configuration
    pub autumn: Feature,
    /// Proxy service configuration
    pub january: Feature,
    /// Voice server configuration
    pub voso: VoiceFeature,
}

/// # Build Information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildInformation {
    /// Commit Hash
    pub commit_sha: String,
    /// Commit Timestamp
    pub commit_timestamp: String,
    /// Git Semver
    pub semver: String,
    /// Git Origin URL
    pub origin_url: String,
    /// Build Timestamp
    pub timestamp: String,
}

/// # Server Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevoltConfig {
    /// Revolt API Version
    pub revolt: String,
    /// Features enabled on this Revolt node
    pub features: RevoltFeatures,
    /// WebSocket URL
    pub ws: String,
    /// URL pointing to the client serving this node
    pub app: String,
    /// Web Push VAPID public key
    pub vapid: String,
    /// Build information
    pub build: Option<BuildInformation>,
}
