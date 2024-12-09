use std::str::FromStr;

use cosmic::iced::{stream, Subscription};
use futures_util::SinkExt;
use mastodon_async::Mastodon;
use reqwest::Url;

use crate::pages;

pub fn timeline(mastodon: Mastodon) -> Subscription<pages::public::Message> {
    Subscription::run_with_id(
        format!("public-timeline-{}", mastodon.data.base),
        stream::channel(1, move |mut output| async move {
            match mastodon.get_public_timeline(false, false).await {
                Ok(statuses) => {
                    let mut urls = Vec::new();
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
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
                            if let Err(err) = output
                                .send(pages::public::Message::FetchHandle(url.clone()))
                                .await
                            {
                                tracing::error!("Failed to send image handle: {}", err);
                            }
                        }
                        urls.clear();
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
            let mut urls = Vec::new();
            match mastodon.get_public_timeline(true, false).await {
                Ok(statuses) => {
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
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
                            if let Err(err) = output
                                .send(pages::public::Message::FetchHandle(url.clone()))
                                .await
                            {
                                tracing::error!("Failed to send image handle: {}", err);
                            }
                        }
                        urls.clear();
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
            let mut urls = Vec::new();
            match mastodon.get_public_timeline(false, true).await {
                Ok(statuses) => {
                    for status in statuses {
                        if let Err(err) = output
                            .send(pages::public::Message::AppendStatus(status.clone()))
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
                            if let Err(err) = output
                                .send(pages::public::Message::FetchHandle(url.clone()))
                                .await
                            {
                                tracing::error!("Failed to send image handle: {}", err);
                            }
                        }
                        urls.clear();
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
