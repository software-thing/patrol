use poem::{
    handler,
    web::{Data, Html, Query},
};
use sea_orm::DatabaseConnection;

use crate::models::users;

#[handler]
pub async fn get(
    db: Data<&DatabaseConnection>,
    username: Query<String>,
) -> anyhow::Result<Html<String>> {
    let is_available = users::Entity::find_by_username(username.0)
        .one(db.0)
        .await?
        .is_none();

    Ok(Html(
        if is_available {
            ""
        } else {
            "Username is taken"
        }
        .to_string(),
    ))
}
