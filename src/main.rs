use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;

use twitch_rs::{games, streams::{self, StreamFilter}, TwitchApi};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CrawlerArgs {
    #[arg(long, env, help = "Value of Warp10 label \"event_name\" of datapoints")]
    event_name: String,
    #[arg(long, env, help = "Twitch client id")]
    twitch_client_id: String,
    #[arg(long, env, help = "Twitch client secret")]
    twitch_client_secret: String,
    #[arg(long, env, help = "Base url of Warp10 database")]
    warp10_url: String,
    #[arg(long, env, help = "Warp10 write token")]
    warp10_write_token: String,
    #[arg(long, env, default_value = "twitch", help = "Warp10 classname prefix of datapoints")]
    warp10_prefix: String,
    #[arg(long, env, value_delimiter = ',', help = "Filter streams on game ids (Max value: 100)")]
    game_ids: Option<Vec<String>>,
    #[arg(long, env, value_delimiter = ',', help = "Filter streams on languages (Max value: 100)")]
    languages: Option<Vec<String>>,
    #[arg(long, env, value_delimiter = ',', help = "Filter streams on user logins")]
    user_logins: Option<Vec<String>>,
    #[arg(long, env, default_value_t = 0, help = "Keep only datapoint viewers count superior to the value")]
    minimum_viewers: u32,
    #[arg(long, env, default_value_t = 15, help = "Interval of seconds between each measurement")]
    interval: u64
}

impl CrawlerArgs {
    fn filters(&self) -> Result<Vec<StreamFilter>, ()> {
        let games_len = self.game_ids.as_ref().unwrap_or(&[].to_vec()).len();
        let languages_len = self.languages.as_ref().unwrap_or(&[].to_vec()).len();
        let users_len = self.user_logins.as_ref().unwrap_or(&[].to_vec()).len();

        if games_len == 0 && languages_len == 0 && users_len == 0 {
            eprintln!("No filters are provisionned");

            return Err(());
        }

        if games_len > 100 || languages_len > 100 {
            eprintln!("GAME_IDS or/and LANGUAGES filters are more than 100 values");

            return Err(());
        }

        let filter = StreamFilter {
            after: None,
            before: None,
            first: Some(100),
            game_ids: self.game_ids.clone(),
            languages: self.languages.clone(),
            user_ids: None,
            user_logins: self.user_logins.clone(),
        };

        return Ok(chunk_filters(filter));
    } 
}

#[actix::main]
async fn main() {
    let config = CrawlerArgs::parse();

    let filters = config.filters().expect("Error with filters");

    let mut api = TwitchApi::new(config.twitch_client_id, config.twitch_client_secret).expect("Failed to create api client");

    api.authorize().await.expect("Failed to get access token");

    let warp10_client = warp10::Client::new(&config.warp10_url).expect("Failed to build warp10 client");
    let writer = warp10_client.get_writer(config.warp10_write_token);

    let mut interval = actix_rt::time::interval(Duration::from_secs(config.interval));
    let mut games_mapping: HashMap<String, String> = HashMap::new();

    loop {
        interval.tick().await;

        let timestamp = OffsetDateTime::now_utc();

        println!("Retrieve metrics at {}", timestamp);

        for selected_filter in &filters {
            let mut filter = selected_filter.clone();
            let mut is_finished = false;

            while !is_finished {
                match streams::get(
                    &api,
                    &filter
                )
                .await {
                    Ok(responses) => {
                        let mut metrics = Vec::new();

                        if responses.data.len() == filter.first.unwrap_or(20) as usize {
                            is_finished = responses.pagination.cursor.is_none();
                            filter.after = responses.pagination.cursor.clone();
                        } else {
                            is_finished = true;
                            filter.after = None;
                        }

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

                            if stream.viewer_count < config.minimum_viewers as i32 {
                                is_finished = true;
                                continue;
                            }

                            metrics.push(warp10::Data::new(
                                timestamp,
                                None,
                                format!("{}.viewers", &config.warp10_prefix),
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
                                eprintln!("Failed to write metrics to warp10");
                                eprintln!("{:?}", error);
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
}

fn chunk_filters(filter: streams::StreamFilter) -> Vec<streams::StreamFilter> {
    let mut filters = Vec::new();

    match &filter.user_logins {
        Some(user_logins) => {
            if user_logins.len() > 100 {
                for chunk in user_logins.chunks(100) {
                    let mut chunked_filter = filter.clone();

                    chunked_filter.user_logins = Some(chunk.to_vec());

                    filters.push(chunked_filter);
                }
            } else {
                filters.push(filter);
            }
        },
        None => {
            filters.push(filter);
        }
    }

    return filters
}
