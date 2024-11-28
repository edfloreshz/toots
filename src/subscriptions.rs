use crate::pages;
use cosmic::iced::{stream, Subscription};
use futures_util::{SinkExt, TryStreamExt};
use mastodon_async::entities::event::Event;
use mastodon_async::Mastodon;

use crate::app;

pub mod home;
pub mod notifications;
pub mod public;

pub fn stream_user_events(mastodon: Mastodon) -> Subscription<app::Message> {
    Subscription::run_with_id(
        "posts",
        stream::channel(1, |output| async move {
            let stream = mastodon.stream_user().await.unwrap();
            stream
                .try_for_each(|(event, _client)| {
                    let mut output = output.clone();
                    async move {
                        match event {
                            Event::Update(ref status) => {
                                if let Err(err) = output
                                    .send(app::Message::Home(pages::home::Message::PrependStatus(
                                        status.clone(),
                                    )))
                                    .await
                                {
                                    tracing::warn!("failed to send post: {}", err);
                                }
                            }
                            Event::Notification(ref notification) => {
                                if let Err(err) = output
                                    .send(app::Message::Notifications(
                                        pages::notifications::Message::PrependNotification(
                                            notification.clone(),
                                        ),
                                    ))
                                    .await
                                {
                                    tracing::warn!("failed to send post: {}", err);
                                }
                            }
                            Event::Delete(ref id) => {
                                if let Err(err) = output
                                    .send(app::Message::Home(pages::home::Message::DeleteStatus(
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
