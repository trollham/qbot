use crate::{database::insert_or_update_user, CLIENT_ID, CLIENT_SECRET};
use async_trait::async_trait;
use errors::{reject::*, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod errors;
pub mod twitch;

pub async fn extract_auth(token: String) -> Result<Claims, warp::Rejection> {
    let unvalidated_token = if token.starts_with("Bearer") {
        match token.split_whitespace().last() {
            Some(t) => t,
            None => return Err(warp::reject::custom(Unauthorized)),
        }
    } else {
        &token
    };
    match validate_jwt(unvalidated_token).await {
        Ok(c) => {
            tracing::info!("JWT successfully validated");
            Ok(c)
        }
        Err(e) => {
            tracing::info!("Failed to validate JWT: {}", &token);
            match e {
                Error::Fetch(_) => Err(warp::reject::custom(BadGateway)),
                _ => Err(warp::reject::custom(Unauthorized)),
            }
        }
    }
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

#[derive(Debug, Deserialize)]
pub(crate) struct AuthCode {
    code: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    iat: usize,
    nonce: Option<String>,
    pub email: String,
    pub preferred_username: String,
}

pub mod endpoints {
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
            .with(warp::trace(
                |info| tracing::info_span!("request", method = %info.method(), path = %info.path(),),
            ))
    }
}

async fn refresh_token(refresh_token: &str, pool: &sqlx::postgres::PgPool) -> Result<(), Error> {
    let redirect_url = if cfg!(debug_assertions) {
        "http://localhost:3000/login-redirect"
    } else {
        "https://brittlq.com/login-redirect"
    };

    let app_root_url = if cfg!(debug_assertions) {
        "http://localhost:8080"
    } else {
        "https://brittlq.com"
    };

    let refresh_url =
        format!("https://id.twitch.tv/oauth2/token?grant_token=refresh_token&refresh_token={}&client_id={}&client_secret={}", urlencoding::encode(&refresh_token), *CLIENT_ID, *CLIENT_SECRET);

    make_auth_request(&refresh_url, pool).await;
    Ok(())
}

#[async_trait]
trait Authenticator {
    async fn authenticate(&self) -> Result<AuthToken, Error>;
    async fn refresh(&self) -> Result<AuthToken, Error>;
    async fn validate_jwt(&self, token: &str) -> Result<Claims, Error>;
}

async fn make_auth_request<T>(authenticator: &T, pool: &sqlx::postgres::PgPool)
where
    T: Authenticator,
{
    let client = reqwest::Client::new();
    let token = client
        .post(url)
        .send()
        .await
        .unwrap() // TODO error handling
        .json::<AuthToken>()
        .await
        .unwrap(); // TODO error handling
    let refresh_token = token.refresh_token;
    let claims = authenticator.validate_jwt(&token.id_token).await.unwrap(); // TODO error handling

    insert_or_update_user(
        &claims.email,
        &claims.preferred_username.to_lowercase(),
        &refresh_token,
        pool,
    )
    .await
    .unwrap();
}

mod handlers {
    use super::AuthCode;
    use crate::server::authentication::make_auth_request;
    use crate::{CLIENT_ID, CLIENT_SECRET};
    use std::convert::Infallible;

    pub(crate) async fn login_redirect(
        unverified_jwt: AuthCode,
        pool: sqlx::postgres::PgPool,
    ) -> Result<impl warp::Reply, Infallible> {
        let redirect_url = if cfg!(debug_assertions) {
            "http://localhost:3000/login-redirect"
        } else {
            "https://brittlq.com/login-redirect"
        };

        let app_root_url = if cfg!(debug_assertions) {
            "http://localhost:8080"
        } else {
            "https://brittlq.com"
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

        make_auth_request(&verification_url, &pool).await; // TODO error handling

        Ok(with_authorization_cookie(
            warp::redirect::see_other(
                warp::http::Uri::from_maybe_shared(format!(
                    "{}/{}",
                    app_root_url,
                    claims.preferred_username.to_lowercase()
                ))
                .unwrap(), // TODO error handling
            ),
            &token.id_token,
        ))
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
