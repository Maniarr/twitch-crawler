use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::responses::*;
use crate::{TwitchApi, Result};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HelixCommenter {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub display_name: String,
    pub name: String,
    pub r#type: String,
    pub bio: Option<String>,
    pub created_at: DateTime::<Utc>,
    pub updated_at: DateTime::<Utc>,
    pub logo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HelixMessageFragment {
    pub text: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HelixUserBadge {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HelixMessage {
    pub body: String,
    pub fragments: Option<Vec<HelixMessageFragment>>,
    pub is_action: bool,
    pub user_badges: Option<Vec<HelixUserBadge>>,
    pub user_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HelixComment {
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub created_at: DateTime::<Utc>,
    pub updated_at: DateTime::<Utc>,
    pub channel_id: String,
    pub content_type: String,
    pub content_id: String,
    pub content_offset_seconds: f64,
    pub commenter: HelixCommenter,
    pub source: String,
    pub state: String,
    pub message: HelixMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelixCommentResponse {
    pub comments: Vec<HelixComment>,
    #[serde(rename="_prev")]
    pub previous: Option<String>,
    #[serde(rename="_next")]
    pub next: Option<String>,
}

impl super::traits::HelixModel for HelixComment {}

pub async fn get(twitch_api: &TwitchApi, video_id: &str, cursor: Option<String>) -> Result<HelixCommentResponse> {
    let mut data = Vec::new();

    if let Some(cursor) = cursor {
        data.push(("cursor", cursor));
    } else {
        data.push(("content_offset_seconds", "0".to_string()));
    }

    Ok(
        serde_json::from_str(
            &twitch_api.get(format!("https://api.twitch.tv/v5/videos/{}/comments", video_id), &data)
                .await?
                .text()
                .await?[..]
        )?
    )
}
