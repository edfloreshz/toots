use cosmic::{widget, Element};
use mastodon_async::prelude::{notification::Type, Notification};

use crate::utils::{self, Cache};

use super::status::StatusOptions;

#[derive(Debug, Clone)]
pub enum Message {
    Status(crate::widgets::status::Message),
}

pub fn notification<'a>(notification: &'a Notification, cache: &'a Cache) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let display_name = notification.account.display_name.clone();

    let action = match notification.notification_type {
        Type::Mention => format!("{} mentioned you", display_name),
        Type::Reblog => format!("{} boosted", display_name),
        Type::Favourite => format!("{} liked", display_name),
        Type::Follow => {
            format!("{} followed you", display_name)
        }
        Type::FollowRequest => format!("{} requested to follow you", display_name),
        Type::Poll => {
            format!("{} created a poll", display_name)
        }
        Type::Status => format!("{} has posted a status", display_name),
        Type::Update => "A post has been edited".to_string(),
        Type::SignUp => "Someone signed up (optionally sent to admins)".to_string(),
        Type::Report => "A new report has been filed".to_string(),
    };

    let action = widget::button::custom(
        widget::row()
            .push(
                cache
                    .handles
                    .get(&notification.account.avatar)
                    .map(|handle| widget::image(handle).width(20))
                    .unwrap_or(utils::fallback_avatar().width(20)),
            )
            .push(widget::text(action))
            .spacing(spacing.space_xs),
    )
    .on_press(Message::Status(
        crate::widgets::status::Message::OpenAccount(notification.account.clone()),
    ));

    let content = notification.status.as_ref().map(|status| {
        widget::container(
            crate::widgets::status(status, StatusOptions::new(false, true, false, true), cache)
                .map(Message::Status),
        )
        .padding(spacing.space_xxs)
        .class(cosmic::theme::Container::Dialog)
    });

    let content = widget::column()
        .push(action)
        .push_maybe(content)
        .spacing(spacing.space_xs);

    widget::settings::flex_item_row(vec![content.into()])
        .padding(spacing.space_xs)
        .into()
}
