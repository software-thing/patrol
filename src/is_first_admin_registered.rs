use std::sync::Arc;

use sea_orm::{prelude::*, QuerySelect};
use tokio::sync::RwLock;

use crate::models::{
    users_roles::{self, EmptyUsersRoles},
    UsersRoles,
};

#[derive(Clone)]
pub struct IsFirstAdminRegistered {
    pub lock: Arc<RwLock<bool>>,
}

pub async fn is_first_admin_registered(
    db: &DatabaseConnection,
) -> anyhow::Result<IsFirstAdminRegistered> {
    let is_first_admin_registered = UsersRoles::find()
        .select_only()
        .filter(users_roles::Column::RoleTitle.eq("admin"))
        .into_partial_model::<EmptyUsersRoles>()
        .one(db)
        .await?
        .is_some();

    Ok(IsFirstAdminRegistered {
        lock: Arc::new(RwLock::new(is_first_admin_registered)),
    })
}
