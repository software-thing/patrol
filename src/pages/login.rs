use poem::{
    error::InternalServerError,
    handler,
    http::StatusCode,
    web::{
        cookie::{Cookie, CookieJar},
        Data, Form, Html, Redirect,
    },
    IntoResponse,
};

use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{crypto, models::users, session, BASE_PATH};

#[handler]
pub async fn get(Data((tera, context)): Data<&(Tera, Context)>) -> anyhow::Result<Html<String>> {
    tera.render("login.html.tera", &context)
        .map(Html)
        .map_err(anyhow::Error::new)
}

#[derive(Clone, Deserialize, Default)]
struct UserLogin {
    username: String,
    password: String,
}

#[handler]
pub async fn post(
    Data((tera, context)): Data<&(Tera, Context)>,
    Data(db): Data<&DatabaseConnection>,
    cookie_jar: &CookieJar,
    user_login: Form<UserLogin>,
) -> poem::Result<poem::Response> {
    let user: users::Model = users::Entity::find_by_username(user_login.username.clone())
        .one(db)
        .await
        .map_err(InternalServerError)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let password_hash = crypto::hashing::parse_hash(&user.password_hash)?;

    // If the password is incorrect, render the login page again
    if !crypto::hashing::verify(user_login.password.as_bytes(), &password_hash) {
        let mut ctx = Context::new();
        ctx.extend(context.clone());
        ctx.insert("username", &user_login.username);

        return tera
            .render("login.html.tera", &ctx)
            .map_err(InternalServerError)
            .map(|html| Html(html).into_response());
    }

    let session_id = session::new(db, user.username)
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    let cookie = Cookie::new_with_str(session::PATROL_COOKIE, session_id);

    cookie_jar.add(cookie);

    return Ok(Redirect::see_other(BASE_PATH.to_string() + "/account").into_response());
}
