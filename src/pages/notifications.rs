use std::collections::VecDeque;

use cosmic::{
    iced::Subscription,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::{
    entities::notification::Notification,
    prelude::{Mastodon, NotificationId},
};

use crate::{
    app::{self, ContextPage},
    utils::Cache,
    widgets::status::StatusHandles,
};

#[derive(Debug, Clone)]
pub struct Notifications {
    pub mastodon: Option<Mastodon>,
    notifications: VecDeque<NotificationId>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendNotification(Notification),
    PrependNotification(Notification),
    Notification(crate::widgets::notification::Message),
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            mastodon: None,
            notifications: VecDeque::new(),
        }
    }

    pub fn view(&self, cache: &Cache) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let notifications: Vec<Element<_>> = self
            .notifications
            .iter()
            .filter_map(|id| cache.notifications.get(&id.to_string()))
            .map(|notification| {
                crate::widgets::notification(
                    notification,
                    &StatusHandles::from_notification(notification, &cache.handles),
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
        let mut tasks = vec![];
        match message {
            Message::SetClient(mastodon) => self.mastodon = mastodon,
            Message::AppendNotification(notification) => {
                self.notifications.push_back(notification.id.clone());
                tasks.push(cosmic::task::message(app::Message::CacheNotification(
                    notification,
                )));
            }
            Message::PrependNotification(notification) => {
                self.notifications.push_front(notification.id.clone());
                tasks.push(cosmic::task::message(app::Message::CacheNotification(
                    notification,
                )));
            }
            Message::Notification(message) => match message {
                crate::widgets::notification::Message::Status(message) => match message {
                    crate::widgets::status::Message::OpenProfile(url) => {
                        _ = open::that_detached(url);
                    }
                    crate::widgets::status::Message::ExpandStatus(status) => {
                        tasks.push(cosmic::task::message(app::Message::ToggleContextPage(
                            ContextPage::Status(status.id.clone()),
                        )));
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
        match self.mastodon.clone() {
            Some(mastodon) => {
                Subscription::batch(vec![crate::subscriptions::notifications::timeline(
                    mastodon,
                )])
            }
            None => Subscription::none(),
        }
    }
}
