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
    context.insert("redirect_to", &params.redirect_to);

    tera.render("login.html.tera", &context)
        .map_err(internal_server_error)
        .map(Html)
}

#[derive(Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct UserLogin {
    redirect_to: Option<String>,

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
    let redirect_to = match user_login.redirect_to.as_ref().map(|url| url.trim()) {
        Some("") | None => BASE_PATH.to_string() + "/account",
        Some(redirect_to) => redirect_to.to_string(),
    };

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
            ctx.insert("messages", &["Invalid username or password"]);
            ctx.insert("redirect_to", &redirect_to);

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
        ctx.insert("messages", &["Invalid username or password"]);
        ctx.insert("redirect_to", &redirect_to);

        return tera
            .render("login.html.tera", &ctx)
            .map_err(internal_server_error)
            .map(|html| Html(html).into_response());
    }

    let session_id = session::new(db, user.username).await?;
    let mut cookie = Cookie::new_with_str(session::PATROL_COOKIE, session_id);
    cookie.set_path("/");

    cookie_jar.add(cookie);

    return Ok(Redirect::see_other(redirect_to).into_response());
}
