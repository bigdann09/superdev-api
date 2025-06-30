use std::sync::Arc;
use axum::{routing::post, Router};

mod handlers;
use handlers::*;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState {});

    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
        .with_state(app_state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3002")
        .await
        .unwrap();

    println!(
        "Listening on  {}",
        listener.local_addr().unwrap()
    );

    // start server
    axum::serve(listener, app).await.unwrap();
}
