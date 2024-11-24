use cosmic::{iced::mouse::Interaction, widget, Apply, Element};
use mastodon_async::prelude::{AccountId, Status, StatusId};

#[derive(Debug, Clone)]
pub enum Message {
    OpenProfile(AccountId),
    ExpandStatus(StatusId),
    Reply(StatusId),
    Favorite(StatusId),
    Boost(StatusId),
    Bookmark(StatusId),
}

pub fn status<'a>(
    status: Status,
    avatar: Option<widget::image::Handle>,
    reblog_avatar: Option<widget::image::Handle>,
) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let (avatar, reblog_avatar) = if status.reblog.is_some() {
        (reblog_avatar.clone(), avatar.clone())
    } else {
        (avatar, reblog_avatar)
    };

    let reblog = status.reblog.as_ref().map(|_| {
        widget::button::custom(
            widget::row()
                .push_maybe(reblog_avatar.map(|handle| widget::image(handle).width(20)))
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

    let content = widget::column()
        .push_maybe(reblog)
        .push(
            widget::row()
                .push_maybe(avatar.map(|handle| {
                    widget::button::image(handle)
                        .width(50)
                        .on_press(Message::OpenProfile(status.account.id.clone()))
                }))
                .push(
                    widget::column()
                        .push(
                            widget::button::link(display_name)
                                .on_press(Message::OpenProfile(status.account.id.clone())),
                        )
                        .push(
                            widget::MouseArea::new(widget::text(content))
                                .interaction(Interaction::Pointer)
                                .on_press(Message::ExpandStatus(status.id.clone())),
                        )
                        .spacing(spacing.space_xxs),
                )
                .spacing(spacing.space_xs),
        )
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
                .spacing(spacing.space_xs),
        )
        .spacing(spacing.space_xs)
        .apply(widget::container);

    widget::settings::flex_item_row(vec![content.into()])
        .padding(spacing.space_xs)
        .into()
}
