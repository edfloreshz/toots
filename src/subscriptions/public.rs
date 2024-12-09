use cosmic::iced::{stream, Subscription};
use futures_util::SinkExt;
use mastodon_async::Mastodon;

use crate::pages;

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::public::Message> {
    Subscription::run_with_id(
        format!("public-timeline-{}", mastodon.data.base),
        stream::channel(1, move |mut output| async move {
            match mastodon.get_public_timeline(false, false).await {
                Ok(statuses) => {
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
                            .await
                        {
                            tracing::warn!("failed to send post: {}", err);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("failed to get local timeline: {}", err);
                }
            }

            std::future::pending().await
        }),
    )
}

pub fn local_timeline(mastodon: Mastodon) -> Subscription<pages::public::Message> {
    Subscription::run_with_id(
        format!("local-timeline-{}", mastodon.data.base),
        stream::channel(1, move |mut output| async move {
            match mastodon.get_public_timeline(true, false).await {
                Ok(statuses) => {
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
                            .await
                        {
                            tracing::warn!("failed to send post: {}", err);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("failed to get local timeline: {}", err);
                }
            }

            std::future::pending().await
        }),
    )
}

pub fn remote_timeline(mastodon: Mastodon) -> Subscription<pages::public::Message> {
    Subscription::run_with_id(
        format!("remote-timeline-{}", mastodon.data.base),
        stream::channel(1, move |mut output| async move {
            match mastodon.get_public_timeline(false, true).await {
                Ok(statuses) => {
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
                            .await
                        {
                            tracing::warn!("failed to send post: {}", err);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("failed to get local timeline: {}", err);
                }
            }

            std::future::pending().await
        }),
    )
}
