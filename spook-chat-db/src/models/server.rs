use super::channel::Channel;
use super::{Invite, User};
use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgQueryResult};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct Server {
    pub server_id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct Permissions {
    pub manage_channels: bool,
    pub manage_users: bool,
    pub manage_invites: bool,
    pub banned: bool,
}

pub struct ChangePermissions {
    pub manage_channels: Option<bool>,
    pub manage_users: Option<bool>,
    pub manage_invites: Option<bool>,
}

impl Server {
    pub fn new(name: &str, user: User) -> Self {
        Self {
            server_id: Uuid::new_v4(),
            name: name.to_string(),
            owner_id: user.user_id,
            created_at: Utc::now(),
        }
    }

    pub async fn save(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO servers (server_id, name, created_at)
            VALUES ($1, $2, $3)",
            self.server_id,
            self.name,
            self.created_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_in_database(&self, pool: &PgPool) -> bool {
        let server: sqlx::Result<Uuid> = sqlx::query_scalar!(
            "SELECT server_id FROM servers WHERE server_id = $1",
            self.server_id
        )
        .fetch_one(pool)
        .await;

        server.is_ok()
    }

    pub async fn add_channel(&self, pool: &PgPool, channel: &Channel) -> sqlx::Result<()> {
        if !self.is_in_database(pool).await {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query!(
            "INSERT INTO channels (server_id, channel_id, name, created_at)
            VALUES ($1, $2, $3, $4)",
            self.server_id,
            channel.channel_id,
            channel.name,
            channel.created_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn create_invite(
        &self,
        pool: &PgPool,
        expires: Option<DateTime<Utc>>,
    ) -> sqlx::Result<Invite> {
        let invite = Invite::new(self, expires);
        invite.save(pool).await?;

        Ok(invite)
    }

    pub async fn get_permissions(
        &self,
        pool: &PgPool,
        user: &User,
    ) -> sqlx::Result<Option<Permissions>> {
        sqlx::query_as!(
            Permissions, 
            "SELECT manage_channels, manage_users, manage_invites, banned 
            FROM users_servers WHERE server_id = $1 AND user_id = $2", 
            self.server_id, 
            user.user_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn change_permissions(
        &self,
        pool: &PgPool,
        user: &User,
        changes: ChangePermissions,
    ) -> sqlx::Result<PgQueryResult> {
        let current_permissions = self
            .get_permissions(pool, user)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        sqlx::query!(
            "UPDATE users_servers SET (manage_channels, manage_users, manage_invites) = ($1, $2, $3)", 
            changes.manage_channels.unwrap_or(current_permissions.manage_channels), 
            changes.manage_users.unwrap_or(current_permissions.manage_users), 
            changes.manage_invites.unwrap_or(current_permissions.manage_invites)
        )
        .execute(pool)
        .await
    }

    pub async fn kick_user(&self, pool: &PgPool, user: &User) -> sqlx::Result<()> {
        sqlx::query!(
            "DELETE FROM users_servers WHERE server_id = $1 AND user_id = $2",
            self.server_id,
            user.user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn ban_user(&self, pool: &PgPool, user: &User) -> sqlx::Result<()> {
        if !self.is_in_database(pool).await {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query!(
            "UPDATE users_servers SET banned = true WHERE server_id = $1 AND user_id = $2",
            self.server_id,
            user.user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn unban_user(&self, pool: &PgPool, user: &User) -> sqlx::Result<()> {
        if !self.is_in_database(pool).await {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query!(
            "UPDATE users_servers SET banned = false WHERE server_id = $1 AND user_id = $2",
            self.server_id,
            user.user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn filter_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Server> {
        let server = sqlx::query_as!(Server, "SELECT * FROM servers WHERE server_id = $1", id)
            .fetch_one(pool)
            .await;

        server
    }

    pub async fn filter_by_name(pool: &PgPool, name: &str) -> sqlx::Result<Vec<Self>> {
        let server = sqlx::query_as!(Server, "SELECT * FROM servers WHERE name = $1", name)
            .fetch_all(pool)
            .await;

        server
    }
}
