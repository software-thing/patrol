use poem::{
    handler,
    web::{cookie::CookieJar, Data, Redirect},
    IntoResponse,
};
use redis::{aio::ConnectionManager, AsyncCommands};
use uuid::Uuid;

use crate::token::{self, Claims};

#[handler]
pub async fn get(
    Data(redis): Data<&ConnectionManager>,
    Data(user): Data<&Claims>,
    cookie_jar: &CookieJar,
) -> anyhow::Result<poem::Response> {
    redis
        .clone()
        .del(format!("token:{}:{}", user.sub, user.jti))
        .await?;

    cookie_jar.remove(token::PATROL_COOKIE);

    Ok(Redirect::see_other("/patrol/login").into_response())
}
