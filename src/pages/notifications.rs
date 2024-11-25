use std::collections::{HashSet, VecDeque};

use cosmic::{
    iced::Subscription,
    iced_core::image,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::{
    entities::notification::Notification as MastodonNotification,
    prelude::{Mastodon, NotificationId},
};

use crate::{app, utils::IMAGE_CACHE};

#[derive(Debug, Clone)]
pub struct Notifications {
    pub mastodon: Option<Mastodon>,
    notifications: VecDeque<Notification>,
    notification_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendNotification(MastodonNotification),
    PrependNotification(MastodonNotification),
    Notification(crate::widgets::notification::Message),
    ResolveAvatar(NotificationId),
    SetAvatars(NotificationId, Option<image::Handle>, Option<image::Handle>),
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
        let mut tasks = vec![];
        match message {
            Message::SetClient(mastodon) => self.mastodon = mastodon,
            Message::AppendNotification(notification) => {
                if !self.notification_ids.contains(&notification.id.to_string()) {
                    self.notifications.push_back(Notification::new(
                        notification.clone(),
                        None,
                        None,
                    ));
                    self.notification_ids.insert(notification.id.to_string());
                    tasks.push(self.update(Message::ResolveAvatar(notification.id.clone())));
                }
            }
            Message::PrependNotification(notification) => {
                if !self.notification_ids.contains(&notification.id.to_string()) {
                    self.notifications.push_front(Notification::new(
                        notification.clone(),
                        None,
                        None,
                    ));
                    self.notification_ids.insert(notification.id.to_string());
                    tasks.push(self.update(Message::ResolveAvatar(notification.id.clone())));
                }
            }
            Message::ResolveAvatar(status_id) => {
                let status = self
                    .notifications
                    .iter()
                    .find(|s| s.notification.id == status_id)
                    .map(|status| status.notification.clone())
                    .expect("status not found");
                tasks.push(Task::perform(
                    async move {
                        let mut image_cache = IMAGE_CACHE.write().await;
                        let handle = image_cache.get(&status.account.avatar_static).await;
                        let reblog_handle = if let Some(reblog) = &status.status {
                            image_cache.get(&reblog.account.avatar_static).await.ok()
                        } else {
                            None
                        };

                        (status_id, handle.ok(), reblog_handle)
                    },
                    |(id, status_avatar, reblog_avatar)| {
                        cosmic::app::message::app(app::Message::Notifications(Message::SetAvatars(
                            id,
                            status_avatar,
                            reblog_avatar,
                        )))
                    },
                ));
            }
            Message::SetAvatars(id, status_avatar, sender_avatar) => {
                let notification = self
                    .notifications
                    .iter_mut()
                    .find(|n| n.notification.id == id);
                if let Some(status) = notification {
                    status.sender_avatar = sender_avatar;
                    status.status_avatar = status_avatar;
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
