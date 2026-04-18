use crate::auth::errors::AuthError;
use crate::auth::token::{GoogleTokenResponse, TokenSet};

#[derive(serde::Deserialize)]
pub struct ServiceAccountKey {
    pub client_email: String,
    pub private_key: String,
}

pub struct ServiceAccountFlow {
    key_json: ServiceAccountKey,
    impersonate: String,
    scopes: Vec<String>,
}

impl ServiceAccountFlow {
    pub fn new(key_json: ServiceAccountKey, impersonate: String, scopes: Vec<String>) -> Self {
        Self {
            key_json,
            impersonate,
            scopes,
        }
    }

    pub async fn access_token(&self) -> Result<TokenSet, AuthError> {
        let now = chrono::Utc::now().timestamp();

        #[derive(serde::Serialize)]
        struct Claims {
            iss: String,
            sub: String,
            aud: String,
            scope: String,
            iat: i64,
            exp: i64,
        }

        let claims = Claims {
            iss: self.key_json.client_email.clone(),
            sub: self.impersonate.clone(),
            aud: "https://oauth2.googleapis.com/token".into(),
            scope: self.scopes.join(" "),
            iat: now,
            exp: now + 3600,
        };
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(self.key_json.private_key.as_bytes())?;
        let assertion = jsonwebtoken::encode(&header, &claims, &key)?;

        let resp: GoogleTokenResponse = reqwest::Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &assertion),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(TokenSet::from(resp))
    }
}
