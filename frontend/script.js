let ws;

document.getElementById("createRoom").addEventListener("click", async () => {
  const name = document.getElementById("playerName").value.trim();
  const duration = parseInt(document.getElementById("roomDuration").value.trim(), 10);
  if (name === "" || isNaN(duration) || duration <= 0) return;
  try {
    const res = await fetch("/create_room", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ duration })
    });
    const data = await res.json();
    const roomId = data.room_id;
    const invitationLink = data.invitation_link;
    document.getElementById("roomInfo").textContent = 
      "Room created: " + roomId + ". Invitation link: " + invitationLink;
    startGame(name, roomId, invitationLink);
  } catch (error) {
    console.error("Error creating room:", error);
  }
});

document.getElementById("joinRoom").addEventListener("click", () => {
  const name = document.getElementById("playerName").value.trim();
  const roomId = document.getElementById("roomId").value.trim();
  if (name && roomId) {
    startGame(name, roomId, null);
  }
});

function startGame(name, roomId, invitationLink) {
  document.getElementById("register").style.display = "none";
  document.getElementById("game").style.display = "block";
  
  // If host, show invitation info and start button.
  if (invitationLink) {
    document.getElementById("invitation").textContent = "Invitation link: " + invitationLink;
    document.getElementById("startBtn").style.display = "block";
  }
  
  const protocol = location.protocol === "https:" ? "wss:" : "ws:";
  ws = new WebSocket(`${protocol}//${location.host}/ws?room_id=${roomId}&name=${encodeURIComponent(name)}`);

  ws.onopen = () => {
    console.log("Connected to server");
  };

  ws.onmessage = (event) => {
    console.log("Received:", event.data);
    const message = event.data;
    if (message.startsWith("COUNTDOWN: ")) {
      document.getElementById("problem").textContent = message.replace("COUNTDOWN: ", "").trim();
    } else if (message.startsWith("INFO: ")) {
      document.getElementById("problem").textContent = message.replace("INFO: ", "").trim();
    } else if (message.startsWith("START: ")) {
      document.getElementById("problem").textContent = message.replace("START: ", "").trim();
    } else if (message.startsWith("PROBLEM: ")) {
      document.getElementById("problem").textContent = message.replace("PROBLEM: ", "").trim();
    } else if (message.startsWith("PROGRESS: ")) {
      document.getElementById("scoreboard").textContent = message.replace("PROGRESS: ", "").trim();
    } else if (message.startsWith("TIMER: ")) {
      document.getElementById("timer").textContent = "Time: " + message.replace("TIMER: ", "").trim();
    } else if (message.startsWith("FINISH: ")) {
      document.getElementById("feedback").textContent = message.replace("FINISH: ", "").trim();
      ws.close();
    }
  };

  ws.onerror = (error) => {
    console.error("WebSocket error:", error);
  };

  // If host, attach start button handler.
  document.getElementById("startBtn").addEventListener("click", () => {
    ws.send("START");
    document.getElementById("startBtn").style.display = "none";
  });

  document.getElementById("answer").addEventListener("input", () => {
    const answer = document.getElementById("answer").value.trim();
    if (!isNaN(answer) && answer !== "") {
      ws.send(answer);
      document.getElementById("answer").value = "";
    }
  });
}
