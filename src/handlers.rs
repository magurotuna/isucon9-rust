use crate::consts;
use crate::models::{
    Category, Item, ItemSimple, ReqInitialize, ResInitialize, ResNewItems, User, UserSimple,
};
use crate::AppState;
use anyhow::{Context, Result as AnyhowResult};
use async_recursion::async_recursion;
use regex::Regex;
use serde::Deserialize;
use sqlx::mysql::{MySqlQueryAs, MySqlRow};
use sqlx::{MySqlPool, Row as _};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use tide::{Body, Result as TideResult, StatusCode};

type Request = tide::Request<AppState>;

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

pub(crate) async fn post_initialize(mut req: Request) -> TideResult<Body> {
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

pub(crate) async fn get_new_items(req: Request) -> TideResult<Body> {
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

    let conn = &req.state().conn;
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
            .fetch_all(conn)
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
            .fetch_all(conn)
            .await?
        }
    };

    let mut item_simples = Vec::new();
    for item in items {
        let seller = get_user_simple_by_id(conn, item.seller_id).await?;
        let category = get_category_by_id(conn, item.category_id).await?;
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

async fn get_user_simple_by_id(conn: &MySqlPool, user_id: u64) -> AnyhowResult<UserSimple> {
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

    Ok(user.into())
}

#[async_recursion]
async fn get_category_by_id(conn: &MySqlPool, category_id: u32) -> AnyhowResult<Category> {
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
    .fetch_one(conn)
    .await?;
    if category.parent_id != 0 {
        category.parent_category_name = get_category_by_id(conn, category.parent_id)
            .await
            .map(|parent_category| Some(parent_category.category_name))
            .unwrap_or(None);
    }
    Ok(category)
}

fn get_image_url(image_name: String) -> String {
    format!("/upload/{}", image_name)
}

pub(crate) async fn get_new_category_items(req: Request) -> TideResult<Body> {
    let re = Regex::new(r"^(.+)\.json$").unwrap();
    let param: String = req.param("root_category_id.json")?;
    let root_category_id = re
        .captures(param.as_str())
        .and_then(|cap| cap.get(1))
        .and_then(|it| it.as_str().parse::<u32>().ok())
        .context("incorrect category id")
        .map_err(with_status(StatusCode::BadRequest))?;

    let conn = &req.state().conn;

    let root_category = get_category_by_id(conn, root_category_id).await?;
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
        .fetch_all(conn)
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
                .fetch_all(conn)
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
                .fetch_all(conn)
                .await?
        }
    };

    let mut item_simples = Vec::new();
    for item in items {
        let seller = get_user_simple_by_id(conn, item.seller_id)
            .await
            .map_err(with_status(StatusCode::NotFound))?;
        let category = get_category_by_id(conn, item.category_id)
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

pub(crate) async fn get_transactions(req: Request) -> TideResult<String> {
    let user = get_user(&req).await?;
    // TODO WIP
    Ok("a".to_string())
}

async fn get_user(req: &Request) -> TideResult<User> {
    let session = req.session();
    let user_id: String = session
        .get("user_id")
        .context("no session")
        .map_err(with_status(StatusCode::NotFound))?;
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

pub(crate) async fn get_item_id(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_item_edit(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_buy(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_sell(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_ship(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_ship_done(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_complete(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn get_qr_code(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_bump(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn get_settings(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_login(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn post_register(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn get_reports(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn get_index(_req: Request) -> TideResult<&'static str> {
    let html = include_str!("../public/index.html");
    Ok(html)
}

pub(crate) async fn get_assets(req: Request) -> TideResult<Body> {
    let mut file_path = env::current_dir()?;
    let path: String = req.param("path")?;
    file_path.push("public");
    file_path.push(path);

    let body = Body::from_file(&file_path)
        .await
        .map_err(with_status(StatusCode::NotFound))?;
    Ok(body)
}
