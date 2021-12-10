use serde::Deserialize;

pub mod responses;
pub mod traits;

pub mod games;
pub mod clips;
pub mod streams;
pub mod users;
pub mod videos;
pub mod comments;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct TwitchApi {
    client_id: String,
    client_secret: String,
    client: reqwest::Client,
    credentials: Option<Credentials>,
}

#[derive(Debug, Deserialize)]
struct Credentials {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

impl TwitchApi {
    pub fn new(client_id: String, client_secret: String) -> Result<TwitchApi> {
        Ok(TwitchApi {
            client_id,
            client_secret,
            client: reqwest::Client::builder().build()?,
            credentials: None,
        })
    }

    pub async fn authorize(&mut self) -> Result<()> {
        let credentials: Credentials = serde_json::from_str(&self.client
            .post("https://id.twitch.tv/oauth2/token")
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials")
            ])
            .send()
            .await?
            .text()
            .await?
        ).unwrap();

        self.credentials = Some(credentials);

        Ok(())
    }

    async fn get(&self, url: String, data: &Vec<(&str, String)>) -> Result<reqwest::Response> {
        let mut request = self.client
            .get(&url[..])
            .header("Client-ID", &self.client_id[..]);

        if let Some(credentials) = &self.credentials {
            request = request.header("Authorization", format!("Bearer {}", credentials.access_token));
        }
        
        Ok(
            request
                .query(&data)
                .send()
                .await?
        )
    }
}
