use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub username: String,

    pub first_name: String,
    pub last_name: String,

    pub password_hash: String,
    pub password_hash_previous: Option<String>,
    pub password_changed_at: Option<DateTimeUtc>,

    pub profile_picture: bool,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::Roles")]
    Roles,
    #[sea_orm(has_many = "super::Tokens")]
    Tokens,
}

impl Related<super::Roles> for Entity {
    fn to() -> RelationDef {
        super::users_roles::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::users_roles::Relation::User.def().rev())
    }
}

impl Related<super::Tokens> for Entity {
    fn to() -> RelationDef {
        Relation::Tokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_username(username: String) -> Select<Entity> {
        Self::find().filter(Column::Username.eq(username))
    }
}
