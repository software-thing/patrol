use chrono::Utc;
use password_hash::{rand_core::OsRng, SaltString};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Form, Html, Redirect},
    IntoResponse,
};
use sea_orm::{prelude::DateTimeUtc, ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{crypto, internal_server_error, models::users, session::Session, BASE_PATH};

#[derive(Deserialize)]
struct PasswordChangeForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

#[handler]
pub async fn get(
    Data(session): Data<&Session>,
    Data((tera, context)): Data<&(Tera, Context)>,
) -> poem::Result<Html<String>> {
    let mut ctx = Context::new();

    ctx.extend(context.clone());
    ctx.insert("user", session);

    tera.render("account.html.tera", &ctx)
        .map_err(internal_server_error)
        .map(Html)
}

#[handler]
pub async fn change_password(
    Data(session): Data<&Session>,
    Data(db): Data<&DatabaseConnection>,
    Data((tera, context)): Data<&(Tera, Context)>,
    Form(form): Form<PasswordChangeForm>,
) -> poem::Result<poem::Response> {
    let mut ctx = Context::new();
    ctx.extend(context.clone());
    ctx.insert("user", session);

    // Validate that new password and confirm password match
    if form.new_password != form.confirm_password {
        ctx.insert("messages", &["New passwords do not match"]);
        return tera
            .render("account.html.tera", &ctx)
            .map_err(internal_server_error)
            .map(|html| Html(html).into_response());
    }

    // Get the current user from database
    let user = users::Entity::find_by_id(&session.username)
        .one(db)
        .await
        .map_err(internal_server_error)?
        .ok_or(poem::Error::from_status(StatusCode::UNAUTHORIZED))?;

    // Verify current password
    let current_hash = crypto::hashing::parse_hash(&user.password_hash).map_err(|e| {
        log::error!("Hash parsing error: {:?}", e);
        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    if !crypto::hashing::verify(form.current_password.as_bytes(), &current_hash) {
        ctx.insert("messages", &["Current password is incorrect"]);
        return tera
            .render("account.html.tera", &ctx)
            .map_err(internal_server_error)
            .map(|html| Html(html).into_response());
    }

    // Generate new password hash
    let salt_string = SaltString::generate(&mut OsRng);
    let salt = salt_string.as_salt();
    let new_password_hash =
        crypto::hashing::hash(&salt, form.new_password.as_bytes()).map_err(|e| {
            log::error!("Password hashing error: {:?}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // Update password in database
    let old_password_hash = user.password_hash.clone();
    let mut user_model: users::ActiveModel = user.into();
    user_model.password_hash = Set(new_password_hash.to_string());
    user_model.password_hash_previous = Set(Some(old_password_hash));
    user_model.password_changed_at = Set(Some(DateTimeUtc::from(Utc::now())));

    user_model.save(db).await.map_err(internal_server_error)?;

    Ok(Redirect::see_other(BASE_PATH.to_string() + "/account").into_response())
}
