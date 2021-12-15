use crate::database::AuthDatabase;
use crate::AuthProvider;

use humphrey::http::headers::RequestHeader;
use humphrey::http::{Request, Response, StatusCode};
use humphrey::App;

use std::sync::{Arc, MutexGuard};

pub trait AuthState<D>
where
    D: AuthDatabase,
{
    fn auth_provider(&self) -> MutexGuard<AuthProvider<D>>;
}

pub trait AuthRequestHandler<S>: Fn(Request, Arc<S>, String) -> Response + Send + Sync {}
impl<T, S> AuthRequestHandler<S> for T where T: Fn(Request, Arc<S>, String) -> Response + Send + Sync
{}

pub trait AuthApp<S, D>
where
    S: AuthState<D>,
    D: AuthDatabase,
{
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

            forbidden(request)
        })
    }
}

fn forbidden(request: Request) -> Response {
    Response::new(StatusCode::Unauthorized, "401 Unauthorized", &request)
}
