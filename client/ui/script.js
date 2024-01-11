const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.tauri;

// messages
function sendMessage() {
  // Get the value from the input
  var messageInput = document.getElementById("message");
  var messageText = messageInput.value;
  var user = document.querySelector('input[name="users"]:checked').value;

  // Clear the input
  messageInput.value = "";

  console.log(messageText, user);
  invoke("send", { user: user, message: messageText });
}

listen("MSG", (message) => {
  window.header.innerHTML = message.payload;

  console.log(message);
  var newDiv = document.createElement("div");
  newDiv.className = "dynamic-div";
  newDiv.textContent = message.payload;

  // Append the new div to the container
  document.getElementById("container").appendChild(newDiv);

  // Scroll to the bottom to keep the new div in view
  document.getElementById("container").scrollTop =
    document.getElementById("container").scrollHeight;
});

// get users
function getUsers() {
  console.log("getting users");

  invoke("getusers");
}

listen("USR", (message) => {
  var arr = message.payload;

  console.log(message, typeof message.payload);

  var div = document.getElementById("radioDiv");
  div.innerHTML = ""; // clear the div

  for (var i = 0; i < arr.length; i++) {
    var radioButton = document.createElement("input");
    radioButton.type = "radio";
    radioButton.name = "users";
    radioButton.id = "radio" + i;
    radioButton.value = arr[i];

    var label = document.createElement("label");
    label.htmlFor = radioButton.id;
    label.appendChild(document.createTextNode(arr[i]));

    div.appendChild(radioButton);
    div.appendChild(label);
  }
});

// login
function login() {
  var username = document.getElementById("username").value;
  var password = document.getElementById("password").value;

  console.log("username:", username, typeof username);
  console.log("password:", password, typeof password);

  invoke("login", { username: username, password: password });
}

listen("LGN", (message) => {
  var logged = message.payload;

  console.log(message);
  if (logged) {
    var div = document.getElementById("no");
    div.style.display = "none";
    var div = document.getElementById("yes");
    div.style.display = "block";
  }
});

listen("ERR", (message) => {
  var logged = message.payload;

  console.log(message);
  if (logged) {
    var div = document.getElementById("no");
    div.style.display = "none";
    var div = document.getElementById("yes");
    div.style.display = "block";
  }
});

listen("OTH", (message) => {
  console.log(message);
});
