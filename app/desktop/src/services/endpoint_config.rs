use std::fs;
use std::path::PathBuf;
use url::Url;

const CONFIG_FILE_NAME: &str = "mensetsu_config.json";

#[derive(Debug, thiserror::Error, Clone)]
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
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
pub struct AppConfigData {
    pub server_url: Option<String>,
}

pub fn validate_and_normalize_url(raw: &str) -> Result<String, EndpointConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EndpointConfigError::EmptyUrl);
    }

    let parsed = Url::parse(trimmed.trim_end_matches('/'))
        .map_err(|e| EndpointConfigError::InvalidUrl(e.to_string()))?;


    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(EndpointConfigError::UserinfoNotAllowed);
    }


    if parsed.query().is_some() {
        return Err(EndpointConfigError::QueryNotAllowed);
    }


    if parsed.fragment().is_some() {
        return Err(EndpointConfigError::FragmentNotAllowed);
    }


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


    let is_loopback = host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "[::1]";

    if scheme == "http" && !is_loopback {
        return Err(EndpointConfigError::InsecureHttp);
    }


    let port_str = parsed.port().map(|p| format!(":{p}")).unwrap_or_default();
    let normalized = format!("{scheme}://{host}{port_str}");
    Ok(normalized)
}

pub fn config_file_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("mensetsu").join(CONFIG_FILE_NAME);
        }
    }
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("mensetsu").join(CONFIG_FILE_NAME);
    }
    PathBuf::from(".mensetsu_config.json")
}

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

pub fn set_persisted_server_url(url: &str) -> Result<(), EndpointConfigError> {
    let normalized = validate_and_normalize_url(url)?;
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| EndpointConfigError::Io(e.to_string()))?;
    }
    let config = AppConfigData {
        server_url: Some(normalized.clone()),
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| EndpointConfigError::Json(e.to_string()))?;
    fs::write(&path, json).map_err(|e| EndpointConfigError::Io(e.to_string()))?;

    Ok(())
}

pub fn resolve_initial_url() -> Option<String> {
    get_persisted_server_url()
}
