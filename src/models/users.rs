use sea_orm::{entity::prelude::*, Set};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique)]
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
    #[sea_orm(has_many = "super::roles::Entity")]
    Roles,
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        super::users_roles::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::users_roles::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            password_hash_previous: Set(None),

            ..ActiveModelTrait::default()
        }
    }
}

impl Entity {
    pub fn find_by_username(username: String) -> Select<Entity> {
        Self::find().filter(Column::Username.eq(username))
    }
}
