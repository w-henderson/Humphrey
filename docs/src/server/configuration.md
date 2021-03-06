# Configuration
Humphrey's configuration format is similar to that of Nginx. Comments begin with a `#` and are ignored by the parser. Separate configuration files can be included with the `include` directive, like in Nginx.

## Locating the Configuration
Humphrey looks in three places for the configuration, before falling back to the default. If no configuration file can be found, the server will log a warning and start using the default configuration.

1. The path specified as a command-line argument, for example when running `humphrey /path/to/config.conf`.
2. The `humphrey.conf` file in the current directory.
3. The path specified in the `HUMPHREY_CONF` environment variable.

It is important to note that if a file is found at any of these locations but is invalid, the server will log an error and exit instead of continuing to the next location.

## Example
An example configuration file with all the supported directives specified is shown below.

```conf
server {
  address   "0.0.0.0"        # Address to host the server on
  port      443              # Port to host the server on
  threads   32               # Number of threads to use for the server
  timeout   5                # Timeout for requests, highly recommended to avoid deadlocking the thread pool

  plugins { # Plugin configuration (only supported with the `plugins` feature)
    include "php.conf"       # Include PHP configuration (see next page)
  }

  tls { # TLS configuration (only supported with the `tls` feature)
    cert_file "cert.pem"     # Path to the TLS certificate
    key_file  "key.pem"      # Path to the TLS key
    force     true           # Whether to force HTTPS on all requests
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