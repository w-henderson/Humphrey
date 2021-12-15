function getAndResetInputs() {
  let username = document.getElementById("username").value;
  let password = document.getElementById("password").value;

  document.getElementById("username").value = "";
  document.getElementById("password").value = "";
  document.getElementById("alert").innerHTML = "Loading...";

  return {
    username,
    password
  };
}

function signIn() {
  let { username, password } = getAndResetInputs();

  fetch("/api/login", {
    method: "POST",
    body: `${username},${password}`,
  }).then(res => res.text()).then(res => {
    if (res === "OK") {
      window.location.reload();
    } else {
      document.getElementById("alert").innerHTML = res;
    }
  });
}

function signUp() {
  let { username, password } = getAndResetInputs();

  fetch("/api/signup", {
    method: "POST",
    body: `${username},${password}`,
  }).then(res => res.text()).then(res => {
    if (res === "OK") {
      document.getElementById("alert").innerHTML = "Signed up successfully, please sign in";
    } else {
      document.getElementById("alert").innerHTML = "Username already exists";
    }
  })
}

function signOut() {
  window.location = "/api/signout";
}

function deleteAccount() {
  window.location = "/api/deleteAccount";
}