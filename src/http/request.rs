use super::headers::{Header, RequestHeader, RequestHeaderMap};
use super::method::Method;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    method: Method,
    url: String,
    version: String,
    headers: RequestHeaderMap,
    content: Option<String>,
}

#[derive(Debug)]
pub struct RequestError;

impl Request {
    pub fn from_string(string: &str) -> Result<Self, RequestError> {
        let mut lines = string.split("\r\n");

        let start_line: Vec<&str> = lines.next().unwrap().split(" ").collect();
        safe_assert(start_line.len() == 3)?;

        let method = Method::from_name(start_line[0])?;
        let url = start_line[1].to_string();
        let version = start_line[2].to_string();

        let mut headers = RequestHeaderMap::new();
        for line in lines.by_ref() {
            if line == "" {
                break;
            }

            let line_parts: Vec<&str> = line.splitn(2, ':').collect();
            safe_assert(line_parts.len() == 2)?;
            headers.insert(
                RequestHeader::from_name(line_parts[0]),
                line_parts[1].trim_start().to_string(),
            );
        }

        let content = lines.next().map(String::from);

        Ok(Self {
            method,
            url,
            version,
            headers,
            content,
        })
    }
}

fn safe_assert(condition: bool) -> Result<(), RequestError> {
    match condition {
        true => Ok(()),
        false => Err(RequestError),
    }
}
