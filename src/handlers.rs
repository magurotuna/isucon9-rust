use crate::consts;
use crate::models::{
    Category, Item, ItemSimple, ReqInitialize, ResInitialize, ResNewItems, UserSimple,
};
use crate::AppState;
use anyhow::Result as AnyhowResult;
use serde::Deserialize;
use sqlx::mysql::MySqlQueryAs;
use sqlx::MySqlPool;
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
        item_id: u64,
        created_at: u64,
    }
    let query: Query = req.query().map_err(with_status(StatusCode::BadRequest))?;
    let Query {
        item_id,
        created_at,
    } = query;

    let conn = &req.state().conn;
    let items: Vec<Item> = if item_id > 0 && created_at > 0 {
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
    } else {
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

async fn get_user_simple_by_id(conn: &MySqlPool, seller_id: i64) -> AnyhowResult<UserSimple> {
    todo!()
}

async fn get_category_by_id(conn: &MySqlPool, category_id: i32) -> AnyhowResult<Category> {
    todo!()
}

fn get_image_url(image_name: String) -> String {
    format!("/upload/{}", image_name)
}

pub(crate) async fn get_root_category_id(req: Request) -> TideResult<String> {
    todo!()
}

pub(crate) async fn get_transactions(req: Request) -> TideResult<String> {
    todo!()
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

    // TODO: return 404 if the file does not exist
    let body = Body::from_file(&file_path).await?;
    Ok(body)
}
