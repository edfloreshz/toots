use std::collections::VecDeque;

use cosmic::{
    iced::{stream, Subscription},
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use mastodon_async::{
    entities::event::Event,
    prelude::{Mastodon, Status},
};

use crate::app;

use super::IMAGE_LOADER;

#[derive(Debug, Clone)]
pub struct Home {
    mastodon: Option<Mastodon>,
    posts: VecDeque<Post>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendPost(Post),
    PrependPost(Post),
    DeletePost(String),
}

#[derive(Debug, Clone)]
pub struct Post {
    status: Status,
    avatar: Option<widget::image::Handle>,
    reglog_avatar: Option<widget::image::Handle>,
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
            mastodon: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let posts: Vec<Element<_>> = self
            .posts
            .iter()
            .map(|post| {
                Self::status(
                    post.status.clone(),
                    post.avatar.clone(),
                    post.reglog_avatar.clone(),
                )
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
        let tasks = vec![];
        match message {
            Message::SetClient(mastodon) => self.mastodon = mastodon,
            Message::AppendPost(post) => self.posts.push_back(post),
            Message::PrependPost(post) => self.posts.push_front(post),
            Message::DeletePost(id) => self.posts.retain(|post| post.status.id.to_string() != id),
        }
        Task::batch(tasks)
    }

    pub fn subscription(&self, valid_page: bool) -> Subscription<Message> {
        let mut subscriptions = vec![];
        if valid_page {
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

            if let Some(mastodon) = self.mastodon.clone() {
                subscriptions.push(Subscription::run_with_id(
                    "posts",
                    stream::channel(1, |output| async move {
                        let stream = mastodon.stream_user().await.unwrap();
                        stream
                            .try_for_each(|(event, _client)| {
                                let mut output = output.clone();
                                async move {
                                    match event {
                                        Event::Update(ref status) => {
                                            let handle = IMAGE_LOADER
                                                .write()
                                                .await
                                                .get(&status.account.avatar_static)
                                                .await;
                                            let reblog_handle = if let Some(reblog) = &status.reblog
                                            {
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
                                                .send(Message::PrependPost(Post::new(
                                                    status.clone(),
                                                    handle.ok(),
                                                    reblog_handle,
                                                )))
                                                .await
                                            {
                                                tracing::warn!("failed to send post: {}", err);
                                            }
                                        }
                                        Event::Notification(ref _notification) => (),
                                        Event::Delete(ref id) => {
                                            if let Err(err) =
                                                output.send(Message::DeletePost(id.clone())).await
                                            {
                                                tracing::warn!("failed to send post: {}", err);
                                            }
                                        }
                                        Event::FiltersChanged => (),
                                    };
                                    Ok(())
                                }
                            })
                            .await
                            .unwrap();

                        std::future::pending().await
                    }),
                ));
            }
        }

        Subscription::batch(subscriptions)
    }

    fn status<'a>(
        status: Status,
        avatar: Option<widget::image::Handle>,
        reblog_avatar: Option<widget::image::Handle>,
    ) -> Element<'a, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let column = if let Some(reblog) = status.reblog {
            let display_name = format!("{} boosted", status.account.display_name);
            widget::column()
                .push(
                    widget::button::custom(
                        widget::row()
                            .push_maybe(avatar.map(|handle| widget::image(handle).width(20)))
                            .push(widget::text(display_name))
                            .spacing(spacing.space_xs),
                    )
                    .padding(spacing.space_xxxs),
                )
                .push(Self::status(*reblog.clone(), reblog_avatar, None))
                .spacing(spacing.space_xs)
                .apply(widget::container)
        } else {
            let display_name = format!(
                "{} @{}",
                status.account.display_name, status.account.username
            );
            let content = html2text::config::rich()
                .string_from_read(status.content.as_bytes(), 700)
                .unwrap();
            widget::row()
                .push_maybe(avatar.map(|handle| widget::image(handle).width(50)))
                .push(
                    widget::column()
                        .push(widget::text(display_name).class(cosmic::style::Text::Accent))
                        .push(widget::text(content))
                        .spacing(spacing.space_xxs),
                )
                .spacing(spacing.space_xs)
                .apply(widget::container)
        };

        widget::settings::flex_item_row(vec![column.into()])
            .padding(spacing.space_xs)
            .into()
    }
}
