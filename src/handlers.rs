use crate::models::{ReqInitialize, ResInitialize};
use crate::AppState;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use tide::{Body, Result, StatusCode};

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

pub(crate) async fn get_new_items(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_root_category_id(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_transactions(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_item_id(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_item_edit(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_buy(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_sell(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_ship(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_ship_done(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_complete(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_qr_code(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_bump(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_settings(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_login(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn post_register(req: Request) -> Result<String> {
    todo!()
}

pub(crate) async fn get_reports(req: Request) -> Result<String> {
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

    // TODO: return 404 if the file does not exist
    let body = Body::from_file(&file_path).await?;
    Ok(body)
}
