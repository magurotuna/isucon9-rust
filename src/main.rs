use std::env;
use anyhow::{Result, Context};
use async_std::prelude::*;

mod consts {
    use std::time::Duration;

    pub(crate) const SESSION_NAME: &str = "session_isucari";

    pub(crate) const DEFAULT_PAYMENT_SERVICE_URL: &str = "http://localhost:5555";
    pub(crate) const DEFAULT_SHIPMENT_SERVICE_URL: &str = "http://localhost:7000";

    pub(crate) const ITEM_MIN_PRICE: i32 = 100;
    pub(crate) const ITEM_MAX_PRICE: i32 = 1_000_000;
    pub(crate) const ITEM_PRICE_ERR_MSG: &str =
        "商品価格は100ｲｽｺｲﾝ以上、1,000,000ｲｽｺｲﾝ以下にしてください";

    pub(crate) const ITEM_STATUS_ON_SALE: &str = "on_sale";
    pub(crate) const ITEM_STATUS_TRADING: &str = "trading";
    pub(crate) const ITEM_STATUS_SOLD_OUT: &str = "sold_out";
    pub(crate) const ITEM_STATUS_STOP: &str = "stop";
    pub(crate) const ITEM_STATUS_CANCEL: &str = "cancel";

    pub(crate) const PAYMENT_SERVICE_ISUCARI_API_KEY: &str =
        "a15400e46c83635eb181-946abb51ff26a868317c";
    pub(crate) const PAYMENT_SERCICE_ISUCARI_SHOP_ID: &str = "11";

    pub(crate) const TRANSACTION_EVIDENCE_STATUS_WAIT_SHIPPING: &str = "wait_shipping";
    pub(crate) const TRANSACTION_EVIDENCE_STATUS_WAIT_DONE: &str = "wait_done";
    pub(crate) const TRANSACTION_EVIDENCE_STATUS_DONE: &str = "done";

    pub(crate) const SHIPPINGS_STATUS_INITIAL: &str = "initial";
    pub(crate) const SHIPPINGS_STATUS_WAIT_PICKUP: &str = "wait_pickup";
    pub(crate) const SHIPPINGS_STATUS_SHIPPING: &str = "shipping";
    pub(crate) const SHIPPINGS_STATUS_DONE: &str = "done";

    pub(crate) const BUMP_HARGE_SECONDS: Duration = Duration::from_secs(3);

    pub(crate) const ITEMS_PER_PAGE: i32 = 48;
    pub(crate) const TRANSACTION_PER_PAGE: i32 = 10;

    pub(crate) const BCRYPT_COST: i32 = 10;
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();

    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    port.parse::<i32>().context("failed to read DB port number from an environment variable MYSQL_PORT.")?;
    let user = env::var("MYSQL_USER").unwrap_or_else(|_| "isucari".to_string());
    let dbname = env::var("MYSQL_DBNAME").unwrap_or_else(|_| "isucari".to_string());
    let password = env::var("MYSQL_PASS").unwrap_or_else(|_| "isucari".to_string());

    let dsn = format!("{}:{}@tcp({}:{})/{}?charset=utf8mb4&parseTime=true&loc=Local", user, password, host, port, dbname);
    let conn = connect(&dsn).await?;
    Ok(())
}

async fn connect(url: &str) -> Result<sqlx::MySqlPool> {
    let pool = sqlx::Pool::new(url).await?;
    Ok(pool)
}
