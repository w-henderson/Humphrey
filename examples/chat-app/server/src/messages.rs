use humphrey_json::error::ParseError;
use humphrey_json::prelude::*;
use humphrey_json::Value;

#[derive(FromJson, IntoJson)]
pub struct ClientMessage {
    pub kind: ClientMessageKind,
    pub message: String,
}

#[derive(FromJson, IntoJson)]
pub struct ServerMessage {
    pub kind: ServerMessageKind,
    pub message: Option<String>,
    #[rename = "senderId"]
    pub sender_id: usize,
    #[rename = "senderName"]
    pub sender_name: Option<String>,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClientMessageKind {
    Register,
    Chat,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ServerMessageKind {
    Id,
    Participants,
    Join,
    Chat,
    Leave,
}

impl FromJson for ClientMessageKind {
    fn from_json(value: &Value) -> Result<Self, ParseError> {
        match value.as_number().map(|f| f as u8) {
            Some(0) => Ok(Self::Register),
            Some(1) => Ok(Self::Chat),
            _ => Err(ParseError::TypeError),
        }
    }
}

impl IntoJson for ClientMessageKind {
    fn to_json(&self) -> Value {
        Value::from(*self as u8)
    }
}

impl FromJson for ServerMessageKind {
    fn from_json(value: &Value) -> Result<Self, ParseError> {
        match value.as_number().map(|f| f as u8) {
            Some(0) => Ok(Self::Id),
            Some(1) => Ok(Self::Participants),
            Some(2) => Ok(Self::Join),
            Some(3) => Ok(Self::Chat),
            Some(4) => Ok(Self::Leave),
            _ => Err(ParseError::TypeError),
        }
    }
}

impl IntoJson for ServerMessageKind {
    fn to_json(&self) -> Value {
        Value::from(*self as u8)
    }
}
