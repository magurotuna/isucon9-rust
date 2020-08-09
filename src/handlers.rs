use std::env;
use tide::{Body, Result};

type Request = tide::Request<crate::AppState>;

pub(crate) async fn post_initialize(req: Request) -> Result<String> {
    todo!()
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
