use anyhow::{anyhow, Result};
use web_sys::Storage;

#[allow(dead_code)] // 为未来功能准备
pub struct StorageService;

impl StorageService {
    #[allow(dead_code)] // 为未来功能准备
    fn get_local_storage() -> Result<Storage> {
        let window = web_sys::window().ok_or_else(|| anyhow!("No window found"))?;
        window
            .local_storage()
            .map_err(|_| anyhow!("Failed to get local storage"))?
            .ok_or_else(|| anyhow!("Local storage not available"))
    }

    #[allow(dead_code)] // 为未来功能准备
    pub fn set_item(key: &str, value: &str) -> Result<()> {
        let storage = Self::get_local_storage()?;
        storage
            .set_item(key, value)
            .map_err(|_| anyhow!("Failed to set item"))
    }

    #[allow(dead_code)] // 为未来功能准备
    pub fn get_item(key: &str) -> Result<Option<String>> {
        let storage = Self::get_local_storage()?;
        storage
            .get_item(key)
            .map_err(|_| anyhow!("Failed to get item"))
    }

    #[allow(dead_code)] // 为未来功能准备
    pub fn remove_item(key: &str) -> Result<()> {
        let storage = Self::get_local_storage()?;
        storage
            .remove_item(key)
            .map_err(|_| anyhow!("Failed to remove item"))
    }
}
