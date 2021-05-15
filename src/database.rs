use sqlx::postgres::PgPool;
use warp::Filter;

pub struct User {
    email: String,
    username: String,
    refresh_token: Option<String>,
    id: i32,
}

pub struct Queue {
    id: i32,
    name: String,
    pub content: serde_json::Value,
}

pub fn with_pool(
    pool: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub async fn select_user(username: &str, pool: &PgPool) -> User {
    sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", &username)
        .fetch_one(pool)
        .await
        .unwrap() // TODO error handling
}

pub async fn insert_or_update_user(
    email: &str,
    username: &str,
    refresh_token: &str,
    pool: &PgPool,
) -> Result<(), std::convert::Infallible> {
    sqlx::query!(
        "INSERT INTO users (email, username, refresh_token) VALUES ($1, $2, $3) ON CONFLICT (email) DO UPDATE SET refresh_token = $3",
        email,
        username,
        refresh_token
    )
    .execute(pool)
    .await
    .unwrap(); // TODO error handling
    Ok(())
}

pub async fn clear_user_refresh_token(
    user: &User,
    pool: &PgPool,
) -> Result<(), std::convert::Infallible> {
    sqlx::query!(
        "UPDATE users SET refresh_token = NULL WHERE id = $1",
        user.id
    )
    .execute(pool)
    .await
    .unwrap(); // TODO error handling
    Ok(())
}

pub async fn get_user_queue(
    username: &str,
    pool: &PgPool,
) -> Result<Queue, std::convert::Infallible> {
    let queue = sqlx::query_as!(
        Queue,
        r#"SELECT queues.id, queues.name, queues.content
    FROM queues INNER JOIN users
        ON queues.owner = users.id AND users.username = $1"#,
        &username
    )
    .fetch_one(pool)
    .await
    .unwrap(); // TODO error handling
    Ok(queue)
}
