use poem::{
    handler,
    web::{cookie::CookieJar, Redirect},
    IntoResponse,
};

use crate::session;

#[handler]
pub async fn get(cookie_jar: &CookieJar) -> anyhow::Result<poem::Response> {
    cookie_jar.remove(session::PATROL_COOKIE);

    Ok(Redirect::see_other("/patrol/login").into_response())
}
