function send() {
  let input = document.querySelector("input");
  let message = input.value;

  if (message.length > 0) {
    input.value = "Posting...";
    input.disabled = true;

    fetch("/api/postMessage", {
      method: "POST",
      body: message
    }).then(() => {
      window.location.reload(true);
    });
  }
}

window.addEventListener("keypress", e => {
  if (e.key === "Enter") {
    send();
  }
});