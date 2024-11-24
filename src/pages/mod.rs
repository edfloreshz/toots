use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::RwLock;

use cosmic::{iced_core::image, widget};

use crate::error::Error;

pub mod home;
pub mod notifications;

pub static IMAGE_LOADER: LazyLock<RwLock<ImageLoader>> =
    LazyLock::new(|| RwLock::new(ImageLoader::new()));

pub struct ImageLoader {
    images: HashMap<String, widget::image::Handle>,
}

impl ImageLoader {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub async fn get(&mut self, url: &str) -> Result<widget::image::Handle, Error> {
        match self.images.get(url) {
            Some(handle) => Ok(handle.clone()),
            None => {
                let response = reqwest::get(url).await?;
                match response.error_for_status() {
                    Ok(response) => {
                        let bytes = response.bytes().await?;
                        let handle = image::Handle::from_bytes(bytes.to_vec());
                        self.images.insert(url.to_string(), handle.clone());
                        Ok(handle)
                    }
                    Err(err) => Err(err.into()),
                }
            }
        }
    }
}
