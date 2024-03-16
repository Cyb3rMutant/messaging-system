let userId = 0;

const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.tauri;

// testing
function getChat() {
  var button = document.querySelector('input[name="users"]:checked');
  var user = parseInt(button.value);
  button.className = "";
  invoke("switch_chat", { user: user }).then(function (messages) {
    console.log(messages, typeof messages);

    document.getElementById("container").innerHTML = "";
    for (let i = 0; i < messages.length; i++) {
      const element = messages[i];

      displayMessage(element.from_me, element.content, element.status);
    }
  });
}

// messages
function sendMessage() {
  // Get the value from the input
  var messageInput = document.getElementById("message");
  var messageText = messageInput.value;
  var user = parseInt(
    document.querySelector('input[name="users"]:checked').value,
  );

  // Clear the input
  messageInput.value = "";

  displayMessage(true, messageText, 1);

  invoke("send", { user: user, message: messageText });
}

listen("MSG", (message) => {
  console.log(message);
  let from_me = message.payload[1].from_me;
  let content = message.payload[1].content;
  let status = message.payload[1].status;
  let id = message.payload[0];
  var activeChat = parseInt(
    document.querySelector('input[name="users"]:checked').value,
  );

  if (id == activeChat) {
    displayMessage(from_me, content, status);
  } else {
    document.getElementById(id).className += " notification";
  }
});

// messages
function setSeen() {
  var user = parseInt(
    document.querySelector('input[name="users"]:checked').value,
  );
  console.log("setting seen for chat ", user);
  // Get the value from the input
  invoke("read_chat", { user: user });
}

listen("STS", (message) => {
  console.log(message);
  let id = message.payload;
  var activeChat = parseInt(
    document.querySelector('input[name="users"]:checked').value,
  );

  if (id == activeChat) {
    // setSeen();
    getChat();
  }
});

function displayMessage(from_me, content, status) {
  console.log("---", from_me, content);
  var newDiv = document.createElement("div");
  newDiv.className = "dynamic-div";
  if (from_me) {
    newDiv.className += " from-me";
  }
  switch (status) {
    case 1:
      newDiv.style.background = "red";
      break;
    case 2:
      newDiv.style.background = "green";
      break;
    default:
      console.log(status, "huh");
      break;
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
  console.log(arr);

  var friendsList = document.getElementById("friends");
  friendsList.innerHTML = ""; // clear the list

  for (var i = 0; i < arr.length; i += 2) {
    var listItem = document.createElement("li");

    var radioButton = document.createElement("input");
    radioButton.type = "radio";
    radioButton.name = "users";
    radioButton.id = arr[i];
    radioButton.value = arr[i];
    radioButton.addEventListener("change", () => {
      // setSeen();
      getChat();
    });

    var label = document.createElement("label");
    label.htmlFor = radioButton.id;
    label.appendChild(document.createTextNode(arr[i + 1]));

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
  userId = message.payload;
  if (userId) {
    document.getElementById("your-username").innerText = userId;
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
