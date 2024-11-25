use cosmic::{iced_core::image, widget};

use crate::error::Error;

pub async fn get_image(url: &str) -> Result<widget::image::Handle, Error> {
    let response = reqwest::get(url).await?;
    match response.error_for_status() {
        Ok(response) => {
            let bytes = response.bytes().await?;
            let handle = image::Handle::from_bytes(bytes.to_vec());
            Ok(handle)
        }
        Err(err) => Err(err.into()),
    }
}

pub fn fallback_avatar<'a>() -> widget::Image<'a> {
    widget::image(image::Handle::from_bytes(
        include_bytes!("../assets/missing.png").to_vec(),
    ))
}

pub fn fallback_handle() -> widget::image::Handle {
    image::Handle::from_bytes(include_bytes!("../assets/missing.png").to_vec())
}
