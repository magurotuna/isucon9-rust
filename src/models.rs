use serde::{Deserialize, Serialize};

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
