use super::request::RequestError;

#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    pub fn from_name(name: &str) -> Result<Self, RequestError> {
        match name {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            _ => Err(RequestError::Request),
        }
    }
}
