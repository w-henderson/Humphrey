use humphrey_server::config::RouteType;
use humphrey_server::AppState;
use humphrey_ws::{Message, WebsocketStream};

use notify::{raw_watcher, Op, RecursiveMode, Watcher};

use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

struct WatchedRoute {
    path: PathBuf,
    url_prefix: String,
}

pub fn init(
    streams: Arc<Mutex<Vec<WebsocketStream>>>,
    state: Arc<AppState>,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx)?;
    let mut watched_routes = Vec::new();

    for route in &state.config.default_host.routes {
        match route.route_type {
            RouteType::File | RouteType::Directory => {
                let path = PathBuf::from(route.path.as_ref().unwrap()).canonicalize()?;
                watcher.watch(&path, RecursiveMode::Recursive)?;

                state.logger.debug(format!(
                    "Hot Reload: Watching for changes on {}",
                    path.display()
                ));

                watched_routes.push(WatchedRoute {
                    path,
                    url_prefix: get_url_prefix(&route.matches)?,
                });
            }
            _ => (),
        }
    }

    spawn(move || {
        // Watcher must be moved onto the thread so it doesn't get dropped.
        // This is because `Drop` disconnects the channel.
        let _watcher_on_thread = watcher;

        loop {
            let event = rx.recv().unwrap();

            println!("event! {:?}", event);

            if event.path.is_none() || event.op.is_err() || event.op.unwrap() != Op::WRITE {
                continue;
            }

            let path = event.path.unwrap();

            let mut streams = streams.lock().unwrap();

            for route in &watched_routes {
                if path.starts_with(&route.path) {
                    let url = route.url_prefix.clone()
                        + path.strip_prefix(&route.path).unwrap().to_str().unwrap();

                    println!("change! {}", &url);

                    for stream in &mut *streams {
                        stream.send(Message::new(url.clone())).unwrap();
                    }
                }
            }
        }
    });

    Ok(())
}

pub fn get_url_prefix(s: &str) -> Result<String, String> {
    let ends_with_wildcard = s.ends_with('*');
    let number_of_wildcards = s.chars().filter(|c| *c == '*').count();

    if ends_with_wildcard && number_of_wildcards == 1 {
        Ok(s.trim_end_matches('*').to_string())
    } else if number_of_wildcards == 0 {
        Ok(s.to_string())
    } else {
        Err("Invalid URL prefix".to_string())
    }
}
