use cosmic::{widget, Element};
use mastodon_async::{entities::notification::NotificationType, prelude::Notification};

use crate::utils::Cache;

#[derive(Debug, Clone)]
pub enum Message {
    Status(crate::widgets::status::Message),
}

pub fn notification<'a>(notification: &'a Notification, cache: &'a Cache) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;

    let display_name = notification.account.display_name.clone();

    let action = match notification.notification_type {
        NotificationType::Mention => format!("{} mentioned you", display_name),
        NotificationType::Reblog => format!("{} boosted", display_name),
        NotificationType::Favourite => format!("{} liked", display_name),
        NotificationType::Follow => {
            format!("{} followed you", display_name)
        }
        NotificationType::FollowRequest => format!("{} requested to follow you", display_name),
        NotificationType::Poll => {
            format!("{} created a poll", display_name)
        }
    };

    let action = widget::button::custom(
        widget::row()
            .push_maybe(
                cache
                    .handles
                    .get(&notification.account.avatar)
                    .map(|handle| widget::image(handle).width(20)),
            )
            .push(widget::text(action))
            .spacing(spacing.space_xs),
    )
    .on_press(Message::Status(
        crate::widgets::status::Message::OpenAccount(notification.account.clone()),
    ));

    let content = notification.status.as_ref().map(|status| {
        widget::container(crate::widgets::status(status, &cache).map(Message::Status))
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
