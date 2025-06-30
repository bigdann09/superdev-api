use axum::{
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use bs58;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
};
use spl_token::instruction as token_instruction;
use std::{str::FromStr};
use thiserror::Error;
use solana_sdk::program_error;
use axum::response::IntoResponse;
use axum::http::StatusCode;

#[derive(Error, Debug)]
enum ApiError {
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Program error: {0}")]
    ProgramError(String),
}

impl From<program_error::ProgramError> for ApiError {
    fn from(err: program_error::ProgramError) -> Self {
        ApiError::ProgramError(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status_code = StatusCode::BAD_REQUEST;
        let body = Json(serde_json::json!({
            "success": false,
            "error": self.to_string()
        }));
        (status_code, body).into_response()
    }
}

#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

impl<T> SuccessResponse<T> {
    fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}


#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

async fn generate_keypair() -> Result<Json<SuccessResponse<KeypairResponse>>, ApiError> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Ok(Json(SuccessResponse::new(KeypairResponse { pubkey, secret })))
}


#[derive(Deserialize)]
struct CreateTokenRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct InstructionResponse {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
}

async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<SuccessResponse<InstructionResponse>>, ApiError> {
    let mint_authority = Pubkey::from_str(&payload.mint_authority)
        .map_err(|_| ApiError::InvalidPubkey(payload.mint_authority.clone()))?;
    let mint = Pubkey::from_str(&payload.mint)
        .map_err(|_| ApiError::InvalidPubkey(payload.mint.clone()))?;

    let instruction = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        payload.decimals,
    )?;

    let accounts = instruction
        .accounts
        .iter()
        .map(|account_meta| AccountInfo {
            pubkey: account_meta.pubkey.to_string(),
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        })
        .collect();

    Ok(Json(SuccessResponse::new(InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(instruction.data),
    })))
}


#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Result<Json<SuccessResponse<InstructionResponse>>, ApiError> {
    let mint = Pubkey::from_str(&payload.mint)
        .map_err(|_| ApiError::InvalidPubkey(payload.mint.clone()))?;
    let destination = Pubkey::from_str(&payload.destination)
        .map_err(|_| ApiError::InvalidPubkey(payload.destination.clone()))?;
    let authority = Pubkey::from_str(&payload.authority)
        .map_err(|_| ApiError::InvalidPubkey(payload.authority.clone()))?;

    let instruction = token_instruction::mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    )?;

    let accounts = instruction
        .accounts
        .iter()
        .map(|account_meta| AccountInfo {
            pubkey: account_meta.pubkey.to_string(),
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        })
        .collect();

    Ok(Json(SuccessResponse::new(InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(instruction.data),
    })))
}


#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignMessageResponse {
    signature: String,
    public_key: String,
    message: String,
}

async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> Result<Json<SuccessResponse<SignMessageResponse>>, ApiError> {
    if payload.message.is_empty() {
        return Err(ApiError::MissingField("message".to_string()));
    }
    if payload.secret.is_empty() {
        return Err(ApiError::MissingField("secret".to_string()));
    }

    let secret_bytes = bs58::decode(&payload.secret)
        .into_vec()
        .map_err(|_| ApiError::InvalidSecretKey("Invalid base58 encoding".to_string()))?;

    let keypair = Keypair::from_bytes(&secret_bytes)
        .map_err(|_| ApiError::InvalidSecretKey("Invalid keypair bytes".to_string()))?;

    let signature = keypair.sign_message(payload.message.as_bytes());
    let signature_base64 = general_purpose::STANDARD.encode(signature.as_ref());

    Ok(Json(SuccessResponse::new(SignMessageResponse {
        signature: signature_base64,
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    })))
}


#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Result<Json<SuccessResponse<VerifyMessageResponse>>, ApiError> {
    if payload.message.is_empty() {
        return Err(ApiError::MissingField("message".to_string()));
    }
    if payload.signature.is_empty() {
        return Err(ApiError::MissingField("signature".to_string()));
    }
    if payload.pubkey.is_empty() {
        return Err(ApiError::MissingField("pubkey".to_string()));
    }

    let pubkey = Pubkey::from_str(&payload.pubkey)
        .map_err(|_| ApiError::InvalidPubkey(payload.pubkey.clone()))?;

    let signature_bytes = general_purpose::STANDARD
        .decode(&payload.signature)
        .map_err(|_| ApiError::InvalidSignature("Invalid base64 encoding".to_string()))?;

    let signature = solana_sdk::signature::Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| ApiError::InvalidSignature("Invalid signature bytes".to_string()))?;

    let valid = signature.verify(pubkey.as_ref(), payload.message.as_bytes());

    Ok(Json(SuccessResponse::new(VerifyMessageResponse {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    })))
}


#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Result<Json<SuccessResponse<InstructionResponse>>, ApiError> {
    if payload.lamports == 0 {
        return Err(ApiError::InvalidAmount);
    }

    let from = Pubkey::from_str(&payload.from)
        .map_err(|_| ApiError::InvalidPubkey(payload.from.clone()))?;
    let to = Pubkey::from_str(&payload.to)
        .map_err(|_| ApiError::InvalidPubkey(payload.to.clone()))?;

    let instruction = system_instruction::transfer(&from, &to, payload.lamports);

    let accounts = instruction
        .accounts
        .iter()
        .map(|account_meta| AccountInfo {
            pubkey: account_meta.pubkey.to_string(),
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        })
        .collect();

    Ok(Json(SuccessResponse::new(InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(instruction.data),
    })))
}


#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Result<Json<SuccessResponse<InstructionResponse>>, ApiError> {
    if payload.amount == 0 {
        return Err(ApiError::InvalidAmount);
    }

    let _mint = Pubkey::from_str(&payload.mint)
        .map_err(|_| ApiError::InvalidPubkey(payload.mint.clone()))?;
    let destination = Pubkey::from_str(&payload.destination)
        .map_err(|_| ApiError::InvalidPubkey(payload.destination.clone()))?;
    let owner = Pubkey::from_str(&payload.owner)
        .map_err(|_| ApiError::InvalidPubkey(payload.owner.clone()))?;

    let source = owner;
    let destination = destination;

    let instruction = token_instruction::transfer(
        &spl_token::id(),
        &source,
        &destination,
        &owner,
        &[],
        payload.amount,
    )?;

    let accounts = instruction
        .accounts
        .iter()
        .map(|account_meta| AccountInfo {
            pubkey: account_meta.pubkey.to_string(),
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        })
        .collect();

    Ok(Json(SuccessResponse::new(InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(instruction.data),
    })))
}

fn create_router() -> Router {
    Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
}

#[tokio::main]
async fn main() {
    let app = create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}