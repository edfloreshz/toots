use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::Mastodon;

use crate::pages;

pub fn user_timeline(mastodon: Mastodon, skip: usize) -> Subscription<pages::home::Message> {
    Subscription::run_with_id(
        format!("timeline-{}-{}", skip, mastodon.data.base),
        stream::channel(1, move |mut output| async move {
            println!("{}", format!("timeline-{}-{}", skip, mastodon.data.base));

            // First fetch the timeline
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
                    .send(pages::home::Message::AppendStatus(status.clone()))
                    .await
                {
                    tracing::warn!("failed to send post: {}", err);
                }
            }

            std::future::pending().await
        }),
    )
}
