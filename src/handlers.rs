use tide::Result;

type Request = tide::Request<crate::AppState>;

pub(crate) async fn index(_req: Request) -> Result<&'static str> {
    let html = include_str!("../public/index.html");
    Ok(html)
}
