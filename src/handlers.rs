use std::env;
use tide::{Body, Result};

type Request = tide::Request<crate::AppState>;

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
