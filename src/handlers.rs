use crate::consts;
use crate::models::{
    APIShipmentStatusReq, APIShipmentStatusRes, Category, Config, Item, ItemDetail, ItemSimple,
    ReqInitialize, ResInitialize, ResNewItems, ResTransactions, Shipping, TransactionEvidence,
    User, UserSimple,
};
use crate::AppState;
use async_recursion::async_recursion;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use sqlx::mysql::{MySql, MySqlRow};
use sqlx::Executor;
use sqlx::Row as _;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use tide::{Body, Result, StatusCode};

type Request = tide::Request<AppState>;

static JSON_PATH_PARAM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.+)\.json$").unwrap());

fn with_status<E>(status_code: StatusCode) -> impl FnOnce(E) -> tide::Error
where
    E: Into<tide::Error>,
{
    move |err: E| -> tide::Error {
        let mut err = err.into();
        err.set_status(status_code);
        err
    }
}

pub(crate) async fn post_initialize(mut req: Request) -> Result<Body> {
    let body: ReqInitialize = req
        .body_json()
        .await
        .map_err(with_status(StatusCode::BadRequest))?;

    {
        let output = Command::new("./sql/init.sh")
            .output()
            .map_err(with_status(StatusCode::InternalServerError))?;
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        lock.write_all(&output.stdout)
            .map_err(with_status(StatusCode::InternalServerError))?;
        lock.write_all(&output.stderr)
            .map_err(with_status(StatusCode::InternalServerError))?;
    }

    let conn = &req.state().conn;
    sqlx::query(
        r"
        INSERT INTO `configs` (`name`, `val`) VALUES (?, ?)
        ON DUPLICATE KEY UPDATE `val` = VALUES(`val`)
        ",
    )
    .bind("payment_service_url")
    .bind(body.payment_service_url)
    .execute(conn)
    .await
    .map_err(with_status(StatusCode::InternalServerError))?;
    sqlx::query(
        r"
        INSERT INTO `configs` (`name`, `val`) VALUES (?, ?)
        ON DUPLICATE KEY UPDATE `val` = VALUES(`val`)
        ",
    )
    .bind("shipment_service_url")
    .bind(body.shipment_service_url)
    .execute(conn)
    .await
    .map_err(with_status(StatusCode::InternalServerError))?;

    let res = ResInitialize {
        campaign: 0,
        language: "Rust".to_string(),
    };
    Ok(Body::from_json(&res)?)
}

pub(crate) async fn get_new_items(req: Request) -> Result<Body> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Query {
        item_id: Option<u64>,
        created_at: Option<u64>,
    }
    let query: Query = req.query().map_err(with_status(StatusCode::BadRequest))?;
    let Query {
        item_id,
        created_at,
    } = query;

    let mut conn = req.state().conn.acquire().await?;
    let items: Vec<Item> = match (item_id, created_at) {
        (Some(item_id), Some(created_at)) => {
            sqlx::query_as(
                r"
                SELECT 
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
                    updated_at
                FROM `items`
                WHERE `status` IN (?,?)
                AND (`created_at` < ? OR (`created_at` <= ? AND `id` < ?))
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
            )
            .bind(consts::ITEM_STATUS_ON_SALE)
            .bind(consts::ITEM_STATUS_SOLD_OUT)
            .bind(created_at)
            .bind(created_at)
            .bind(item_id)
            .bind(consts::ITEMS_PER_PAGE + 1)
            .fetch_all(&mut conn)
            .await?
        }
        _ => {
            sqlx::query_as(
                r"
                SELECT 
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
                    updated_at
                FROM `items`
                WHERE `status` IN (?,?)
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
            )
            .bind(consts::ITEM_STATUS_ON_SALE)
            .bind(consts::ITEM_STATUS_SOLD_OUT)
            .bind(consts::ITEMS_PER_PAGE + 1)
            .fetch_all(&mut conn)
            .await?
        }
    };

    let mut item_simples = Vec::new();
    for item in items {
        let seller = get_user_simple_by_id(&mut conn, item.seller_id).await?;
        let category = get_category_by_id(&mut conn, item.category_id).await?;
        item_simples.push(ItemSimple {
            id: item.id,
            seller_id: item.seller_id,
            seller,
            status: item.status,
            name: item.name,
            price: item.price,
            image_url: get_image_url(item.image_name),
            category_id: item.category_id,
            category,
            created_at: item.created_at.timestamp(),
        });
    }

    let mut has_next = false;
    if item_simples.len() > consts::ITEMS_PER_PAGE as usize {
        has_next = true;
        item_simples.truncate(consts::ITEMS_PER_PAGE as usize);
    }

    let res = ResNewItems {
        root_category_id: None,
        root_category_name: None,
        has_next,
        items: item_simples,
    };

    Ok(Body::from_json(&res)?)
}

async fn get_user_simple_by_id<'e, E>(executor: &'e mut E, user_id: u64) -> Result<UserSimple>
where
    &'e mut E: Executor<'e, Database = MySql>,
{
    let user: User = sqlx::query_as(
        r"
        SELECT
            id,
            account_name,
            hashed_password,
            address,
            num_sell_items,
            last_bump,
            created_at
        FROM `users`
        WHERE `id` = ?
        ",
    )
    .bind(user_id)
    .fetch_one(executor)
    .await?;

    Ok(user.into())
}

#[async_recursion]
async fn get_category_by_id<E>(executor: &mut E, category_id: u32) -> Result<Category>
where
    E: Send,
    for<'e> &'e mut E: Executor<'e, Database = MySql>,
{
    let mut category: Category = sqlx::query_as(
        r"
        SELECT
            id,
            parent_id,
            category_name
        FROM `categories`
        WHERE `id` = ?
        ",
    )
    .bind(category_id)
    .fetch_one(&mut *executor)
    .await?;
    if category.parent_id != 0 {
        category.parent_category_name = get_category_by_id(&mut *executor, category.parent_id)
            .await
            .map(|parent_category| Some(parent_category.category_name))
            .unwrap_or(None);
    }
    Ok(category)
}

fn get_image_url(image_name: String) -> String {
    format!("/upload/{}", image_name)
}

pub(crate) async fn get_new_category_items(req: Request) -> Result<Body> {
    let param: String = req.param("root_category_id.json")?;
    let root_category_id = JSON_PATH_PARAM_RE
        .captures(param.as_str())
        .and_then(|cap| cap.get(1))
        .and_then(|it| it.as_str().parse::<u32>().ok())
        .ok_or_else(|| tide::Error::from_str(StatusCode::BadRequest, "incorrect category id"))?;

    let mut conn = req.state().conn.acquire().await?;

    let root_category = get_category_by_id(&mut conn, root_category_id).await?;
    if root_category.parent_id != 0 {
        return Err(tide::Error::from_str(
            StatusCode::NotFound,
            "category not found",
        ));
    }

    let category_ids = sqlx::query("SELECT id FROM `categories` WHERE parent_id=?")
        .bind(root_category.id)
        .try_map(|row: MySqlRow| {
            let id = row.try_get::<u32, _>("id")?;
            Ok(id.to_string())
        })
        .fetch_all(&mut conn)
        .await?
        .join(",");

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Query {
        item_id: Option<u64>,
        created_at: Option<u64>,
    }
    let query: Query = req.query().map_err(with_status(StatusCode::BadRequest))?;
    let Query {
        item_id,
        created_at,
    } = query;

    let items: Vec<Item> = match (item_id, created_at) {
        (Some(item_id), Some(created_at)) => {
            // [How to bind Vec<i64> arguments for IN operator? · Issue #528 · launchbadge/sqlx](https://github.com/launchbadge/sqlx/issues/528)
            let sql = format!(
                r"
                SELECT
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
                    updated_at
                FROM `items`
                WHERE `status` IN (?,?)
                AND `category_id` IN ({})
                AND (`created_at` < ? OR (`created_at` <= ? AND `id` < ?))
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
                &category_ids
            );
            sqlx::query_as(&sql)
                .bind(consts::ITEM_STATUS_ON_SALE)
                .bind(consts::ITEM_STATUS_SOLD_OUT)
                .bind(created_at)
                .bind(created_at)
                .bind(item_id)
                .bind(consts::ITEMS_PER_PAGE + 1)
                .fetch_all(&mut conn)
                .await?
        }
        _ => {
            let sql = format!(
                r"
                SELECT
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
                    updated_at
                FROM `items`
                WHERE `status` IN (?,?)
                AND category_id IN ({})
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
                &category_ids
            );
            sqlx::query_as(&sql)
                .bind(consts::ITEM_STATUS_ON_SALE)
                .bind(consts::ITEM_STATUS_SOLD_OUT)
                .bind(consts::ITEMS_PER_PAGE + 1)
                .fetch_all(&mut conn)
                .await?
        }
    };

    let mut item_simples = Vec::new();
    for item in items {
        let seller = get_user_simple_by_id(&mut conn, item.seller_id)
            .await
            .map_err(with_status(StatusCode::NotFound))?;
        let category = get_category_by_id(&mut conn, item.category_id)
            .await
            .map_err(with_status(StatusCode::NotFound))?;
        item_simples.push(ItemSimple {
            id: item.id,
            seller_id: item.seller_id,
            seller,
            status: item.status,
            name: item.name,
            price: item.price,
            image_url: get_image_url(item.image_name),
            category_id: item.category_id,
            category,
            created_at: item.created_at.timestamp(),
        })
    }

    let mut has_next = false;
    if item_simples.len() > consts::ITEMS_PER_PAGE as usize {
        has_next = true;
        item_simples.truncate(consts::ITEMS_PER_PAGE as usize);
    }

    let res = ResNewItems {
        root_category_id: Some(root_category.id),
        root_category_name: Some(root_category.category_name),
        items: item_simples,
        has_next,
    };

    Ok(Body::from_json(&res)?)
}

pub(crate) async fn get_transactions(req: Request) -> Result<Body> {
    let user = get_user(&req).await?;

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Query {
        item_id: Option<u64>,
        created_at: Option<u64>,
    }
    let Query {
        item_id,
        created_at,
    } = req.query().map_err(with_status(StatusCode::BadRequest))?;

    let mut tx = req.state().conn.begin().await?;

    let items: Vec<Item> = match (item_id, created_at) {
        (Some(item_id), Some(created_at)) => {
            sqlx::query_as(
                r"
                SELECT 
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
                    updated_at
                FROM `items`
                WHERE (`seller_id` = ? OR `buyer_id` = ?)
                AND `status` IN (?,?,?,?,?)
                AND (`created_at` < ? OR (`created_at` <= ? AND `id` < ?))
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
            )
            .bind(user.id)
            .bind(user.id)
            .bind(consts::ITEM_STATUS_ON_SALE)
            .bind(consts::ITEM_STATUS_TRADING)
            .bind(consts::ITEM_STATUS_SOLD_OUT)
            .bind(consts::ITEM_STATUS_CANCEL)
            .bind(consts::ITEM_STATUS_STOP)
            .bind(created_at)
            .bind(created_at)
            .bind(item_id)
            .bind(consts::TRANSACTION_PER_PAGE + 1)
            .fetch_all(&mut tx)
            .await?
        }
        _ => {
            sqlx::query_as(
                r"
                SELECT 
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
                    updated_at
                FROM `items`
                WHERE (`seller_id` = ? OR `buyer_id` = ?)
                AND `status` IN (?,?,?,?,?)
                ORDER BY `created_at` DESC, `id` DESC
                LIMIT ?
                ",
            )
            .bind(user.id)
            .bind(user.id)
            .bind(consts::ITEM_STATUS_ON_SALE)
            .bind(consts::ITEM_STATUS_TRADING)
            .bind(consts::ITEM_STATUS_SOLD_OUT)
            .bind(consts::ITEM_STATUS_CANCEL)
            .bind(consts::ITEM_STATUS_STOP)
            .bind(consts::TRANSACTION_PER_PAGE + 1)
            .fetch_all(&mut tx)
            .await?
        }
    };

    let mut item_details: Vec<ItemDetail> = Vec::new();
    for item in items {
        let seller = get_user_simple_by_id(&mut tx, item.seller_id).await?;
        let category = get_category_by_id(&mut tx, item.category_id).await?;
        let mut item_detail = ItemDetail {
            id: item.id,
            seller_id: item.seller_id,
            seller,
            buyer_id: None,
            buyer: None,
            status: item.status,
            name: item.name,
            price: item.price,
            description: item.description,
            image_url: get_image_url(item.image_name),
            category_id: item.category_id,
            transaction_evidence_id: None,
            transaction_evidence_status: None,
            shipping_status: None,
            category,
            created_at: item.created_at,
        };

        if item.buyer_id != 0 {
            let buyer = get_user_simple_by_id(&mut tx, item.buyer_id).await?;
            item_detail.buyer_id = Some(item.buyer_id);
            item_detail.buyer = Some(buyer);
        }

        let transaction_evidence: sqlx::Result<TransactionEvidence> = sqlx::query_as(
            r"
            SELECT
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
                updated_at
            FROM `transaction_evidences`
            WHERE `item_id` = ?
            ",
        )
        .bind(item.id)
        .fetch_one(&mut tx)
        .await;

        match transaction_evidence {
            Ok(t) => {
                let shipping: Shipping = sqlx::query_as(
                    r"
                    SELECT 
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
                        updated_at
                    FROM `shippings`
                    WHERE `transaction_evidence_id` = ?
                    ",
                )
                .bind(t.id)
                .fetch_one(&mut tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => tide::Error::new(StatusCode::NotFound, e),
                    _ => tide::Error::new(StatusCode::InternalServerError, e),
                })?;
                let ssr = api_shipment_status(
                    get_shipment_service_url(&mut tx).await,
                    APIShipmentStatusReq {
                        reserve_id: shipping.reserve_id,
                    },
                )
                .await?;

                item_detail.transaction_evidence_id = Some(t.id);
                item_detail.transaction_evidence_status = Some(t.status);
                item_detail.shipping_status = Some(ssr.status);
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => {
                return Err(tide::Error::new(StatusCode::InternalServerError, e));
            }
        }

        item_details.push(item_detail);
    }

    tx.commit().await?;

    let mut has_next = false;
    if item_details.len() > consts::TRANSACTION_PER_PAGE as usize {
        has_next = true;
        item_details.truncate(consts::TRANSACTION_PER_PAGE as usize);
    }

    let res = ResTransactions {
        has_next,
        items: item_details,
    };

    Ok(Body::from_json(&res)?)
}

async fn get_user(req: &Request) -> Result<User> {
    let session = req.session();
    let user_id: String = session
        .get("user_id")
        .ok_or_else(|| tide::Error::from_str(StatusCode::NotFound, "no session"))?;
    let conn = &req.state().conn;
    let user: User = sqlx::query_as(
        r"
        SELECT 
            id,
            account_name,
            hashed_password,
            address,
            num_sell_items,
            last_bump,
            created_at
        FROM `users`
        WHERE `id` = ?
        ",
    )
    .bind(user_id)
    .fetch_one(conn)
    .await?;
    Ok(user)
}

async fn get_config_by_name<'e, E>(executor: &'e mut E, name: impl AsRef<str>) -> Result<String>
where
    &'e mut E: Executor<'e, Database = MySql>,
{
    let config: Config = sqlx::query_as("SELECT name, val FROM `configs` WHERE `name` = ?")
        .bind(name.as_ref())
        .fetch_one(executor)
        .await?;

    Ok(config.val)
}

async fn get_shipment_service_url<'e, E>(executor: &'e mut E) -> String
where
    &'e mut E: Executor<'e, Database = MySql>,
{
    get_config_by_name(executor, "payment_service_url")
        .await
        .unwrap_or(consts::DEFAULT_SHIPMENT_SERVICE_URL.to_string())
}

async fn api_shipment_status(
    shipment_url: String,
    param: APIShipmentStatusReq,
) -> Result<APIShipmentStatusRes> {
    let res = surf::get(format!("{}/status", shipment_url))
        .body_json(&param)?
        .set_header("User-Agent", consts::USER_AGENT)
        .set_header("Authorization", consts::ISUCARI_API_TOKEN)
        .recv_json()
        .await?;

    Ok(res)
}

pub(crate) async fn get_item(req: Request) -> Result<Body> {
    let param: String = req.param("item_id.json")?;
    let item_id = JSON_PATH_PARAM_RE
        .captures(param.as_str())
        .and_then(|cap| cap.get(1))
        .and_then(|it| it.as_str().parse::<u32>().ok())
        .ok_or_else(|| tide::Error::from_str(StatusCode::BadRequest, "incorrect item id"))?;

    let user = get_user(&req).await?;

    let mut conn = req.state().conn.acquire().await?;
    let item: Item = sqlx::query_as(
        r"
        SELECT
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
            updated_at
        FROM `items`
        WHERE `id` = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut conn)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => tide::Error::new(StatusCode::NotFound, e),
        _ => tide::Error::new(StatusCode::InternalServerError, e),
    })?;

    let category = get_category_by_id(&mut conn, item.category_id).await?;
    let seller = get_user_simple_by_id(&mut conn, item.seller_id).await?;

    let mut item_detail = ItemDetail {
        id: item.id,
        seller_id: item.seller_id,
        seller,
        buyer_id: None,
        buyer: None,
        status: item.status,
        name: item.name,
        price: item.price,
        description: item.description,
        image_url: get_image_url(item.image_name),
        category_id: item.category_id,
        transaction_evidence_id: None,
        transaction_evidence_status: None,
        shipping_status: None,
        category,
        created_at: item.created_at,
    };

    if (user.id == item.seller_id || user.id == item.buyer_id) && item.buyer_id != 0 {
        let buyer = get_user_simple_by_id(&mut conn, item.buyer_id).await?;
        item_detail.buyer_id = Some(item.buyer_id);
        item_detail.buyer = Some(buyer);

        let transaction_evidence: sqlx::Result<TransactionEvidence> = sqlx::query_as(
            r"
            SELECT
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
                updated_at
            FROM `transaction_evidence`
            WHERE `item_id` = ?
            ",
        )
        .bind(item.id)
        .fetch_one(&mut conn)
        .await;

        match transaction_evidence {
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => {
                return Err(tide::Error::new(StatusCode::NotFound, e));
            }
            Ok(t) => {
                let shipping: Shipping = sqlx::query_as(
                    r"
                    SELECT
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
                        updated_at
                    FROM `shippings`
                    WHERE `transaction_evidence_id` = ?
                    ",
                )
                .bind(t.id)
                .fetch_one(&mut conn)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => tide::Error::new(StatusCode::NotFound, e),
                    _ => tide::Error::new(StatusCode::InternalServerError, e),
                })?;
                item_detail.transaction_evidence_id = Some(t.id);
                item_detail.transaction_evidence_status = Some(t.status);
                item_detail.shipping_status = Some(shipping.status);
            }
        }
    }

    Ok(Body::from_json(&item_detail)?)
}

pub(crate) async fn post_item_edit(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_buy(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_sell(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_ship(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_ship_done(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_complete(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn get_qr_code(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_bump(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn get_settings(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_login(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn post_register(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn get_reports(req: Request) -> Result<Body> {
    todo!()
}

pub(crate) async fn get_index(_req: Request) -> Result<&'static str> {
    let html = include_str!("../public/index.html");
    Ok(html)
}

pub(crate) async fn get_assets(req: Request) -> Result<Body> {
    let mut file_path = env::current_dir()?;
    let path: String = req.param("path")?;
    file_path.push("public");
    file_path.push(path);

    let body = Body::from_file(&file_path)
        .await
        .map_err(with_status(StatusCode::NotFound))?;
    Ok(body)
}
