// Connect to the WebSocket server.
// const ws = new WebSocket(`ws://${location.host}/ws`);

const protocol = location.protocol === "https:" ? "wss:" : "ws:";
const ws = new WebSocket(`${protocol}//${location.host}/ws`);

ws.onopen = () => {
  console.log("Connected to the server.");
};

ws.onmessage = (event) => {
  console.log("Message received: " + event.data);
  const message = event.data;

  if (message.startsWith("PROBLEM: ")) {
    document.getElementById("problem").textContent = message.replace("PROBLEM: ", "");
    document.getElementById("feedback").textContent = "";
  } else if (message.startsWith("CORRECT! New PROBLEM: ")) {
    document.getElementById("problem").textContent = message.replace("CORRECT! New PROBLEM: ", "");
    document.getElementById("feedback").textContent = "Correct!";
    answerInput.value = ""; // Clear input after a correct answer
  } else {
    document.getElementById("feedback").textContent = message;
  }
};

ws.onerror = (error) => {
  console.error("WebSocket error: ", error);
};

const answerInput = document.getElementById("answer");
answerInput.addEventListener("input", () => {
  const answer = answerInput.value.trim();
  if (!isNaN(answer) && answer !== "") {
    ws.send(answer);
  }
});
