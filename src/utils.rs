use futures_util::future::join_all;
use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::RwLock;

use cosmic::{iced_core::image, widget};

use crate::error::Error;

pub static IMAGE_CACHE: LazyLock<RwLock<ImageCache>> =
    LazyLock::new(|| RwLock::new(ImageCache::new()));

pub struct ImageCache {
    images: HashMap<String, widget::image::Handle>,
    client: reqwest::Client,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            client: reqwest::Client::builder().build().unwrap(),
        }
    }

    pub async fn get_batch(&mut self, urls: Vec<String>) -> HashMap<String, widget::image::Handle> {
        let mut handles = HashMap::new();
        let mut to_fetch = Vec::new();

        for url in urls {
            if let Some(handle) = self.images.get(&url) {
                handles.insert(url, handle.clone());
            } else {
                to_fetch.push(url);
            }
        }

        let futures = to_fetch.iter().map(|url| {
            let client = &self.client;
            let url = url.clone();
            async move {
                match client.get(&url).send().await.ok()?.bytes().await {
                    Ok(bytes) => Some((url, image::Handle::from_bytes(bytes.to_vec()))),
                    Err(e) => {
                        tracing::error!("Failed to fetch image {}: {}", url, e);
                        None
                    }
                }
            }
        });

        for (url, handle) in join_all(futures).await.into_iter().flatten() {
            self.images.insert(url.clone(), handle.clone());
            handles.insert(url, handle);
        }

        handles
    }

    pub async fn get(&mut self, url: &str) -> Result<widget::image::Handle, Error> {
        match self.images.get(url) {
            Some(handle) => Ok(handle.clone()),
            None => {
                let response = self.client.get(url).send().await?;
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
