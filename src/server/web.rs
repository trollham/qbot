use crate::{CLIENT_ID, CLIENT_SECRET};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use warp::Filter;

pub fn build_server(
    db_pool: sqlx::postgres::PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    /*
    Look if id_token cookie exists ----No---> /index.html
        |
        No
        |
        v
    Validate cookie ----Yes---> /dashboard/user
        |
        No
        |
        v
    refresh cookie ----Yes---> /dashboard/user with new id_token cookie
        |
        No
        |
        v
    clear refresh token from db and id_token cookie and redirect to /index.html
    */

    // let auth_cookie = warp::cookie::<String>("id_token");
    // let cookie_validatation = auth_cookie.and(validate_jwt);
    // warp::any().and(warp::get()).and(
    //     warp::cookie("id_token")
    //     .and_then(validate_jwt) // TODO map error into a warp::Rejection
    //     .and(warp::fs::file("./www/dashboard/"))
    //     .or(warp::fs::file("./www/"))
    // )
    endpoints::login_redirect(db_pool).or(warp::fs::dir("./www/"))
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthToken {
    access_token: String,
    expires_in: usize,
    id_token: String,
    refresh_token: String,
    scope: Vec<String>,
    token_type: String,
}

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

#[derive(Debug, Deserialize)]
pub(crate) struct AuthCode {
    code: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    iat: usize,
    nonce: Option<String>,
    email: String,
    preferred_username: String,
}

async fn validate_jwt(
    unverified_jwt: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, Vec<jsonwebtoken::errors::ErrorKind>> {
    let jwk_set: TwitchJwkSet = reqwest::get("https://id.twitch.tv/oauth2/keys")
        .await
        .unwrap() // TODO error handling
        .json()
        .await
        .unwrap(); // TODO error handling

    let mut err_vec: Vec<jsonwebtoken::errors::ErrorKind> = Vec::new();

    for jwk in jwk_set.keys.iter() {
        // Initializing this (unchanging) HashSet seems wasteful, but its required by jsonwebtoken in order to validate the audience is correct
        let mut audience = HashSet::new();
        audience.insert((*CLIENT_ID).clone());

        match jsonwebtoken::decode::<Claims>(
            &unverified_jwt,
            &jsonwebtoken::DecodingKey::from_rsa_components(&jwk.n, &jwk.e),
            &jsonwebtoken::Validation {
                aud: Some(audience),
                iss: Some("https://id.twitch.tv/oauth2".to_string()),
                algorithms: vec![jsonwebtoken::Algorithm::RS256],
                ..jsonwebtoken::Validation::default()
            },
        ) {
            Ok(verified_token) => {
                println!("========{:?}", verified_token);
                return Ok(verified_token);
            }
            Err(e) => {
                err_vec.push(e.into_kind());
            }
        }
    }
    Err(err_vec)
}

mod handlers {
    use super::{validate_jwt, AuthCode, AuthToken};
    use crate::database::insert_or_update_user;
    use crate::{CLIENT_ID, CLIENT_SECRET};
    use std::convert::Infallible;

    pub(crate) async fn login_redirect(
        unverified_jwt: AuthCode,
        pool: sqlx::postgres::PgPool,
    ) -> Result<impl warp::Reply, Infallible> {
        let redirect_url = if cfg!(debug_assertions) {
            "http://localhost:8080/login-redirect"
        } else {
            "https://brittlq.com/login-redirect"
        };

        let verification_url = format!(
            "https://id.twitch.tv/oauth2/token\
            ?client_id={}\
            &client_secret={}\
            &code={}\
            &grant_type=authorization_code\
            &redirect_uri={}",
            *CLIENT_ID, *CLIENT_SECRET, &unverified_jwt.code, &redirect_url
        );

        let client = reqwest::Client::new();
        let token = client
            .post(verification_url)
            .send()
            .await
            .unwrap()
            .json::<AuthToken>()
            .await
            .unwrap(); // TODO error handling
        let refresh_token = token.refresh_token;
        let valid_id_token = validate_jwt(&token.id_token).await.unwrap(); // TODO error handling
        let claims = valid_id_token.claims;
        insert_or_update_user(&claims.email, &refresh_token, &pool)
            .await
            .unwrap();
        Ok(with_authorization_cookie(warp::reply(), &token.id_token))
        // Ok(warp::reply())
    }

    fn with_cookie<T: warp::reply::Reply>(
        reply: T,
        key: &str,
        value: &str,
    ) -> warp::reply::WithHeader<T> {
        warp::reply::with_header(reply, key, value)
    }

    fn with_authorization_cookie<T: warp::reply::Reply>(
        reply: T,
        auth_value: &str,
    ) -> warp::reply::WithHeader<T> {
        with_cookie(
            reply,
            "Set-Cookie",
            &format!("id_token={}; HttpOnly; Secure", auth_value),
        )
    }
}
fn refresh_token(refresh_token: &str) {
    let client = reqwest::Client::new();
    // TODO Send post request to:
    //  https://id.twitch.tv/oauth2/token?grant_type=refresh_token&refresh_token=<your refresh token>&client_id=<your client ID>&client_secret=<your client secret>
    // If 401, delete the refresh token from the DB. Otherwise,
}

mod endpoints {
    use super::handlers;
    use super::AuthCode;
    use crate::database::with_pool;
    use warp::Filter;

    pub fn login_redirect(
        pool: sqlx::postgres::PgPool,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("login-redirect")
            .and(warp::query::<AuthCode>())
            .and(with_pool(pool))
            .and_then(handlers::login_redirect)
    }
}
