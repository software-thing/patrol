use sea_orm::{entity::prelude::*, FromQueryResult};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    user_username: String,

    #[sea_orm(primary_key, auto_increment = false)]
    role_title: String,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "Model")]
pub struct EmptyUsersRoles {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Users",
        from = "Column::UserUsername",
        to = "super::users::Column::Username"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::Roles",
        from = "Column::RoleTitle",
        to = "super::roles::Column::Title"
    )]
    Role,
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_user_and_role(
        user_username: String,
        role_title: impl Into<String>,
    ) -> Select<Entity> {
        Self::find_by_id((user_username, role_title.into()))
    }

    pub fn find_by_user(user_username: String) -> Select<Entity> {
        Self::find().filter(Column::UserUsername.eq(user_username))
    }
}
