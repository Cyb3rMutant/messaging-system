let userName = "";

const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.tauri;

// testing
function getChat() {
  var button = document.querySelector('input[name="users"]:checked');
  var user = button.value;
  button.className = "";
  invoke("switch_chat", { user: user }).then(function(messages) {
    console.log(messages, typeof messages);

    document.getElementById("container").innerHTML = "";
    for (let i = 0; i < messages.length; i++) {
      const element = messages[i];

      displayMessage(element.from, element.content);
    }
  });
}

// messages
function sendMessage() {
  // Get the value from the input
  var messageInput = document.getElementById("message");
  var messageText = messageInput.value;
  var user = document.querySelector('input[name="users"]:checked').value;

  // Clear the input
  messageInput.value = "";

  displayMessage(userName, messageText);

  invoke("send", { user: user, message: messageText });
}

listen("MSG", (message) => {
  let from = message.payload.from;
  let content = message.payload.content;
  var activeChat = document.querySelector('input[name="users"]:checked').value;

  if (from == activeChat) {
    displayMessage(from, content);
  } else {
    document.getElementById(from).className += " notification";
  }
});

function displayMessage(from, content) {
  console.log("---", from, content);
  var newDiv = document.createElement("div");
  newDiv.className = "dynamic-div";
  if (from == userName) {
    newDiv.className += " from-me";
  }
  newDiv.textContent = content;

  // Append the new div to the container
  document.getElementById("container").appendChild(newDiv);

  // Scroll to the bottom to keep the new div in view
  document.getElementById("container").scrollTop =
    document.getElementById("container").scrollHeight;
}

// get users
function getUsers() {
  invoke("getusers");
}

listen("USR", (message) => {
  var arr = message.payload;

  var friendsList = document.getElementById("friends");
  friendsList.innerHTML = ""; // clear the list

  for (var i = 0; i < arr.length; i++) {
    var listItem = document.createElement("li");

    var radioButton = document.createElement("input");
    radioButton.type = "radio";
    radioButton.name = "users";
    radioButton.id = arr[i];
    radioButton.value = arr[i];
    radioButton.addEventListener("change", getChat);

    var label = document.createElement("label");
    label.htmlFor = radioButton.id;
    label.appendChild(document.createTextNode(arr[i]));

    listItem.appendChild(radioButton);
    listItem.appendChild(label);

    friendsList.appendChild(listItem);
  }
});

// register
function register() {
  var username = document.getElementById("username").value;
  var password = document.getElementById("password").value;

  invoke("register", { username: username, password: password });
}

listen("REG", (message) => {
  var elem = document.getElementById("register-message");
  if (message.payload == "Y") {
    elem.innerText = "you are registered";
  } else {
    elem.innerText = "user already exists";
  }
});

// login
function login() {
  var username = document.getElementById("username").value;
  var password = document.getElementById("password").value;

  invoke("login", { username: username, password: password });
}

listen("LGN", (message) => {
  userName = message.payload;
  if (userName) {
    document.getElementById("your-username").innerText = userName;
    var div = document.getElementById("no");
    div.style.display = "none";
    var div = document.getElementById("yes");
    div.style.display = "block";
  }
});

listen("ERR", (message) => {
  var err = message.payload;

  var elem = document.getElementById("register-message");
  if (err == "PWD") {
    elem.innerText = "wrong password";
  } else {
    elem.innerText = "user does not exist";
  }
});

listen("OTH", (message) => {
  console.log(message);
});
