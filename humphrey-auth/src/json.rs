use crate::session::Session;
use crate::user::User;

use humphrey_json::prelude::*;

json_map! {
    User,
    uid => "uid",
    session => "session",
    password_hash => "password_hash"
}

json_map! {
    Session,
    token => "token",
    expiry => "expiry"
}
