use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
    pub exposure: Option<String>,
    pub quota_max_size: Option<i64>,
    pub quota_max_objects: Option<i64>,
    pub organization_id: Option<Uuid>,
    pub resource_group_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBucketRequest {
    pub exposure: Option<String>,
    pub quota_max_size: Option<i64>,
    pub quota_max_objects: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BucketResponse {
    pub id: Uuid,
    pub name: String,
    pub global_alias: String,
    pub exposure: String,
    pub quota_max_size: Option<i64>,
    pub quota_max_objects: Option<i64>,
    pub status: String,
    pub organization_id: Option<Uuid>,
    pub resource_group_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
