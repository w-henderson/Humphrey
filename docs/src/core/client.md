# Using as a Client
Humphrey Core also provides client functionality, which allows dependent programs to send HTTP requests. It optionally supports TLS with the `tls` feature, the setup for which was discussed in the [Using HTTPS](https.md) section. This section assumes the TLS feature is enabled.

## Sending a Simple Request
A simple request can be sent by creating a `Client` object, creating and sending a GET request, then parsing the response from the body into a string. This basic example shows how to use the Ipify API to get your public IP address.

```rs
use humphrey::Client;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new();
    let response = client.get("https://api.ipify.org")?.send()?;

    println!("IP address: {}", response.text().ok_or("Invalid text")?);

    Ok(())
}
```

## Adding Headers and Following Redirects
Headers can be added to the request by using the `with_header` method on the `ClientRequest` struct. For this example, we'll use the `User-Agent` header to identify the client. Redirects can be followed by using `with_redirects` and specifying to follow redirects.

```rs
use humphrey::http::headers::HeaderType;
use humphrey::Client;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new();
    let response = client
        .get("https://api.ipify.org")?
        .with_redirects(true)
        .with_header(HeaderType::UserAgent, "HumphreyExample/1.0")
        .send()?;

    println!("IP address: {}", response.text().ok_or("Invalid text")?);

    Ok(())
}
```

## Using HTTPS
You'll notice that the previous examples have requested the HTTPS endpoint for the API. If we were to run these examples without the TLS feature enabled, an error would be encountered. Furthermore, creating the `Client` object with TLS enabled is an expensive operation since certificates must be loaded from the operating system, so it is advisable to create one client per application instead of one per request.

## Conclusion
In conclusion, Humphrey provides a powerful way to make requests as well as to serve them. If you want to learn more about Humphrey, consider exploring the [API reference](https://docs.rs/humphrey) or reading the [WebSocket guide](../websocket/index.md).