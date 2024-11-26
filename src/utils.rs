use std::collections::HashMap;

use cosmic::{
    iced_core::image,
    widget::{self, image::Handle},
};
use mastodon_async::prelude::*;

#[derive(Debug, Clone)]
pub struct Cache {
    pub handles: HashMap<String, Handle>,
    pub statuses: HashMap<String, Status>,
    pub notifications: HashMap<String, Notification>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            statuses: HashMap::new(),
            notifications: HashMap::new(),
        }
    }

    pub fn insert_status(&mut self, status: Status) {
        self.statuses.insert(status.id.to_string(), status.clone());
        if let Some(reblog) = status.reblog {
            self.statuses.insert(reblog.id.to_string(), *reblog);
        }
    }

    pub fn insert_notification(&mut self, notification: Notification) {
        self.notifications
            .insert(notification.id.to_string(), notification.clone());
        if let Some(status) = notification.status {
            self.insert_status(status.clone());
        }
    }

    pub fn insert_handle(&mut self, url: String, handle: Handle) {
        self.handles.insert(url, handle);
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
