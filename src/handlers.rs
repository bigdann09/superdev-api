use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use spl_token::instruction as token_instruction;

use base58::{FromBase58, ToBase58};
use base64::{engine::general_purpose, Engine as _};

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

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct InstructionResponse {
    program_id: String,
    accounts: Vec<AccountMetaResponse>,
    instruction_data: String,
}

#[derive(Serialize)]
struct AccountMetaResponse {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

pub async fn create_token(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    match (
        Pubkey::from_str_const(&payload.mint_authority),
        Pubkey::from_str_const(&payload.mint),
    ) {
        (mint_authority, mint) => {
            let instruction = token_instruction::initialize_mint(
                &spl_token::id(),
                &mint,
                &mint_authority,
                None,
                payload.decimals,
            ).unwrap();

            let response = ApiResponse {
                success: true,
                data: Some(InstructionResponse {
                    program_id: spl_token::id().to_string(),
                    accounts: instruction
                        .accounts
                        .iter()
                        .map(|acc| AccountMetaResponse {
                            pubkey: acc.pubkey.to_string(),
                            is_signer: acc.is_signer,
                            is_writable: acc.is_writable,
                        })
                        .collect(),
                    instruction_data: general_purpose::STANDARD.encode(&instruction.data),
                }),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
    }
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