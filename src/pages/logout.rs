use poem::{
    handler,
    web::{cookie::CookieJar, Data, Redirect},
    IntoResponse,
};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{
    internal_server_error,
    models::Sessions,
    session::{self},
    BASE_PATH,
};

#[handler]
pub async fn get(
    Data(db): Data<&DatabaseConnection>,
    cookie_jar: &CookieJar,
) -> poem::Result<poem::Response> {
    if let Some(cookie) = cookie_jar.get(session::PATROL_COOKIE) {
        Sessions::delete_by_id(cookie.value_str())
            .exec(db)
            .await
            .map_err(internal_server_error)?;

        cookie_jar.remove(session::PATROL_COOKIE);
    }

    Ok(Redirect::see_other(BASE_PATH.to_string() + "/login").into_response())
}
