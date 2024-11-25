use std::collections::HashMap;

use cosmic::{
    iced::mouse::Interaction,
    widget::{self, image::Handle},
    Element,
};
use mastodon_async::prelude::{AccountId, Notification, Status, StatusId};

#[derive(Debug, Clone, PartialEq)]
pub struct StatusHandles {
    pub primary: Option<Handle>,
    pub secondary: Option<Handle>,
}

impl StatusHandles {
    pub fn new(primary: Option<&Handle>, secondary: Option<&Handle>) -> Self {
        Self {
            primary: primary.cloned(),
            secondary: secondary.cloned(),
        }
    }

    pub fn from_status(status: &Status, handles: &HashMap<String, Handle>) -> Self {
        let (primary, secondary) = (
            handles.get(&status.account.avatar.to_string()),
            status
                .reblog
                .as_ref()
                .map(|reblog| handles.get(&reblog.account.avatar.to_string()))
                .unwrap_or_default(),
        );
        Self {
            primary: primary.cloned(),
            secondary: secondary.cloned(),
        }
    }

    pub fn from_notification(
        notification: &Notification,
        handles: &HashMap<String, Handle>,
    ) -> Self {
        let (primary, secondary) = (
            handles.get(&notification.account.avatar.to_string()),
            notification
                .status
                .as_ref()
                .and_then(|status| handles.get(&status.account.avatar.to_string())),
        );
        Self {
            primary: primary.cloned(),
            secondary: secondary.cloned(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenProfile(AccountId),
    ExpandStatus(Status),
    Reply(StatusId),
    Favorite(StatusId),
    Boost(StatusId),
    Bookmark(StatusId),
    OpenLink(String),
}

pub fn status<'a>(status: &Status, handles: &StatusHandles) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let (status_avatar, reblog_avatar) = if status.reblog.is_some() {
        (handles.secondary.clone(), handles.primary.clone())
    } else {
        (handles.primary.clone(), handles.secondary.clone())
    };

    let reblog = status.reblog.as_ref().map(|_| {
        widget::button::custom(
            widget::row()
                .push(
                    reblog_avatar
                        .map(|avatar| widget::image(avatar).width(20).height(20))
                        .unwrap_or(crate::utils::fallback_avatar()),
                )
                .push(widget::text(format!(
                    "{} boosted",
                    status.account.display_name
                )))
                .spacing(spacing.space_xs),
        )
        .on_press(Message::OpenProfile(status.account.id.clone()))
    });

    let status = status.reblog.as_deref().unwrap_or(&status);
    let display_name = format!(
        "{} @{}",
        status.account.display_name, status.account.username
    );
    let content = html2text::config::rich()
        .string_from_read(status.content.as_bytes(), 700)
        .unwrap();

    let tags: Option<Element<_>> = (!status.tags.is_empty()).then(|| {
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

    let content = widget::column()
        .push_maybe(reblog)
        .push(
            widget::row()
                .push(
                    widget::button::image(status_avatar.unwrap_or(crate::utils::fallback_handle()))
                        .width(50)
                        .height(50)
                        .on_press(Message::OpenProfile(status.account.id.clone())),
                )
                .push(
                    widget::column()
                        .push(
                            widget::button::link(display_name)
                                .on_press(Message::OpenProfile(status.account.id.clone())),
                        )
                        .push(
                            widget::MouseArea::new(widget::text(content))
                                .interaction(Interaction::Pointer)
                                .on_press(Message::ExpandStatus(status.clone())),
                        )
                        .spacing(spacing.space_xxs),
                )
                .spacing(spacing.space_xs),
        )
        .push_maybe(tags)
        .push(
            widget::row()
                .push(
                    widget::button::icon(widget::icon::from_name("mail-replied-symbolic"))
                        .label(status.replies_count.unwrap_or_default().to_string())
                        .on_press(Message::Reply(status.id.clone())),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("emblem-shared-symbolic"))
                        .label(status.reblogs_count.to_string())
                        .on_press(Message::Boost(status.id.clone())),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("starred-symbolic"))
                        .label(status.favourites_count.to_string())
                        .class(if status.favourited.unwrap() {
                            cosmic::theme::Button::Link
                        } else {
                            cosmic::theme::Button::Standard
                        })
                        .on_press(Message::Favorite(status.id.clone())),
                )
                .push(
                    widget::button::icon(widget::icon::from_name("bookmark-new-symbolic"))
                        .on_press(Message::Bookmark(status.id.clone())),
                )
                .padding(spacing.space_xs)
                .spacing(spacing.space_xs),
        )
        .spacing(spacing.space_xs);

    widget::settings::flex_item_row(vec![content.into()])
        .padding(spacing.space_xs)
        .into()
}
