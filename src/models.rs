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

#[derive(Deserialize)]
pub(crate) struct APIShipmentStatusReq {
    pub(crate) reserve_id: String,
}

#[derive(Serialize)]
pub(crate) struct APIShipmentStatusRes {
    pub(crate) status: String,
    pub(crate) reserve_time: u64,
}

#[derive(Serialize)]
pub(crate) struct ResTransactions {
    pub(crate) has_next: bool,
    pub(crate) items: Vec<ItemDetail>,
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

impl<'c> FromRow<'c, MySqlRow> for Item {
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
    pub(crate) id: u64,
    pub(crate) account_name: String,
    #[serde(skip)]
    pub(crate) hashed_password: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) address: Option<String>,
    pub(crate) num_sell_items: i32,
    #[serde(skip)]
    pub(crate) last_bump: Time,
    #[serde(skip)]
    pub(crate) created_at: Time,
}

impl<'c> FromRow<'c, MySqlRow> for User {
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
pub(crate) struct ItemDetail {
    pub(crate) id: u64,
    pub(crate) seller_id: u64,
    pub(crate) seller: UserSimple,
    pub(crate) buyer_id: Option<u64>,
    pub(crate) buyer: Option<UserSimple>,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) price: i32,
    pub(crate) description: String,
    pub(crate) image_url: String,
    pub(crate) category_id: u32,
    pub(crate) category: Category,
    pub(crate) transaction_evidence_id: Option<u64>,
    pub(crate) transaction_evidence_status: Option<String>,
    pub(crate) shipping_status: Option<String>,
    pub(crate) created_at: Time,
}

#[derive(Serialize)]
pub(crate) struct TransactionEvidence {
    pub(crate) id: u64,
    pub(crate) seller_id: u64,
    pub(crate) buyer_id: u64,
    pub(crate) status: String,
    pub(crate) item_id: u64,
    pub(crate) item_name: String,
    pub(crate) item_price: i32,
    pub(crate) item_description: String,
    pub(crate) item_category_id: u32,
    pub(crate) item_root_category_id: u32,
    #[serde(skip)]
    pub(crate) created_at: Time,
    #[serde(skip)]
    pub(crate) updated_at: Time,
}

impl<'c> FromRow<'c, MySqlRow> for TransactionEvidence {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id: u64 = row.try_get("id")?;
        let seller_id: u64 = row.try_get("seller_id")?;
        let buyer_id: u64 = row.try_get("buyer_id")?;
        let status: String = row.try_get("status")?;
        let item_id: u64 = row.try_get("item_id")?;
        let item_name: String = row.try_get("item_name")?;
        let item_price: i32 = row.try_get("item_price")?;
        let item_description: String = row.try_get("item_description")?;
        let item_category_id: u32 = row.try_get("item_category_id")?;
        let item_root_category_id: u32 = row.try_get("item_root_category_id")?;
        let created_at: Time = row.try_get("created_at")?;
        let updated_at: Time = row.try_get("updated_at")?;
        Ok(TransactionEvidence {
            id,
            seller_id,
            buyer_id,
            status,
            item_id,
            item_name,
            item_price,
            item_description,
            item_category_id,
            item_root_category_id,
            created_at,
            updated_at,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct Shipping {
    pub(crate) transaction_evidence_id: u64,
    pub(crate) status: String,
    pub(crate) item_name: String,
    pub(crate) item_id: u64,
    pub(crate) reserve_id: String,
    pub(crate) reserve_time: u64,
    pub(crate) to_address: String,
    pub(crate) to_name: String,
    pub(crate) from_address: String,
    pub(crate) from_name: String,
    #[serde(skip)]
    pub(crate) img_binary: Vec<u8>,
    #[serde(skip)]
    pub(crate) created_at: Time,
    #[serde(skip)]
    pub(crate) updated_at: Time,
}

impl<'c> FromRow<'c, MySqlRow> for Shipping {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let transaction_evidence_id: u64 = row.try_get("transaction_evidence_id")?;
        let status: String = row.try_get("status")?;
        let item_name: String = row.try_get("item_name")?;
        let item_id: u64 = row.try_get("item_id")?;
        let reserve_id: String = row.try_get("reserve_id")?;
        let reserve_time: u64 = row.try_get("reserve_time")?;
        let to_address: String = row.try_get("to_address")?;
        let to_name: String = row.try_get("to_name")?;
        let from_address: String = row.try_get("from_address")?;
        let from_name: String = row.try_get("from_name")?;
        let img_binary: Vec<u8> = row.try_get("img_binary")?;
        let created_at: Time = row.try_get("created_at")?;
        let updated_at: Time = row.try_get("updated_at")?;
        Ok(Shipping {
            transaction_evidence_id,
            status,
            item_name,
            item_id,
            reserve_id,
            reserve_time,
            to_address,
            to_name,
            from_address,
            from_name,
            img_binary,
            created_at,
            updated_at,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct Category {
    pub(crate) id: u32,
    pub(crate) parent_id: u32,
    pub(crate) category_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_category_name: Option<String>,
}

impl<'c> FromRow<'c, MySqlRow> for Category {
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
