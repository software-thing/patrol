use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Query},
};
use sea_orm::{DatabaseConnection, EntityTrait, JoinType, QuerySelect, RelationTrait};
use serde::Deserialize;

use crate::{
    models::{sessions, users, Sessions},
    session::Session,
};

#[derive(Deserialize)]
struct SessionParams {
    id: String,
}

#[handler]
pub async fn get(
    Data(db): Data<&DatabaseConnection>,
    Query(session_params): Query<SessionParams>,
) -> Result<Json<Session>, poem::Error> {
    let user = Sessions::find_by_id(session_params.id)
        .join(JoinType::InnerJoin, sessions::Relation::User.def())
        .select_also(users::Entity)
        .one(db)
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .and_then(|(_session, user)| user)
        .ok_or(poem::Error::from_status(StatusCode::UNAUTHORIZED))?;

    Ok(Json(Session {
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        profile_picture: user.profile_picture,
        created_at: user.created_at,
    }))
}
