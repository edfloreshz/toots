use std::collections::VecDeque;

use cosmic::{
    iced::Subscription,
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::prelude::{Mastodon, Status, StatusId};

use crate::{
    app,
    utils::Cache,
    widgets::{self, status::StatusHandles},
};

#[derive(Debug, Clone)]
pub struct Home {
    pub mastodon: Option<Mastodon>,
    statuses: VecDeque<StatusId>,
    skip: usize,
    loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetClient(Option<Mastodon>),
    AppendStatus(Status),
    PrependStatus(Status),
    DeleteStatus(String),
    Status(crate::widgets::status::Message),
    LoadMore(bool),
}

impl Home {
    pub fn new() -> Self {
        Self {
            mastodon: None,
            statuses: VecDeque::new(),
            skip: 0,
            loading: false,
        }
    }

    pub fn view(&self, cache: &Cache) -> Element<Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let statuses: Vec<Element<_>> = self
            .statuses
            .iter()
            .filter_map(|id| cache.statuses.get(&id.to_string()))
            .map(|status| {
                crate::widgets::status(status, &StatusHandles::from_status(status, &cache.handles))
                    .map(Message::Status)
            })
            .collect();

        widget::scrollable(widget::settings::section().extend(statuses))
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
            Message::AppendStatus(status) => {
                self.loading = false;
                self.statuses.push_back(status.id.clone());
                tasks.push(cosmic::task::message(app::Message::CachceStatus(status)));
            }
            Message::PrependStatus(status) => {
                self.loading = false;
                self.statuses.push_front(status.id.clone());
                tasks.push(cosmic::task::message(app::Message::CachceStatus(status)));
            }
            Message::DeleteStatus(id) => self
                .statuses
                .retain(|status_id| *status_id.to_string() != id),
            Message::Status(message) => tasks.push(widgets::status::update(message)),
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
