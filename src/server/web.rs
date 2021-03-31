use warp::Filter;

use crate::chatbot;

pub fn build_server(
    tx: chatbot::Tx,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    endpoints::token(tx).or(warp::fs::dir("./www/dist/"))
}

mod handlers {
    use std::convert::Infallible;

    use crate::{chatbot, Token};

    pub async fn send_token(token: Token, tx: chatbot::Tx) -> Result<impl warp::Reply, Infallible> {
        Ok(warp::reply::json(
            &tx.send(chatbot::Commands::Token(token)).await.unwrap(),
        ))
    }
}

mod endpoints {
    use super::handlers::send_token;
    use crate::chatbot;
    use crate::with_tx;
    use warp::Filter;

    pub fn token(
        tx: chatbot::Tx,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("queue" / "token")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_tx(tx))
            .and_then(send_token)
    }
}
