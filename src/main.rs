use std::collections::HashMap;
use std::sync::Arc;
use warp::Filter;
use futures_util::{StreamExt, SinkExt};
use rand::Rng;
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
struct GameState {
    // (For simplicity, player tracking is left as a placeholder)
    players: HashMap<usize, Player>,
    current_problem: MathProblem,
    // A broadcast channel to send messages (e.g., new problems) to all clients.
    broadcaster: broadcast::Sender<String>,
}

#[derive(Clone)]
struct Player {
    name: String,
    score: u32,
}

#[derive(Clone)]
struct MathProblem {
    problem: String,
    answer: i32,
}

impl MathProblem {
    fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let a: i32 = rng.gen_range(1..50);
        let b: i32 = rng.gen_range(0..50);
        MathProblem {
            problem: format!("{} X {}", a, b),
            answer: a * b,
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel with a capacity of 16 messages.
    let (tx, _) = broadcast::channel(16);
    let game_state = Arc::new(Mutex::new(GameState {
        players: HashMap::new(),
        current_problem: MathProblem::generate(),
        broadcaster: tx,
    }));

    // Define a filter to inject our shared state.
    let state_filter = warp::any().map({
        let game_state = game_state.clone();
        move || game_state.clone()
    });

    // Set up a route at "/ws" to upgrade HTTP requests to WebSocket connections.
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter)
        .map(|ws: warp::ws::Ws, game_state| {
            ws.on_upgrade(move |socket| client_connection(socket, game_state))
        });

    println!("Server running on ws://127.0.0.1:3030/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn client_connection(ws: warp::ws::WebSocket, game_state: Arc<Mutex<GameState>>) {
    // Split the WebSocket into a sender and receiver.
    let (ws_tx, mut ws_rx) = ws.split();
    // Wrap the sender in an Arc<Mutex<...>> to share it between tasks.
    let ws_tx = Arc::new(Mutex::new(ws_tx));

    // Each client subscribes to the broadcast channel to receive updates.
    let mut broadcast_rx = {
        let state = game_state.lock().await;
        state.broadcaster.subscribe()
    };

    // Spawn a task to forward broadcast messages to the connected client.
    let tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let mut tx = tx_clone.lock().await;
            if tx.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // When the client connects, send the current math problem.
    {
        let state = game_state.lock().await;
        let init_msg = format!("PROBLEM: {}", state.current_problem.problem);
        let mut tx = ws_tx.lock().await;
        if tx.send(warp::ws::Message::text(init_msg)).await.is_err() {
            return;
        }
    }

    // Process incoming messages from the client.
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_str().unwrap_or("");
                    // Expect the message to be the answer (e.g., "7")
                    if let Ok(answer) = text.trim().parse::<i32>() {
                        let mut state = game_state.lock().await;
                        if answer == state.current_problem.answer {
                            // If correct, generate a new problem.
                            state.current_problem = MathProblem::generate();
                            let broadcast_msg = format!(
                                "CORRECT! New PROBLEM: {}",
                                state.current_problem.problem
                            );
                            // Broadcast the new problem to all clients.
                            let _ = state.broadcaster.send(broadcast_msg);
                        } else {
                            // Optionally, send feedback only to the sender.
                            let mut tx = ws_tx.lock().await;
                            let _ = tx
                                .send(warp::ws::Message::text("Incorrect, try again.".to_string()))
                                .await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}
