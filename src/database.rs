use serde_json::Value as json_value;
use sqlx::postgres::PgPool;

#[allow(dead_code)]
pub struct Queue {
    id: i32,
    name: String,
    owner: String,
    content: Option<json_value>,
}

#[allow(dead_code)]
pub struct ChatBot {
    id: i32,
    oauth_token: String,
    refresh_token: String,
    owner: String,
}

pub async fn get_queue(email: &str, pool: &PgPool) -> anyhow::Result<Queue> {
    let queue_row = sqlx::query_as!(Queue, "SELECT * FROM queue WHERE owner = $1", &email)
        .fetch_one(pool) // TODO we could get multiple queues back here, we should support `fetch` instead
        .await?;

    Ok(queue_row)
}

pub async fn insert_queue(queue: Queue, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO queue ( name, owner, content ) VALUES ( $1, $2, $3 )",
        queue.name,
        queue.owner,
        queue.content
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_chat_bot(email: &str, pool: &PgPool) -> anyhow::Result<ChatBot> {
    let chat_bot = sqlx::query_as!(ChatBot, "SELECT * from chat_bot WHERE owner = $1", &email)
        .fetch_one(pool)
        .await?;

    Ok(chat_bot)
}

pub async fn insert_chat_bot(chat_bot: ChatBot, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO chat_bot ( oauth_token, refresh_token, owner ) VALUES ( $1, $2, $3 )",
        chat_bot.oauth_token,
        chat_bot.refresh_token,
        chat_bot.owner
    )
    .execute(pool)
    .await?;
    Ok(())
}
