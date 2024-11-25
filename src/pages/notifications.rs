use std::collections::{HashSet, VecDeque};

use cosmic::{
    iced::Subscription,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::{
    entities::notification::Notification as MastodonNotification, prelude::Mastodon,
};

use crate::app;

#[derive(Debug, Clone)]
pub struct Notifications {
    pub mastodon: Option<Mastodon>,
    notifications: VecDeque<Notification>,
    notification_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendNotification(Notification),
    PrependNotification(Notification),
    Notification(crate::widgets::notification::Message),
}

#[derive(Debug, Clone)]
pub struct Notification {
    notification: MastodonNotification,
    sender_avatar: Option<widget::image::Handle>,
    status_avatar: Option<widget::image::Handle>,
}

impl Notification {
    pub fn new(
        notification: MastodonNotification,
        avatar: Option<widget::image::Handle>,
        reglog_avatar: Option<widget::image::Handle>,
    ) -> Self {
        Self {
            notification,
            sender_avatar: avatar,
            status_avatar: reglog_avatar,
        }
    }
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            notification_ids: HashSet::new(),
            mastodon: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let notifications: Vec<Element<_>> = self
            .notifications
            .iter()
            .map(|post| {
                crate::widgets::notification(
                    post.notification.clone(),
                    post.sender_avatar.clone(),
                    post.status_avatar.clone(),
                )
                .map(Message::Notification)
            })
            .collect();

        widget::scrollable(widget::settings::section().extend(notifications))
            .direction(Direction::Vertical(
                Scrollbar::default().spacing(spacing.space_xxs),
            ))
            .apply(widget::container)
            .max_width(700)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<app::Message> {
        let tasks = vec![];
        match message {
            Message::SetClient(mastodon) => self.mastodon = mastodon,
            Message::AppendNotification(notification) => {
                if !self
                    .notification_ids
                    .contains(&notification.notification.id.to_string())
                {
                    self.notifications.push_back(notification.clone());
                    self.notification_ids
                        .insert(notification.notification.id.to_string());
                }
            }
            Message::PrependNotification(notification) => {
                if !self
                    .notification_ids
                    .contains(&notification.notification.id.to_string())
                {
                    self.notifications.push_front(notification.clone());
                    self.notification_ids
                        .insert(notification.notification.id.to_string());
                }
            }
            Message::Notification(message) => match message {
                crate::widgets::notification::Message::Status(message) => match message {
                    crate::widgets::status::Message::OpenProfile(account_id) => {
                        tracing::info!("open profile: {}", account_id)
                    }
                    crate::widgets::status::Message::ExpandStatus(status_id) => {
                        tracing::info!("expand status: {}", status_id)
                    }
                    crate::widgets::status::Message::Reply(status_id) => {
                        tracing::info!("reply: {}", status_id)
                    }
                    crate::widgets::status::Message::Favorite(status_id) => {
                        tracing::info!("favorite: {}", status_id)
                    }
                    crate::widgets::status::Message::Boost(status_id) => {
                        tracing::info!("boost: {}", status_id)
                    }
                    crate::widgets::status::Message::Bookmark(status_id) => {
                        tracing::info!("bookmark: {}", status_id)
                    }
                    crate::widgets::status::Message::OpenLink(url) => {
                        if let Err(err) = open::that_detached(url) {
                            tracing::error!("{err}")
                        }
                    }
                },
            },
        }
        Task::batch(tasks)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![];
        if let Some(mastodon) = self.mastodon.clone() {
            subscriptions.push(crate::subscriptions::notifications::timeline(mastodon));
        }

        Subscription::batch(subscriptions)
    }
}
