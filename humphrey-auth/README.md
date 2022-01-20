<div align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=150>

  <h3 align="center">Humphrey Auth</h3>

  <p align="center">
    A simple authentication system which integrates with Humphrey.<br>
    <a href="https://humphrey.whenderson.dev/auth/index.html">Guide</a> ·
    <a href="https://docs.rs/humphrey-auth">API Reference</a><br><br>
  </p>
</div>

<hr><br>

Web applications commonly need a way of authenticating users. This crate provides an easy and secure way to do this, integrating with Humphrey using the `AuthApp` trait and allowing complete control over the database users are stored in. Humphrey Auth does not come with a database, but the `AuthDatabase` trait is implemented for `Vec<User>` to get started. For a production use, you should use a proper database and implement the `AuthDatabase` trait for it.

Learn more about Humphrey Auth [here](https://humphrey.whenderson.dev/auth/index.html).