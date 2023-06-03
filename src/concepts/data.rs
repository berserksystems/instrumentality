//! Data enums for content, presence and profile metadata.
//!
//! # A note on Data
//!
//! The quality of all data provided to Instrumentality will determine its
//! usefulness. For example, if one data provider posts full profile metadata
//! and another posts the minimum subset in quick succession, Instrumentality
//! must discard the full profile in favour of the minimum subset as it has no
//! ability to determine that all the previous data was not removed by the user
//! between posts. This also applies to content.
//!
//! In the future, profiles with lots of coverage could be used to sniff out
//! lazy data providers automatically or through a reputation system. However,
//! at this stage, data providers posting all available data is key to the
//! utility of the platform.
//!
//! # Content
//! Content exists to represent any event occurring at a discrete point in time.
//!
//! Examples of content include:
//! - a blog entry.
//! - a video.
//! - an item coming back into stock on an online store.
//!
//! The only requirements of content are that it must have a subject, a content
//! type and a time retrieved. For example,
//! ```json
//! {
//!     "id": "123456789",
//!     "platform": "twitter",
//!     "content_type": "tweet",
//!     "created_at": "2038-01-19T03:14:07Z",
//!     "body": "I love my epoch.",
//! };
//! ```
//! When handling URLs, we store the original URL of the content and have a
//! separate media table in MongoDB to retrieve the content at the point at
//! which we want to reconstruct the post. This is true for image, audio, video
//! and any further content that cannot be reasonably represented in UTF-8.
//!
//! The URLs in media should be direct links to the files themselves, not a page
//! with the media present on it. If need be, this may involve the extractor
//! manually grabbing the media and hosting it until Instrumentality has
//! confirmed it as received.
//!
//! ## IDs
//! In order to continue attributing new content to the correct user after a
//! username change, you must fill the id field with a unique user ID.
//! Instrumentality will not stop you submitting content with a username as the
//! subject but this is suboptimal.
//!
//! ## Content types
//! Platforms cannot said to be solely made up of one type of content. For
//! example, 'stories' are a common temporary post feature that exist on top a
//! platforms 'bread and butter' content. In order to differentiate between
//! content types on the same platform we tag them with a type. For example,
//! ```json
//! {
//!     "id": "123456789",
//!     "platform": "instagram",
//!     "content_type": "post",
//!     "created_at": "2022-01-01T00:00:05Z",
//!     "body": "Happy new year!",
//!     "media": ["https://..."]
//! };
//! ```
//! and
//! ```json
//! {
//!     "id": "123456789",
//!     "platform": "instagram",
//!     "content_type": "story",
//!     "created_at": "2022-01-01T00:00:05Z",
//!     "body": "Happy new year!",
//!     "media": ["https://..."]
//! };
//! ```
//! are distinct types of content that are still tied to the id '123456789' on
//! 'instagram'. Content can only be of types specified within Instrumentality.
//!
//! ## Activity
//! Updates to the user are always tagged as 'activity'. These are distinct from
//! other types of content in that they are not content in and of themselves but
//! do represent some action that the user has taken.
//!
//! ## Limitations
//! The Content struct is not intended to perfectly mirror all types of content
//! on every platform, it is merely a first approximation. Certain information
//! is inevitably lost during the process of mirroring content, such as the
//! positions of tags on group photos.
//!
//! # Presence
//! Presence exists to represent a user being 'active' or present in a
//! continuous manner. Obviously, these are discrete observations of continuous
//! behaviour but labeling them accordingly makes this apparent to the system.
//!
//! One must be wary of attempting to interpret discrete observations to
//! continuous data.
//!
//! An example of this is a Twitch livestream being live. This isn't content
//! because it can't be said to have 'happened' at a discrete point in time
//! until it is finished, at which point you would post it as content. A Twitch
//! livestream going live could be considered to be content as it happens at a
//! discrete time.
//!
//! These are expected to make up the bulk of traffic as presence changes occur
//! far more often than content posts.
//!
//! For example,
//! ```json
//! {
//!     "id": "123456789",
//!     "platform": "twitch",
//!     "presence_type": "livestream",
//!     "retrieved_at": "2022-01-01T00:00:00Z",
//! };
//! ```
//!
//! # Meta
//! Profile metadata changes regularly and sometimes silently. Without data
//! providers keeping a local copy of the data, it's difficult to determine what
//! has changed from fetch to fetch. Given that each request of the profile will
//! generally contain a full copy of that profile, it's easier to post the
//! entire profile to Instrumentality to determine changes.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{config::IConfig, routes::queue::InternalQueueItem};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Timestamp {
    Instant(DateTime<Utc>),
    Span(DateTime<Utc>, DateTime<Utc>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Data {
    Presence {
        id: String,
        platform: String,
        presence_type: String,
        retrieved_at: DateTime<Utc>,
        added_by: Option<String>,
        added_at: Option<DateTime<Utc>>,
    },
    Content {
        id: String,
        platform: String,
        content_type: String,
        retrieved_at: DateTime<Utc>,
        content_id: String,
        deleted: Option<bool>,
        retrieved_from: Option<String>,
        created_at: Option<Timestamp>,
        body: Option<String>,
        media: Option<Vec<String>>,
        references: Option<HashMap<String, String>>,
        added_by: Option<String>,
        added_at: Option<DateTime<Utc>>,
    },
    Meta {
        id: String,
        platform: String,
        username: String,
        private: bool,
        banned: bool,
        retrieved_at: DateTime<Utc>,
        display_name: Option<String>,
        profile_picture: Option<String>,
        bio: Option<String>,
        verified: Option<bool>,
        references: Option<HashMap<String, String>>,
        link: Option<String>,
        added_by: Option<String>,
        added_at: Option<DateTime<Utc>>,
    },
}

impl Data {
    pub fn tag(self, uuid: &str) -> Self {
        match self {
            Data::Presence {
                id,
                platform,
                presence_type,
                retrieved_at,
                ..
            } => Data::Presence {
                id,
                platform,
                presence_type,
                retrieved_at,
                added_by: Some(uuid.to_string()),
                added_at: Some(Utc::now()),
            },
            Data::Content {
                id,
                platform,
                content_type,
                retrieved_at,
                content_id,
                deleted,
                retrieved_from,
                created_at,
                body,
                media,
                references,
                ..
            } => Data::Content {
                id,
                platform,
                content_type,
                retrieved_at,
                content_id,
                deleted,
                retrieved_from,
                created_at,
                body,
                media,
                references,
                added_by: Some(uuid.to_string()),
                added_at: Some(Utc::now()),
            },
            Data::Meta {
                id,
                platform,
                username,
                private,
                banned,
                display_name,
                profile_picture,
                bio,
                verified,
                references,
                link,
                retrieved_at,
                ..
            } => Data::Meta {
                id,
                platform,
                username,
                private,
                banned,
                display_name,
                profile_picture,
                bio,
                verified,
                references,
                link,
                retrieved_at,
                added_by: Some(uuid.to_string()),
                added_at: Some(Utc::now()),
            },
        }
    }

    pub fn verify(&self, config: &IConfig) -> bool {
        match self {
            Data::Presence {
                platform,
                presence_type,
                ..
            } => config.valid_presence_type(platform, presence_type),
            Data::Content {
                platform,
                content_type,
                ..
            } => config.valid_content_type(platform, content_type),
            Data::Meta { platform, .. } => config.valid_platform(platform),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Datas {
    pub queue_id: Option<String>,
    pub data: Vec<Data>,
}

impl Datas {
    pub fn tag(self, uuid: &str) -> Self {
        let mut tagged_data = Vec::new();
        for data in self.data {
            tagged_data.push(data.tag(uuid))
        }

        Datas {
            data: tagged_data,
            ..self
        }
    }

    pub fn verify_for_config(self, config: &IConfig) -> Self {
        let mut verified_data = Vec::new();
        for data in self.data {
            if data.verify(config) {
                verified_data.push(data);
            }
        }

        Datas {
            data: verified_data,
            ..self
        }
    }

    pub fn meta(&self) -> Option<Data> {
        for data in &self.data {
            if let Data::Meta { .. } = data {
                return Some(data.clone());
            }
        }
        None
    }

    pub fn info(&self) -> (String, String, String, Option<String>) {
        if let Some(meta) = self.meta() {
            let (platform_id, platform, added_by, username) = match meta {
                Data::Meta {
                    id,
                    platform,
                    added_by,
                    username,
                    ..
                } => (id, platform, added_by, Some(username)),
                _ => panic!("Expected Data::Meta."),
            };
            (platform_id, platform, added_by.unwrap(), username)
        } else {
            let data = self.data[0].clone();
            let (platform_id, platform, added_by) = match data {
                Data::Presence {
                    id,
                    platform,
                    added_by,
                    ..
                } => (id, platform, added_by),
                Data::Content {
                    id,
                    platform,
                    added_by,
                    ..
                } => (id, platform, added_by),
                _ => panic!("Expected Data::Presence or Data::Content."),
            };
            (platform_id, platform, added_by.unwrap(), None)
        }
    }

    pub fn verify_for_queue(self, queue_item: InternalQueueItem) -> Self {
        let mut verified_data = Vec::new();
        // let queue_id = self.queue_id.clone();
        for data in &self.data {
            let verified: bool = match &data {
                Data::Meta { platform, id, .. } => {
                    &queue_item.platform == platform
                        && (&queue_item.platform_id == id
                            || !&queue_item.confirmed_id)
                }
                Data::Content { platform, id, .. } => {
                    &queue_item.platform_id == id
                        && &queue_item.platform == platform
                }
                Data::Presence { platform, id, .. } => {
                    &queue_item.platform_id == id
                        && &queue_item.platform == platform
                }
            };

            if verified {
                verified_data.push(data.clone());
            }
        }

        Datas {
            data: verified_data,
            ..self
        }
    }
}
