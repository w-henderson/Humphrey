use humphrey::App;
use std::net::SocketAddr;

fn main() {
    let mut app = App::new();
    app.run(&("127.0.0.1:80".parse().unwrap()));
}
