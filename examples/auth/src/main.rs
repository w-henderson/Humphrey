mod database;

use database::WrappedDatabase;

use humphrey::handlers::serve_dir;
use humphrey::http::headers::ResponseHeader;
use humphrey::http::method::Method;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use humphrey_auth::app::{AuthApp, AuthState};
use humphrey_auth::AuthProvider;

use jasondb::{Database, JasonDB};

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};

struct AppState {
    db: WrappedDatabase,
    auth: Mutex<AuthProvider<WrappedDatabase>>,
}

impl AuthState<WrappedDatabase> for AppState {
    fn auth_provider(&self) -> MutexGuard<AuthProvider<WrappedDatabase>> {
        self.auth.lock().unwrap()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let database = JasonDB::open("database.jdb").unwrap_or_else(new_db);
    let database = WrappedDatabase::new(database);
    let provider = AuthProvider::new(database.clone());

    let state = AppState {
        db: database,
        auth: Mutex::new(provider),
    };

    let app = App::new_with_config(32, state)
        .with_route("/api/login", login)
        .with_route("/api/signup", signup)
        .with_auth_route("/profile", profile)
        .with_path_aware_route("/*", serve_dir("./static"));

    app.run("0.0.0.0:80")?;

    Ok(())
}

fn login(request: Request, state: Arc<AppState>) -> Response {
    println!("login request");

    if request.method != Method::Post {
        return Response::new(
            StatusCode::MethodNotAllowed,
            b"Method Not Allowed",
            &request,
        );
    }

    let body_str = String::from_utf8(request.content.as_ref().unwrap().clone()).unwrap();
    let mut body_split = body_str.split(',');
    let username = body_split.next().unwrap();
    let password = body_split.next().unwrap();

    println!("username={} password={}", username, password);

    let db = state.db.0.read();
    let users = db.collection("users").unwrap();

    let uid = users
        .list()
        .iter()
        .find(|doc| doc.json == username)
        .map(|doc| doc.id.clone());

    drop(db);

    if let Some(uid) = uid {
        println!("uid={}", uid);

        let mut provider = state.auth.lock().unwrap();
        let verify = provider.verify(&uid, password);

        if verify {
            if let Ok(token) = provider.create_session(uid) {
                println!("token={}", token);

                return Response::empty(StatusCode::OK)
                    .with_header(
                        ResponseHeader::SetCookie,
                        format!("HumphreyToken={}; Path=/", token),
                    )
                    .with_bytes(b"OK")
                    .with_request_compatibility(&request)
                    .with_generated_headers();
            } else {
                return Response::new(StatusCode::NotFound, b"Already logged in", &request);
            }
        } else {
            return Response::new(StatusCode::NotFound, b"Invalid credentials", &request);
        }
    }

    Response::new(StatusCode::NotFound, b"User not found", &request)
}

fn signup(request: Request, state: Arc<AppState>) -> Response {
    println!("signup request");

    if request.method != Method::Post {
        return Response::new(
            StatusCode::MethodNotAllowed,
            b"Method Not Allowed",
            &request,
        );
    }

    let body_str = String::from_utf8(request.content.as_ref().unwrap().clone()).unwrap();
    let mut body_split = body_str.split(',');
    let username = body_split.next().unwrap();
    let password = body_split.next().unwrap();

    println!("username={} password={}", username, password);

    let db = state.db.0.write();
    let users = db.collection("users").unwrap();

    if !users.list().iter().any(|doc| doc.json == username) {
        drop(db);

        println!("no existing uid");

        let uid = {
            let mut provider = state.auth.lock().unwrap();
            provider.create_user(password).unwrap()
        };

        println!("uid={}", uid);

        let mut db = state.db.0.write();
        let users = db.collection_mut("users").unwrap();
        users.set(uid, username);

        return Response::new(StatusCode::OK, b"OK", &request);
    }

    Response::new(StatusCode::NotFound, b"User not found", &request)
}

fn profile(request: Request, _: Arc<AppState>, uid: String) -> Response {
    Response::new(StatusCode::OK, b"Hello", &request)
}

/// Create a new database, automatically starting the background thread to synchronize the database to disk.
fn new_db(_: Box<dyn Error>) -> JasonDB {
    // Create the database and the `messages` collection.
    let mut db = Database::new("database.jdb");
    db.create_collection("auth").unwrap();
    db.create_collection("users").unwrap();

    // Initialise the JasonDB instance with the pre-existing database.
    JasonDB::init(db, "database.jdb")
}
