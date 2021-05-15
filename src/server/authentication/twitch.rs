use super::{errors::Error, AuthToken, Authenticator, Claims};
use crate::CLIENT_ID;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize)]
struct TwitchJwk {
    alg: String,
    e: String,
    kid: String,
    kty: String,
    n: String,
    #[serde(rename = "use")]
    _use: String,
}

#[derive(Deserialize)]
struct TwitchJwkSet {
    keys: Vec<TwitchJwk>,
}

pub struct TwitchAuth(reqwest::Client);

#[async_trait]
impl Authenticator for TwitchAuth {
    async fn authenticate(&self) -> Result<AuthToken, Error> {}

    async fn refresh(&self) -> Result<AuthToken, Error> {}

    async fn validate_jwt(&self, unverified_jwt: &str) -> Result<Claims, Error> {
        let jwk_set: TwitchJwkSet = self
            .0
            .get("https://id.twitch.tv/oauth2/keys")
            .send()
            .await?
            .json()
            .await?;

        let mut err = Error::Unknown;

        for json_web_key in jwk_set.keys.iter() {
            // Initializing this (unchanging) HashSet seems wasteful, but its required by
            // jsonwebtoken in order to validate the aud field is correct
            let mut audience = HashSet::new();
            audience.insert((*CLIENT_ID).clone());

            match jsonwebtoken::decode::<Claims>(
                &unverified_jwt,
                &jsonwebtoken::DecodingKey::from_rsa_components(&json_web_key.n, &json_web_key.e),
                &jsonwebtoken::Validation {
                    aud: Some(audience),
                    iss: Some("https://id.twitch.tv/oauth2".to_string()),
                    algorithms: vec![jsonwebtoken::Algorithm::RS256],
                    ..jsonwebtoken::Validation::default()
                },
            ) {
                Ok(verified_token) => {
                    println!("========{:?}", verified_token);
                    return Ok(verified_token.claims);
                }
                Err(e) => match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        tracing::info!("JWT expired, attempting to refresh token and try again");
                    }
                    _ => {
                        tracing::warn!("Failed to validate JWT: {}", &e);
                        err = Error::JwtValidation(e);
                    }
                },
            }
        }
        Err(err)
    }
}
