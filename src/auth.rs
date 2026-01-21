use crate::types::{Scope, Tokens};
use reqwest::Client;

#[derive(Debug)]
pub struct AuthClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl AuthClient {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        Self {
            client_id: String::from(client_id),
            client_secret: String::from(client_secret),
            redirect_uri: String::from(redirect_uri),
        }
    }

    pub fn get_authorization_url(&self, scope: Scope) -> String {
        let scope_ = if scope == Scope::Trading {
            "trading"
        } else {
            "accounts"
        };
        format!(
            "https://id.ctrader.com/my/settings/openapi/grantingaccess/?client_id={}&redirect_uri={}&scope={scope_}&product=web",
            self.client_id, self.redirect_uri
        )
    }

    pub async fn get_access_tokens(
        &self,
        code: &str,
    ) -> Result<Tokens, Box<dyn std::error::Error>> {
        let client = Client::new();

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = client
            .post("https://openapi.ctrader.com/apps/token")
            .form(&params)
            .header("Accept", "application_json")
            .send()
            .await?;

        let token = response.json::<Tokens>().await?;

        println!("{:#?}", token);

        Ok(token)
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Tokens, Box<dyn std::error::Error>> {
        let client = Client::new();

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];
        let response = client
            .post("https://openapi.ctrader.com/apps/token")
            .form(&params)
            .send()
            .await?;
        let token = response.json::<Tokens>().await?;

        Ok(token)
    }
}
