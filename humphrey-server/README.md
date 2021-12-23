<p align="center">
  <img src="https://raw.githubusercontent.com/w-henderson/Humphrey/master/assets/logo.png" width=250><br><br>
  <img src="https://img.shields.io/badge/language-rust-b07858?style=for-the-badge&logo=rust" style="margin-right:5px">
  <img src="https://img.shields.io/github/workflow/status/w-henderson/Humphrey/CI?style=for-the-badge" style="margin-right:5px">
  <img src="https://img.shields.io/crates/v/humphrey-server?style=for-the-badge" style="margin-right:5px">
</p>

# Humphrey: A Performance-Focused, Dependency-Free Web Server.
Humphrey is a very fast, robust and flexible HTTP/1.1 web server, with support for static and dynamic content through its plugin system. It has no dependencies when only using default features, and is easily extensible with a configuration file and dynamically-loaded plugins.

## Installation
To install the binary, run `cargo install humphrey_server` and it will be automatically downloaded, compiled and added to your path as `humphrey`. Alternatively, you can find precompiled binaries from the [latest GitHub release](https://github.com/w-henderson/Humphrey/releases).

## Configuration
The Humphrey executable is run with a maximum of one argument, which specifies the path to the configuration file (defaulting to `humphrey.conf` in the current directory). The configuration file is where all configuration for Humphrey and any plugins is stored. The syntax is similar to Nginx, with comments starting with a `#`. Other configuration files can be included with the `include` directive, like in Nginx. Below is an example of a configuration file with every supported field specified. Unless specified otherwise, all fields are optional.

```conf
server {
  address   "0.0.0.0"        # Address to host the server on
  port      80               # Port to host the server on
  threads   32               # Number of threads to use for the server

  plugins {
    include "php.conf"       # Include PHP configuration (see below)
  }

  blacklist {
    file "conf/blacklist.txt" # Text file containing blacklisted addresses, one per line
    mode "block"              # Method of enforcing the blacklist, "block" or "forbidden" (which returns 403 Forbidden)
  }

  log {
    level   "info"         # Log level, from most logging to least logging: "debug", "info", "warn", "error"
    console true           # Whether to log to the console
    file    "humphrey.log" # Filename to log to
  }

  cache {
    size 128M # Size limit of the cache
    time 60   # Max time to cache files for, in seconds
  }

  host "127.0.0.1" { # Configuration for connecting through the host 127.0.0.1
    route /* {
      redirect "http://localhost/" # Redirect to localhost
    }
  }

  route /ws {
    websocket "localhost:1234" # Address to connect to for WebSocket connections
  }

  route /proxy/* {
    proxy              "127.0.0.1:8000,127.0.0.1:8080" # Comma-separated proxy targets
    load_balancer_mode "round-robin"                   # Load balancing mode, either "round-robin" or "random"
  }

  route /static/*, /images/* {
    directory "/var/static" # Serve content from this directory to both paths
  }

  route /logo.png {
    file "/var/static/logo_256x256.png" # Serve this file to this route
  }

  route /home {
    redirect "/" # Redirect this route with 302 Moved Permanently
  }

  route /* {
    directory "/var/www" # Serve content from this directory
  }
}
```

## Using with PHP
To use Humphrey with PHP, compile the [PHP plugin in the plugins folder](https://github.com/w-henderson/Humphrey/tree/master/plugins/php) and add the path to the output file to your plugin configuration (also available precompiled in the GitHub releases). You'll need Humphrey installed with plugins enabled (using `cargo install humphrey_server --features plugins`) and you'll also need PHP-CGI or PHP-FPM. Start the PHP server first, and specify its address in the Humphrey configuration file as specified below. Ensure your PHP configuration allows for multithreading if you set more than one thread in the configuration. Finally, you can start Humphrey in the normal way and it will work with PHP.

In the `php.conf` file which is included in the main configuration through the `include` directive:
```conf
php {
  library "plugins/php/target/release/php.dll" # Path to compiled library, `.dll` on Windows and `.so` on Linux
  address "127.0.0.1"                          # Address to connect to the PHP-CGI interpreter
  port    9000                                 # Port of the interpreter
  threads 8                                    # Number of threads to connect to the interpreter with
}
```

**Note:** by default, the PHP interpreter is single-threaded, so don't increase the threads in the configuration unless you also change the PHP-CGI configuration.

## Creating a Plugin
To create a plugin, take a look at the [plugin example](https://github.com/w-henderson/Humphrey/tree/master/examples/plugin). In short, you need to create a library crate with type `cdylib` to compile to a DLL, then implement the `humphrey_server::plugins::plugin::Plugin` trait for a struct and declare it with the `declare_plugin!` macro.