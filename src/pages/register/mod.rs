use std::borrow::Cow;

use password_hash::{rand_core::OsRng, SaltString};
use poem::{
    self, handler,
    http::StatusCode,
    web::{Data, Form, Html, Redirect},
};
use sea_orm::{sqlx, ActiveModelTrait, DatabaseConnection, DbErr, RuntimeErr, Set};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{
    crypto, internal_server_error,
    is_first_admin_registered::IsFirstAdminRegistered,
    models::{users, users_roles},
    BASE_PATH,
};

pub mod is_available;

#[handler]
pub async fn get(Data((tera, _context)): Data<&(Tera, Context)>) -> poem::Result<Html<String>> {
    let context = Context::new();

    tera.render("register.html.tera", &context)
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        .map(Html)
}

#[derive(Deserialize)]
struct UserRegister {
    #[serde(rename = "first-name")]
    first_name: String,
    #[serde(rename = "last-name")]
    last_name: String,

    username: String,
    password: String,
}

#[handler]
pub async fn post(
    Data(db): Data<&DatabaseConnection>,
    Data(is_first_admin_registered): Data<&IsFirstAdminRegistered>,
    user_register: Form<UserRegister>,
) -> poem::Result<poem::web::Redirect> {
    let salt_string = SaltString::generate(&mut OsRng);
    let salt = salt_string.as_salt();
    let password_hash = crypto::hashing::hash(&salt, user_register.password.as_bytes())?;

    let is_first_admin_registered = &is_first_admin_registered.lock;

    // The new user is an admin only if no admin has been registered before
    let is_admin = !*is_first_admin_registered.read().await;
    println!(
        "DEBUG: is_first_admin_registered state: {}",
        *is_first_admin_registered.read().await
    );
    println!(
        "DEBUG: New user '{}' will be admin: {}",
        user_register.username, is_admin
    );

    let maybe_user: Result<users::Model, DbErr> = users::ActiveModel {
        username: Set(user_register.username.clone()),
        first_name: Set(user_register.first_name.clone()),
        last_name: Set(user_register.last_name.clone()),
        password_hash: Set(password_hash.to_string()),

        ..Default::default()
    }
    .insert(db)
    .await;

    let user = match maybe_user {
        Err(DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(err)))) => {
            log::debug!("{:?}", err);
            if err.code() == Some(Cow::Borrowed("1555")) {
                log::debug!("Username '{}' already exists", user_register.username);
                return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
            } else {
                return Err(poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR));
            }
        }
        Err(err) => return Err(internal_server_error(err)),
        Ok(user) => user,
    };

    if is_admin {
        println!("DEBUG: Assigning admin role to user '{}'", user.username);
        users_roles::ActiveModel {
            user_username: Set(user.username.clone()),
            role_title: Set("admin".to_string()),

            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(internal_server_error)?;

        println!(
            "DEBUG: Successfully assigned admin role to '{}'",
            user.username
        );
        *is_first_admin_registered.write().await = true;
        println!("DEBUG: Updated is_first_admin_registered state to true");
    } else {
        println!(
            "DEBUG: User '{}' will NOT be admin (admin already exists)",
            user.username
        );
    }

    Ok(Redirect::see_other(BASE_PATH.to_string() + "/login"))
}
