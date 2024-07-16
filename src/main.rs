use std::env;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, query_as, Acquire, Pool, Postgres};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load env variables");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env variable not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Error connecting to database");

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = create_user_in_database(payload, &pool).await;

    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
}

struct CreateLogEntry {
    content: String,
}

struct LogEntry {
    id: i32,
    content: String,
}

async fn create_user_in_database<'a, A: Acquire<'a, Database = Postgres> + Send>(
    user: CreateUser,
    conn: A,
) -> User {
    let mut conn = conn.acquire().await.unwrap();

    create_log_entry_in_database(
        CreateLogEntry {
            content: "Creating new user".to_string(),
        },
        &mut *conn,
    )
    .await;

    query_as!(
        User,
        "INSERT INTO users (name) VALUES ($1) RETURNING id, name",
        user.name
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap()
}

async fn create_log_entry_in_database<'a, A: Acquire<'a, Database = Postgres> + Send>(
    entry: CreateLogEntry,
    conn: A,
) -> LogEntry {
    let mut conn = conn.acquire().await.unwrap();

    query_as!(
        LogEntry,
        "INSERT INTO logs (content) VALUES ($1) RETURNING id, content",
        entry.content
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap()
}
