use super::{channel::Channel, user::User};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct Message {
    pub message_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(content: &str) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            content: content.to_string(),
            created_at: Utc::now(),
        }
    }

    pub async fn save(&self, pool: &PgPool, user: User, channel: Channel) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO messages (message_id, content, created_at, channel_id, user_id)
            VALUES ($1, $2, $3, $4, $5)",
            self.message_id,
            self.content,
            self.created_at,
            channel.channel_id,
            user.user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
