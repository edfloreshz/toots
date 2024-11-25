use std::collections::{HashMap, VecDeque};

use cosmic::{
    iced::Subscription,
    iced_core::image,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget::{self, image::Handle},
    Apply, Element, Task,
};
use mastodon_async::{entities::notification::Notification, prelude::Mastodon};

use crate::{app, utils, widgets::status::StatusHandles};

#[derive(Debug, Clone)]
pub struct Notifications {
    pub mastodon: Option<Mastodon>,
    notifications: VecDeque<Notification>,
    handles: HashMap<String, Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendNotification(Notification),
    PrependNotification(Notification),
    Notification(crate::widgets::notification::Message),
    ResolveAvatar(Notification),
    SetAvatars(Notification, Option<image::Handle>, Option<image::Handle>),
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            mastodon: None,
            notifications: VecDeque::new(),
            handles: HashMap::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let notifications: Vec<Element<_>> = self
            .notifications
            .iter()
            .map(|notification| {
                crate::widgets::notification(
                    notification,
                    &StatusHandles::from_notification(notification, &self.handles),
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
                self.notifications.push_back(notification.clone());
                if !self.handles.contains_key(&notification.account.avatar) {
                    tasks.push(self.update(Message::ResolveAvatar(notification)));
                }
            }
            Message::PrependNotification(notification) => {
                if !self.handles.contains_key(&notification.account.avatar) {
                    self.notifications.push_front(notification.clone());
                    tasks.push(self.update(Message::ResolveAvatar(notification)));
                }
            }
            Message::ResolveAvatar(notification) => {
                let handles = StatusHandles::from_notification(&notification, &self.handles);
                match (handles.primary, handles.secondary) {
                    (None, None) => {
                        tasks.push(Task::perform(
                            async {
                                let handle = utils::get_image(&notification.account.avatar).await;
                                let reblog_handle = if let Some(reblog) = &notification.status {
                                    utils::get_image(&reblog.account.avatar).await.ok()
                                } else {
                                    None
                                };

                                (notification, handle, reblog_handle)
                            },
                            |(notification, status_avatar, reblog_avatar)| {
                                cosmic::app::message::app(app::Message::Notifications(
                                    Message::SetAvatars(
                                        notification.clone(),
                                        status_avatar.ok(),
                                        reblog_avatar,
                                    ),
                                ))
                            },
                        ));
                    }
                    (status_avatar, reblog_avatar) => tasks.push(self.update(Message::SetAvatars(
                        notification.clone(),
                        status_avatar,
                        reblog_avatar,
                    ))),
                }
            }
            Message::SetAvatars(notification, status_avatar, sender_avatar) => {
                if let Some(status_avatar) = status_avatar {
                    self.handles
                        .insert(notification.account.avatar, status_avatar);
                }
                if let Some(sender_avatar) = sender_avatar {
                    self.handles
                        .insert(notification.status.unwrap().account.avatar, sender_avatar);
                }
            }
            Message::Notification(message) => match message {
                crate::widgets::notification::Message::Status(message) => match message {
                    crate::widgets::status::Message::OpenProfile(account_id) => {
                        tracing::info!("open profile: {}", account_id)
                    }
                    crate::widgets::status::Message::ExpandStatus(status) => {
                        tracing::info!("expand status: {}", status.id)
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
