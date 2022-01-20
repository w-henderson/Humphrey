# Getting Started

## Installation
You can find the latest binaries at the [Releases page](https://github.com/w-henderson/Humphrey/releases) on GitHub. If you want to use plugins, ensure you download the version which supports them. It is also advisable to add the executable to your `PATH`, so you can run `humphrey` from anywhere. This is automatically done if you install via `cargo`, as outlined below.

## Building from Source
To download and build the server, run the following command:

```sh
$ cargo install humphrey_server
```

If you are going to use plugins with the server, including the PHP plugin, you'll need to compile in plugin support, which can be done with the argument `--features plugins`. If you want to serve content over HTTPS, you'll need to compile in TLS support, which can be done with the argument `--features tls`. The following command automatically does both:

```sh
$ cargo install humphrey_server --all-features
```

## Running the Server
Once Humphrey Server is installed, you can simply run `humphrey` anywhere to serve the content of the current working directory. It has only one optional argument, which is the path to its configuration file, and this defaults to `humphrey.conf`.

You'll see a warning that no configuration file was found. In the next section, [Configuration](configuration.md), we'll learn how to use Humphrey's advanced configuration format to configure the server.