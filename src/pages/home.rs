use std::collections::{HashSet, VecDeque};

use cosmic::{
    iced::Subscription,
    iced_core::image,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::prelude::{Mastodon, Status as MastodonStatus, StatusId};

use crate::{
    app::{self, ContextPage},
    utils::IMAGE_CACHE,
};

#[derive(Debug, Clone)]
pub struct Home {
    pub mastodon: Option<Mastodon>,
    posts: VecDeque<Status>,
    post_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendPost(MastodonStatus),
    PrependPost(MastodonStatus),
    DeletePost(String),
    Status(crate::widgets::status::Message),
    ResolveAvatar(StatusId),
    SetAvatars(StatusId, Option<image::Handle>, Option<image::Handle>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub status: MastodonStatus,
    pub status_avatar: widget::image::Handle,
    pub reblog_avatar: widget::image::Handle,
}

impl Status {
    pub fn new(
        status: MastodonStatus,
        status_avatar: Option<widget::image::Handle>,
        reglog_avatar: Option<widget::image::Handle>,
    ) -> Self {
        Self {
            status,
            status_avatar: status_avatar.unwrap_or(image::Handle::from_bytes(vec![])),
            reblog_avatar: reglog_avatar.unwrap_or(image::Handle::from_bytes(vec![])),
        }
    }
}

impl Home {
    pub fn new() -> Self {
        Self {
            posts: VecDeque::new(),
            post_ids: HashSet::new(),
            mastodon: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let posts: Vec<Element<_>> = self
            .posts
            .iter()
            .map(|post| {
                crate::widgets::status(Status::new(
                    post.status.clone(),
                    Some(post.status_avatar.clone()),
                    Some(post.reblog_avatar.clone()),
                ))
                .map(Message::Status)
            })
            .collect();

        widget::scrollable(widget::settings::section().extend(posts))
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
            Message::AppendPost(status) => {
                if !self.post_ids.contains(&status.id.to_string()) {
                    self.posts
                        .push_back(Status::new(status.clone(), None, None));
                    self.post_ids.insert(status.id.to_string());
                    tasks.push(self.update(Message::ResolveAvatar(status.id.clone())));
                }
            }
            Message::PrependPost(status) => {
                if !self.post_ids.contains(&status.id.to_string()) {
                    self.posts
                        .push_front(Status::new(status.clone(), None, None));
                    self.post_ids.insert(status.id.to_string());
                    tasks.push(self.update(Message::ResolveAvatar(status.id.clone())));
                }
            }
            Message::DeletePost(id) => {
                self.post_ids.remove(&id);
                self.posts.retain(|post| post.status.id.to_string() != id)
            }
            Message::ResolveAvatar(status_id) => {
                let status = self
                    .posts
                    .iter()
                    .find(|s| s.status.id == status_id)
                    .map(|status| status.status.clone())
                    .expect("status not found");
                tasks.push(Task::perform(
                    async move {
                        let mut image_cache = IMAGE_CACHE.write().await;
                        let handle = image_cache.get(&status.account.avatar_static).await;
                        let reblog_handle = if let Some(reblog) = &status.reblog {
                            image_cache.get(&reblog.account.avatar_static).await.ok()
                        } else {
                            None
                        };

                        (status_id, handle.ok(), reblog_handle)
                    },
                    |(id, status_avatar, reblog_avatar)| {
                        cosmic::app::message::app(app::Message::Home(Message::SetAvatars(
                            id,
                            status_avatar,
                            reblog_avatar,
                        )))
                    },
                ));
            }
            Message::SetAvatars(id, status_avatar, reblog_avatar) => {
                let status = self.posts.iter_mut().find(|s| s.status.id == id);
                if let Some(status) = status {
                    status.status_avatar =
                        status_avatar.unwrap_or(image::Handle::from_bytes(vec![]));
                    status.reblog_avatar =
                        reblog_avatar.unwrap_or(image::Handle::from_bytes(vec![]));
                }
            }
            Message::Status(status_msg) => match status_msg {
                crate::widgets::status::Message::OpenProfile(account_id) => {
                    tracing::info!("open profile: {}", account_id)
                }
                crate::widgets::status::Message::ExpandStatus(status_id) => {
                    if let Some(post) = self.posts.iter().find(|s| s.status.id == status_id) {
                        tasks.push(cosmic::task::message(app::Message::ToggleContextPage(
                            ContextPage::Status(post.clone()),
                        )));
                    }
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
        }
        Task::batch(tasks)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![];
        if let Some(mastodon) = self.mastodon.clone() {
            subscriptions.push(crate::subscriptions::home::timeline(mastodon));
        }

        Subscription::batch(subscriptions)
    }
}
