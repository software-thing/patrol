use anyhow;
use password_hash::{rand_core::OsRng, SaltString};
use poem::{
    self, handler,
    web::{Data, Form, Html},
};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, Set, TransactionTrait};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::{
    crypto,
    is_first_admin_registered::IsFirstAdminRegistered,
    models::{users, users_roles},
};

#[handler]
pub async fn get(tera: Data<&Tera>) -> anyhow::Result<Html<String>> {
    let context = Context::new();

    tera.render("register.html.tera", &context)
        .map_err(anyhow::Error::new)
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
    db: Data<&DatabaseConnection>,
    is_first_admin_registered: Data<&IsFirstAdminRegistered>,
    user_register: Form<UserRegister>,
) -> anyhow::Result<()> {
    let salt_string = SaltString::generate(&mut OsRng);
    let salt = salt_string.as_salt();
    let password_hash = crypto::hashing::hash(&salt, user_register.password.as_bytes())?;

    let is_first_admin_registered = &is_first_admin_registered.0.lock;

    // The new user is an admin only if no admin has been registered before
    let is_admin = !*is_first_admin_registered.read().await;

    let txn = db.0.begin().await?;

    let user: users::Model = users::ActiveModel {
        username: Set(user_register.username.clone()),
        first_name: Set(user_register.first_name.clone()),
        last_name: Set(user_register.last_name.clone()),
        password_hash: Set(password_hash.to_string()),

        ..users::ActiveModel::new()
    }
    .insert(&txn)
    .await
    .map_err(anyhow::Error::new)?;

    if is_admin {
        users_roles::ActiveModel {
            user_id: Set(user.id),
            role_title: Set("admin".to_string()),
        }
        .insert(&txn)
        .await
        .map_err(anyhow::Error::new)?;

        *is_first_admin_registered.write().await = true;
    }

    txn.commit().await.map_err(anyhow::Error::new)?;

    Ok(())
}
