use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Query},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait, SelectColumns,
};
use serde::Deserialize;

use crate::{
    internal_server_error,
    models::{sessions, users, users_roles, Roles, Sessions, UsersRoles},
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
        .map_err(internal_server_error)?
        .and_then(|(_session, user)| user)
        .ok_or(poem::Error::from_status(StatusCode::UNAUTHORIZED))?;

    let roles = UsersRoles::find()
        .filter(users_roles::Column::UserUsername.eq(user.username.clone()))
        .all(db)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(Session {
        username: user.username,
        first_name: user.first_name,
        last_name: user.last_name,
        profile_picture: user.profile_picture,
        created_at: user.created_at,
        roles: roles.into_iter().map(|role| role.role_title).collect(),
    }))
}
