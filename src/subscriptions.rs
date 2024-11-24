use crate::pages;
use crate::utils::IMAGE_CACHE;
use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use mastodon_async::entities::event::Event;
use mastodon_async::Mastodon;

use crate::app;

pub fn mastodon_user_events(mastodon: Mastodon) -> Subscription<app::Message> {
    Subscription::run_with_id(
        "posts",
        stream::channel(1, |output| async move {
            let stream = mastodon.stream_user().await.unwrap();
            stream
                .try_for_each(|(event, _client)| {
                    let mut output = output.clone();
                    async move {
                        let mut image_cache = IMAGE_CACHE.write().await;
                        match event {
                            Event::Update(ref status) => {
                                let handle = image_cache.get(&status.account.avatar_static).await;
                                let reblog_handle = if let Some(reblog) = &status.reblog {
                                    image_cache.get(&reblog.account.avatar_static).await.ok()
                                } else {
                                    None
                                };
                                if let Err(err) = output
                                    .send(app::Message::Home(pages::home::Message::PrependPost(
                                        pages::home::Post::new(
                                            status.clone(),
                                            handle.ok(),
                                            reblog_handle,
                                        ),
                                    )))
                                    .await
                                {
                                    tracing::warn!("failed to send post: {}", err);
                                }
                            }
                            Event::Notification(ref notification) => {
                                let handle =
                                    image_cache.get(&notification.account.avatar_static).await;
                                let reblog_handle = if let Some(status) = &notification.status {
                                    image_cache.get(&status.account.avatar_static).await.ok()
                                } else {
                                    None
                                };
                                if let Err(err) = output
                                    .send(app::Message::Notifications(
                                        pages::notifications::Message::PrependNotification(
                                            pages::notifications::Notification::new(
                                                notification.clone(),
                                                handle.ok(),
                                                reblog_handle,
                                            ),
                                        ),
                                    ))
                                    .await
                                {
                                    tracing::warn!("failed to send post: {}", err);
                                }
                            }
                            Event::Delete(ref id) => {
                                if let Err(err) = output
                                    .send(app::Message::Home(pages::home::Message::DeletePost(
                                        id.clone(),
                                    )))
                                    .await
                                {
                                    tracing::warn!("failed to send post: {}", err);
                                }
                            }
                            Event::FiltersChanged => (),
                        };
                        Ok(())
                    }
                })
                .await
                .unwrap();

            std::future::pending().await
        }),
    )
}

pub fn mastodon_home_timeline(mastodon: Mastodon) -> Subscription<pages::home::Message> {
    Subscription::run_with_id(
        "timeline",
        stream::channel(1, |mut output| async move {
            tokio::task::spawn(async move {
                let mut image_cache = IMAGE_CACHE.write().await;
                let mut stream = Box::pin(
                    mastodon
                        .get_home_timeline()
                        .await
                        .unwrap()
                        .items_iter()
                        .take(100),
                );
                while let Some(status) = stream.next().await {
                    let handle = image_cache.get(&status.account.avatar_static).await;
                    let reblog_handle = if let Some(reblog) = &status.reblog {
                        image_cache.get(&reblog.account.avatar_static).await.ok()
                    } else {
                        None
                    };

                    if let Err(err) = output
                        .send(pages::home::Message::AppendPost(pages::home::Post::new(
                            status,
                            handle.ok(),
                            reblog_handle,
                        )))
                        .await
                    {
                        tracing::warn!("failed to send set avatar: {}", err);
                    }
                }
            })
            .await
            .unwrap();

            std::future::pending().await
        }),
    )
}

pub fn mastodon_notifications_timeline(
    mastodon: Mastodon,
) -> Subscription<pages::notifications::Message> {
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
                let mut image_cache = IMAGE_CACHE.write().await;
                while let Some(notification) = stream.next().await {
                    let handle = image_cache.get(&notification.account.avatar_static).await;
                    let reblog_handle = if let Some(status) = &notification.status {
                        image_cache.get(&status.account.avatar_static).await.ok()
                    } else {
                        None
                    };

                    if let Err(err) = output
                        .send(pages::notifications::Message::AppendNotification(
                            pages::notifications::Notification::new(
                                notification,
                                handle.ok(),
                                reblog_handle,
                            ),
                        ))
                        .await
                    {
                        tracing::warn!("failed to send set avatar: {}", err);
                    }
                }
            })
            .await
            .unwrap();

            std::future::pending().await
        }),
    )
}
