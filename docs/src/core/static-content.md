# Static Content
This chapter covers how to serve static content with Humphrey. Serving static content is a vital part of many web applications, and Humphrey provides a simple way to do this.

The [`handlers`](https://docs.rs/humphrey/*/humphrey/handlers) module provides a number of useful built-in functions to handle requests for static content.

## Serving a File
The `serve_file` handler is the simplest way to serve a single file.

```rs
use humphrey::handlers::serve_file;
use humphrey::App;

fn main() {
    let app: App<()> = App::new()
        .with_route("/foo", serve_file("./bar.html"));

    app.run("0.0.0.0:80").unwrap();
}
```

## Serving a Directory
The `serve_dir` handler allows you to serve a directory of files. The path you specify should be relative to the current directory.

This handler must be applied using the `with_path_aware_route` method, since the path is used to determine how to locate the requested content. For example, a request to `/static/foo.html` with a handler that looks for `/static/*` should find the file at `/static/foo.html`, not `/static/static/foo.html`.

```rs
use humphrey::handlers::serve_dir;
use humphrey::App;

fn main() {
    let app: App<()> = App::new()
        .with_path_aware_route("/static/*", serve_dir("./static"));

    app.run("0.0.0.0:80").unwrap();
}
```

## Redirecting Requests
The `redirect` handler allows you to redirect requests to a different path, whether it be on the same domain or a different domain.

```rs
use humphrey::handlers::redirect;
use humphrey::App;

fn main() {
    let app: App<()> = App::new()
        .with_route("/foo", redirect("https://www.example.com"));

    app.run("0.0.0.0:80").unwrap();
}
```

## Conclusion
In this section, we've learnt how to use Humphrey's built-in handlers to serve static content from a Humphrey web application. In the next section, we'll explore how to use HTTPS (TLS) with Humphrey using the `rustls` crate.