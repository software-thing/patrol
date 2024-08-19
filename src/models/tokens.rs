use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub jti: Uuid,

    pub sub: Uuid,

    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Users",
        from = "Column::Sub",
        to = "super::users::Column::Username"
    )]
    User,
}

impl Related<super::Users> for Entity {
    fn to() -> RelationDef {
        super::users::Relation::Tokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
