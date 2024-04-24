let userId = 0;
let search_arr = [];
let search_idx = 0;
let private = 0;

const { listen, emit } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.tauri;

function getActiveChat() {
  let c = document.querySelector('input[name="friends"]:checked');
  if (c == null) {
    return -1;
  }
  return parseInt(c.value);
}

// testing
function getChat() {
  var button = document.querySelector('input[name="friends"]:checked');
  var user = parseInt(button.value);
  button.className = "";
  invoke("switch_chat", { user: user }).then(function (messages) {
    console.log(messages, typeof messages);

    document.getElementById("container").innerHTML = "";
    for (let i = 0; i < messages.length; i++) {
      const element = messages[i];

      displayMessage(
        element.message_id,
        element.from_me,
        element.content,
        element.status,
      );
    }
  });
}

// messages
function sendMessage() {
  // Get the value from the input
  var messageInput = document.getElementById("message");
  var messageText = messageInput.value;
  var user = getActiveChat();
  console.log(user);
  if (user < 0) {
    return;
  }
  // Clear the input
  messageInput.value = "";

  // displayMessage(0, true, messageText, 1);

  invoke("send", { user: user, message: messageText });
}

listen("MSG", (message) => {
  console.log(message);
  let message_id = message.payload[1].message_id;
  let from_me = message.payload[1].from_me;
  let content = message.payload[1].content;
  let status = message.payload[1].status;
  let id = message.payload[0];
  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  if (id == activeChat) {
    setSeen();
    displayMessage(message_id, from_me, content, 2);
  } else {
    document.getElementById(id).className += " notification";
  }
});

listen("MID", (message) => {
  console.log(message);
  let message_id = message.payload[1].message_id;
  let from_me = message.payload[1].from_me;
  let content = message.payload[1].content;
  let status = message.payload[1].status;
  let id = message.payload[0];
  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  if (id == activeChat) {
    displayMessage(message_id, from_me, content, status);
  }
});

// messages
function setSeen() {
  var user = getActiveChat();
  console.log("setting seen for chat ", user);
  if (user < 0) {
    return;
  }
  console.log("setting seen for chat ", user);
  // Get the value from the input
  if (private) {
    return;
  }
  invoke("read_chat", { user: user });
}

listen("STS", (message) => {
  console.log(message);
  let id = message.payload;
  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  if (id == activeChat) {
    let ch = document.getElementById("container").children;
    for (let i = ch.length - 1; i >= 0; i--) {
      let element = ch[i];
      console.log(element, element.style, element.style.background);
      if (
        element.classList.contains("from-me") &&
        element.style.background == "green"
      ) {
        break;
      }
      if (element.style.background != "blue") {
        continue;
      }
      element.style.background = "green";
    }
  }
});

function displayMessage(message_id, from_me, content, status) {
  var newDiv = document.createElement("div");
  newDiv.className = "dynamic-div";
  newDiv.id = "m" + message_id;
  if (from_me) {
    newDiv.className += " from-me";
  }
  switch (status) {
    case 1:
      newDiv.style.background = "blue";
      break;
    case 2:
      newDiv.style.background = "green";
      break;
    case 3:
      newDiv.style.background = "red";
      break;
    case 4:
      newDiv.style.background = "purple";
      break;
    default:
      console.log(status, "huh");
      break;
  }

  var messageContent = document.createElement("span");
  messageContent.textContent = content;

  if (from_me && status != 3) {
    var deleteButton = document.createElement("button");
    deleteButton.textContent = "Delete";
    deleteButton.onclick = function () {
      // Call delete function
      deleteMessage(message_id);
    };
    newDiv.appendChild(deleteButton);
  }
  newDiv.appendChild(messageContent);

  // Append the new div to the container
  document.getElementById("container").appendChild(newDiv);

  // Scroll to the bottom to keep the new div in view
  document.getElementById("container").scrollTop =
    document.getElementById("container").scrollHeight;
}

// get users
function getAll() {
  invoke("get_all");
}
function getFriends() {
  invoke("get_friends");
}
function getBlocked() {
  invoke("get_blocked");
}

listen("FRD", (message) => {
  var arr = message.payload;
  console.log(arr);

  var friendsList = document.getElementById("friends");
  if (arr.length < 2) {
    friendsList.innerHTML = "lonely lol";
    return;
  }
  friendsList.innerHTML = ""; // clear the list

  for (var i = 0; i < arr.length; i += 2) {
    var listItem = document.createElement("li");

    var radioButton = document.createElement("input");
    radioButton.type = "radio";
    radioButton.name = "friends";
    radioButton.id = arr[i];
    radioButton.value = arr[i];
    radioButton.addEventListener("change", () => {
      setSeen();
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
listen("BKS", (message) => {
  var arr = message.payload;
  console.log(arr);

  var friendsList = document.getElementById("blocked");
  for (var i = 0; i < arr.length; i += 2) {
    console.log(arr[i], document.getElementById(arr[i]));
    (function (index) {
      // Create a new scope
      var listItem = document.createElement("li");

      var button = document.createElement("button");
      button.id = "ub" + arr[index];
      button.value = arr[index + 1];
      button.textContent = "unblock";
      button.onclick = function () {
        console.log(arr[index]); // Use the captured index
        unblock(parseInt(arr[index]));
      };
      var label = document.createElement("label");
      label.htmlFor = button.id;
      label.appendChild(document.createTextNode(arr[index + 1]));

      listItem.appendChild(button);
      listItem.appendChild(label);

      friendsList.appendChild(listItem);
    })(i); // Pass the current value of i to the IIFE
  }
});
listen("ALL", (message) => {
  var arr = message.payload;
  console.log(arr);

  var friendsList = document.getElementById("all");
  for (var i = 0; i < arr.length; i += 2) {
    console.log(arr[i], document.getElementById(arr[i]));
    (function (index) {
      // Create a new scope
      var listItem = document.createElement("li");

      var friend_button = document.createElement("button");
      friend_button.id = "u" + arr[index];
      friend_button.value = arr[index + 1];
      friend_button.textContent = "add friend";
      friend_button.onclick = function () {
        console.log(arr[index]); // Use the captured index
        connect(parseInt(arr[index]));
      };
      listItem.appendChild(friend_button);
      var block_button = document.createElement("button");
      block_button.id = "b" + arr[index];
      block_button.value = arr[index + 1];
      block_button.textContent = "block";
      block_button.onclick = function () {
        console.log(arr[index]); // Use the captured index
        block(parseInt(arr[index]));
      };
      listItem.appendChild(block_button);
      var label = document.createElement("label");
      // label.htmlFor = button.id;
      label.appendChild(document.createTextNode(arr[index + 1]));

      listItem.appendChild(label);

      friendsList.appendChild(listItem);
    })(i); // Pass the current value of i to the IIFE
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
    getFriends();
    getAll();
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

function deleteMessage(message_id) {
  let elem = document.getElementById("m" + message_id);
  elem.style.background = "red";
  elem.getElementsByTagName("span")[0].textContent = "";
  elem.getElementsByTagName("button")[0].remove();

  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  invoke("delete", { user: activeChat, messageId: message_id });
}

listen("DEL", (message) => {
  console.log(message);
  let message_id = message.payload[1];
  let id = message.payload[0];
  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  if (id == activeChat) {
    let elem = document.getElementById("m" + message_id);
    elem.style.background = "red";
    elem.getElementsByTagName("span")[0].textContent = "";
  }
});

function editMessage() {
  var messageInput = document.getElementById("message");
  var messageText = messageInput.value;
  var user = getActiveChat();
  console.log(user);
  if (user < 0) {
    return;
  }
  // Clear the input
  messageInput.value = "";

  let ch = document.getElementById("container").children;
  for (let i = ch.length - 1; i >= 0; i--) {
    let element = ch[i];
    if (element.classList.contains("from-me")) {
      var elem = element;
      break;
    }
  }

  elem.style.background = "purple";
  elem.getElementsByTagName("span")[0].textContent = messageText;

  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  invoke("update", {
    user: activeChat,
    messageId: parseInt(parseInt(elem.id.substring(1))),
    content: messageText,
  });
}

listen("UPD", (message) => {
  console.log(message);
  let content = message.payload[2];
  let message_id = message.payload[1];
  let id = message.payload[0];
  var activeChat = getActiveChat();
  console.log(activeChat);
  if (activeChat < 0) {
    return;
  }
  if (id == activeChat) {
    let elem = document.getElementById("m" + message_id);
    elem.style.background = "purple";
    elem.getElementsByTagName("span")[0].textContent = content;
  }
});

function connect(id) {
  console.log(id);
  if (private) {
    return;
  }
  invoke("connect", { id: parseInt(id) });
}
function block(id) {
  console.log(id);
  if (private) {
    return;
  }
  let elem = document.getElementById("b" + id);
  emit("BKS", [id, elem.value]);
  elem.parentElement.remove();
  invoke("block", { id: parseInt(id) });
}
function unblock(id) {
  console.log(id);
  if (private) {
    return;
  }
  let elem = document.getElementById("ub" + id);
  emit("ALL", [id, elem.value]);
  elem.parentElement.remove();
  invoke("unblock", { id: parseInt(id) });
}

listen("CNT", (message) => {
  console.log(message);
  let chat_id = message.payload[0];
  let user_id = message.payload[1];
  let a = message.payload[2];

  var listItem = document.createElement("li");
  console.log("u" + user_id);
  let name = document.getElementById("u" + user_id).value;

  var radioButton = document.createElement("input");
  radioButton.type = "radio";
  radioButton.name = "friends";
  radioButton.id = chat_id;
  radioButton.value = chat_id;
  radioButton.addEventListener("change", () => {
    setSeen();
    getChat();
  });

  var label = document.createElement("label");
  label.htmlFor = radioButton.id;
  label.appendChild(document.createTextNode(name));

  listItem.appendChild(radioButton);
  listItem.appendChild(label);

  let l = document.getElementById("friends");
  if (l.innerHTML.indexOf("lonely lol") !== -1) {
    l.innerHTML = "";
  }
  l.appendChild(listItem);
  document.getElementById("u" + user_id).parentElement.remove();
  invoke("send_a", { user: chat_id, a: a });
});
listen("BLK", (message) => {
  console.log(message, message.payload);
  let user_id = message.payload;
  console.log(user_id);
  try {
    document.getElementById("u" + user_id).parentElement.remove();
  } catch (error) {
    console.log(error);
  }
  try {
    document.getElementById(user_id).parentElement.remove();
  } catch (error) {
    console.log(error);
  }
});

function search() {
  // Get the value from the input
  var messageInput = document.getElementById("search");
  var messageText = messageInput.value;
  if (messageText.length == 0) {
    next_search();
    return;
  }
  var user = getActiveChat();
  console.log(user);
  if (user < 0) {
    return;
  }
  // Clear the input
  messageInput.value = "";

  // displayMessage(0, true, messageText, 1);

  invoke("search", { user: user, message: messageText }).then(
    function (messages) {
      console.log(messages, typeof messages);
      search_arr = messages;
      search_idx = 0;
      next_search();
    },
  );
}
function next_search() {
  console.log(search_idx);
  document.getElementById("m" + search_arr[search_idx]).scrollIntoView();
  if (++search_idx >= search_arr.length) {
    search_idx = 0;
  }
}
function set_private() {
  let elem = document.getElementById("private-button");
  private ^= 1;
  if (private) {
    elem.textContent = "private on";
    elem.style.background = "red";
  } else {
    elem.style.background = "green";
    elem.textContent = "private off";
  }
}
