use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    user_id: Uuid,

    #[sea_orm(primary_key, auto_increment = false)]
    role_title: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Users",
        from = "Column::UserId",
        to = "super::users::Column::Id"
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
    pub fn find_by_user_and_role(user_id: Uuid, role_title: impl Into<String>) -> Select<Entity> {
        Self::find_by_id((user_id, role_title.into()))
    }

    pub fn find_by_user(user_id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserId.eq(user_id))
    }
}
