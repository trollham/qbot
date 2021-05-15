use std::convert::Infallible;

use crate::{chatbot::*, StateCommand, StateRx, StateTx};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::oneshot::error::RecvError;
use warp::Filter;

use super::authentication::{self, errors::reject::Unauthorized, Claims};

#[derive(Debug, Deserialize)]
pub struct NextQueryArg {
    count: Option<u16>,
}

async fn dispatch<T>(tx: StateTx, rx: StateRx<T>, command: StateCommand) -> Result<T, RecvError> {
    tx.send(command).await.unwrap(); // TODO error handling
    rx.await
}

pub fn build_server(
    tx: StateTx,
    chatbot_tx: Tx,
    pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    endpoints::queue_get(tx.clone(), pool.clone())
        .or(endpoints::queue_pop(tx.clone(), chatbot_tx.clone()))
        .or(endpoints::queue_toggle(tx.clone(), chatbot_tx))
        .or(endpoints::user_delete(tx))
        .or(authentication::endpoints::login_redirect(pool))
        .recover(handle_rejection)
}

#[derive(Default, Deserialize, Serialize)]
struct QueueApiResponse {
    #[serde(flatten)]
    queue: Value,
    is_owner: Option<bool>,
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(Unauthorized) = err.find::<Unauthorized>() {
        code = warp::http::StatusCode::UNAUTHORIZED;
        message = "Unauthorized";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

mod handlers {
    use chatbot::Commands;
    use tokio::sync::oneshot;

    use crate::{
        chatbot, database,
        server::{api::dispatch, authentication::Claims},
        StateCommand, StateTx,
    };
    use std::convert::Infallible;

    use super::{NextQueryArg, QueueApiResponse};

    pub async fn delete_user(
        queue_owner: String,
        user: String,
        tx: StateTx,
        auth: String,
    ) -> Result<impl warp::Reply, Infallible> {
        println!("Auth: {}", auth);
        let (resp_tx, resp_rx) = oneshot::channel();
        let removed_users = dispatch(tx, resp_rx, StateCommand::RemoveUser { user, tx: resp_tx })
            .await
            .unwrap(); // TODO error handling
        Ok(warp::reply::json(&removed_users))
    }

    pub async fn get_queue(
        user: String,
        claims: Claims,
        tx: StateTx,
        pool: sqlx::PgPool,
    ) -> Result<impl warp::Reply, Infallible> {
        let queue = database::get_user_queue(&user, &pool)
            .await
            .unwrap()
            .content; // TODO error handling
        let is_owner = if claims.preferred_username.to_lowercase() == user.to_lowercase() {
            Some(true)
        } else {
            None
        };

        let response = QueueApiResponse { queue, is_owner };

        Ok(warp::reply::json(&response))
    }

    pub async fn toggle_queue(
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> Result<impl warp::Reply, Infallible> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let queue_status = dispatch(tx, resp_rx, StateCommand::ToggleQueue(resp_tx))
            .await
            .unwrap(); // TODO error handling
        chatbot_tx
            .send(Commands::SendMessage(format!(
                "The queue is now {}.",
                if queue_status { "open" } else { "closed" }
            )))
            .await
            .unwrap(); // TODO error handling
        Ok(warp::reply::json(&queue_status))
    }

    pub async fn pop_queue(
        args: NextQueryArg,
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> Result<impl warp::Reply, Infallible> {
        let (resp_tx, resp_rx) = oneshot::channel();
        tracing::debug!("Popping: {}", args.count.unwrap_or(4)); // TODO error handling
        let popped_entries = dispatch(
            tx,
            resp_rx,
            StateCommand::PopQueue {
                count: args.count.unwrap_or(4), // TODO error handling
                tx: resp_tx,
            },
        )
        .await
        .unwrap(); // TODO error handling
        if let Some(popped) = &popped_entries {
            let temp_users = popped
                .iter()
                .map(|u| u.nickname.clone())
                .collect::<Vec<String>>();
            let names_message = temp_users.join(", @");
            chatbot_tx
                .send(Commands::SendMessage(format!(
                    "Up next: @{}. You can reach BK in game with the following message: @brittleknee Hi.",
                    names_message
                )))
                .await
                .unwrap(); // TODO error handling
        }
        Ok(warp::reply::json(&popped_entries))
    }
}

pub fn validate() -> impl Filter<Extract = (Claims,), Error = warp::Rejection> + Clone {
    warp::cookie("id_token").and_then(crate::server::authentication::extract_auth)
}

mod endpoints {
    use super::handlers::{delete_user, get_queue, pop_queue, toggle_queue};
    use super::NextQueryArg;
    use crate::{chatbot, server::api::validate, StateTx};
    use crate::{database::with_pool, with_tx};
    use warp::Filter;

    // TODO all endpoints need to have the name or ID of the queue
    // GET /queue
    pub fn queue_get(
        tx: StateTx,
        pool: sqlx::PgPool,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors().allow_credentials(true).allow_any_origin();

        warp::path!("queue" / String)
            .and(warp::get())
            .and(validate())
            .and(with_tx(tx))
            .and(with_pool(pool))
            .and_then(get_queue)
            .with(cors)
            .with(warp::trace(
                |info| tracing::info_span!("request", method = %info.method(), path = %info.path(),),
            ))
    }

    // TODO all endpoints below this point need to be protected by the Authorization: Bearer header
    // DELETE /queue/:name
    pub fn user_delete(
        tx: StateTx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_origin("http://localhost:3000")
            .allow_credentials(true)
            .allow_methods(vec!["GET", "POST"]);

        warp::path!("queue" / String / "user" / String)
            .and(warp::delete())
            .and(with_tx(tx))
            .and(warp::header("Authorization"))
            .and_then(delete_user)
            .with(cors)
            .with(warp::trace(
                |info| tracing::info_span!("request", method = %info.method(), path = %info.path(),),
            ))
    }

    // GET /queue/toggle
    pub fn queue_toggle(
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_origin("http://localhost:3000")
            .allow_credentials(true)
            .allow_methods(vec!["GET", "POST"]);

        warp::path!("queue" / "toggle")
            .and(warp::get())
            .and(with_tx(tx))
            .and(with_tx(chatbot_tx))
            .and_then(toggle_queue)
            .with(cors)
            .with(warp::trace(
                |info| tracing::info_span!("request", method = %info.method(), path = %info.path(),),
            ))
    }

    // GET /queue/pop?:u16
    pub fn queue_pop(
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_origin("http://localhost:3000")
            .allow_credentials(true)
            .allow_methods(vec!["GET", "POST"]);
        warp::path!("queue" / "pop")
            .and(warp::get())
            .and(warp::query::<NextQueryArg>())
            .and(with_tx(tx))
            .and(with_tx(chatbot_tx))
            .and_then(pop_queue)
            .with(cors)
            .with(warp::trace(
                |info| tracing::info_span!("request", method = %info.method(), path = %info.path(),),
            ))
    }
}
