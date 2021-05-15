use chrono::prelude::*;
use dotenv::dotenv;
use env_logger::Env;
use qbot::*;
use std::collections::VecDeque;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    tracing::info!("starting application");
    dotenv().ok();

    let (state_tx, mut state_rx) = tokio::sync::mpsc::channel(32);
    let (chat_tx, mut chat_rx) = tokio::sync::mpsc::channel(4);
    let bot_state_tx = state_tx.clone();

    let state_task = tokio::spawn(async move {
        let mut state = Queue {
            queue: VecDeque::new(),
            is_open: false,
        };

        while let Some(command) = state_rx.recv().await {
            match command {
                StateCommand::AddUser { user, tx } => {
                    let pos = find(&user, &state.queue);

                    if let Some(index) = pos {
                        tx.send(index).unwrap(); // TODO error handling
                    } else {
                        state.queue.push_back(UserEntry {
                            nickname: user,
                            time_joined: Local::now(),
                            id: Uuid::new_v4(),
                        });
                        tx.send(state.queue.len() - 1).unwrap(); // TODO error handling
                    }
                }
                StateCommand::GetQueue(tx) => {
                    tx.send(serde_json::to_value(&state).unwrap()).unwrap(); // TODO error handling
                }

                StateCommand::GetQueueStatus(tx) => {
                    tx.send(state.is_open).unwrap(); // TODO error handling
                }

                StateCommand::FindUser { name, tx } => {
                    tx.send(find(&name, &state.queue)).unwrap(); // TODO error handling
                }

                StateCommand::PeekQueue { count, tx } => {
                    let first_n: Vec<_> =
                        state.queue.iter().take(count as usize).cloned().collect();
                    tx.send(first_n).unwrap(); // TODO error handling
                }

                StateCommand::PopQueue { count, tx } => {
                    let popped_users = pop(count, &mut state.queue);
                    tx.send(popped_users).unwrap(); // TODO error handling
                }

                StateCommand::RemoveUser { user, tx } => {
                    tx.send(remove(&user, &mut state.queue)).unwrap(); // TODO error handling
                }

                StateCommand::ToggleQueue(tx) => {
                    state.is_open = !state.is_open;
                    tx.send(state.is_open).unwrap(); // TODO error handling
                }
            }
        }
        // Ok(()) as anyhow::Result<()>
    });

    // Initialize DB connections
    let database_url = std::env::var("DATABASE_URL").unwrap(); // TODO error handling
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .unwrap(); // TODO error handling

    // let web_task = tokio::spawn(async move {
    //     let server = warp::serve(server::web::build_server(pool));
    //     server.run(([127, 0, 0, 1], 8080)).await;
    //     // Ok(()) as anyhow::Result<()>
    // });
    let api_task = tokio::spawn(async move {
        let server = warp::serve(server::api::build_server(state_tx, chat_tx, pool));
        server.run(([127, 0, 0, 1], 3000)).await;
        // Ok(()) as anyhow::Result<()>
    });

    let mut auth = String::new();
    if let Some(chatbot::Commands::Token(token)) = chat_rx.recv().await {
        auth = format!("oauth:{}", token.access_token);
    }

    let mut bot = chatbot::Bot::new(get_user_config(&auth), chat_rx)
        .await
        .unwrap(); // TODO error handling

    let bot_task = tokio::spawn(async move {
        chatbot::build_bot(&mut bot);
        bot.run(bot_state_tx).await
    });

    tokio::select! {
        _ = bot_task => {
            tracing::debug!("Bot task exited.");
            // Ok(()) as anyhow::Result<()>
        }
        // _ = web_task => {
        //     tracing::debug!("Web server task exited.");
        //     // Ok(()) as anyhow::Result<()>
        // }
        _ = api_task => {
            tracing::debug!("Api server task exited.");
            // Ok(()) as anyhow::Result<()>
        }
        _ = state_task => {
            tracing::debug!("State task exited.");
            // Ok(()) as anyhow::Result<()>
        }
    }
}
