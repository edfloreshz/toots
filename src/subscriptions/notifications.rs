use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::Mastodon;

use crate::pages;

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::notifications::Message> {
    Subscription::run_with_id(
        "notifications",
        stream::channel(1, |mut output| async move {
            tokio::task::spawn(async move {
                let mut stream = Box::pin(
                    mastodon
                        .notifications()
                        .await
                        .unwrap()
                        .items_iter()
                        .take(100),
                );
                while let Some(notification) = stream.next().await {
                    if let Err(err) = output
                        .send(pages::notifications::Message::AppendNotification(
                            notification.clone(),
                        ))
                        .await
                    {
                        tracing::warn!("failed to send post: {}", err);
                    }
                }
            })
            .await
            .unwrap();

            std::future::pending().await
        }),
    )
}
