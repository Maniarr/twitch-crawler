use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::responses::*;
use crate::{TwitchApi, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelixStream {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub game_id: String,
    pub r#type: String,
    pub title: String,
    pub viewer_count: i32,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String
}

impl super::traits::HelixModel for HelixStream {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StreamFilter {
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: Option<i64>,
    pub game_ids: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub user_ids: Option<Vec<String>>,
    pub user_logins: Option<Vec<String>>,
}

impl Default for StreamFilter {
    fn default() -> Self {
        Self {
            after: None,
            before: None,
            first: Some(100),
            game_ids: None,
            languages: None,
            user_ids: None,
            user_logins: None,
        }
    }
}

pub async fn get(twitch_api: &TwitchApi, filter: &StreamFilter) -> Result<HelixPaginatedResponse<HelixStream>> {
    let mut data: Vec<(&str, String)> = Vec::new();

    if let Some(after) = &filter.after {
        data.push(("after", after.to_string()));
    }
    
    if let Some(before) = &filter.before {
        data.push(("before", before.to_string()));
    }

    if let Some(first) = filter.first {
        data.push(("first", first.to_string()));
    }

    if let Some(game_ids) = &filter.game_ids {
        for game_id in game_ids {
            data.push(("game_id", game_id.to_string()));
        }
    }

    if let Some(languages) = &filter.languages {
        for language in languages {
            data.push(("language", language.to_string()));
        }
    }

    if let Some(user_ids) = &filter.user_ids {
        for user_id in user_ids {
            data.push(("user_id", user_id.to_string()));
        }
    }

    if let Some(user_logins) = &filter.user_logins {
        for user_login in user_logins {
            data.push(("user_login", user_login.to_string()));
        }
    }

    Ok(
        serde_json::from_str(
            &twitch_api.get(String::from("https://api.twitch.tv/helix/streams"), &data)
                .await?
                .text()
                .await?[..]
        )?
    )
}

pub async fn get_from_games(twitch_api: &TwitchApi, game_ids: &Vec<String>, first: i32, after: Option<String>, before: Option<String>) -> Result<HelixPaginatedResponse<HelixStream>> {
    let mut data: Vec<(&str, String)> = vec![
        ("first", first.to_string())
    ];

    for game_id in game_ids {
        data.push(("game_id", String::from(game_id)));
    }

    if let Some(value) = after {
        data.push(("after", value));
    }

    if let Some(value) = before {
        data.push(("before", value));
    }

    Ok(
        serde_json::from_str(
            &twitch_api.get(String::from("https://api.twitch.tv/helix/streams"), &data)
                .await?
                .text()
                .await?[..]
        )?
    )
}

pub async fn get_from_users(twitch_api: &TwitchApi, user_ids: &Vec<String>, first: i32, after: Option<String>, before: Option<String>) -> Result<HelixPaginatedResponse<HelixStream>> {
    let mut data: Vec<(&str, String)> = vec![
        ("first", first.to_string())
    ];

    for user_id in user_ids {
        data.push(("user_id", String::from(user_id)));
    }

    if let Some(value) = after {
        data.push(("after", value));
    }

    if let Some(value) = before {
        data.push(("before", value));
    }

    Ok(
        serde_json::from_str(
            &twitch_api.get(String::from("https://api.twitch.tv/helix/streams"), &data)
                .await?
                .text()
                .await?[..]
        )?
    )
}

pub async fn get_from_users_login(twitch_api: &TwitchApi, user_logins: &Vec<String>, first: i32, after: Option<String>, before: Option<String>) -> Result<HelixPaginatedResponse<HelixStream>> {
    let mut data: Vec<(&str, String)> = vec![
        ("first", first.to_string())
    ];

    for user_login in user_logins {
        data.push(("user_login", String::from(user_login)));
    }

    if let Some(value) = after {
        data.push(("after", value));
    }

    if let Some(value) = before {
        data.push(("before", value));
    }

    Ok(
        serde_json::from_str(
            &twitch_api.get(String::from("https://api.twitch.tv/helix/streams"), &data)
                .await?
                .text()
                .await?[..]
        )?
    )
}
