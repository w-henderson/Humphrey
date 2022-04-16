use humphrey_json::prelude::*;

pub struct UserInfo {
    pub(crate) uid: String,
    pub(crate) name: String,
}

json_map! {
    UserInfo,
    uid => "uid",
    name => "name"
}
