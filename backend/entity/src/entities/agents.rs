use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub hostname: String,
    pub ip_address: Option<String>,
    pub agent_version: String,
    pub os_type: String,
    pub os_version: String,
    pub architecture: String,
    pub status: String,
    pub last_heartbeat: Option<DateTime>,
    pub registered_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub organization_id: Option<Uuid>,
    pub tags: Option<serde_json::Value>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agent_metrics::Entity")]
    AgentMetrics,
}

impl Related<super::agent_metrics::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AgentMetrics.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
