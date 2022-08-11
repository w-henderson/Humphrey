# Hot Reload
Humphrey supports hot reload through a first-party plugin, provided that the server was compiled with the `plugins` feature enabled and the plugin is installed.

The Hot Reload plugin is able to automatically reload webpages when the source code changes. It is not recommended for use in production, but is useful for development. It should also be noted that, when using a front-end framework such as React, the framework's built-in HMR (hot module reloading) functionality should be used instead of this plugin.

HTML pages are reloaded by requesting the updated page through a `fetch` call, then writing this to the page. This avoids the need for the page to be reloaded manually. CSS and JavaScript are reloaded by requesting the updated data, then replacing the old script or stylesheet. Images are reloaded in the same way. Other resources are currently unable to be dynamically reloaded.

When JavaScript is reloaded, the updated script will be executed upon load in the same context as the old script. This means that any `const` declarations may cause errors, but this is unavoidable as without executing the new script, none of the changes can be used. For this reason, the Hot Reload plugin is more suitable for design changes than for functionality changes.

**Warning:** Hot Reload disables caching so that changes are immediately visible.

## Configuration
In the plugins section of the configuration file, add the following:

```conf
hot-reload {
  library "path/to/hot-reload.dll" # Path to the compiled library
  ws_route "/ws"                   # Route to the WebSocket endpoint
}
```

Specifying the WebSocket route is optional. If not specified, the default is `/__hot-reload-ws` in order to avoid conflicts with other configured WebSocket endpoints.