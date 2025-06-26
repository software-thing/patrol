use std::sync::Arc;

use sea_orm::prelude::*;
use tokio::sync::RwLock;

use crate::models::{
    users_roles::{self},
    UsersRoles,
};

#[derive(Clone)]
pub struct IsFirstAdminRegistered {
    pub lock: Arc<RwLock<bool>>,
}

pub async fn is_first_admin_registered(
    db: &DatabaseConnection,
) -> anyhow::Result<IsFirstAdminRegistered> {
    let admin_users = UsersRoles::find()
        .filter(users_roles::Column::RoleTitle.eq("admin"))
        .all(db)
        .await?;

    let is_first_admin_registered = !admin_users.is_empty();

    println!("DEBUG: Found {} admin users", admin_users.len());
    for admin in &admin_users {
        println!("DEBUG: Admin user: {}", admin.user_username);
    }

    if is_first_admin_registered {
        log::info!("Admin(s) already exist - first admin has been registered");
        println!("Admin(s) already exist - first admin has been registered");
    } else {
        log::info!("No admin registered yet - next user will become admin");
        println!("No admin registered yet - next user will become admin");
    }

    Ok(IsFirstAdminRegistered {
        lock: Arc::new(RwLock::new(is_first_admin_registered)),
    })
}
