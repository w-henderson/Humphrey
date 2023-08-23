# Tokio
This chapter covers how to use Humphrey with the Tokio async runtime. Currently, only Humphrey Core supports integration with Tokio.

## Enabling Tokio
To enable Tokio support, enable the `tokio` feature of the `humphrey` crate in your `Cargo.toml` file. You'll also need Tokio as a direct dependency of your project.

```toml
[dependencies]
humphrey = { version = "0.7", features = ["tokio"] }
tokio = { version = "1", features = ["full"] }
```

## Using Tokio
With the Tokio feature enabled, everything you would expect to be asynchronous is now asynchronous. That's it!