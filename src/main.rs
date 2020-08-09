use std::env;
use anyhow::{Result, Context};

mod handlers;
mod consts;

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    port.parse::<i32>().context("failed to read DB port number from an environment variable MYSQL_PORT.")?;
    let user = env::var("MYSQL_USER").unwrap_or_else(|_| "isucari".to_string());
    let dbname = env::var("MYSQL_DBNAME").unwrap_or_else(|_| "isucari".to_string());
    let password = env::var("MYSQL_PASS").unwrap_or_else(|_| "isucari".to_string());

    let dsn = format!("{}:{}@tcp({}:{})/{}?charset=utf8mb4&parseTime=true&loc=Local", user, password, host, port, dbname);
    let conn = connect(&dsn).await?;
    let state = AppState { conn };

    let mut app = tide::with_state(state);
    // API
    // TODO

    // Frontend
    app.at("/").get(handlers::index);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn connect(url: &str) -> Result<sqlx::MySqlPool> {
    let pool = sqlx::Pool::new(url).await?;
    Ok(pool)
}

#[derive(Clone)]
struct AppState {
    conn: sqlx::MySqlPool,
}
