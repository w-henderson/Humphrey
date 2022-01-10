const ws = new WebSocket(`ws://${window.location.hostname}/ws`);

function write(message) {
  document.querySelector("div").innerText += `[server]  ${message}`;
  document.querySelector("div").innerHTML += "<br>";
}

ws.onmessage = (e) => write(e.data);
ws.onopen = () => write("<connected>");
ws.onclose = () => write("<disconnected>");
ws.onerror = () => write("<error>");

function sendMessage() {
  let data = document.querySelector("#message").value;
  document.querySelector("div").innerText += `[client]  ${data}`;
  document.querySelector("div").innerHTML += "<br>";
  ws.send(data);
  document.querySelector("#message").value = "";
}

document.onkeydown = (e) => {
  if (e.key === "Enter" && document.querySelector("#message").value !== "") {
    sendMessage();
  }
}