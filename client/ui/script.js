const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.tauri;

listen("received", (message) => {
  window.header.innerHTML = message.payload;

  var newDiv = document.createElement("div");
  newDiv.className = "dynamic-div";
  newDiv.textContent = message.payload;

  // Append the new div to the container
  document.getElementById("container").appendChild(newDiv);

  // Scroll to the bottom to keep the new div in view
  document.getElementById("container").scrollTop =
    document.getElementById("container").scrollHeight;
});

function sendMessage() {
  // Get the value from the input
  var messageText = document.getElementById("message").value;

  // Clear the input
  messageText.value = "";

  console.log(messageText, typeof messageText);
  invoke("send", { message: messageText });
}
