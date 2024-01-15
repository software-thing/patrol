use poem::{
    handler,
    web::{
        cookie::{Cookie, CookieJar},
        Data, Form, Html, Redirect,
    },
    IntoResponse,
};
use sea_orm::{DatabaseConnection, QuerySelect};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{
    crypto,
    models::{users, users_roles},
    token,
};

#[handler]
pub async fn get(tera: Data<&Tera>) -> anyhow::Result<Html<String>> {
    let context = Context::new();

    tera.render("login.html.tera", &context)
        .map_err(anyhow::Error::new)
        .map(Html)
}

#[derive(Deserialize)]
struct UserLogin {
    username: String,
    password: String,
}

#[handler]
pub async fn post(
    tera: Data<&Tera>,
    db: Data<&DatabaseConnection>,
    cookie_jar: &CookieJar,
    user_login: Form<UserLogin>,
) -> anyhow::Result<poem::Response> {
    let user: users::Model = users::Entity::find_by_username(user_login.username.clone())
        .one(db.0)
        .await
        .map_err(anyhow::Error::new)?
        .unwrap();

    let password_hash = crypto::hashing::parse_hash(&user.password_hash)?;

    if crypto::hashing::verify(user_login.password.as_bytes(), &password_hash) {
        let roles = users_roles::Entity::find_by_user(user.id)
            .select_only()
            .column(users_roles::Column::RoleTitle)
            .into_tuple()
            .all(db.0)
            .await
            .map_err(anyhow::Error::new)?;

        let claims = token::Claims::new(user, roles);
        let token = token::new(claims).await?;
        let cookie = Cookie::new_with_str(token::PATROL_COOKIE, token);

        cookie_jar.add(cookie);

        return Ok(Redirect::see_other("/account").into_response());
    } else {
        let context = Context::new();

        return tera
            .render("login.html.tera", &context)
            .map_err(anyhow::Error::new)
            .map(|html| Html(html).into_response());
    }
}
