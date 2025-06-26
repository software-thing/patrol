use poem::{
    handler,
    web::{
        cookie::{Cookie, CookieJar},
        Data, Form, Html, Query, Redirect,
    },
    IntoResponse,
};

use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{crypto, internal_server_error, models::users, session, BASE_PATH};

#[derive(Deserialize)]
struct LoginParams {
    redirect_to: Option<String>,
}

#[handler]
pub async fn get(
    Data((tera, context)): Data<&(Tera, Context)>,
    Query(params): Query<LoginParams>,
) -> poem::Result<Html<String>> {
    let mut context = context.clone();
    context.insert(
        "redirect_to",
        &params
            .redirect_to
            .unwrap_or_else(|| BASE_PATH.to_string() + "/account"),
    );

    tera.render("login.html.tera", &context)
        .map_err(internal_server_error)
        .map(Html)
}

#[derive(Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct UserLogin {
    redirect_to: String,

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
    let user: users::Model = match users::Entity::find_by_username(user_login.username.clone())
        .one(db)
        .await
        .map_err(internal_server_error)?
    {
        Some(user) => user,
        None => {
            let mut ctx = Context::new();
            ctx.extend(context.clone());
            ctx.insert("username", &user_login.username);
            ctx.insert("redirect_to", &user_login.redirect_to);
            ctx.insert("messages", &["Invalid username or password"]);

            return tera
                .render("login.html.tera", &ctx)
                .map_err(internal_server_error)
                .map(|html| Html(html).into_response());
        }
    };

    let password_hash = crypto::hashing::parse_hash(&user.password_hash)?;

    // If the password is incorrect, render the login page again
    if !crypto::hashing::verify(user_login.password.as_bytes(), &password_hash) {
        let mut ctx = Context::new();
        ctx.extend(context.clone());
        ctx.insert("username", &user_login.username);
        ctx.insert("redirect_to", &user_login.redirect_to);
        ctx.insert("messages", &["Invalid username or password"]);

        return tera
            .render("login.html.tera", &ctx)
            .map_err(internal_server_error)
            .map(|html| Html(html).into_response());
    }

    let session_id = session::new(db, user.username).await?;
    let mut cookie = Cookie::new_with_str(session::PATROL_COOKIE, session_id);
    cookie.set_path("/");

    cookie_jar.add(cookie);

    return Ok(Redirect::see_other(user_login.redirect_to.clone()).into_response());
}
