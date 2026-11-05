use keyring::Entry;

const SERVICE_NAME: &str = "mensetsu_desktop";
const USER_NAME: &str = "refresh_token";

#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

pub struct KeyringStore;

impl KeyringStore {
    pub fn new() -> Self {
        Self
    }

    pub fn get_refresh_token(&self) -> Result<Option<String>, KeyringError> {
        let entry = Entry::new(SERVICE_NAME, USER_NAME)?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeyringError::Keyring(e)),
        }
    }

    pub fn set_refresh_token(&self, token: &str) -> Result<(), KeyringError> {
        let entry = Entry::new(SERVICE_NAME, USER_NAME)?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn delete_refresh_token(&self) -> Result<(), KeyringError> {
        let entry = Entry::new(SERVICE_NAME, USER_NAME)?;
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(KeyringError::Keyring(e)),
        }
    }
}
