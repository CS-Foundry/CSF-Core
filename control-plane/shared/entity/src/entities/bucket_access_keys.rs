use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bucket_access_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub bucket_id: Uuid,
    pub name: String,
    pub garage_key_id: String,
    pub permissions: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub last_rotated_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::buckets::Entity",
        from = "Column::BucketId",
        to = "super::buckets::Column::Id",
        on_delete = "Cascade"
    )]
    Bucket,
}

impl Related<super::buckets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bucket.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
