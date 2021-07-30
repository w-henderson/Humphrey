use std::convert::TryFrom;

use super::headers::{RequestHeader, RequestHeaderMap};
use super::method::Method;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub version: String,
    pub headers: RequestHeaderMap,
    pub content: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestError;

impl TryFrom<&str> for Request {
    fn try_from(string: &str) -> Result<Self, RequestError> {
        println!("{}", string);

        let mut lines = string.split("\r\n");

        let start_line: Vec<&str> = lines.next().unwrap().split(" ").collect();
        println!("{:?}", start_line);

        safe_assert(start_line.len() == 3)?;

        println!("split right");

        let method = Method::from_name(start_line[0])?;
        let url = start_line[1].to_string();
        let version = start_line[2].to_string();

        println!("parsed opening line");

        let mut headers = RequestHeaderMap::new();
        for line in lines.by_ref() {
            if line == "" {
                break;
            }

            let line_parts: Vec<&str> = line.splitn(2, ':').collect();
            safe_assert(line_parts.len() == 2)?;
            println!("split header right");
            headers.insert(
                RequestHeader::from(line_parts[0]),
                line_parts[1].trim_start().to_string(),
            );
        }

        println!("parsed headers");

        let content = lines.next().map(String::from);

        println!("parsed content");

        Ok(Self {
            method,
            url,
            version,
            headers,
            content,
        })
    }

    type Error = RequestError;
}

fn safe_assert(condition: bool) -> Result<(), RequestError> {
    match condition {
        true => Ok(()),
        false => Err(RequestError),
    }
}
