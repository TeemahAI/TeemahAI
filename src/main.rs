mod agent;
mod llm;
mod web3;
mod launchpad_services;
mod wallet;

use axum::{
    extract::State,
    routing::{get, post},
    Router, Json,
    http::{HeaderValue, Method},
};
use tower_http::cors::Any;
use crate::agent::TransactionData;

use ethers_core::rand;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use rand::Rng;

// Import your services and agent
use crate::agent::IntentAgent;
use crate::web3::Web3Service;
use crate::wallet::WalletManager;

#[derive(Clone)]
struct AppState {
    intent_agent: Arc<RwLock<Option<IntentAgent>>>,
    wallet_manager: Arc<WalletManager>,
}

#[derive(Deserialize)]
struct HelloRequest {
    name: String,
}

#[derive(Deserialize)]
struct SignTransactionRequest {
    transaction_data: TransactionData,
    address: String,
    chain_id: u64,
}
#[derive(Serialize)]
struct SignTransactionResponse {
    success: bool,
    transaction_hash: Option<String>,
    message: String,
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
    timestamp: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    agent_initialized: bool,
    wallet_connected: bool,
}

#[derive(Deserialize)]
struct CreateIntentRequest {
    user_input: String,
    user_id: Option<i64>,
}

#[derive(Deserialize)]
struct SignedIntentRequest {
    user_input: String,
    signature: String,
    address: String,
    chain_id: u64,
}

#[derive(Serialize)]
struct IntentResponse {
    intent_id: String,
    status: String,
    message: String,
    ai_message: String,
    transaction_hash: Option<String>,
    transaction_data: Option<TransactionData>,  // Add this line!
    data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct InitializeAgentRequest {
    deepseek_api_key: String,
    rpc_url: String,
    contract_address: String,
}

#[derive(Deserialize)]
struct WalletConnectRequest {
    address: String,
    chain_id: u64,
    wallet_type: String,
}

#[derive(Serialize)]
struct WalletStatusResponse {
    connected: bool,
    address: Option<String>,
    chain_id: Option<u64>,
    balance: Option<String>,
    balance_eth: Option<String>,
    wallet_type: Option<String>,
    connected_at: Option<String>,
}

#[derive(Serialize)]
struct WalletConnectResponse {
    success: bool,
    message: String,
    address: Option<String>,
    chain_id: Option<u64>,
}

#[derive(Serialize)]
struct WalletBalanceResponse {
    success: bool,
    balance: Option<String>,
    balance_eth: Option<String>,
    message: Option<String>,
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting Teemah AI Backend...");

    // Initialize wallet manager with default RPC
    let default_rpc = "https://data-seed-prebsc-1-s1.binance.org:8545".to_string();
    let wallet_manager = match WalletManager::new(&default_rpc) {
        Ok(wm) => {
            println!("✅ Wallet manager initialized");
            Arc::new(wm)
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize wallet manager: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize shared state
    let app_state = AppState {
        intent_agent: Arc::new(RwLock::new(None)),
        wallet_manager: wallet_manager.clone(),
    };
  
   let cors = CorsLayer::new()
    .allow_origin(Any)  // Allow ALL origins
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
    .allow_headers(Any)
    .allow_credentials(false);  // Important: must be false with wildcard
    let app = Router::new()
        // Basic routes
        .route("/", get(root))
        .route("/api/hello", post(hello_handler))
        .route("/api/health", get(health_handler))
        
        // Intent routes
        .route("/api/intents", post(create_intent))
        .route("/api/intents/signed", post(create_signed_intent))
        
        // Agent routes
        .route("/api/agent/initialize", post(initialize_agent))
        .route("/api/agent/status", get(get_agent_status))
        
        // Wallet routes
        .route("/api/wallet/connect", post(connect_wallet))
        .route("/api/wallet/disconnect", post(disconnect_wallet))
        .route("/api/wallet/status", get(get_wallet_status))
        .route("/api/wallet/balance", get(get_wallet_balance))
        .route("/api/wallet/sign-message", post(sign_message))
        .route("/api/transactions/sign", post(sign_transaction))

        
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(&addr).await.unwrap();
    
    println!("✅ Server running on http://{}", addr);
    println!("📡 Health: GET http://{}/api/health", addr);
    println!("🤝 Hello: POST http://{}/api/hello", addr);
    println!("🎯 Intents: POST http://{}/api/intents", addr);
    println!("🔐 Signed Intents: POST http://{}/api/intents/signed", addr);
    println!("🤖 Agent Init: POST http://{}/api/agent/initialize", addr);
    println!("👛 Wallet Connect: POST http://{}/api/wallet/connect", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Teemah AI Backend Server v0.1.0"
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let agent_guard = state.intent_agent.read().await;
    let agent_initialized = agent_guard.is_some();
    let wallet_connected = state.wallet_manager.is_connected().await;
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        agent_initialized,
        wallet_connected,
    })
}

async fn hello_handler(Json(payload): Json<HelloRequest>) -> Json<HelloResponse> {
    println!("👋 Hello request from: {}", payload.name);
    
    Json(HelloResponse {
        message: format!("Hello {} from Teemah AI Backend! 🚀", payload.name),
        timestamp: chrono::Local::now().to_rfc3339(),
    })
}

async fn initialize_agent(
    State(state): State<AppState>,
    Json(payload): Json<InitializeAgentRequest>,
) -> Json<serde_json::Value> {
    println!("🤖 Initializing Intent Agent...");
    
    println!("📡 RPC URL: {}", payload.rpc_url);
    println!("📜 Contract Address: {}", payload.contract_address);
    println!("🧠 DeepSeek API Key (first 10 chars): {}...", 
             &payload.deepseek_api_key[..10.min(payload.deepseek_api_key.len())]);
    
    match Web3Service::new_without_signer(
        &payload.rpc_url,
        &payload.contract_address,
    ) {
        Ok(web3_service) => {
            println!("✅ Web3Service initialized successfully (read-only)");
            
            let agent = IntentAgent::new("TeemahAgent", &payload.deepseek_api_key, web3_service);
            println!("✅ IntentAgent created");
            
            let mut agent_guard = state.intent_agent.write().await;
            *agent_guard = Some(agent);
            
            Json(serde_json::json!({
                "success": true,
                "message": "Intent agent initialized successfully (awaiting wallet connection)",
                "agent_name": "TeemahAgent",
                "read_only": true,
                "wallet_required": true
            }))
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize agent: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to initialize agent: {}", e)
            }))
        }
    }
}

async fn get_agent_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let agent_guard = state.intent_agent.read().await;
    let wallet_connected = state.wallet_manager.is_connected().await;
    
    if let Some(agent) = agent_guard.as_ref() {
        let intents = agent.get_active_intents().await;
        
        Json(serde_json::json!({
            "initialized": true,
            "agent_name": agent.name,
            "active_intents": intents.len(),
            "wallet_connected": wallet_connected,
            "requires_wallet": true,
            "status": "ready"
        }))
    } else {
        Json(serde_json::json!({
            "initialized": false,
            "message": "Agent not initialized",
            "wallet_connected": wallet_connected,
            "requires_wallet": true,
            "status": "uninitialized"
        }))
    }
}

async fn connect_wallet(
    State(state): State<AppState>,
    Json(payload): Json<WalletConnectRequest>,
) -> Json<WalletConnectResponse> {
    println!("👛 Connecting wallet: {} ({})", payload.address, payload.wallet_type);
    
    let wallet_type = match payload.wallet_type.to_lowercase().as_str() {
        "metamask" => crate::wallet::WalletType::MetaMask,
        "walletconnect" => crate::wallet::WalletType::WalletConnect,
        "coinbase" => crate::wallet::WalletType::CoinbaseWallet,
        "phantom" => crate::wallet::WalletType::Phantom,
        "trustwallet" => crate::wallet::WalletType::Other("TrustWallet".to_string()),
        "rainbow" => crate::wallet::WalletType::Other("Rainbow".to_string()),
        "argent" => crate::wallet::WalletType::Other("Argent".to_string()),
        _ => crate::wallet::WalletType::Other(payload.wallet_type.clone()),
    };
    
    match state.wallet_manager.connect_wallet(
        payload.address.clone(),
        payload.chain_id,
        wallet_type,
    ).await {
        Ok(_) => {
            println!("✅ Wallet connected successfully: {}", payload.address);
            //  let mut agent_guard = state.intent_agent.write().await;
            //  if let Some(agent) = agent_guard.as_mut() {
            //     // In production, you'd get the private key securely
            //     // For now, we'll use a placeholder or read from env
            //     if let Ok(private_key) = std::env::var("WALLET_PRIVATE_KEY") {
            //         if let Err(e) = agent.set_wallet_signer(&private_key, payload.chain_id).await {
            //             eprintln!("⚠️ Failed to update agent with wallet: {}", e);
            //         } else {
            //             println!("✅ Agent Web3Service updated with wallet");
            //         }
            //     }
            // }
            WalletConnectResponse {
                success: true,
                message: "Wallet connected successfully".to_string(),
                address: Some(payload.address),
                chain_id: Some(payload.chain_id),
            }.into()
        }
        Err(e) => {
            eprintln!("❌ Failed to connect wallet: {}", e);
            WalletConnectResponse {
                success: false,
                message: format!("Failed to connect wallet: {}", e),
                address: None,
                chain_id: None,
            }.into()
        }
    }
}

async fn disconnect_wallet(
    State(state): State<AppState>,
) -> Json<WalletConnectResponse> {
    match state.wallet_manager.disconnect_wallet().await {
        Ok(_) => {
            println!("🔌 Wallet disconnected");
            WalletConnectResponse {
                success: true,
                message: "Wallet disconnected successfully".to_string(),
                address: None,
                chain_id: None,
            }.into()
        }
        Err(e) => {
            eprintln!("❌ Failed to disconnect wallet: {}", e);
            WalletConnectResponse {
                success: false,
                message: format!("Failed to disconnect wallet: {}", e),
                address: None,
                chain_id: None,
            }.into()
        }
    }
}
async fn sign_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SignTransactionRequest>,
) -> Json<SignTransactionResponse> {
    println!("📝 Received transaction signing request from: {}", payload.address);
    
    // Verify wallet is connected
    let wallet_connected = state.wallet_manager.is_connected().await;
    if !wallet_connected {
        return Json(SignTransactionResponse {
            success: false,
            transaction_hash: None,
            message: "Wallet not connected. Please connect your wallet first.".to_string(),
        });
    }
    
    println!("⚠️ IMPORTANT: In production, transactions should be signed client-side with ethers.js");
    println!("📋 Transaction details:");
    println!("   To: {}", payload.transaction_data.to);
    println!("   Description: {}", payload.transaction_data.description);
    println!("   Chain ID: {}", payload.transaction_data.chain_id);
    println!("   Value: {}", payload.transaction_data.value);
    
    // Simulate a transaction hash
    let mock_hash = format!("0x{:x}", rand::random::<u64>());
    
    Json(SignTransactionResponse {
        success: true,
        transaction_hash: Some(mock_hash),
        message: "Transaction signing simulated. In production, this would be signed by the user's wallet.".to_string(),
    })
}
async fn get_wallet_status(
    State(state): State<AppState>,
) -> Json<WalletStatusResponse> {
    let connected = state.wallet_manager.is_connected().await;
    
    if connected {
        if let Some(wallet_info) = state.wallet_manager.get_wallet_info().await {
            let balance_result = state.wallet_manager.get_balance().await;
            let (balance, balance_eth) = match balance_result {
                Ok(bal) => {
                    let eth_balance = bal.as_u128() as f64 / 1e18;
                    (Some(bal.to_string()), Some(format!("{:.6}", eth_balance)))
                }
                Err(_) => (None, None),
            };
            
            let wallet_type_str = match &wallet_info.wallet_type {
                crate::wallet::WalletType::MetaMask => "MetaMask",
                crate::wallet::WalletType::WalletConnect => "WalletConnect",
                crate::wallet::WalletType::CoinbaseWallet => "CoinbaseWallet",
                crate::wallet::WalletType::Phantom => "Phantom",
                crate::wallet::WalletType::Other(name) => name.as_str(),
            }.to_string();
            
            return Json(WalletStatusResponse {
                connected: true,
                address: Some(wallet_info.address),
                chain_id: Some(wallet_info.chain_id),
                balance,
                balance_eth,
                wallet_type: Some(wallet_type_str.to_string()),
                connected_at: Some(wallet_info.connected_at.to_rfc3339()),
            });
        }
    }
    
    Json(WalletStatusResponse {
        connected: false,
        address: None,
        chain_id: None,
        balance: None,
        balance_eth: None,
        wallet_type: None,
        connected_at: None,
    })
}

async fn get_wallet_balance(
    State(state): State<AppState>,
) -> Json<WalletBalanceResponse> {
    match state.wallet_manager.get_balance().await {
        Ok(balance) => {
            let eth_balance = balance.as_u128() as f64 / 1e18;
            Json(WalletBalanceResponse {
                success: true,
                balance: Some(balance.to_string()),
                balance_eth: Some(format!("{:.6}", eth_balance)),
                message: None,
            })
        }
        Err(e) => {
            Json(WalletBalanceResponse {
                success: false,
                balance: None,
                balance_eth: None,
                message: Some(format!("Failed to get balance: {}", e)),
            })
        }
    }
}

async fn sign_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let message = payload.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    
    if message.is_empty() {
        return Json(serde_json::json!({
            "success": false,
            "message": "No message provided"
        }));
    }
    
    match state.wallet_manager.sign_message(message).await {
        Ok(signature) => {
            Json(serde_json::json!({
                "success": true,
                "signature": signature,
                "message": "Message signed successfully"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to sign message: {}", e)
            }))
        }
    }
}

async fn create_intent(
    State(state): State<AppState>,
    Json(payload): Json<CreateIntentRequest>,
) -> Json<IntentResponse> {
    println!("🎯 Intent received (unsigned): {}", payload.user_input);
    
    let agent_guard = state.intent_agent.read().await;
    
    match agent_guard.as_ref() {
        Some(agent) => {
            let user_id = payload.user_id.unwrap_or_else(|| {
                let mut rng = rand::thread_rng();
                rng.gen_range(100000..999999) as i64
            });
            
            println!("🔑 Using user_id: {} (generated automatically)", user_id);
            
            match agent.process_user_intent(user_id, &payload.user_input).await {
                Ok(result) => {
                      println!("📦 Intent result: success={}, has_tx_data={}", 
                             result.success, result.transaction_data.is_some());
            
                  if let Some(tx_data) = &result.transaction_data {
                    println!("📝 Transaction data chain_id: {}", tx_data.chain_id);
                    }
                    Json(IntentResponse {
                        intent_id: result.intent_id,
                        status: if result.success { "completed".to_string() } else { "failed".to_string() },
                        message: result.message,
                        ai_message: result.ai_message,
                        transaction_hash: result.transaction_hash,
                        transaction_data: result.transaction_data,  // Add this line!
                        data: result.data,
                    })
                }
                Err(e) => {
                    let error_message = format!("Intent processing failed: {}", e);
                    let fallback_ai_response = format!("❌ Sorry! There was an error processing your request: {}", e);
                    
                    Json(IntentResponse {
                        intent_id: Uuid::new_v4().to_string(),
                        status: "failed".to_string(),
                        message: error_message,
                        ai_message: fallback_ai_response,
                        transaction_hash: None,
                        transaction_data: None,  // Add this line!
                        data: None,
                    })
                }
            }
        }
        None => {
            Json(IntentResponse {
                intent_id: Uuid::new_v4().to_string(),
                status: "failed".to_string(),
                message: "Intent agent not initialized. Please initialize the agent first.".to_string(),
                ai_message: "🤖 Please initialize the AI agent first by clicking 'Initialize Agent' in the settings.".to_string(),
                transaction_hash: None,
                transaction_data: None,  // Add this line!
                data: None,
            })
        }
    }
}

async fn create_signed_intent(
    State(state): State<AppState>,
    Json(payload): Json<SignedIntentRequest>,
) -> Json<IntentResponse> {
    println!("🔐 Signed intent from: {}", payload.address);
    
    // Check if wallet is connected
    let wallet_connected = state.wallet_manager.is_connected().await;
    if !wallet_connected {
        return Json(IntentResponse {
            intent_id: Uuid::new_v4().to_string(),
            status: "failed".to_string(),
            message: "No wallet connected".to_string(),
            ai_message: "🔐 Please connect your wallet first to execute transactions!".to_string(),
            transaction_hash: None,
            transaction_data: None,  // Add this line!
            data: None,
        });
    }
    
    println!("📝 Signature provided: {}", payload.signature);
    println!("⚠️ Note: Signature verification is simulated in this example");
    
    let agent_guard = state.intent_agent.read().await;
    
    match agent_guard.as_ref() {
        Some(agent) => {
            // Generate user_id from address hash
            let user_id = (payload.address.clone() + &payload.chain_id.to_string())
                .bytes()
                .fold(0i64, |acc, b| acc.wrapping_add(b as i64));
            
            match agent.process_user_intent(user_id, &payload.user_input).await {
                Ok(result) => {
                    println!("🔄 Intent result received:");
                    println!("   Success: {}", result.success);
                    println!("   Has transaction_data: {}", result.transaction_data.is_some());
                    println!(" transation data {}", result.transaction_data);
                    println!("   Has data: {}", result.data.is_some());
        
                 if let Some(tx_data) = &result.transaction_data {
                    println!("   Transaction data details:");
                    println!("     To: {}", tx_data.to);
                    println!("     Chain ID: {}", tx_data.chain_id);
                    println!("     Description: {}", tx_data.description);
                    }
                    let mut data = result.data.unwrap_or(serde_json::json!({}));
                    if let serde_json::Value::Object(ref mut map) = data {
                        map.insert("wallet_address".to_string(), serde_json::Value::String(payload.address.clone()));
                        map.insert("chain_id".to_string(), serde_json::Value::Number(payload.chain_id.into()));
                        map.insert("signed".to_string(), serde_json::Value::Bool(true));
                    }
                    
                    Json(IntentResponse {
                        intent_id: result.intent_id,
                        status: if result.success { "completed".to_string() } else { "failed".to_string() },
                        message: result.message,
                        ai_message: result.ai_message,
                        transaction_hash: result.transaction_hash,
                        transaction_data: result.transaction_data,  // Add this line!
                        data: Some(data),
                    })
                }
                Err(e) => {
                    let error_message = format!("Intent processing failed: {}", e);
                    let fallback_ai_response = format!("❌ Sorry! There was an error processing your signed request: {}", e);
                    
                    Json(IntentResponse {
                        intent_id: Uuid::new_v4().to_string(),
                        status: "failed".to_string(),
                        message: error_message,
                        ai_message: fallback_ai_response,
                        transaction_hash: None,
                        transaction_data: None,  // Add this line!
                        data: None,
                    })
                }
            }
        }
        None => {
            Json(IntentResponse {
                intent_id: Uuid::new_v4().to_string(),
                status: "failed".to_string(),
                message: "Intent agent not initialized".to_string(),
                ai_message: "🤖 Please initialize the AI agent first.".to_string(),
                transaction_hash: None,
                transaction_data: None,  // Add this line!
                data: None,
            })
        }
    }
}