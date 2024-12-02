use std::str::FromStr;

use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::Mastodon;
use reqwest::Url;

use crate::{pages, utils};

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::notifications::Message> {
    Subscription::run_with_id(
        format!("notifications-{}", mastodon.data.base),
        stream::channel(1, |mut output| async move {
            println!("{}", format!("notifications-{}", mastodon.data.base));
            let mut stream = Box::pin(
                mastodon
                    .notifications()
                    .await
                    .unwrap()
                    .items_iter()
                    .take(100),
            );

            let mut urls = Vec::new();
            while let Some(notification) = stream.next().await {
                urls.push(notification.account.avatar.clone());
                urls.push(notification.account.header.clone());

                if let Some(status) = &notification.status {
                    urls.push(status.account.avatar.clone());
                    urls.push(status.account.header.clone());
                    if let Some(card) = &status.card {
                        if let Some(image) = &card.image {
                            if let Ok(url) = Url::from_str(image) {
                                urls.push(url);
                            }
                        }
                    }
                    for attachment in &status.media_attachments {
                        urls.push(attachment.preview_url.clone());
                    }
                }

                if let Err(err) = output
                    .send(pages::notifications::Message::AppendNotification(
                        notification.clone(),
                    ))
                    .await
                {
                    tracing::warn!("failed to send post: {}", err);
                }

                // Fetch and send each image
                for url in &urls {
                    match utils::get(&url).await {
                        Ok(handle) => {
                            if let Err(err) = output
                                .send(pages::notifications::Message::CacheHandle(
                                    url.clone(),
                                    handle,
                                ))
                                .await
                            {
                                tracing::error!("Failed to send image handle: {}", err);
                            }
                        }
                        Err(err) => {
                            tracing::error!("Failed to fetch image: {}", err);
                        }
                    }
                }
                urls.clear();
            }

            std::future::pending().await
        }),
    )
}
