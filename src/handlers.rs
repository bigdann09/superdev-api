use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
// use solana_sdk::signature::Keypair;
use solana_sdk::{signature::Keypair, signer::Signer};

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

pub async fn create_token() {

}

pub async fn mint_token() {
    
}

pub async fn sign_message() {
    
}

pub async fn verify_message() {
    
}

pub async fn send_sol() {
    
}

pub async fn send_token() {
    
}

pub async fn generate_keypair() -> impl IntoResponse {
    let keypair = Keypair::new();
    let response = ApiResponse {
        success: true,
        data: Some(KeypairResponse {
            pubkey: keypair.pubkey().to_string(),
            secret: keypair.to_base58_string()
        }),
        error: None,
    };

    (StatusCode::OK, Json(response))
}