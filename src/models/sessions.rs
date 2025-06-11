use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    pub user_username: String,

    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Users",
        from = "Column::UserUsername",
        to = "super::users::Column::Username"
    )]
    User,
}

impl Related<super::Users> for Entity {
    fn to() -> RelationDef {
        super::users::Relation::Sessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
