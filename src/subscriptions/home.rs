use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::Mastodon;

use crate::pages;

pub fn timeline(mastodon: Mastodon, skip: usize) -> Subscription<pages::home::Message> {
    Subscription::run_with_id(
        format!("timeline-{}", skip),
        stream::channel(1, move |mut output| async move {
            let mut stream = Box::pin(
                mastodon
                    .get_home_timeline()
                    .await
                    .unwrap()
                    .items_iter()
                    .skip(skip)
                    .take(20),
            );

            while let Some(status) = stream.next().await {
                if let Err(err) = output
                    .send(pages::home::Message::AppendPost(status.clone()))
                    .await
                {
                    tracing::warn!("failed to send post: {}", err);
                }
            }

            std::future::pending().await
        }),
    )
}
