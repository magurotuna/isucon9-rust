use anyhow::{Context, Result};
use std::env;
use tide::sessions::{MemoryStore, SessionMiddleware};

mod consts;
mod handlers;
mod models;

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    port.parse::<i32>()
        .context("failed to read DB port number from an environment variable MYSQL_PORT.")?;
    let user = env::var("MYSQL_USER").unwrap_or_else(|_| "isucari".to_string());
    let dbname = env::var("MYSQL_DBNAME").unwrap_or_else(|_| "isucari".to_string());
    let password = env::var("MYSQL_PASS").unwrap_or_else(|_| "isucari".to_string());

    let dsn = format!(
        "mysql://{}:{}@{}:{}/{}?charset=utf8mb4&parseTime=true&loc=Local",
        user, password, host, port, dbname
    );
    let conn = connect(&dsn).await?;
    let state = AppState { conn };

    let mut app = tide::with_state(state);
    app.with(SessionMiddleware::new(
        MemoryStore::new(),
        consts::SESSION_SECRET.as_bytes(),
    ));

    // API
    app.at("/initialize").post(handlers::post_initialize);
    app.at("/new_items.json").get(handlers::get_new_items);
    app.at("/new_items/:root_category_id.json")
        .get(handlers::get_new_category_items);
    app.at("users/transactions.json")
        .get(handlers::get_transactions);
    app.at("/items/:item_id.json").get(handlers::get_item_id);
    app.at("/items/edit").post(handlers::post_item_edit);
    app.at("/buy").post(handlers::post_buy);
    app.at("/sell").post(handlers::post_sell);
    app.at("/ship").post(handlers::post_ship);
    app.at("/ship_done").post(handlers::post_ship_done);
    app.at("/complete").post(handlers::post_complete);
    app.at("/transactions/:transaction_evidence_id.png")
        .get(handlers::get_qr_code);
    app.at("/bump").post(handlers::post_bump);
    app.at("/settings").get(handlers::get_settings);
    app.at("/login").post(handlers::post_login);
    app.at("/register").post(handlers::post_register);
    app.at("/reports.json").get(handlers::get_reports);

    // Frontend
    app.at("/").get(handlers::get_index);
    app.at("/login").get(handlers::get_index);
    app.at("/register").get(handlers::get_index);
    app.at("/timeline").get(handlers::get_index);
    app.at("/categories/:category_id/items")
        .get(handlers::get_index);
    app.at("/sell").get(handlers::get_index);
    app.at("/items/:item_id").get(handlers::get_index);
    app.at("/items/:item_id/edit").get(handlers::get_index);
    app.at("/items/:item_id/buy").get(handlers::get_index);
    app.at("/buy/complete").get(handlers::get_index);
    app.at("/transactions/:transaction_id")
        .get(handlers::get_index);
    app.at("/users/:user_id").get(handlers::get_index);
    app.at("/users/setting").get(handlers::get_index);

    // Assets
    app.at("/*path").get(handlers::get_assets);

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
