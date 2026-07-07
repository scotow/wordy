use crate::game::Game;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::{Json, Router, routing};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::MissedTickBehavior;
use tokio::{task, time};
use tower_http::cors;
use uuid::Uuid;

// mod frontend;
mod game;

#[tokio::main]
async fn main() {
    let games = Arc::new(Mutex::new(HashMap::new()));
    task::spawn(garbage_collection_loop(Arc::clone(&games)));

    let router = Router::new()
        .route("/", routing::get(index_handler))
        .route("/games", routing::post(create_game_handler))
        .route("/games/{id}", routing::get(game_status_handler))
        .route("/games/{id}/words", routing::get(game_next_words_handler))
        .with_state(games)
        .layer(cors::CorsLayer::permissive());

    let _ = tokio::join!(
        axum::serve(
            TcpListener::bind((Ipv4Addr::UNSPECIFIED, 8080))
                .await
                .unwrap(),
            router.clone(),
        ),
        axum::serve(
            TcpListener::bind((Ipv4Addr::UNSPECIFIED, 7200))
                .await
                .unwrap(),
            router,
        )
    );
}

async fn garbage_collection_loop(games: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>>) {
    let mut ticker = time::interval(Duration::from_secs(60));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        let mut games = games.lock().await;
        let mut expired = Vec::new();
        for (&id, game) in &*games {
            if game.lock().await.is_expired() {
                expired.push(id);
            }
        }
        for id in expired {
            games.remove(&id);
        }
    }
}

async fn index_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../frontend/dist/index.html"),
    )
}

async fn game_status_handler(
    State(games): State<Arc<Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if games.lock().await.contains_key(&id) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn create_game_handler(
    State(games): State<Arc<Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>>>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    games
        .lock()
        .await
        .insert(id, Arc::new(Mutex::new(Game::new())));
    (StatusCode::CREATED, Json(json!({ "id": id }))).into_response()
}

#[derive(Deserialize, Debug)]
struct GameNextWordsQueryParams {
    count: Option<u16>,
}

async fn game_next_words_handler(
    State(games): State<Arc<Mutex<HashMap<Uuid, Arc<Mutex<Game>>>>>>,
    Path(id): Path<Uuid>,
    Query(params): Query<GameNextWordsQueryParams>,
) -> impl IntoResponse {
    let game = match games.lock().await.get(&id) {
        Some(game) => Arc::clone(game),
        None => return Err((StatusCode::NOT_FOUND, Json(json!({})))),
    };
    let mut game_lock = game.lock().await;
    let words = game_lock
        .word_catalogue
        .by_ref()
        .take(params.count.unwrap_or(1) as usize)
        .map(|(category, colors, word)| {
            json!({
                "number": word.number,
                "category": category,
                "colors": colors,
                "word": word.word,
                "difficulty": word.difficulty,
                "exclusions": word.exclusions,
            })
        })
        .collect::<Vec<_>>();
    Ok((StatusCode::OK, Json(words)))
}
