use crate::{guards::LoginGuard, quick_response, MyState};
use rocket::{
    http::Status,
    serde::{json::Json, uuid::Uuid, Deserialize},
    State,
};
use spook_chat_db::models::{Invite, Permissions, Server, User};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct NewInviteConfig {
    server_id: Uuid,
    expires: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct BanUserData {
    server_id: Uuid,
    user_id: Uuid,
}

// keeping copy due to planned changes to BanUserData
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct KickUserData {
    server_id: Uuid,
    user_id: Uuid,
}

enum PermissionError {
    SqlxError(sqlx::Error),
    MissingPermissions,
    NoEntry,
    UserNoExist(Uuid),
}

impl From<sqlx::Error> for PermissionError {
    fn from(e: sqlx::Error) -> Self {
        Self::SqlxError(e)
    }
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for PermissionError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            PermissionError::SqlxError(e) => Ok(quick_response(
                Status::InternalServerError,
                e.to_string().as_str(),
            )),
            PermissionError::MissingPermissions => Ok(quick_response(
                Status::Forbidden,
                "You are missing permissions required to perform this action",
            )),
            PermissionError::NoEntry => Ok(quick_response(
                Status::Forbidden,
                "You do not appear to be part of this server",
            )),
            PermissionError::UserNoExist(id) => Ok(quick_response(
                Status::Forbidden,
                format!("User with the {id} does not exist"),
            )),
        }
    }
}

enum JoinError {
    SqlxError(sqlx::Error),
    InviteNotExist,
    InviteExpired,
}

impl From<sqlx::Error> for JoinError {
    fn from(e: sqlx::Error) -> Self {
        Self::SqlxError(e)
    }
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for JoinError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            JoinError::SqlxError(e) => Ok(quick_response(
                Status::InternalServerError,
                e.to_string().as_str(),
            )),
            JoinError::InviteNotExist => {
                Ok(quick_response(Status::BadRequest, "This invite is invalid"))
            }
            JoinError::InviteExpired => Ok(quick_response(
                Status::BadRequest,
                "This Invite has expired",
            )),
        }
    }
}

#[get("/invite/<id>")]
async fn join_server(
    state: &State<MyState>,
    login: LoginGuard,
    id: Uuid,
) -> Result<Status, JoinError> {
    let invite = Invite::filter_by_id(&state.conn, id)
        .await?
        .ok_or(JoinError::InviteNotExist)?;
    if invite.is_valid() {
        login
            .user
            .add_to_server(&state.conn, invite.server_id)
            .await?;
        Ok(Status::Ok)
    } else {
        Err(JoinError::InviteExpired)
    }
}

#[post("/invite", data = "<id>")]
async fn join_server_post(
    state: &State<MyState>,
    login: LoginGuard,
    id: Json<Uuid>,
) -> Result<Status, JoinError> {
    let invite = Invite::filter_by_id(&state.conn, id.0)
        .await?
        .ok_or(JoinError::InviteNotExist)?;
    if invite.is_valid() {
        login
            .user
            .add_to_server(&state.conn, invite.server_id)
            .await?;
        Ok(Status::Ok)
    } else {
        Err(JoinError::InviteExpired)
    }
}

#[post("/new/invite", data = "<config>")]
async fn create_invite(
    state: &State<MyState>,
    login: LoginGuard,
    config: Json<NewInviteConfig>,
) -> Result<(Status, String), PermissionError> {
    let server = Server::filter_by_id(&state.conn, config.server_id).await?;

    let permissions: Permissions = server
        .get_permissions(&state.conn, &login.user)
        .await?
        .ok_or(PermissionError::NoEntry)?;

    if permissions.manage_invites {
        let invite = server.create_invite(&state.conn, config.expires).await?;

        Ok((Status::Ok, invite.invite_id.to_string()))
    } else {
        Err(PermissionError::MissingPermissions)
    }
}

#[post("/user/ban", data = "<data>")]
async fn ban_user(
    state: &State<MyState>,
    login: LoginGuard,
    data: Json<BanUserData>,
) -> Result<String, PermissionError> {
    let server = Server::filter_by_id(&state.conn, data.server_id).await?;

    let permissions = server
        .get_permissions(&state.conn, &login.user)
        .await?
        .ok_or(PermissionError::NoEntry)?;

    if permissions.manage_users {
        let user_to_ban = User::filter_by_id(&state.conn, data.user_id)
            .await?
            .ok_or(PermissionError::UserNoExist(data.user_id))?;
        server.ban_user(&state.conn, &user_to_ban).await?;

        Ok(format!("User {} banned", data.user_id))
    } else {
        Err(PermissionError::MissingPermissions)
    }
}

#[post("/user/unban", data = "<data>")]
async fn unban_user(
    state: &State<MyState>,
    login: LoginGuard,
    data: Json<BanUserData>,
) -> Result<String, PermissionError> {
    let server = Server::filter_by_id(&state.conn, data.server_id).await?;

    let permissions = server
        .get_permissions(&state.conn, &login.user)
        .await?
        .ok_or(PermissionError::NoEntry)?;

    if permissions.manage_users {
        let user_to_unban = User::filter_by_id(&state.conn, data.user_id)
            .await?
            .ok_or(PermissionError::UserNoExist(data.user_id))?;
        server.unban_user(&state.conn, &user_to_unban).await?;

        Ok(format!("User {} unbanned", data.user_id))
    } else {
        Err(PermissionError::MissingPermissions)
    }
}

#[post("/user/kick", data = "<data>")]
async fn kick_user(
    state: &State<MyState>,
    login: LoginGuard,
    data: Json<KickUserData>,
) -> Result<String, PermissionError> {
    let server = Server::filter_by_id(&state.conn, data.server_id).await?;

    let permissions = server
        .get_permissions(&state.conn, &login.user)
        .await?
        .ok_or(PermissionError::NoEntry)?;

    if permissions.manage_users {
        let user_to_kick = User::filter_by_id(&state.conn, data.user_id)
            .await?
            .ok_or(PermissionError::UserNoExist(data.user_id))?;
        server.kick_user(&state.conn, &user_to_kick).await?;

        Ok(format!("User {} kicked", data.user_id))
    } else {
        Err(PermissionError::MissingPermissions)
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        join_server,
        join_server_post,
        create_invite,
        kick_user,
        ban_user,
        unban_user
    ]
}
