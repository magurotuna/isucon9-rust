use std::time::Duration;

pub(crate) const SESSION_NAME: &str = "session_isucari";
pub(crate) const SESSION_SECRET: &str = "THIS_IS_SESSION_SECRET_KEY_FOR_ISUCARI_APP";

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

pub(crate) const USER_AGENT: &str = "isucon9-qualify-webapp";
pub(crate) const ISUCARI_API_TOKEN: &str = "Bearer 75ugk2m37a750fwir5xr-22l6h4wmue1bwrubzwd0";
