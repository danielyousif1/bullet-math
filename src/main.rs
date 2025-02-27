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
        let a: i32 = rng.gen_range(1..5);
        let b: i32 = rng.gen_range(0..5);
        // Randomly pick an operator: 0 for addition, 1 for subtraction, 2 for multiplication.
        let op = rng.gen_range(0..3);
        match op {
            0 => MathProblem {
                problem: format!("{} + {}", a, b),
                answer: a + b,
            },
            1 => {
                // Ensure subtraction results in a non-negative answer.
                let (minuend, subtrahend) = if a >= b { (a, b) } else { (b, a) };
                MathProblem {
                    problem: format!("{} - {}", minuend, subtrahend),
                    answer: minuend - subtrahend,
                }
            },
            _ => MathProblem {
                problem: format!("{} X {}", a, b),
                answer: a * b,
            },
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

    // Shared state filter.
    let state_filter = warp::any().map({
        let game_state = game_state.clone();
        move || game_state.clone()
    });

    // WebSocket route.
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter.clone())
        .map(|ws: warp::ws::Ws, game_state| {
            ws.on_upgrade(move |socket| client_connection(socket, game_state))
        });

    // Serve frontend static files.
    // This serves index.html when someone visits the root path.
    let static_files = warp::fs::dir("frontend");

    // Combine the routes.
    let routes = ws_route.or(static_files);

    println!("Server running on http://127.0.0.1:3030");
    // warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    // get env var PORT and made it available in all interfaces
    let port: u16 = std::env::var("PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(3030);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn client_connection(ws: warp::ws::WebSocket, game_state: Arc<Mutex<GameState>>) {
    let (ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));

    // Subscribe to broadcast channel.
    let mut broadcast_rx = {
        let state = game_state.lock().await;
        state.broadcaster.subscribe()
    };

    // Forward broadcast messages to the client.
    let tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let mut tx = tx_clone.lock().await;
            if tx.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Send the current problem when the client connects.
    {
        let state = game_state.lock().await;
        let init_msg = format!("PROBLEM: {}", state.current_problem.problem);
        let mut tx = ws_tx.lock().await;
        if tx.send(warp::ws::Message::text(init_msg)).await.is_err() {
            return;
        }
    }

    // Process incoming messages.
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_str().unwrap_or("");
                    if let Ok(answer) = text.trim().parse::<i32>() {
                        let mut state = game_state.lock().await;
                        if answer == state.current_problem.answer {
                            // Correct answer: generate and broadcast a new problem.
                            state.current_problem = MathProblem::generate();
                            let broadcast_msg = format!(
                                "CORRECT! New PROBLEM: {}",
                                state.current_problem.problem
                            );
                            let _ = state.broadcaster.send(broadcast_msg);
                        } else {
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
