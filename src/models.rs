use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row};

type Time = DateTime<Utc>;

#[derive(Deserialize)]
pub(crate) struct ReqInitialize {
    pub(crate) payment_service_url: String,
    pub(crate) shipment_service_url: String,
}

#[derive(Serialize)]
pub(crate) struct ResInitialize {
    pub(crate) campaign: i32,
    pub(crate) language: String,
}

pub(crate) struct Item {
    pub(crate) id: i64,
    pub(crate) seller_id: i64,
    pub(crate) buyer_id: i64,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) price: i32,
    pub(crate) description: String,
    pub(crate) image_name: String,
    pub(crate) category_id: i32,
    pub(crate) created_at: Time,
    pub(crate) updated_at: Time,
}

impl<'c> FromRow<'c, MySqlRow<'c>> for Item {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.try_get("id")?;
        let seller_id: i64 = row.try_get("seller_id")?;
        let buyer_id: i64 = row.try_get("buyer_id")?;
        let status: String = row.try_get("status")?;
        let name: String = row.try_get("name")?;
        let price: i32 = row.try_get("price")?;
        let description: String = row.try_get("description")?;
        let image_name: String = row.try_get("image_name")?;
        let category_id: i32 = row.try_get("category_id")?;
        let created_at: Time = row.try_get("created_at")?;
        let updated_at: Time = row.try_get("updated_at")?;
        Ok(Item {
            id,
            seller_id,
            buyer_id,
            status,
            name,
            price,
            description,
            image_name,
            category_id,
            created_at,
            updated_at,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct UserSimple {
    pub(crate) id: i64,
    pub(crate) account_name: String,
    pub(crate) num_sell_items: i32,
}

#[derive(Serialize)]
pub(crate) struct ItemSimple {
    pub(crate) id: i64,
    pub(crate) seller_id: i64,
    pub(crate) seller: UserSimple,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) price: i32,
    pub(crate) image_url: String,
    pub(crate) category_id: i32,
    pub(crate) category: Category,
    pub(crate) created_at: i64,
}

#[derive(Serialize)]
pub(crate) struct Category {
    pub(crate) id: i32,
    pub(crate) parent_id: i32,
    pub(crate) category_name: String,
    pub(crate) parent_category_name: String,
}

#[derive(Serialize)]
pub(crate) struct ResNewItems {
    pub(crate) root_category_id: Option<i32>,
    pub(crate) root_category_name: Option<String>,
    pub(crate) has_next: bool,
    pub(crate) items: Vec<ItemSimple>,
}
