use humphrey::http::headers::RequestHeader;
use humphrey::Client;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new();

    // Use the Ipify API to get current IP.
    let response = client.get("https://api.ipify.org")?.send()?;
    let text = String::from_utf8(response.body)?;
    println!("Your IP is: {}", text);

    // Post mock data to the JSON placeholder API.
    let data = "{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}".into();
    let response = client
        .post("https://jsonplaceholder.typicode.com/posts", data)?
        .with_header(
            RequestHeader::ContentType,
            "application/json; charset=UTF-8",
        )
        .send()?;
    let text = String::from_utf8(response.body)?;
    println!("Response from JSON placeholder API: {}", text);

    Ok(())
}
