use sqlx::postgres::PgPool;
use warp::Filter;

#[derive(sqlx::FromRow)]
pub struct User {
    email: String,
    refresh_token: Option<String>,
    id: i32,
}

pub fn with_pool(
    pool: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

pub async fn select_user(email: &str, pool: &PgPool) -> User {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", &email)
        .fetch_one(pool)
        .await
        .unwrap() // TODO error handling
}

pub async fn insert_or_update_user(
    email: &str,
    refresh_token: &str,
    pool: &PgPool,
) -> Result<(), std::convert::Infallible> {
    sqlx::query!(
        "INSERT INTO users (email, refresh_token) VALUES ($1, $2) ON CONFLICT (email) DO UPDATE SET refresh_token = $2",
        email,
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
