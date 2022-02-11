use std::net::SocketAddr;
use std::sync::Arc;

use crate::State;

#[derive(Clone)]
pub struct User {
    pub id: usize,
    pub name: String,
    pub loaded: bool,
}

pub trait UserManager {
    fn get_user(&self, addr: SocketAddr) -> Option<User>;
    fn set_user(&self, addr: SocketAddr, user: User);
    fn remove_user(&self, addr: SocketAddr);
}

impl UserManager for Arc<State> {
    fn get_user(&self, addr: SocketAddr) -> Option<User> {
        self.users.read().unwrap().get(&addr).cloned()
    }

    fn set_user(&self, addr: SocketAddr, user: User) {
        self.users.write().unwrap().insert(addr, user);
    }

    fn remove_user(&self, addr: SocketAddr) {
        self.users.write().unwrap().remove(&addr);
    }
}
