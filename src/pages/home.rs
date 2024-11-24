use std::collections::{HashSet, VecDeque};

use cosmic::{
    iced::{stream, Subscription},
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::prelude::{Mastodon, Status};

use crate::app::{self, ContextPage};

use super::IMAGE_LOADER;

#[derive(Debug, Clone)]
pub struct Home {
    pub mastodon: Option<Mastodon>,
    posts: VecDeque<Post>,
    post_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendPost(Post),
    PrependPost(Post),
    DeletePost(String),
    Status(crate::widgets::status::Message),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Post {
    pub status: Status,
    pub avatar: Option<widget::image::Handle>,
    pub reglog_avatar: Option<widget::image::Handle>,
}

impl Post {
    pub fn new(
        status: Status,
        avatar: Option<widget::image::Handle>,
        reglog_avatar: Option<widget::image::Handle>,
    ) -> Self {
        Self {
            status,
            avatar,
            reglog_avatar,
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
                crate::widgets::status(
                    post.status.clone(),
                    post.avatar.clone(),
                    post.reglog_avatar.clone(),
                )
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
            Message::AppendPost(post) => {
                if !self.post_ids.contains(&post.status.id.to_string()) {
                    self.posts.push_back(post.clone());
                    self.post_ids.insert(post.status.id.to_string());
                }
            }
            Message::PrependPost(post) => {
                if !self.post_ids.contains(&post.status.id.to_string()) {
                    self.posts.push_front(post.clone());
                    self.post_ids.insert(post.status.id.to_string());
                }
            }
            Message::DeletePost(id) => {
                self.post_ids.remove(&id);
                self.posts.retain(|post| post.status.id.to_string() != id)
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
            subscriptions.push(Subscription::run_with_id(
                "timeline",
                stream::channel(1, |mut output| async move {
                    tokio::task::spawn(async move {
                        let mut stream = Box::pin(
                            mastodon
                                .get_home_timeline()
                                .await
                                .unwrap()
                                .items_iter()
                                .take(100),
                        );
                        while let Some(status) = stream.next().await {
                            let handle = IMAGE_LOADER
                                .write()
                                .await
                                .get(&status.account.avatar_static)
                                .await;
                            let reblog_handle = if let Some(reblog) = &status.reblog {
                                IMAGE_LOADER
                                    .write()
                                    .await
                                    .get(&reblog.account.avatar_static)
                                    .await
                                    .ok()
                            } else {
                                None
                            };

                            if let Err(err) = output
                                .send(Message::AppendPost(Post::new(
                                    status,
                                    handle.ok(),
                                    reblog_handle,
                                )))
                                .await
                            {
                                tracing::warn!("failed to send set avatar: {}", err);
                            }
                        }
                    })
                    .await
                    .unwrap();

                    std::future::pending().await
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }
}
