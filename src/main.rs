use std::collections::HashMap;
use std::time::Duration;
use time::OffsetDateTime;
use clap::Parser;
use twitch_api2::{
    helix::streams::GetStreamsRequest,
    HelixClient,
    types::{
        CategoryId,
        UserName
    },
};
use twitch_oauth2::{
    AppAccessToken,
    Scope,
};


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
    minimum_viewers: usize,
    #[arg(long, env, default_value_t = 15, help = "Interval of seconds between each measurement")]
    interval: u64
}

impl CrawlerArgs {
    fn filters(&self) -> Result<Vec<GetStreamsRequest>, ()> {
        let games_len = self.game_ids.as_ref().unwrap_or(&[].to_vec()).len();
        let languages_len = self.languages.as_ref().unwrap_or(&[].to_vec()).len();
        let users_len = self.user_logins.as_ref().unwrap_or(&[].to_vec()).len();

        if games_len == 0 && languages_len == 0 && users_len == 0 {
            log::error!("No filters are provisionned");

            return Err(());
        }

        if games_len > 100 || languages_len > 100 {
            log::error!("GAME_IDS or/and LANGUAGES filters are more than 100 values");

            return Err(());
        }

        let languages = if let Some(languages) = &self.languages {
            Some(languages.join(","))
        } else { 
            None
        };

        let mut filters = vec![];

        if let Some(user_logins) = &self.user_logins {
            for chunk in user_logins.chunks(100) {
                filters.push(
                    GetStreamsRequest::builder()
                        .first(Some(100))
                        .language(languages.clone())
                        .game_id(self.game_ids.clone().unwrap_or(vec![]).iter().map(|name| CategoryId::from(name.as_str())).collect())
                        .user_login(chunk.iter().map(|name| UserName::from(name.as_str())).collect())
                        .build()
                );
            }
        } else {
            filters.push(
                GetStreamsRequest::builder()
                    .first(Some(100))
                    .language(languages.clone())
                    .game_id(self.game_ids.clone().unwrap_or(vec![]).iter().map(|name| CategoryId::from(name.as_str())).collect())
                    .build()
            );
        }

        return Ok(filters);
    } 
}

#[actix::main]
async fn main() {
    env_logger::init();

    let config = CrawlerArgs::parse();

    let filters = config.filters().expect("Error with filters");

    let client: HelixClient<reqwest::Client> = HelixClient::default();

    let token = AppAccessToken::get_app_access_token(
        &client,
        config.twitch_client_id.into(),
        config.twitch_client_secret.into(),
        Scope::all()
    ).await.expect("Failed to get twitch api token");

    let warp10_client = warp10::Client::new(&config.warp10_url).expect("Failed to build warp10 client");
    let writer = warp10_client.get_writer(config.warp10_write_token);

    let mut interval = actix_rt::time::interval(Duration::from_secs(config.interval));
    let mut games_mapping: HashMap<String, String> = HashMap::new();

    loop {
        interval.tick().await;

        let timestamp = OffsetDateTime::now_utc();

        log::info!("Retrieve metrics");

        for req in &filters {
            let mut filter = req.clone();
            let mut is_finished = false;

            while !is_finished {
                match client.req_get(filter.clone(), &token).await {
                    Ok(responses) => {
                        let mut metrics = Vec::new();

                        if responses.data.len() == filter.first.unwrap_or(20) as usize {
                            is_finished = responses.pagination.is_none();
                            filter.after = responses.pagination.clone();
                        } else {
                            is_finished = true;
                            filter.after = None;
                        }

                        for stream in responses.data {
                            let game_name = if let Some(name) = games_mapping.get(&stream.game_id.to_string()) {
                                name.clone()
                            } else {
                                let name = match client.get_games_by_id(&vec![stream.game_id.clone()], &token).await {
                                    Ok(response) => {
                                        match response.get(&stream.game_id) {
                                            Some(game) => {
                                                game.name.clone()
                                            },
                                            None => {
                                                "Pas de catégorie".to_string()
                                            }
                                        }
                                    },
                                    Err(_) => {
                                        "Pas de catégorie".to_string()
                                    }
                                };

                                games_mapping.insert(stream.game_id.to_string(), name.clone());

                                name
                            };

                            if stream.viewer_count < config.minimum_viewers {
                                is_finished = true;
                                continue;
                            }

                            metrics.push(warp10::Data::new(
                                timestamp,
                                None,
                                format!("{}.viewers", &config.warp10_prefix),
                                vec![
                                    warp10::Label::new("event_name", &config.event_name),
                                    warp10::Label::new("stream_id", stream.id.as_str()),
                                    warp10::Label::new("game_id", stream.game_id.as_str()),
                                    warp10::Label::new("game_name", &game_name),
                                    warp10::Label::new("user_id", stream.user_id.as_str()),
                                    warp10::Label::new("user_name", stream.user_login.as_str()),
                                ],
                                warp10::Value::Int(stream.viewer_count.try_into().unwrap()),
                            ));
                        }

                        let metrics_count = metrics.len();

                        match writer.post_sync(metrics) {
                            Ok(_) => {
                                log::info!("{} metrics wrote to Warp10", metrics_count);
                            }
                            Err(error) => {
                                log::error!("Failed to write metrics to Warp10: {}", error.to_string());
                                log::debug!("{:?}", error);
                            }
                        }
                    }
                    Err(error) => {
                        log::error!("{}", error);
                    }
                };
            }
        }
    }
}
