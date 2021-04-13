use crate::{chatbot::*, StateCommand, StateRx, StateTx};
use serde::Deserialize;
use tokio::sync::oneshot::error::RecvError;
use warp::Filter;
// TODO expand on the Claims struct
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
    iat: usize,
    nonce: Option<String>,
    email: String,
    email_verified: bool,
}

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
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    endpoints::queue_get(tx.clone())
        .or(endpoints::queue_pop(tx.clone(), chatbot_tx.clone()))
        .or(endpoints::queue_toggle(tx.clone(), chatbot_tx))
        .or(endpoints::user_delete(tx))
}

mod handlers {
    use super::{dispatch, NextQueryArg};
    use crate::*;
    use crate::{StateCommand, StateTx};
    use std::convert::Infallible;
    use tokio::sync::oneshot;

    pub async fn delete_user(
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

    pub async fn get_queue(tx: StateTx) -> Result<impl warp::Reply, Infallible> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let queue_status = dispatch(tx, resp_rx, StateCommand::GetQueue(resp_tx))
            .await
            .unwrap(); // TODO error handling
        Ok(warp::reply::json(&queue_status))
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
            .send(chatbot::Commands::SendMessage(format!(
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
        log::debug!("Popping: {}", args.count.unwrap_or(4)); // TODO error handling
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
                .send(chatbot::Commands::SendMessage(format!(
                    "Up next: @{}. You can reach BK in game with the following message: @brittleknee Hi.",
                    names_message
                )))
                .await
                .unwrap(); // TODO error handling
        }
        Ok(warp::reply::json(&popped_entries))
    }
}

mod endpoints {
    use warp::Filter;

    use super::NextQueryArg;
    use crate::with_tx;
    use crate::{chatbot, StateTx};

    use super::handlers::{delete_user, get_queue, pop_queue, toggle_queue};

    // TODO all endpoints need to have the name or ID of the queue
    // GET /queue
    pub fn queue_get(
        tx: StateTx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("queue")
            .and(warp::get())
            .and(with_tx(tx))
            .and_then(get_queue)
    }

    // TODO all endpoints below this point need to be protected by the Authorization: Bearer header
    // DELETE /queue/:name
    pub fn user_delete(
        tx: StateTx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("queue" / String)
            .and(warp::delete())
            .and(with_tx(tx))
            .and(warp::header("Authorization"))
            .and_then(delete_user)
    }

    // GET /queue/toggle
    pub fn queue_toggle(
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("queue" / "toggle")
            .and(warp::get())
            .and(with_tx(tx))
            .and(with_tx(chatbot_tx))
            .and_then(toggle_queue)
    }

    // GET /queue/pop?:u16
    pub fn queue_pop(
        tx: StateTx,
        chatbot_tx: chatbot::Tx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("queue" / "pop")
            .and(warp::get())
            .and(warp::query::<NextQueryArg>())
            .and(with_tx(tx))
            .and(with_tx(chatbot_tx))
            .and_then(pop_queue)
    }
}
