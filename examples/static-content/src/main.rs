//! Static content server example.
//!
//! ## Important
//! This example must be run from the `static-content` directory to successfully find the paths.
//! This is because content is found relative to the CWD instead of the binary.

use humphrey::handlers;
use humphrey::App;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app: App<()> = App::new()
        // Serve the "/" route with the specified file
        .with_route("/", handlers::serve_file("./static/pages/index.html"))
        // Serve the "/img/*" route with files stored in the "./static/images" directory.
        // Strip the "/img/" prefix from the request URI before it is concatenated with the directory path.
        // For example, "/img/ferris.png" would go to "ferris.png" and then to "./static/images/ferris.png".
        .with_route("/img/*", handlers::serve_dir("./static/images", "/img/"))
        // Serve a regular file path in the current directory.
        // This means simply appending the request URI to the directory path and looking for a file there.
        // This is equivalent to `serve_dir` with a strip prefix value of `""`.
        .with_route("/src/*", handlers::serve_as_file_path("."));

    app.run("0.0.0.0:80")?;

    Ok(())
}
