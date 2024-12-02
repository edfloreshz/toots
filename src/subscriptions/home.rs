use std::str::FromStr;

use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::Mastodon;
use reqwest::Url;

use crate::{pages, utils};

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

            let mut urls = Vec::new();
            while let Some(status) = stream.next().await {
                if let Err(err) = output
                    .send(pages::home::Message::AppendStatus(status.clone()))
                    .await
                {
                    tracing::warn!("failed to send post: {}", err);
                }

                urls.push(status.account.avatar.clone());
                urls.push(status.account.header.clone());

                if let Some(reblog) = &status.reblog {
                    urls.push(reblog.account.avatar.clone());
                    urls.push(reblog.account.header.clone());
                    if let Some(card) = &reblog.card {
                        if let Some(image) = &card.image {
                            if let Ok(url) = Url::from_str(image) {
                                urls.push(url);
                            }
                        }
                    }
                    for attachment in &reblog.media_attachments {
                        urls.push(attachment.preview_url.clone());
                    }
                }

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

                for url in &urls {
                    match utils::get(&url).await {
                        Ok(handle) => {
                            if let Err(err) = output
                                .send(pages::home::Message::CacheHandle(url.clone(), handle))
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
