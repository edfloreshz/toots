use cosmic::iced::futures::channel::mpsc::Sender;
use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt};
use mastodon_async::{prelude::Notification, Mastodon};

use crate::pages;
use crate::utils::IMAGE_CACHE;

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::notifications::Message> {
    Subscription::run_with_id(
        "notifications",
        stream::channel(1, |mut output| async move {
            tokio::task::spawn(async move {
                let batch_size = 10;
                let mut batch = Vec::new();

                let mut stream = Box::pin(
                    mastodon
                        .notifications()
                        .await
                        .unwrap()
                        .items_iter()
                        .take(100),
                );
                while let Some(notification) = stream.next().await {
                    batch.push(notification.clone());

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

pub async fn process_avatars(
    notifications: &[Notification],
    output: &mut Sender<pages::notifications::Message>,
) {
    // Collect avatar URLs for the batch
    let avatar_urls: Vec<String> = notifications
        .iter()
        .flat_map(|notification| {
            let mut urls = vec![notification.account.avatar_static.clone()];
            if let Some(status) = &notification.status {
                urls.push(status.account.avatar_static.clone());
            }
            urls
        })
        .collect();

    // Fetch avatars for the batch
    let mut image_cache = IMAGE_CACHE.write().await;
    let avatar_handles = image_cache.get_batch(avatar_urls).await;

    // Send statuses with their avatars
    for notification in notifications {
        let avatar = avatar_handles
            .get(&notification.account.avatar_static)
            .cloned();

        let reblog_avatar = notification
            .status
            .as_ref()
            .and_then(|reblog| avatar_handles.get(&reblog.account.avatar_static))
            .cloned();

        if let Err(err) = output
            .send(pages::notifications::Message::AppendNotification(
                pages::notifications::Notification::new(
                    notification.clone(),
                    avatar,
                    reblog_avatar,
                ),
            ))
            .await
        {
            tracing::warn!("failed to send post: {}", err);
        }
    }
}
