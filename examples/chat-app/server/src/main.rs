mod messages;
mod user;

use crate::messages::{ClientMessage, ClientMessageKind, ServerMessage, ServerMessageKind};
use crate::user::{User, UserManager};

use humphrey::handlers::serve_dir;
use humphrey::App;

use humphrey_json::error::ParseError;
use humphrey_json::prelude::*;

use humphrey_ws::async_app::{AsyncStream, AsyncWebsocketApp};
use humphrey_ws::handler::async_websocket_handler;
use humphrey_ws::message::Message;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

static NEXT_UID: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
pub struct State {
    users: RwLock<HashMap<SocketAddr, User>>,
}

fn main() {
    let websocket_app: AsyncWebsocketApp<State> = AsyncWebsocketApp::new_unlinked()
        .with_message_handler(message_handler)
        .with_connect_handler(connect_handler)
        .with_disconnect_handler(disconnect_handler);

    let mut args = std::env::args();

    let client_dir: &'static str = Box::leak(Box::new(if let Some(path) = args.nth(1) {
        path
    } else {
        PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("client")
            .join("build")
            .to_string_lossy()
            .to_string()
    }));

    let humphrey_app: App<()> = App::new()
        .with_path_aware_route("/*", serve_dir(client_dir))
        .with_websocket_route(
            "/ws",
            async_websocket_handler(websocket_app.connect_hook().unwrap()),
        );

    spawn(move || humphrey_app.run("0.0.0.0:80").unwrap());

    websocket_app.run();
}

fn connect_handler(stream: AsyncStream, state: Arc<State>) {
    let user = User {
        id: NEXT_UID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        name: "".to_string(),
        loaded: false,
    };

    let message = ServerMessage {
        kind: ServerMessageKind::Id,
        message: None,
        sender_id: user.id,
        sender_name: None,
    };

    stream.send(Message::new(humphrey_json::to_string(&message)));

    state.set_user(stream.peer_addr(), user);
}

fn disconnect_handler(stream: AsyncStream, state: Arc<State>) {
    let user = state.get_user(stream.peer_addr()).unwrap();

    state.remove_user(stream.peer_addr());

    let message = ServerMessage {
        kind: ServerMessageKind::Leave,
        message: None,
        sender_id: user.id,
        sender_name: Some(user.name),
    };

    stream.broadcast(Message::new(humphrey_json::to_string(&message)));
}

fn message_handler(stream: AsyncStream, message: Message, state: Arc<State>) {
    let mut user = state.get_user(stream.peer_addr()).unwrap();

    let client_message: Result<ClientMessage, ParseError> =
        humphrey_json::from_str(&message.text().unwrap());

    let client_message = if let Ok(client_message) = client_message {
        client_message
    } else {
        return;
    };

    if user.loaded && client_message.kind == ClientMessageKind::Chat {
        let message = ServerMessage {
            kind: ServerMessageKind::Chat,
            message: Some(client_message.message),
            sender_id: user.id,
            sender_name: Some(user.name),
        };

        stream.broadcast(Message::new(humphrey_json::to_string(&message)));
    } else if client_message.kind == ClientMessageKind::Register {
        user.name = client_message.message;
        user.loaded = true;

        state.set_user(stream.peer_addr(), user.clone());

        let broadcast_message = ServerMessage {
            kind: ServerMessageKind::Join,
            message: None,
            sender_id: user.id,
            sender_name: Some(user.name),
        };

        let private_message = json!({
            "kind": (ServerMessageKind::Participants),
            "participants": (state.list_users())
        });

        stream.broadcast(Message::new(humphrey_json::to_string(&broadcast_message)));
        stream.send(Message::new(private_message.serialize()));
    };
}
