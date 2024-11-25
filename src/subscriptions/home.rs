use crate::utils::IMAGE_CACHE;
use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::prelude::Status;
use mastodon_async::Mastodon;

use cosmic::iced::futures::channel::mpsc::Sender;

use crate::pages;

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::home::Message> {
    Subscription::run_with_id(
        "timeline",
        stream::channel(1, |mut output| async move {
            let batch_size = 10;
            let mut batch = Vec::new();
            tokio::task::spawn(async move {
                let mut stream = Box::pin(
                    mastodon
                        .get_home_timeline()
                        .await
                        .unwrap()
                        .items_iter()
                        .take(100),
                );
                while let Some(status) = stream.next().await {
                    batch.push(status.clone());

                    if batch.len() >= batch_size {
                        process_avatars(&batch, &mut output).await;
                        batch.clear();
                    }

                    if !batch.is_empty() {
                        process_avatars(&batch, &mut output).await;
                    }
                }
            })
            .await
            .unwrap();

            std::future::pending().await
        }),
    )
}

pub async fn process_avatars(statuses: &[Status], output: &mut Sender<pages::home::Message>) {
    // Collect avatar URLs for the batch
    let avatar_urls: Vec<String> = statuses
        .iter()
        .flat_map(|status| {
            let mut urls = vec![status.account.avatar_static.clone()];
            if let Some(reblog) = &status.reblog {
                urls.push(reblog.account.avatar_static.clone());
            }
            urls
        })
        .collect();

    // Fetch avatars for the batch
    let mut image_cache = IMAGE_CACHE.write().await;
    let avatar_handles = image_cache.get_batch(avatar_urls).await;

    // Send statuses with their avatars
    for status in statuses {
        let avatar = avatar_handles.get(&status.account.avatar_static).cloned();

        let reblog_avatar = status
            .reblog
            .as_ref()
            .and_then(|reblog| avatar_handles.get(&reblog.account.avatar_static))
            .cloned();

        if let Err(err) = output
            .send(pages::home::Message::AppendPost(pages::home::Post::new(
                status.clone(),
                avatar,
                reblog_avatar,
            )))
            .await
        {
            tracing::warn!("failed to send post: {}", err);
        }
    }
}
