use std::collections::{HashMap, VecDeque};

use cosmic::{
    iced::Subscription,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget::{self, image::Handle},
    Apply, Element, Task,
};
use mastodon_async::prelude::{Mastodon, Status};

use crate::{
    app::{self, ContextPage},
    utils,
    widgets::status::StatusHandles,
};

#[derive(Debug, Clone)]
pub struct Home {
    pub mastodon: Option<Mastodon>,
    statuses: VecDeque<Status>,
    handles: HashMap<String, Handle>,
    skip: usize,
    loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendPost(Status),
    PrependPost(Status),
    DeletePost(String),
    Status(crate::widgets::status::Message),
    ResolveAvatars(Status),
    SetAvatars(Status, Option<Handle>, Option<Handle>),
    LoadMore(bool),
}

impl Home {
    pub fn new() -> Self {
        Self {
            mastodon: None,
            statuses: VecDeque::new(),
            handles: HashMap::new(),
            skip: 0,
            loading: false,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let posts: Vec<Element<_>> = self
            .statuses
            .iter()
            .map(|status| {
                crate::widgets::status(status, &StatusHandles::from_status(status, &self.handles))
                    .map(Message::Status)
            })
            .collect();

        widget::scrollable(widget::settings::section().extend(posts))
            .direction(Direction::Vertical(
                Scrollbar::default().spacing(spacing.space_xxs),
            ))
            .on_scroll(|viewport| {
                Message::LoadMore(!self.loading && viewport.relative_offset().y >= 0.85)
            })
            .apply(widget::container)
            .max_width(700)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<app::Message> {
        let mut tasks = vec![];
        match message {
            Message::SetClient(mastodon) => self.mastodon = mastodon,
            Message::LoadMore(load) => {
                if !self.loading && load {
                    self.loading = true;
                    self.skip += 20;
                }
            }
            Message::AppendPost(status) => {
                self.loading = false;
                self.statuses.push_back(status.clone());
                if !self.handles.contains_key(&status.account.avatar) {
                    tasks.push(self.update(Message::ResolveAvatars(status)));
                }
            }
            Message::PrependPost(status) => {
                self.loading = false;
                self.statuses.push_front(status.clone());
                if !self.handles.contains_key(&status.account.avatar) {
                    tasks.push(self.update(Message::ResolveAvatars(status)));
                }
            }
            Message::DeletePost(id) => self.statuses.retain(|status| status.id.to_string() != id),
            Message::ResolveAvatars(status) => {
                let handles = StatusHandles::from_status(&status, &self.handles);
                match (handles.primary, handles.secondary) {
                    (None, None) => {
                        tasks.push(Task::perform(
                            async {
                                let handle = utils::get_image(&status.account.avatar).await;
                                let reblog_handle = if let Some(reblog) = &status.reblog {
                                    utils::get_image(&reblog.account.avatar).await.ok()
                                } else {
                                    None
                                };

                                (status, handle, reblog_handle)
                            },
                            |(status, status_avatar, reblog_avatar)| {
                                cosmic::app::message::app(app::Message::Home(Message::SetAvatars(
                                    status.clone(),
                                    status_avatar.ok(),
                                    reblog_avatar,
                                )))
                            },
                        ));
                    }
                    (status_avatar, reblog_avatar) => tasks.push(self.update(Message::SetAvatars(
                        status.clone(),
                        status_avatar,
                        reblog_avatar,
                    ))),
                }
            }
            Message::SetAvatars(status, status_avatar, reblog_avatar) => {
                if let Some(status_avatar) = status_avatar {
                    self.handles.insert(status.account.avatar, status_avatar);
                }
                if let Some(reblog_avatar) = reblog_avatar {
                    self.handles
                        .insert(status.reblog.unwrap().account.avatar, reblog_avatar);
                }
            }
            Message::Status(status_msg) => match status_msg {
                crate::widgets::status::Message::OpenProfile(account_id) => {
                    tracing::info!("open profile: {}", account_id)
                }
                crate::widgets::status::Message::ExpandStatus(status) => {
                    if let Some(status) = self.statuses.iter().find(|status| status.id == status.id)
                    {
                        tasks.push(cosmic::task::message(app::Message::ToggleContextPage(
                            ContextPage::Status((
                                status.clone(),
                                StatusHandles::from_status(&status, &self.handles),
                            )),
                        )));
                    }
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
        }
        Task::batch(tasks)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.mastodon.clone() {
            Some(mastodon) => Subscription::batch(vec![crate::subscriptions::home::timeline(
                mastodon, self.skip,
            )]),
            None => Subscription::none(),
        }
    }
}
