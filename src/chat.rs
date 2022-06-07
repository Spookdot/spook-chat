use crate::{guards::LoginGuard, quick_response, MyState};
use rocket::{
    http::Status,
    response::stream::{Event, EventStream},
    serde::{json::Json, uuid::Uuid, Deserialize},
    tokio::sync::broadcast::error,
    State,
};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct MessageData<'a> {
    channel: Uuid,
    message: &'a str,
}

enum ChatError {
    SqlxError(sqlx::Error),
    SendError(error::SendError<String>),
    MissingPermission,
    NoChannelFound,
}

impl From<sqlx::Error> for ChatError {
    fn from(e: sqlx::Error) -> Self {
        Self::SqlxError(e)
    }
}

impl From<error::SendError<String>> for ChatError {
    fn from(e: error::SendError<String>) -> Self {
        Self::SendError(e)
    }
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for ChatError {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            Self::SendError(e) => Ok(quick_response(Status::InternalServerError, e.to_string())),
            Self::SqlxError(e) => Ok(quick_response(Status::InternalServerError, e.to_string())),
            Self::MissingPermission => Ok(quick_response(
                Status::BadRequest,
                "You do not have permission or this channel doesn't exist",
            )),
            Self::NoChannelFound => Ok(quick_response(
                Status::BadRequest,
                "This channel does not exist",
            )),
        }
    }
}

#[get("/subscribe?<channel>")]
async fn subscribe(
    state: &State<MyState>,
    login: LoginGuard,
    channel: Uuid,
) -> Result<EventStream![], ChatError> {
    if !login
        .user
        .has_access_to_channel(&state.conn, channel)
        .await?
    {
        return Err(ChatError::MissingPermission);
    };

    let tx = state
        .channels
        .get(&channel)
        .ok_or(ChatError::NoChannelFound)?;

    let mut rx = tx.subscribe();
    let event_stream = EventStream! { loop {
        if let Ok(msg) = rx.recv().await {
            yield Event::data(msg).event("message");
        }
    }};

    return Ok(event_stream);
}

#[post("/send", data = "<message>")]
async fn send(
    state: &State<MyState>,
    login: LoginGuard,
    message: Json<MessageData<'_>>,
) -> Result<String, ChatError> {
    if !login
        .user
        .has_access_to_channel(&state.conn, message.channel)
        .await?
    {
        return Err(ChatError::MissingPermission);
    };

    let tx = state
        .channels
        .get(&message.channel)
        .ok_or(ChatError::NoChannelFound)?;

    tx.send(message.message.to_string())?;
    return Ok("Message has been sent".to_string());
}

pub fn routes() -> Vec<rocket::Route> {
    routes![subscribe, send]
}
