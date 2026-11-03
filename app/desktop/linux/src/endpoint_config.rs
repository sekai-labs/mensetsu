use std::fs;
use std::path::PathBuf;
use url::Url;

const CONFIG_FILE_NAME: &str = "mensetsu_config.json";

#[derive(Debug, thiserror::Error)]
pub enum EndpointConfigError {
    #[error("Server URL cannot be empty")]
    EmptyUrl,
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("URL must not include credentials/userinfo")]
    UserinfoNotAllowed,
    #[error("URL must not include a query string")]
    QueryNotAllowed,
    #[error("URL must not include a fragment identifier")]
    FragmentNotAllowed,
    #[error("URL must not include a subpath (enter server origin only)")]
    PathNotAllowed,
    #[error("Non-loopback servers must use HTTPS")]
    InsecureHttp,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
pub struct AppConfigData {
    pub server_url: Option<String>,
}

/// Normalizes and validates the server URL.
/// Rules:
/// - Strips trailing slashes, trims whitespace
/// - Must be an absolute http or https URL
/// - Non-loopback hosts must use HTTPS (localhost, 127.0.0.1, ::1 allow http)
pub fn validate_and_normalize_url(raw: &str) -> Result<String, EndpointConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EndpointConfigError::EmptyUrl);
    }

    let parsed = Url::parse(trimmed.trim_end_matches('/'))
        .map_err(|e| EndpointConfigError::InvalidUrl(e.to_string()))?;

    // Reject userinfo (credentials in URL)
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(EndpointConfigError::UserinfoNotAllowed);
    }

    // Reject query string
    if parsed.query().is_some() {
        return Err(EndpointConfigError::QueryNotAllowed);
    }

    // Reject fragment
    if parsed.fragment().is_some() {
        return Err(EndpointConfigError::FragmentNotAllowed);
    }

    // Reject non-empty path beyond "/"
    let path = parsed.path();
    if path != "" && path != "/" {
        return Err(EndpointConfigError::PathNotAllowed);
    }

    let scheme = parsed.scheme().to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(EndpointConfigError::InvalidUrl("Scheme must be http or https".to_string()));
    }

    let host = match parsed.host_str() {
        Some(h) => h.to_ascii_lowercase(),
        None => return Err(EndpointConfigError::InvalidUrl("Missing host".to_string())),
    };

    // 0.0.0.0 is a bind address, not a client loopback address
    let is_loopback = host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "[::1]";

    if scheme == "http" && !is_loopback {
        return Err(EndpointConfigError::InsecureHttp);
    }

    // Build normalized base URL: scheme://host[:port]
    let port_str = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();
    let normalized = format!("{scheme}://{host}{port_str}");
    Ok(normalized)
}

/// Resolve the path to the configuration directory: $XDG_CONFIG_HOME/mensetsu or ~/.config/mensetsu
fn config_file_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("mensetsu").join(CONFIG_FILE_NAME);
        }
    }
    glib::user_config_dir().join("mensetsu").join(CONFIG_FILE_NAME)
}

/// Reads the persisted server URL from disk if configured, validating and normalizing it.
pub fn get_persisted_server_url() -> Option<String> {
    let path = config_file_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let config: AppConfigData = serde_json::from_str(&content).ok()?;
    let raw = config.server_url?;
    validate_and_normalize_url(&raw).ok()
}

/// Saves the server URL to disk.
pub fn set_persisted_server_url(url: &str) -> Result<(), EndpointConfigError> {
    let normalized = validate_and_normalize_url(url)?;
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let config = AppConfigData {
        server_url: Some(normalized),
    };
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(path, json)?;
    Ok(())
}
/// Reads the persisted server URL from disk if configured, validating and normalizing it.
/// Returns None on fresh launch (unconfigured), which prompts the user.
pub fn resolve_initial_url() -> Option<String> {
    get_persisted_server_url()
}
