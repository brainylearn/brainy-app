use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "file")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub path: String,
    pub is_folder: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Cell,
    Repetition,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Cell => Entity::has_many(super::cell::Entity).into(),
            Self::Repetition => Entity::has_many(super::repetition::Entity).into(),
        }
    }
}

impl Related<super::cell::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cell.def()
    }
}

impl Related<super::repetition::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repetition.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
