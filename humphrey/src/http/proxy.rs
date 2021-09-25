use crate::http::headers::ResponseHeader;
use crate::http::response::ResponseError;
use crate::http::{Request, Response, StatusCode};

use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub fn proxy_request(request: &Request, target: SocketAddr, timeout: Duration) -> Response {
    match proxy_request_internal(request, target, timeout) {
        Ok(response) => response.with_header(
            ResponseHeader::Custom {
                name: "X-Forwarded-For".into(),
            },
            request.address.origin_addr.to_string(),
        ),
        Err(_) => Response::new(StatusCode::BadGateway)
            .with_bytes(b"<html><body><h1>502 Bad Gateway</h1></body></html>".to_vec())
            .with_request_compatibility(request)
            .with_generated_headers(),
    }
}

fn proxy_request_internal(
    request: &Request,
    target: SocketAddr,
    timeout: Duration,
) -> Result<Response, ResponseError> {
    let mut stream =
        TcpStream::connect_timeout(&target, timeout).map_err(|_| ResponseError::Stream)?;
    let request_bytes: Vec<u8> = request.clone().into();
    stream
        .write_all(&request_bytes)
        .map_err(|_| ResponseError::Stream)?;

    Response::from_stream(&mut stream)
}
