use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use rand::distributions::Alphanumeric;
use warp::Filter;
use tokio::sync::{broadcast, Mutex};
use serde::Deserialize;

#[derive(Clone)]
struct Player {
    name: String,
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
        let op = rng.gen_range(0..3);
        match op {
            0 => MathProblem {
                problem: format!("{} + {}", a, b),
                answer: a + b,
            },
            1 => {
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

struct Room {
    id: String,
    players: HashMap<u32, Player>,
    current_problem: MathProblem,
    broadcaster: broadcast::Sender<String>,
    progress: HashMap<u32, u32>, // player_id -> score
    start_time: Option<Instant>,
    duration: Duration,
    started: bool,
}

impl Room {
    fn new(id: String, duration: Duration) -> Self {
        let (tx, _) = broadcast::channel(16);
        Room {
            id,
            players: HashMap::new(),
            current_problem: MathProblem::generate(),
            broadcaster: tx,
            progress: HashMap::new(),
            start_time: None,
            duration,
            started: false,
        }
    }
}

struct AppState {
    rooms: HashMap<String, Arc<Mutex<Room>>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            rooms: HashMap::new(),
        }
    }
}

#[derive(Deserialize)]
struct WSQuery {
    room_id: String,
    name: String,
}

#[derive(Deserialize)]
struct CreateRoomRequest {
    duration: u64,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let create_room = warp::path("create_room")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_create_room);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::query::<WSQuery>())
        .and(with_state(state.clone()))
        .map(|ws: warp::ws::Ws, ws_query: WSQuery, state: Arc<Mutex<AppState>>| {
            ws.on_upgrade(move |socket| client_connection(socket, ws_query, state))
        });

    let static_files = warp::fs::dir("frontend");

    let routes = create_room.or(ws_route).or(static_files);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3030);
    println!("Server running on port {}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn with_state(state: Arc<Mutex<AppState>>) -> impl Filter<Extract = (Arc<Mutex<AppState>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn handle_create_room(req: CreateRoomRequest, state: Arc<Mutex<AppState>>) -> Result<impl warp::Reply, warp::Rejection> {
    let room_id: String = {
        let mut rng = rand::thread_rng();
        (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect()
    };

    let duration = Duration::from_secs(req.duration);
    let room = Arc::new(Mutex::new(Room::new(room_id.clone(), duration)));

    {
        let mut state_guard = state.lock().await;
        state_guard.rooms.insert(room_id.clone(), room);
    }

    let invitation_link = format!("http://localhost:3030/?room_id={}", room_id);

    Ok(warp::reply::json(&serde_json::json!({
        "room_id": room_id,
        "invitation_link": invitation_link
    })))
}

fn start_room_timer(room: Arc<Mutex<Room>>) {
    tokio::spawn(async move {
        let start = Instant::now();
        {
            let mut room_guard = room.lock().await;
            room_guard.start_time = Some(start);
        }
        loop {
            let elapsed = Instant::now().duration_since(start);
            let room_duration = {
                let room_guard = room.lock().await;
                room_guard.duration
            };
            let remaining = if elapsed >= room_duration {
                0
            } else {
                room_duration.as_secs() - elapsed.as_secs()
            };
            {
                let room_guard = room.lock().await;
                let _ = room_guard.broadcaster.send(format!("TIMER: {}", remaining));
            }
            if remaining == 0 {
                let room_guard = room.lock().await;
                let _ = room_guard.broadcaster.send("FINISH: Time's up!".to_string());
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}

async fn client_connection(ws: warp::ws::WebSocket, ws_query: WSQuery, state: Arc<Mutex<AppState>>) {
    let (ws_tx, mut ws_rx) = ws.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));

    let room = {
        let state_guard = state.lock().await;
        if let Some(room) = state_guard.rooms.get(&ws_query.room_id) {
            room.clone()
        } else {
            return;
        }
    };

    let player_id: u32 = rand::random();
    {
        let mut room_guard = room.lock().await;
        room_guard.players.insert(player_id, Player { name: ws_query.name.clone() });
        room_guard.progress.insert(player_id, 0);

        let mut tx = ws_tx.lock().await;
        if !room_guard.started {
            // Inform players that the game is waiting for the host to start.
            let _ = tx.send(warp::ws::Message::text("INFO: Waiting for host to start the game.")).await;
        } else {
            let init_msg = format!("PROBLEM: {}", room_guard.current_problem.problem);
            let progress_msg = format!("PROGRESS: {}",
                room_guard.players.iter()
                    .map(|(id, player)| {
                        let score = room_guard.progress.get(id).unwrap_or(&0);
                        format!("{}: {}", player.name, score)
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            let _ = tx.send(warp::ws::Message::text(init_msg)).await;
            let _ = tx.send(warp::ws::Message::text(progress_msg)).await;
        }
    }

    let mut broadcast_rx = {
        let room_guard = room.lock().await;
        room_guard.broadcaster.subscribe()
    };

    let tx_clone = ws_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let mut tx = tx_clone.lock().await;
            if tx.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_str().unwrap_or("");
                    if text.trim() == "START" {
                        let mut room_guard = room.lock().await;
                        if !room_guard.started {
                            room_guard.started = true;
                            drop(room_guard);
                            let room_clone = room.clone();
                            tokio::spawn(async move {
                                for i in (1..=3).rev() {
                                    let countdown_msg = format!("COUNTDOWN: {}", i);
                                    {
                                        let room_guard = room_clone.lock().await;
                                        let _ = room_guard.broadcaster.send(countdown_msg);
                                    }
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                }
                                {
                                    let room_guard = room_clone.lock().await;
                                    let _ = room_guard.broadcaster.send("START: GO".to_string());
                                }
                                start_room_timer(room_clone.clone());
                                {
                                    let room_guard = room_clone.lock().await;
                                    let problem_msg = format!("PROBLEM: {}", room_guard.current_problem.problem);
                                    let progress_msg = format!("PROGRESS: {}",
                                        room_guard.players.iter()
                                            .map(|(id, player)| {
                                                let score = room_guard.progress.get(id).unwrap_or(&0);
                                                format!("{}: {}", player.name, score)
                                            })
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    );
                                    let _ = room_guard.broadcaster.send(problem_msg);
                                    let _ = room_guard.broadcaster.send(progress_msg);
                                }
                            });
                        }
                    } else if let Ok(answer) = text.trim().parse::<i32>() {
                        let room_guard = room.lock().await;
                        if room_guard.started && answer == room_guard.current_problem.answer {
                            drop(room_guard);
                            let mut room_guard = room.lock().await;
                            let entry = room_guard.progress.entry(player_id).or_insert(0);
                            *entry += 1;
                            room_guard.current_problem = MathProblem::generate();
                            let problem_msg = format!("PROBLEM: {}", room_guard.current_problem.problem);
                            let progress_msg = format!("PROGRESS: {}",
                                room_guard.players.iter()
                                    .map(|(id, player)| {
                                        let score = room_guard.progress.get(id).unwrap_or(&0);
                                        format!("{}: {}", player.name, score)
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            );
                            let _ = room_guard.broadcaster.send(problem_msg);
                            let _ = room_guard.broadcaster.send(progress_msg);
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
