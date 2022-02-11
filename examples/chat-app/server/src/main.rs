mod serialise;
mod user;

use crate::serialise::BroadcastMessage;
use crate::user::{User, UserManager};

use humphrey::handlers::serve_dir;
use humphrey::App;

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

    let client_dir: &'static str = Box::leak(Box::new(
        PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("client")
            .join("build")
            .to_string_lossy()
            .to_string(),
    ));

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

    stream.send(Message::new_binary(user.id.to_be_bytes()));
    state.set_user(stream.peer_addr(), user);
}

fn disconnect_handler(stream: AsyncStream, state: Arc<State>) {
    let user = state.get_user(stream.peer_addr()).unwrap();

    state.remove_user(stream.peer_addr());

    let broadcast = BroadcastMessage {
        message: format!("{} has left the chat", user.name),
        sender_id: 0,
        sender_name: None,
    };

    stream.broadcast(broadcast.serialise())
}

fn message_handler(stream: AsyncStream, message: Message, state: Arc<State>) {
    let mut user = state.get_user(stream.peer_addr()).unwrap();

    if user.loaded {
        let broadcast = BroadcastMessage {
            message: message.text().unwrap().to_string(),
            sender_id: user.id,
            sender_name: Some(user.name),
        };

        stream.broadcast(broadcast.serialise());
    } else {
        user.name = message.text().unwrap().to_string();
        user.loaded = true;

        state.set_user(stream.peer_addr(), user);

        let broadcast = BroadcastMessage {
            message: format!("{} has joined the chat", message.text().unwrap()),
            sender_id: 0,
            sender_name: None,
        };

        stream.broadcast(broadcast.serialise());
    };
}
