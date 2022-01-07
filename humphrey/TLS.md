# Using HTTPS/TLS with Humphrey
Humphrey provides a crate feature called `tls` which allows you to use TLS with your web application. This depends upon the `rustls` crate for the cryptography implementation.

## Installation
Add the `tls` feature to the Humphrey dependency in your `Cargo.toml`, as follows (although you should specify a concrete version):
```toml
[dependencies]
humphrey = { version = "*", features = ["tls"] }
```

## Setting up TLS
To run a server with TLS using the `run_tls` method on the `App` struct, you must provide the paths to the certificate and key files. For local development, these can be generated using [`mkcert`](https://github.com/FiloSottile/mkcert). Install the tool and generate the required files as outlined on the `mkcert` repository, then provide paths to these files (absolute or relative) in the `run_tls` method.

Example `mkcert` usage:
```bash
$ mkcert -install
$ mkcert localhost
```

## Example Usage
A simple example program demonstrating TLS usage can be found [here](https://github.com/w-henderson/Humphrey/tree/master/examples/tls) in the examples directory.