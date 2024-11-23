use cosmic::{widget, Apply, Element, Task};
use futures_util::StreamExt;
use mastodon_async::prelude::*;

use crate::app;

#[derive(Debug, Clone)]
pub struct Home {
    statuses: Vec<AvatarStatus>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FetchTimeline(Option<Mastodon>),
    FetchAvatars(Vec<Status>),
    SetTimeline(Vec<AvatarStatus>),
}

#[derive(Debug, Clone)]
pub struct AvatarStatus {
    status: Status,
    avatar: Option<widget::image::Handle>,
}

impl AvatarStatus {
    pub fn new(status: Status, avatar: Option<widget::image::Handle>) -> Self {
        Self { status, avatar }
    }
}

impl Home {
    pub fn new() -> Self {
        Self {
            statuses: Vec::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut list = widget::list_column();
        for status in &self.statuses {
            list = list.add(Self::status(status.status.clone(), status.avatar.clone()));
        }
        widget::scrollable(list)
            .apply(widget::container)
            .max_width(1000.)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<app::Message> {
        let mut tasks = vec![];
        match message {
            Message::FetchTimeline(mastodon) => {
                if let Some(mastodon) = mastodon {
                    let task = Task::perform(
                        async move {
                            mastodon
                                .get_home_timeline()
                                .await
                                .unwrap()
                                .items_iter()
                                .take(100)
                                .collect::<Vec<Status>>()
                                .await
                        },
                        |rows| {
                            cosmic::app::message::app(app::Message::Home(Message::FetchAvatars(
                                rows,
                            )))
                        },
                    );
                    tasks.push(task);
                }
            }
            Message::FetchAvatars(statuses) => {
                let task = Task::perform(
                    async move {
                        let mut avatar_statuses = Vec::new();
                        for status in statuses {
                            // let status = match reqwest::get(&status.account.avatar).await {
                            //     Ok(response) => {
                            //         if response.status().is_success() {
                            //             match response.bytes().await {
                            //                 Ok(bytes) => {
                            //                     let handle =
                            //                         image::Handle::from_bytes(bytes.to_vec());
                            //                     AvatarStatus::new(status, Some(handle))
                            //                 }
                            //                 Err(err) => {
                            //                     tracing::error!("{err}");
                            //                     AvatarStatus::new(status, None)
                            //                 }
                            //             }
                            //         } else {
                            //             AvatarStatus::new(status, None)
                            //         }
                            //     }
                            //     Err(err) => {
                            //         tracing::error!("{err}");
                            //         AvatarStatus::new(status, None)
                            //     }
                            // };
                            avatar_statuses.push(AvatarStatus::new(status, None));
                        }
                        avatar_statuses
                    },
                    |rows| {
                        cosmic::app::message::app(app::Message::Home(Message::SetTimeline(rows)))
                    },
                );
                tasks.push(task);
            }
            Message::SetTimeline(statuses) => {
                self.statuses = statuses;
            }
        }
        Task::batch(tasks)
    }

    fn status<'a>(status: Status, avatar: Option<widget::image::Handle>) -> Element<'a, Message> {
        let username = widget::text(status.account.username);

        let mut items = vec![widget::column()
            .push(username)
            .push(widget::text(
                html2text::from_read(status.content.as_bytes(), 1000).unwrap(),
            ))
            .into()];

        if let Some(avatar) = avatar {
            items.insert(0, widget::image(avatar).width(40.).into());
        }

        widget::settings::flex_item_row(items).into()
    }
}
