use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    title: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users::Entity")]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::users_roles::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::users_roles::Relation::Role.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
