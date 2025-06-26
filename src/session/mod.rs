use poem::{http::StatusCode, Endpoint, Request};
use sea_orm::{
    prelude::DateTimeUtc, ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, Set,
};
use serde::{Deserialize, Serialize};

use crate::{crypto, models::sessions};

pub mod ep;

pub const PATROL_COOKIE: &'static str = "patrol";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub username: String,

    pub first_name: String,
    pub last_name: String,

    pub profile_picture: bool,

    pub roles: Vec<String>,

    pub created_at: DateTimeUtc,
}

pub async fn session_middleware<E: Endpoint>(next: E, mut req: Request) -> poem::Result<E::Output> {
    // Try to extract the cookie's value
    if let Some(token) = req.header("X-Patrol") {
        let session: Session = serde_json::from_str(token).map_err(|_| {
            poem::Error::from_string(
                "Failed to parse the X-Patrol header with user information",
                StatusCode::UNPROCESSABLE_ENTITY,
            )
        })?;

        req.extensions_mut().insert(session);

        return next.call(req).await;
    }

    Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
}

pub async fn new(db: &DatabaseConnection, username: String) -> anyhow::Result<String> {
    let session_id = crypto::id();

    sessions::ActiveModel {
        id: Set(session_id.clone()),
        user_username: Set(username),

        ..sessions::ActiveModel::new()
    }
    .insert(db)
    .await?;

    Ok(session_id)
}
