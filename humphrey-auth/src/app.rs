use crate::database::AuthDatabase;
use crate::AuthProvider;

use humphrey::http::headers::RequestHeader;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use std::sync::{Arc, MutexGuard};

/// Represents a state which contains an `AuthProvider`.
/// This must be implemented on the state in order to use authentication.
///
/// # Example
/// ```
/// type DatabaseWrapper = Arc<RwLock<MyDatabase>>;
///
/// struct MyState {
///     db: DatabaseWrapper,
///     auth_provider: Mutex<AuthProvider<DatabaseWrapper>>
/// }
///
/// impl AuthState<DatabaseWrapper> for MyState {
///    fn auth_provider(&self) -> MutexGuard<AuthProvider<DatabaseWrapper>> {
///       self.auth_provider.lock().unwrap()
///   }
/// }
/// ```
pub trait AuthState<D>
where
    D: AuthDatabase,
{
    fn auth_provider(&self) -> MutexGuard<AuthProvider<D>>;
}

/// Represents a function able to handle an authenticated request.
/// This is passed the request, the state, and the UID of the authenticated user.
///
/// # Example
/// ```
/// fn auth_req_handler(req: Request, state: Arc<MyState>, uid: String) -> Response {
///     Response::new(StatusCode::OK, uid, &request)
/// }
/// ```
pub trait AuthRequestHandler<S>: Fn(Request, Arc<S>, String) -> Response + Send + Sync {}
impl<T, S> AuthRequestHandler<S> for T where T: Fn(Request, Arc<S>, String) -> Response + Send + Sync
{}

/// Represents a Humphrey application with authentication enabled.
/// This is implemented on Humphrey's `App` type provided that the state implements `AuthState`
///   and the database implements `AuthDatabase`.
pub trait AuthApp<S, D>
where
    S: AuthState<D>,
    D: AuthDatabase,
{
    /// Adds an authenticated route and associated handler to the server.
    /// Routes can include wildcards, such as `/blog/*`.
    ///
    /// ## Panics
    /// This function will panic if the route string cannot be converted to a `Uri` object.
    fn with_auth_route<T>(self, route: &str, handler: T) -> Self
    where
        T: AuthRequestHandler<S> + 'static;
}

impl<S, D> AuthApp<S, D> for App<S>
where
    S: AuthState<D> + Send + Sync,
    D: AuthDatabase,
{
    fn with_auth_route<T>(self, route: &str, handler: T) -> Self
    where
        T: AuthRequestHandler<S> + 'static,
    {
        self.with_route(route, move |request: Request, state: Arc<S>| {
            if let Some(cookie) = request.headers.get(&RequestHeader::Cookie) {
                let token = cookie
                    .split(';')
                    .find(|s| s.trim().starts_with("HumphreyToken="))
                    .map(|s| s.trim().strip_prefix("HumphreyToken=").unwrap());

                if let Some(token) = token {
                    let uid = state.auth_provider().get_uid_by_token(token);

                    if let Ok(uid) = uid {
                        return (handler)(request, state, uid);
                    }
                }
            }

            forbidden()
        })
    }
}

fn forbidden() -> Response {
    Response::new(StatusCode::Unauthorized, "401 Unauthorized")
}
