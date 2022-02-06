use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;

use twitch_rs::{games, streams, TwitchApi};

use isahc::ReadResponseExt;

#[derive(Debug, Deserialize, Clone)]
struct TwitchConfig {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Warp10Config {
    url: String,
    write_token: String,
    prefix: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    twitch: TwitchConfig,
    warp10: Warp10Config,
    event_name: String,
    streamers: Vec<String>,
}

fn get_config() -> Result<Config, ()> {
    let args: Vec<String> = std::env::args().collect();

    if let Some(file_name) = args.get(1) {
        Ok(serde_yaml::from_str(
            &std::fs::read_to_string(&file_name).expect(&format!("Failed to read {}", &file_name)),
        )
        .expect(&format!("Not valid yaml in {}", &file_name)))
    } else {
        Ok(Config {
            twitch: TwitchConfig {
                client_id: std::env::var("TWITCH_CLIENT_ID").expect("Missing env TWITCH_CLIENT_ID"),
                client_secret: std::env::var("TWITCH_CLIENT_SECRET")
                    .expect("Missing env TWITCH_CLIENT_ID"),
            },
            warp10: Warp10Config {
                url: std::env::var("WARP10_URL").expect("Missing env WARP10_URL"),
                write_token: std::env::var("WARP10_WRITE_TOKEN")
                    .expect("Missing env WARP10_WRITE_TOKEN"),
                prefix: std::env::var("WARP10_PREFIX").expect("Missing env WARP10_PREFIX"),
            },
            event_name: std::env::var("EVENT_NAME").expect("Missing env EVENT_NAME"),
            streamers: std::env::var("STREAMERS")
                .expect("Missing env STREAMERS")
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct Emote {
    id: String,
    emote: String,
    amount: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamElementsStats {
    channel: String,
    total_messages: i32,
    bttv_emotes: Vec<Emote>,
    twitch_emotes: Vec<Emote>,
}

#[actix::main]
async fn main() {
    let config_original = get_config().expect("Missing configuration");

    let config = config_original.clone();

    let mut api = TwitchApi::new(config.twitch.client_id, config.twitch.client_secret)
        .expect("Failed to create api client");

    api.authorize().await.expect("Failed to get access token");

    let warp10_client =
        warp10::Client::new(&config.warp10.url).expect("Failed to build warp10 client");
    let writer = warp10_client.get_writer(config.warp10.write_token);

    let mut interval = actix_rt::time::interval(Duration::from_secs(10));
    let mut games_mapping: HashMap<String, String> = HashMap::new();

    loop {
        interval.tick().await;

        let timestamp = OffsetDateTime::now_utc();

        println!("Run twitch at {}", timestamp);

        for streamers in config.streamers.chunks(100) {
            match streams::get_from_users_login(
                &api,
                &streamers.iter().map(|i| i.clone()).collect(),
                100,
                None,
                None,
            )
            .await
            {
                Ok(responses) => {
                    let mut metrics = Vec::new();

                    for stream in responses.data {
                        let game_name = if let Some(name) = games_mapping.get(&stream.game_id) {
                            name.clone()
                        } else {
                            let name = if let Ok(response) =
                                games::get(&api, &vec![stream.game_id.clone()]).await
                            {
                                if let Some(game) = response.data.get(0) {
                                    game.name.clone()
                                } else {
                                    dbg!(&stream);
                                    "Pas de catégorie".to_string()
                                }
                            } else {
                                dbg!(&stream);
                                "Pas de catégorie".to_string()
                            };

                            games_mapping.insert(stream.game_id.clone(), name.clone());

                            name
                        };

                        metrics.push(warp10::Data::new(
                            timestamp,
                            None,
                            format!("{}.viewers", &config.warp10.prefix),
                            vec![
                                warp10::Label::new("event_name", &config.event_name),
                                warp10::Label::new("stream_id", &stream.id),
                                warp10::Label::new("game_id", &stream.game_id),
                                warp10::Label::new("game_name", &game_name),
                                warp10::Label::new("user_id", &stream.user_id),
                                warp10::Label::new("user_name", &stream.user_login),
                            ],
                            warp10::Value::Int(stream.viewer_count),
                        ));
                    }

                    let metrics_count = metrics.len();

                    match writer.post_sync(metrics) {
                        Ok(_) => {
                            println!("{} metrics wrote to warp10", metrics_count);
                        }
                        Err(error) => {
                            eprintln!("Error to write metrics: {:?}", error);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("{}", error);
                }
            };
        }
    }
}
