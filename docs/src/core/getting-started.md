# Getting Started
This chapter will walk you through the steps to get started with Humphrey Core. It assumes that you already have Rust and Cargo installed, but installation steps for these can be found in the [Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html).

## Creating a New Project
A Humphrey Core web application is a Rust binary crate, so to begin we'll need to create a new project using Cargo.

```bash
$ cargo new my-app
```

Next, we need to add the `humphrey` crate as a dependency of our project, which can be done by editing the `Cargo.toml` file within the project directory as follows. Please note that it is not good practice to use the `*` version number for a real project, as if a new version of Humphrey adds breaking changes, this could cause your application to stop working properly. In that case, you should check the latest Humphrey version on [crates.io](https://crates.io/crates/humphrey) and use that version number.

```toml
[dependencies]
humphrey = "*"
```

With that, our project will compile and run, but at the moment it doesn't do anything.

## Creating a Humphrey App
We can now initialise a new Humphrey application in the `main.rs` file of the `src` directory.

```rs
use humphrey::App;
use humphrey::http::{Response, StatusCode};

fn main() {
  let app = App::new()
    .with_route("/", |request, _| {
      Response::new(StatusCode::OK, "Hello, Humphrey!", &request)
    });

  app.run("0.0.0.0:80").unwrap();
}
```

If we now run `cargo run`, our application will successfully compile and start the server, which you can access at [localhost](http://localhost).