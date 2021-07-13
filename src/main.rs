use humphrey::http::request::Request;

fn main() {
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: Chrome\r\nAccept:text/html\r\n";
    let request2 = "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 3\r\n\r\npog";

    let parsed_request = Request::from_string(request2).unwrap();
    println!("{:?}", parsed_request);
}
