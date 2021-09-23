<p align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=250><br><br>
  <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
  <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey?style=for-the-badge" style="margin-right:5px">
</p>

# Humphrey: A Performance-Focused, Dependency-Free Web Server.
Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its plugin system. It has no dependencies when only using default features, and is easily extensible with a configuration file and dynamically-loaded plugins.

## Installation
To install the binary, run `cargo install humphrey_server` and it will be automatically downloaded, compiled and added to your path as `humphrey`. Alternatively, you can find precompiled binaries from the [latest GitHub release](https://github.com/w-henderson/Humphrey/releases).

## Configuration
The Humphrey executable is run with a maximum of one argument, which specifies the path to the configuration file (defaulting to `humphrey.ini` in the current directory). The configuration file is where all configuration for Humphrey and any plugins is stored. It is a basic INI file with support for comments and sections. Below is an example of a configuration file with every supported field specified (note that since the mode is set to static, the proxy and load balancer configuration options would be ignored). Unless specified otherwise, all fields are optional.

```ini
; Humphrey Configuration File

[server]
address = "0.0.0.0" ; The address to bind the server to
port = 80           ; The port to bind the server to
threads = 32        ; The number of threads to use, must be at least one
mode = "static"     ; The server mode, usually "static" serving files from a directory, but can be "proxy" or "load_balancer", required.

[log]
level = "warn"          ; The log level, from least logging to most logging: "error", "warn", "info", "debug"
console = false         ; Whether to log to the console, defaults to true
file = "humphrey.log"   ; A file to log to, if not specified Humphrey will not log to a file

[blacklist]
file = "conf/blacklist.txt"  ; A text file containing one IP to block on each line
mode = "block"               ; The method of blacklisting, either "block" to block connections or "forbidden" to return 403 Forbidden

[static]
directory = "/var/www"        ; The directory to serve files from, defaults to directory the executable was run in if unset
cache = 16M                   ; Maximum cache size, cache disabled if not specified
cache_time = 60               ; Maximium time to cache content for in seconds, defaults to 60 seconds if not specified
plugins = "conf/plugins.txt"  ; A text file containing one plugin library file path on each line

[proxy]
target = "localhost:8000"  ; The address to proxy traffic to, required if the mode is set to proxy

[load_balancer]
targets = "conf/targets.txt"  ; A text file containing one target on each line to balance traffic between, required if the mode is set to load_balancer
mode = "round-robin"          ; The algorithm for load balancing, either "round-robin" (default) or "random"
```

## Using with PHP
To use Humphrey with PHP, compile the [PHP plugin in the plugins folder](https://github.com/w-henderson/Humphrey/tree/master/plugins/php) and add the path to the output file to your plugins list as specified in the static configuration (also available precompiled in the GitHub releases). You'll need Humphrey installed with plugins enabled (using `cargo install humphrey_server --features plugins`) and you'll also need PHP-CGI or PHP-FPM. Start the PHP server first, and specify its address in the Humphrey configuration file as specified below. Ensure your PHP configuration allows for multithreading if you set more than one thread in the configuration. Finally, you can start Humphrey in the normal way and it will work with PHP.

```ini
; Additional PHP configuration
[php]
address = "127.0.0.1"
port = 9000
threads = 8
```

## Creating a Plugin
To create a plugin, take a look at the [plugin example](https://github.com/w-henderson/Humphrey/tree/master/examples/plugin). In short, you need to create a library crate with type `cdylib` to compile to a DLL, then implement the `humphrey_server::plugins::plugin::Plugin` trait for a struct and declare it with the `declare_plugin!` macro.