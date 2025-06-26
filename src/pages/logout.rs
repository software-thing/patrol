use poem::{
    handler,
    web::{cookie::CookieJar, Redirect},
    IntoResponse,
};

use crate::{session, BASE_PATH};

#[handler]
pub async fn get(cookie_jar: &CookieJar) -> anyhow::Result<poem::Response> {
    cookie_jar.remove(session::PATROL_COOKIE);

    Ok(Redirect::see_other(BASE_PATH.to_string() + "/login").into_response())
}
