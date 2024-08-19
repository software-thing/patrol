use poem::{
    error::{InternalServerError, Unauthorized},
    handler,
    http::StatusCode,
    web::{
        cookie::{Cookie, CookieJar},
        Data, Form, Html, Redirect,
    },
    IntoResponse,
};
use redis::{aio::ConnectionManager, AsyncCommands};
use sea_orm::{DatabaseConnection, ModelTrait, QuerySelect};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{
    crypto,
    models::{roles, users, users_roles},
    token,
};

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
    Data(redis): Data<&ConnectionManager>,
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

    let roles = user
        .find_related(roles::Entity)
        .select_only()
        .column(users_roles::Column::RoleTitle)
        .into_tuple()
        .all(db)
        .await
        .map_err(anyhow::Error::new)?;

    let claims = token::Claims::new(redis, user, roles).await?;
    let token = token::new(claims).await?;

    let cookie = Cookie::new_with_str(token::PATROL_COOKIE, token);

    cookie_jar.add(cookie);

    return Ok(Redirect::see_other("/patrol/account").into_response());
}
