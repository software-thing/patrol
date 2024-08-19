use poem::{handler, web::Json};
use serde_json::Value;

use crate::keys;

#[handler]
pub async fn jwks() -> Json<Option<Value>> {
    Json(keys::jwk().await.ok().cloned())
}
