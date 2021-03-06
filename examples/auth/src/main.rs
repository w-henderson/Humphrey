mod database;
mod user;

use database::WrappedDatabase;
use user::UserInfo;

use humphrey::handlers::serve_dir;
use humphrey::http::cookie::{SameSite, SetCookie};
use humphrey::http::headers::HeaderType;
use humphrey::http::method::Method;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use humphrey_auth::app::{AuthApp, AuthState};
use humphrey_auth::config::AuthConfig;
use humphrey_auth::AuthProvider;

use jasondb::query;
use jasondb::Database;

use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

struct AppState {
    db: Mutex<Database<UserInfo>>,
    auth: Mutex<AuthProvider<WrappedDatabase>>,
}

impl AuthState<WrappedDatabase> for AppState {
    fn auth_provider(&self) -> MutexGuard<AuthProvider<WrappedDatabase>> {
        self.auth.lock().unwrap()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Use JasonDB to create or open a database, wrapping it so we can implement `AuthDatabase` on it in `database.rs`.
    let auth_db = Database::new("auth_db.jdb")?.with_index("session.token")?;
    let auth_db = WrappedDatabase::new(auth_db);

    // Set up the authentication provider.
    let config = AuthConfig::default()
        .with_default_lifetime(30) // sessions expire after 30 seconds
        .with_pepper("hunter42"); // pepper is used when hashing passwords, this should be kept very safe
    let provider = AuthProvider::new(auth_db).with_config(config);

    // Create a database to store user information.
    let database = Database::new("users_db.jdb")?.with_index("name")?;

    // Set up the app's state.
    let state = AppState {
        db: Mutex::new(database),
        auth: Mutex::new(provider),
    };

    // Create a new Humphrey application.
    let app = App::new_with_config(32, state)
        .with_route("/api/login", login) // login API endpoint
        .with_route("/api/signup", signup) // sign up API endpoint
        .with_auth_route("/api/signout", sign_out) // sign out API endpoint (requires auth)
        .with_auth_route("/api/deleteAccount", delete_account) // delete account API endpoint (requires auth)
        .with_auth_route("/profile.html", profile) // profile page (requires auth)
        .with_path_aware_route("/*", serve_dir("./static")); // serve static files from the static directory

    // Run the app.
    app.run("0.0.0.0:80")?;

    Ok(())
}

/// Login API endpoint handler.
fn login(request: Request, state: Arc<AppState>) -> Response {
    // If the request is not a POST, return a 405 error.
    if request.method != Method::Post {
        return Response::new(StatusCode::MethodNotAllowed, b"Method Not Allowed");
    }

    // Get the username and password from the request body.
    let body_str = String::from_utf8(request.content.as_ref().unwrap().clone()).unwrap();
    let mut body_split = body_str.split(',');
    let username = body_split.next().unwrap();
    let password = body_split.next().unwrap();

    // Get the UID of the user with the given username from the database.
    let uid = {
        let mut db = state.db.lock().unwrap();

        db.query(query!(name == username))
            .ok()
            .and_then(|i| i.flatten().next())
            .map(|(_, user)| user.uid)
    };

    if let Some(uid) = uid {
        // If the user was found, check the password.

        let mut provider = state.auth.lock().unwrap();
        let verify = provider.verify(&uid, password);

        if verify {
            // If the password is correct, create a session for the user.

            if let Ok(token) = provider.create_session(uid) {
                // If the session was created, return a 200 response with the token.

                return Response::empty(StatusCode::OK)
                    .with_cookie(
                        SetCookie::new("HumphreyToken", token)
                            .with_path("/")
                            .with_max_age(Duration::from_secs(3600))
                            .with_same_site(SameSite::Strict),
                    )
                    .with_bytes(b"OK");
            } else {
                // If the session could not be created, return an error.

                return Response::new(StatusCode::NotFound, b"Already logged in");
            }
        } else {
            // If the password was incorrect, return an error.

            return Response::new(StatusCode::NotFound, b"Invalid credentials");
        }
    }

    // If the user was not found, return an error.
    Response::new(StatusCode::NotFound, b"User not found")
}

/// Sign up API endpoint handler.
fn signup(request: Request, state: Arc<AppState>) -> Response {
    // If the request is not a POST, return a 405 error.
    if request.method != Method::Post {
        return Response::new(StatusCode::MethodNotAllowed, b"Method Not Allowed");
    }

    // Get the username and password from the request body.
    let body_str = String::from_utf8(request.content.as_ref().unwrap().clone()).unwrap();
    let mut body_split = body_str.split(',');
    let username = body_split.next().unwrap();
    let password = body_split.next().unwrap();

    // Check whether a user with the given username already exists.
    let existing_user = {
        let mut db = state.db.lock().unwrap();

        db.query(query!(name == username)).unwrap().count() > 0
    };

    if !existing_user {
        // If no user exists with the given username, create a new user.

        // Use the auth provider to create a user and get the UID.
        let uid = {
            let mut provider = state.auth.lock().unwrap();
            provider.create_user(password).unwrap()
        };

        // Add the user to the database.
        // It is important to note that the user's ID and password are already in the auth database,
        //   but we need to add the user's ID and username into the users database.
        let mut db = state.db.lock().unwrap();
        db.set(
            &uid,
            UserInfo {
                uid: uid.to_string(),
                name: username.to_string(),
            },
        )
        .unwrap();

        // Return a successful response.
        return Response::new(StatusCode::OK, b"OK");
    }

    // If a user already exists with the given username, return an error.
    Response::new(StatusCode::NotFound, b"User not found")
}

/// Sign out API endpoint handler.
fn sign_out(_: Request, state: Arc<AppState>, uid: String) -> Response {
    // Use the auth provider to invalidate the user's session.
    let mut provider = state.auth.lock().unwrap();
    provider.invalidate_user_session(uid);

    // Return a response which redirects the client to the homepage as well as resets the cookie.
    Response::empty(StatusCode::Found)
        .with_bytes("OK")
        .with_header(HeaderType::Location, "/")
        .with_cookie(
            SetCookie::new("HumphreyToken", "deleted")
                .with_path("/")
                .with_expires("Thu, 01 Jan 1970 00:00:00 GMT"),
        )
}

/// Delete account API endpoint handler.
fn delete_account(_: Request, state: Arc<AppState>, uid: String) -> Response {
    // Remove the user from the users section of the database.
    {
        let mut db = state.db.lock().unwrap();
        db.delete(&uid).unwrap();
    }

    // Use the auth provider to remove the user from the auth section of the database.
    let mut provider = state.auth.lock().unwrap();
    provider.remove_user(&uid).unwrap();

    // Return a response which redirects the client to the homepage as well as resets the cookie.
    Response::empty(StatusCode::Found)
        .with_bytes("OK")
        .with_header(HeaderType::Location, "/")
        .with_cookie(
            SetCookie::new("HumphreyToken", "deleted")
                .with_path("/")
                .with_expires("Thu, 01 Jan 1970 00:00:00 GMT"),
        )
}

/// Profile page handler.
fn profile(_: Request, state: Arc<AppState>, uid: String) -> Response {
    // Use the database to get the username of the authenticated user.
    let mut db = state.db.lock().unwrap();
    let user = db.get(uid).unwrap();

    // Format the HTML template with the username.
    let html = include_str!("../static/profile.html").replace("{username}", &user.name);

    // Return the response.
    Response::empty(StatusCode::OK)
        .with_header(HeaderType::ContentType, "text/html")
        .with_bytes(html)
}
