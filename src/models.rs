use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row as _};

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

#[derive(Serialize)]
pub(crate) struct Item {
    pub(crate) id: u64,
    pub(crate) seller_id: u64,
    pub(crate) buyer_id: u64,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) price: i32,
    pub(crate) description: String,
    pub(crate) image_name: String,
    pub(crate) category_id: u32,
    #[serde(skip)]
    pub(crate) created_at: Time,
    #[serde(skip)]
    pub(crate) updated_at: Time,
}

impl<'c> FromRow<'c, MySqlRow<'c>> for Item {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: u64 = row.try_get("id")?;
        let seller_id: u64 = row.try_get("seller_id")?;
        let buyer_id: u64 = row.try_get("buyer_id")?;
        let status: String = row.try_get("status")?;
        let name: String = row.try_get("name")?;
        let price: i32 = row.try_get("price")?;
        let description: String = row.try_get("description")?;
        let image_name: String = row.try_get("image_name")?;
        let category_id: u32 = row.try_get("category_id")?;
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
pub(crate) struct User {
    id: u64,
    account_name: String,
    #[serde(skip)]
    hashed_password: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    num_sell_items: i32,
    #[serde(skip)]
    last_bump: Time,
    #[serde(skip)]
    created_at: Time,
}

impl<'c> FromRow<'c, MySqlRow<'c>> for User {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: u64 = row.try_get("id")?;
        let account_name: String = row.try_get("account_name")?;
        let hashed_password: Vec<u8> = row.try_get("hashed_password")?;
        let address: Option<String> = row.try_get("address")?;
        let num_sell_items: i32 = row.try_get("num_sell_items")?;
        let last_bump: Time = row.try_get("last_bump")?;
        let created_at: Time = row.try_get("created_at")?;
        Ok(User {
            id,
            account_name,
            hashed_password,
            address,
            num_sell_items,
            last_bump,
            created_at,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct UserSimple {
    pub(crate) id: u64,
    pub(crate) account_name: String,
    pub(crate) num_sell_items: i32,
}

impl From<User> for UserSimple {
    fn from(user: User) -> Self {
        UserSimple {
            id: user.id,
            account_name: user.account_name,
            num_sell_items: user.num_sell_items,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ItemSimple {
    pub(crate) id: u64,
    pub(crate) seller_id: u64,
    pub(crate) seller: UserSimple,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) price: i32,
    pub(crate) image_url: String,
    pub(crate) category_id: u32,
    pub(crate) category: Category,
    pub(crate) created_at: i64,
}

#[derive(Serialize)]
pub(crate) struct Category {
    pub(crate) id: u32,
    pub(crate) parent_id: u32,
    pub(crate) category_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_category_name: Option<String>,
}

impl<'c> FromRow<'c, MySqlRow<'c>> for Category {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: u32 = row.try_get("id")?;
        let parent_id: u32 = row.try_get("parent_id")?;
        let category_name: String = row.try_get("category_name")?;
        Ok(Category {
            id,
            parent_id,
            category_name,
            parent_category_name: None,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct ResNewItems {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root_category_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root_category_name: Option<String>,
    pub(crate) has_next: bool,
    pub(crate) items: Vec<ItemSimple>,
}
