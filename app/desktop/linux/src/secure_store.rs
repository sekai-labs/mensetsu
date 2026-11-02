use oo7::Keyring;

const SECRET_SERVICE_APP: &str = "mensetsu_desktop_linux";
const SECRET_SERVICE_ATTR: &str = "refresh_token";
static ATTRIBUTES: std::sync::LazyLock<Vec<(&str, &str)>> =
    std::sync::LazyLock::new(|| vec![("application", SECRET_SERVICE_APP), ("type", SECRET_SERVICE_ATTR)]);

#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("Secret service error: {0}")]
    Keyring(#[from] oo7::Error),
    #[error("Secret encoding error: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
}

pub struct KeyringStore;

impl KeyringStore {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_refresh_token(&self) -> Result<Option<String>, KeyringError> {
        let keyring = Keyring::new().await?;
        let items = keyring
            .search_items(&*ATTRIBUTES)
            .await?;
        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            let token = String::from_utf8(secret.to_vec())?;
            return Ok(Some(token));
        }
        Ok(None)
    }

    pub async fn set_refresh_token(&self, token: &str) -> Result<(), KeyringError> {
        let keyring = Keyring::new().await?;
        keyring
            .create_item(
                "Mensetsu Refresh Token",
                &*ATTRIBUTES,
                token.as_bytes(),
                true, // replace
            )
            .await?;
        Ok(())
    }

    pub async fn delete_refresh_token(&self) -> Result<(), KeyringError> {
        let keyring = Keyring::new().await?;
        let items = keyring
            .search_items(&*ATTRIBUTES)
            .await?;
        for item in items {
            item.delete().await?;
        }
        Ok(())
    }
}
