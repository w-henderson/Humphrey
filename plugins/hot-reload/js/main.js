if (typeof __HUMPHREY_INIT === "undefined" || __HUMPHREY_INIT !== true) {
  var __HUMPHREY_INIT = false;

  var __HUMPHREY_WS_ADDR = window.location.protocol === "https:" ? "wss" : "ws"
    + `://${window.location.host}`
    + __HUMPHREY_WS_ROUTE;

  var __HUMPHREY_LAST_UPDATES = {};

  const __HUMPHREY_WS = new WebSocket(__HUMPHREY_WS_ADDR);

  __HUMPHREY_WS.onopen = () => {
    __HUMPHREY_INIT = true;
    console.log("[Humphrey Hot Reload] Connected to server, waiting for changes");
  }

  __HUMPHREY_WS.onmessage = async (e) => {
    const data = e.data;
    const url = window.location.pathname;

    // If the last update happened too recently, ignore it
    // I'm not sure why this happens but it might be due to applications using multiple writes to save a file
    if (__HUMPHREY_LAST_UPDATES[data] && __HUMPHREY_LAST_UPDATES[data] > new Date().getTime() - 200) {
      return;
    }

    __HUMPHREY_LAST_UPDATES[data] = new Date().getTime();

    // If the current page was changed, reload it.
    if (data === url
      || (url.endsWith('/') && data === url + "index.html")
      || (url.endsWith('/') && data === url + "index.htm")) {
      console.log("[Humphrey Hot Reload] Reloading page");

      return await reloadPage();
    }

    // Update any `src` attributes that point to the changed URL.
    const srcElements = Array.from(document.querySelectorAll("[src]"));
    const sources = srcElements.map(e => e.src);
    const indexes = sources.reduce((previous, current, index) => {
      if (current === data) return [...previous, index];
      return previous;
    }, []);

    for (let index of indexes) {
      const source = sources[index];
      const srcUrl = new URL(source.src);
      srcUrl.hash = new Date().getTime();
      source.src = srcUrl;
    }

    // Update any CSS `<link>` tags that point to the changed URL.
    const cssElements = Array.from(document.querySelectorAll("link[href]"));
    const cssSources = cssElements.map(e => e.href);
    const cssIndexes = cssSources.reduce((previous, current, index) => {
      if (current === data) return [...previous, index];
      return previous;
    }, []);

    for (let index of cssIndexes) {
      const source = cssSources[index];
      const srcUrl = new URL(source.href);
      srcUrl.hash = new Date().getTime();
      source.href = srcUrl;
    }
  };
}

async function reloadPage() {
  return fetch(window.location.href)
    .then(res => res.text())
    .then(text => {
      document.open();
      document.write(text);
      document.close();
    });
}