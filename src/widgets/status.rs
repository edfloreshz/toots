use cosmic::{
    iced::{mouse::Interaction, Alignment},
    iced_widget::scrollable::{Direction, Scrollbar},
    widget, Apply, Element, Task,
};
use mastodon_async::{
    prelude::{Account, Status, StatusId},
    NewStatus,
};

use crate::{app, utils::Cache};

#[derive(Debug, Clone)]
pub enum Message {
    OpenAccount(Account),
    ExpandStatus(StatusId),
    Reply(StatusId, String),
    Favorite(StatusId, bool),
    Boost(StatusId, bool),
    OpenLink(String),
}

pub struct StatusOptions {
    media: bool,
    tags: bool,
    actions: bool,
    expand: bool,
}

impl StatusOptions {
    pub fn new(media: bool, tags: bool, actions: bool, expand: bool) -> Self {
        Self {
            media,
            tags,
            actions,
            expand,
        }
    }

    pub fn all() -> StatusOptions {
        StatusOptions::new(true, true, true, true)
    }

    pub fn none() -> StatusOptions {
        StatusOptions::new(false, false, false, false)
    }
}

pub fn status<'a>(
    status: &'a Status,
    options: StatusOptions,
    cache: &'a Cache,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let status = if let Some(reblog) = &status.reblog {
        let reblog = cache.statuses.get(&reblog.id.to_string()).unwrap_or(reblog);

        let indicator = widget::button::custom(
            widget::row()
                .push(
                    cache
                        .handles
                        .get(&status.account.avatar)
                        .map(|avatar| widget::image(avatar).width(20).height(20))
                        .unwrap_or(crate::utils::fallback_avatar().width(20).height(20)),
                )
                .push(widget::text(format!(
                    "{} boosted",
                    status.account.display_name
                )))
                .spacing(spacing.space_xs),
        )
        .on_press(Message::OpenAccount(reblog.account.clone()));

        widget::column().push(indicator).push(
            self::status(&*reblog, options, cache)
                .apply(widget::container)
                .class(cosmic::theme::Container::Dialog),
        )
    } else {
        let display_name = format!(
            "{} @{}",
            status.account.display_name, status.account.username
        );

        let mut content: Element<_> = widget::text(
            html2text::config::rich()
                .string_from_read(status.content.as_bytes(), 700)
                .unwrap(),
        )
        .into();

        if options.expand {
            content = widget::MouseArea::new(content)
                .on_press(Message::ExpandStatus(status.id.clone()))
                .interaction(Interaction::Pointer)
                .into();
        }

        let header = widget::row()
            .push(
                widget::button::image(
                    cache
                        .handles
                        .get(&status.account.avatar)
                        .cloned()
                        .unwrap_or(crate::utils::fallback_handle()),
                )
                .width(50)
                .height(50)
                .on_press(Message::OpenAccount(status.account.clone())),
            )
            .push(
                widget::button::link(display_name)
                    .on_press(Message::OpenAccount(status.account.clone())),
            )
            .align_y(Alignment::Center)
            .spacing(spacing.space_xs);

        let tags: Option<Element<_>> = (!status.tags.is_empty() && options.tags).then(|| {
            widget::row()
                .spacing(spacing.space_xxs)
                .extend(
                    status
                        .tags
                        .iter()
                        .map(|tag| {
                            widget::button::suggested(format!("#{}", tag.name.clone()))
                                .on_press(Message::OpenLink(tag.url.clone()))
                                .into()
                        })
                        .collect::<Vec<Element<Message>>>(),
                )
                .into()
        });

        let attachments = status
            .media_attachments
            .iter()
            .map(|media| {
                widget::button::image(
                    cache
                        .handles
                        .get(&media.preview_url.to_string())
                        .cloned()
                        .unwrap_or(crate::utils::fallback_handle()),
                )
                .on_press_maybe(media.url.as_ref().cloned().map(Message::OpenLink))
                .into()
            })
            .collect::<Vec<Element<Message>>>();

        let media = (!status.media_attachments.is_empty() && options.media).then_some({
            widget::scrollable(widget::row().extend(attachments).spacing(spacing.space_xxs))
                .direction(Direction::Horizontal(Scrollbar::new()))
        });

        let actions = (options.actions).then_some({
            widget::row()
                .push(
                    widget::button::icon(widget::icon::from_name("mail-replied-symbolic"))
                        .label(status.replies_count.unwrap_or_default().to_string())
                        .on_press(Message::Reply(
                            status.id.clone(),
                            status.account.username.clone(),
                        )),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("emblem-shared-symbolic"))
                        .label(status.reblogs_count.to_string())
                        .class(if status.reblogged.unwrap() {
                            cosmic::theme::Button::Suggested
                        } else {
                            cosmic::theme::Button::Icon
                        })
                        .on_press(Message::Boost(status.id.clone(), status.reblogged.unwrap())),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("starred-symbolic"))
                        .label(status.favourites_count.to_string())
                        .class(if status.favourited.unwrap() {
                            cosmic::theme::Button::Suggested
                        } else {
                            cosmic::theme::Button::Icon
                        })
                        .on_press(Message::Favorite(
                            status.id.clone(),
                            status.favourited.unwrap(),
                        )),
                )
                .spacing(spacing.space_xs)
        });

        widget::column()
            .push(header)
            .push(content)
            .push_maybe(media)
            .push_maybe(tags)
            .push_maybe(actions)
    };

    widget::settings::flex_item_row(vec![status
        .padding(spacing.space_xs)
        .spacing(spacing.space_xs)
        .into()])
    .into()
}

pub fn update(message: Message) -> Task<app::Message> {
    match message {
        Message::OpenAccount(account) => cosmic::task::message(app::Message::ToggleContextPage(
            app::ContextPage::Account(account),
        )),
        Message::ExpandStatus(id) => cosmic::task::message(app::Message::ToggleContextPage(
            app::ContextPage::Status(id),
        )),
        Message::Reply(status_id, username) => {
            let mut new_status = NewStatus::default();
            new_status.in_reply_to_id = Some(status_id.to_string());
            new_status.status = Some(format!("@{} ", username));
            cosmic::task::message(app::Message::Dialog(app::DialogAction::Open(
                app::Dialog::Reply(new_status),
            )))
        }
        Message::Favorite(status_id, favorited) => cosmic::task::message(app::Message::Status(
            Message::Favorite(status_id, favorited),
        )),
        Message::Boost(status_id, boosted) => {
            cosmic::task::message(app::Message::Status(Message::Boost(status_id, boosted)))
        }
        Message::OpenLink(url) => cosmic::task::message(app::Message::Open(url)),
    }
}
