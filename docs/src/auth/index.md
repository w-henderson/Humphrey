# Humphrey Auth
Humphrey Auth is simple authentication crate for Humphrey applications. It provides a simple and database-agnostic way to authenticate users and handle sessions and tokens. It does depend upon the [`argon2`](https://docs.rs/argon2), [`uuid`](https://docs.rs/uuid) and [`rand_core`](https://docs.rs/rand_core) crates to ensure that it is secure.

Humphrey Auth needs to be integrated into a full-stack Humphrey application with endpoints for all the authentication-related methods, such as signing in and out. Therefore, this guide does not provide step-by-step instructions on how to use it.

It is easiest to learn how to use Humphrey Auth from the [full example](https://github.com/w-henderson/Humphrey/blob/master/examples/auth/src/main.rs). Alongside this, it may be useful to refer to the [API reference](https://docs.rs/humphrey_auth) for more information.

### Note for Contributors
If you would like to add a step-by-step guide for Humphrey Auth, please [open an issue](https://github.com/w-henderson/Humphrey/issues/new). Your help would be greatly appreciated!