use super::server::Server;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct Channel {
    pub channel_id: Uuid,
    pub name: String,
    pub server_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Channel {
    pub fn new(name: &str, server: Server) -> Self {
        Self {
            channel_id: Uuid::new_v4(),
            name: name.to_string(),
            server_id: server.server_id,
            created_at: Utc::now(),
        }
    }

    pub async fn fetch_all_ids(pool: &PgPool) -> sqlx::Result<Vec<Uuid>> {
        sqlx::query_scalar!("SELECT channel_id FROM channels")
            .fetch_all(pool)
            .await
    }
}
