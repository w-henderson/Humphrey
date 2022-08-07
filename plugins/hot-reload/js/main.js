const addr = window.location.protocol === "https:" ? "wss" : "ws"
  + `://${window.location.host}`
  + __HUMPHREY_WS_ROUTE;

const ws = new WebSocket(addr);
ws.onmessage = e => console.log(e.data);