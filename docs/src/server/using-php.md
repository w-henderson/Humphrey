# Using PHP
Humphrey supports PHP over the FastCGI protocol, provided that it was compiled with the `plugins` feature enabled and the PHP plugin is installed. You'll also need PHP-CGI or PHP-FPM installed and running to allow Humphrey to connect to the PHP interpreter.

## Configuration
In the previous configuration example, we used included a file called `php.conf` into the configuration. You'll need to create this file with the following contents:

```conf
php {
  library "path/to/php.dll" # Path to the compiled library
  address "127.0.0.1"       # Address of the interpreter
  port    9000              # Port of the interpreter
  threads 8                 # Threads to use (see below)
}
```

## Multi-Threading
The PHP plugin supports multi-threading to improve performance, but this requires some tweaks to the PHP FastCGI server configuration. PHP is by default single-threaded, so you'll need to increase the PHP threads to match the number you specify in your `php.conf` file.