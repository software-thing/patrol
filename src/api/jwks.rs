use poem_openapi::{payload::Json, OpenApi};
use serde_json::Value;

use crate::keys;

#[derive(Debug, Clone)]
pub struct Api;

#[OpenApi(prefix_path = "/jwks")]
impl Api {
    #[oai(path = "/", method = "get")]
    async fn jwks(&self) -> Json<Option<Value>> {
        Json(keys::jwk().await.ok().cloned())
    }
}
