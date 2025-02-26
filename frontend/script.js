// Connect to the WebSocket server.
const ws = new WebSocket('ws://127.0.0.1:3030/ws');

ws.onopen = () => {
  console.log("Connected to the server.");
};

ws.onmessage = (event) => {
  console.log("Message received: " + event.data);
  const message = event.data;

  // If the message contains a problem (either initial or new problem)
  if (message.startsWith("PROBLEM: ")) {
    document.getElementById("problem").textContent = message.replace("PROBLEM: ", "");
    document.getElementById("feedback").textContent = "";
  } else if (message.startsWith("CORRECT! New PROBLEM: ")) {
    document.getElementById("problem").textContent = message.replace("CORRECT! New PROBLEM: ", "");
    document.getElementById("feedback").textContent = "Correct!";
  } else {
    // Display any other messages as feedback.
    document.getElementById("feedback").textContent = message;
  }
};

ws.onerror = (error) => {
  console.error("WebSocket error: ", error);
};

// Handle answer input
const answerInput = document.getElementById("answer");
answerInput.addEventListener("keypress", (e) => {
  if (e.key === "Enter") {
    const answer = answerInput.value;
    ws.send(answer);
    answerInput.value = ""; // Clear input after sending.
  }
});
