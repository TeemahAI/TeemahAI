use crate::llm::DeepSeekClient;
use crate::web3::Web3Service;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use ethers::{abi::encode, types::U256};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    CreateProject { name: String, symbol: String },
    Invest { project_id: String, amount: f64 },
    ClaimTokens { project_id: String },
    GetProjectInfo { project_id: String },
    ListProjects,
    GetUserBalance,
    GetProjectStatistics,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub to: String,
    pub data: String,
    pub value: String,
    pub chain_id: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub intent_id: String,
    pub success: bool,
    pub message: String,
    pub ai_message: String,
    pub transaction_data: Option<TransactionData>,
    pub transaction_hash: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct IntentAgent {
    pub name: String,
    llm_client: DeepSeekClient,
    web3_service: Web3Service,
    active_intents: HashMap<String, IntentResult>,
}

impl IntentAgent {
    pub fn new(name: &str, api_key: &str, web3_service: Web3Service) -> Self {
        Self {
            name: name.to_string(),
            llm_client: DeepSeekClient::new(api_key),
            web3_service,
            active_intents: HashMap::new(),
        }
    }
    
    pub async fn process_user_intent(&self, user_id: i64, user_input: &str) -> Result<IntentResult> {
        let intent_id = Uuid::new_v4().to_string();
        println!("🎯 Processing intent from user {}: {}", user_id, user_input);
        println!("📝 Created intent: {}", intent_id);
        
        // Parse intent using LLM
        println!("🧠 Parsing intent with LLM: {}", intent_id);
        let parsed_intent = self.parse_intent_with_llm(user_input).await?;
        println!("✅ Parsed intent: {:?}", parsed_intent);
        
        // Execute intent
        println!("🚀 Executing intent: {:?}", parsed_intent);
        let result = self.execute_intent(&parsed_intent, user_input).await?;
        
        // Generate AI response
        println!("🤖 Generating AI response for intent result...");
        let ai_message = self.generate_ai_response(&result).await?;
        println!("✅ AI response generated successfully");
        
        let final_result = IntentResult {
            intent_id: intent_id.clone(),
            success: result.success,
            message: result.message,
            ai_message,
            transaction_data: result.transaction_data,
            transaction_hash: result.transaction_hash,
            data: result.data,
        };
        
        // Store the intent (optional)
        // self.active_intents.insert(intent_id, final_result.clone());
        
        Ok(final_result)
    }
    
    async fn parse_intent_with_llm(&self, user_input: &str) -> Result<Intent> {
        let prompt = format!(
            "Analyze the user's intent from their message. Classify it into one of these categories:
            1. CreateProject - When user wants to create a new token/project
            2. Invest - When user wants to invest in a project
            3. ClaimTokens - When user wants to claim tokens from a project
            4. GetProjectInfo - When user wants information about a project
            5. ListProjects - When user wants to see available projects
            6. GetUserBalance - When user wants to check their wallet balance
            7. GetProjectStatistics - When user wants statistics about projects
            8. Unknown - If none of the above match
            
            User message: \"{}\"
            
            Respond with ONLY the category name and any extracted parameters in JSON format.
            Example responses:
            - {{\"intent\": \"CreateProject\", \"name\": \"MyToken\", \"symbol\": \"MTK\"}}
            - {{\"intent\": \"Invest\", \"project_id\": \"0x123...\", \"amount\": 0.5}}
            - {{\"intent\": \"GetUserBalance\"}}
            
            If parameters can't be extracted, use null or best guess.
            ",
            user_input
        );
        
        let response = self.llm_client.generate_response(&prompt).await?;
        
        // Parse the JSON response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];
        
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Failed to parse LLM response as JSON: {}", e))?;
        
        let intent_str = parsed.get("intent")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        
        match intent_str {
            "CreateProject" => {
                let name = parsed.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let symbol = parsed.get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(Intent::CreateProject { name, symbol })
            },
            "Invest" => {
                let project_id = parsed.get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let amount = parsed.get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                Ok(Intent::Invest { project_id, amount })
            },
            "ClaimTokens" => {
                let project_id = parsed.get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(Intent::ClaimTokens { project_id })
            },
            "GetProjectInfo" => {
                let project_id = parsed.get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(Intent::GetProjectInfo { project_id })
            },
            "ListProjects" => Ok(Intent::ListProjects),
            "GetUserBalance" => Ok(Intent::GetUserBalance),
            "GetProjectStatistics" => Ok(Intent::GetProjectStatistics),
            _ => Ok(Intent::Unknown),
        }
    }
    
    async fn execute_intent(&self, intent: &Intent, original_input: &str) -> Result<IntentResult> {
        match intent {
            Intent::CreateProject { name, symbol } => {
                self.execute_create_project(name, symbol).await
            },
            Intent::Invest { project_id, amount } => {
                self.execute_invest(project_id, *amount).await
            },
            Intent::ClaimTokens { project_id } => {
                self.execute_claim_tokens(project_id).await
            },
            Intent::GetProjectInfo { project_id } => {
                self.execute_get_project_info(project_id).await
            },
            Intent::ListProjects => {
                self.execute_list_projects().await
            },
            Intent::GetUserBalance => {
                self.execute_get_user_balance().await
            },
            Intent::GetProjectStatistics => {
                self.execute_get_project_statistics().await
            },
            Intent::Unknown => {
                Ok(IntentResult {
                    intent_id: Uuid::new_v4().to_string(),
                    success: false,
                    message: format!("Could not understand intent: {}", original_input),
                    ai_message: String::new(),
                    transaction_data: None,
                    transaction_hash: None,
                    data: None,
                })
            },
        }
    }

    async fn execute_create_project(&self, name: &str, symbol: &str) -> Result<IntentResult> {
     println!("🏗️ Creating project: {} ({})", name, symbol);
    
     // Get the actual chain ID from web3_service
     let chain_id = self.web3_service.chain_id();
     println!("🔗 Using chain ID: {}", chain_id);
    
     // Get contract address
     let contract_address = self.web3_service.get_contract_address().await
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    
     println!("📜 Contract address: {}", contract_address);
    
     // IMPORTANT: Make sure chain_id is not 0
     if chain_id == 0 {
        println!("⚠️ WARNING: Chain ID is 0, defaulting to 97 (BSC Testnet)");
     }
    
     let final_chain_id = if chain_id == 0 { 97 } else { chain_id };
    
     // Create proper contract call data for createProjectWithTokenViaTelegram
     // Function signature: createProjectWithTokenViaTelegram(address, string, string, string, uint8, uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint16, uint16, uint256)
    
     // Get default values for parameters (you might want to make these configurable)
     let creator_address = "0x0000000000000000000000000000000000000000".to_string(); // Will be replaced by user's wallet
     let token_name = format!("{} Token", name);
     let token_decimals = 18u8;
     let initial_supply = U256::from(1_000_000_000_000_000_000_000u128); // 1000 tokens with 18 decimals
     let soft_cap = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
     let hard_cap = U256::from(10_000_000_000_000_000_000u128); // 10 ETH
     let start_time = U256::from(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 300); // Start in 5 minutes
     let end_time = start_time + U256::from(30 * 24 * 60 * 60); // 30 days duration
     let token_price = U256::from(100_000_000_000_000u128); // 0.0001 ETH per token
     let tokens_for_sale = initial_supply * U256::from(70) / U256::from(100); // 70% of tokens for sale
     let liquidity_percent = 3000u16; // 30%
     let marketing_percent = 500u16; // 5%
     let marketing_telegram_id = U256::from(0u64); // Default to 0
    
     println!("📊 Project parameters:");
     println!("   Creator: {}", creator_address);
     println!("   Token Name: {}", token_name);
     println!("   Decimals: {}", token_decimals);
     println!("   Initial Supply: {}", initial_supply);
     println!("   Soft Cap: {}", soft_cap);
     println!("   Hard Cap: {}", hard_cap);
     println!("   Start Time: {}", start_time);
     println!("   End Time: {}", end_time);
     println!("   Token Price: {}", token_price);
     println!("   Tokens for Sale: {}", tokens_for_sale);
     println!("   Liquidity %: {}", liquidity_percent);
     println!("   Marketing %: {}", marketing_percent);
    
     // Create transaction data
     let tx_data = TransactionData {
        to: contract_address.clone(),
        data: "0x".to_string(), // We'll handle encoding client-side for now
        value: "0".to_string(),
        chain_id: final_chain_id,
        description: format!("Create {} token with symbol {}", name, symbol),
     };
    
     println!("📝 Created transaction data:");
     println!("   To: {}", tx_data.to);
     println!("   Chain ID: {}", tx_data.chain_id);
     println!("   Description: {}", tx_data.description);
    
     Ok(IntentResult {
        intent_id: Uuid::new_v4().to_string(),
        success: true,
        message: "Transaction prepared for project creation".to_string(),
        ai_message: String::new(),
        transaction_data: Some(tx_data),
        transaction_hash: None,
        data: Some(serde_json::json!({
            "action": "create_project",
            "name": name,
            "symbol": symbol,
            "contract_address": contract_address,
            "chain_id": final_chain_id,
            "requires_signing": true,
            "is_contract_call": true,
            "function_name": "createProjectWithTokenViaTelegram",
            "parameters": {
                "creator": creator_address,
                "project_name": name,
                "token_name": token_name,
                "token_symbol": symbol,
                "token_decimals": token_decimals,
                "initial_supply": initial_supply.to_string(),
                "soft_cap": soft_cap.to_string(),
                "hard_cap": hard_cap.to_string(),
                "start_time": start_time.to_string(),
                "end_time": end_time.to_string(),
                "token_price": token_price.to_string(),
                "tokens_for_sale": tokens_for_sale.to_string(),
                "liquidity_percent": liquidity_percent,
                "marketing_percent": marketing_percent,
                "marketing_telegram_id": marketing_telegram_id.to_string()
             }
            })),
        })
    }
    
    // async fn execute_create_project(&self, name: &str, symbol: &str) -> Result<IntentResult> {
    //  println!("🏗️ Creating project: {} ({})", name, symbol);
    
    //  // Get the actual chain ID from web3_service
    //  let chain_id = self.web3_service.chain_id();
    //  println!("🔗 Using chain ID: {}", chain_id);
    
    //  // Get contract address
    //  let contract_address = self.web3_service.get_contract_address().await
    //     .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    
    //  println!("📜 Contract address: {}", contract_address);
    
    //  // IMPORTANT: Make sure chain_id is not 0
    //  if chain_id == 0 {
    //     println!("⚠️ WARNING: Chain ID is 0, defaulting to 97 (BSC Testnet)");
    //  }
    
    //  let final_chain_id = if chain_id == 0 { 97 } else { chain_id };
    
    //  // Create transaction data
    //  let tx_data = TransactionData {
    //     to: contract_address.clone(),
    //     data: "0x".to_string(), // For now, empty data
    //     value: "0".to_string(),
    //     chain_id: final_chain_id,
    //     description: format!("Create {} token with symbol {}", name, symbol),
    //  };
    
    //  println!("📝 Created transaction data:");
    //  println!("   To: {}", tx_data.to);
    //  println!("   Chain ID: {}", tx_data.chain_id);
    //  println!("   Value: {}", tx_data.value);
    //  println!("   Description: {}", tx_data.description);
    
    //  Ok(IntentResult {
    //     intent_id: Uuid::new_v4().to_string(),
    //     success: true,
    //     message: "Transaction prepared for project creation".to_string(),
    //     ai_message: String::new(),
    //     transaction_data: Some(tx_data),
    //     transaction_hash: None,
    //     data: Some(serde_json::json!({
    //         "action": "create_project",
    //         "name": name,
    //         "symbol": symbol,
    //         "contract_address": contract_address,
    //         "chain_id": final_chain_id,
    //         "requires_signing": true,
    //         "transaction_data": {
    //             "to": contract_address,
    //             "chain_id": final_chain_id,
    //             "description": format!("Create {} token with symbol {}", name, symbol)
    //         }
    //      })),
    //     })
    // }
    async fn execute_invest(&self, project_id: &str, amount: f64) -> Result<IntentResult> {
        println!("💰 Investing {} in project: {}", amount, project_id);
        
        let tx_data = TransactionData {
            to: project_id.to_string(),
            data: format!("invest:{}", amount),
            value: (amount * 1e18).to_string(), // Convert ETH to wei
            chain_id: self.web3_service.chain_id(),
            description: format!("Invest {} ETH in project {}", amount, project_id),
        };
        
        Ok(IntentResult {
            intent_id: Uuid::new_v4().to_string(),
            success: true,
            message: "Transaction prepared for investment".to_string(),
            ai_message: String::new(),
            transaction_data: Some(tx_data),
            transaction_hash: None,
            data: Some(serde_json::json!({
                "action": "invest",
                "project_id": project_id,
                "amount": amount,
                "requires_signing": true,
            })),
        })
    }
    
    async fn execute_claim_tokens(&self, project_id: &str) -> Result<IntentResult> {
        println!("🎫 Claiming tokens from project: {}", project_id);
        
        let tx_data = TransactionData {
            to: project_id.to_string(),
            data: "claimTokens".to_string(),
            value: "0".to_string(),
            chain_id: self.web3_service.chain_id(),
            description: format!("Claim tokens from project {}", project_id),
        };
        
        Ok(IntentResult {
            intent_id: Uuid::new_v4().to_string(),
            success: true,
            message: "Transaction prepared for token claim".to_string(),
            ai_message: String::new(),
            transaction_data: Some(tx_data),
            transaction_hash: None,
            data: Some(serde_json::json!({
                "action": "claim_tokens",
                "project_id": project_id,
                "requires_signing": true,
            })),
        })
    }
    
    async fn execute_get_project_info(&self, project_id: &str) -> Result<IntentResult> {
        println!("📊 Getting project info: {}", project_id);
        
        // This is a read-only operation, no transaction needed
        match self.web3_service.get_project(project_id).await {
            Ok(Some(project)) => {
                Ok(IntentResult {
                    intent_id: Uuid::new_v4().to_string(),
                    success: true,
                    message: "Project information retrieved".to_string(),
                    ai_message: String::new(),
                    transaction_data: None,
                    transaction_hash: None,
                    data: Some(serde_json::json!({
                        "project": project,
                        "action": "get_project_info",
                        "requires_signing": false,
                    })),
                })
            },
            Ok(None) => {
                Ok(IntentResult {
                    intent_id: Uuid::new_v4().to_string(),
                    success: false,
                    message: format!("Project not found: {}", project_id),
                    ai_message: String::new(),
                    transaction_data: None,
                    transaction_hash: None,
                    data: None,
                })
            },
            Err(e) => {
                Err(anyhow!("Failed to get project info: {}", e))
            }
        }
    }
    
    async fn execute_list_projects(&self) -> Result<IntentResult> {
        println!("📋 Listing all projects");
        
        match self.web3_service.get_all_projects().await {
            Ok(projects) => {
                Ok(IntentResult {
                    intent_id: Uuid::new_v4().to_string(),
                    success: true,
                    message: format!("Found {} projects", projects.len()),
                    ai_message: String::new(),
                    transaction_data: None,
                    transaction_hash: None,
                    data: Some(serde_json::json!({
                        "projects": projects,
                        "count": projects.len(),
                        "action": "list_projects",
                        "requires_signing": false,
                    })),
                })
            },
            Err(e) => {
                Err(anyhow!("Failed to list projects: {}", e))
            }
        }
    }
    
    async fn execute_get_user_balance(&self) -> Result<IntentResult> {
        println!("💰 Getting user balance");
        
        // This would need a wallet address, but we'll return generic info
        Ok(IntentResult {
            intent_id: Uuid::new_v4().to_string(),
            success: true,
            message: "Balance check prepared".to_string(),
            ai_message: String::new(),
            transaction_data: None,
            transaction_hash: None,
            data: Some(serde_json::json!({
                "action": "get_balance",
                "requires_signing": false,
                "note": "Connect wallet to see specific balance",
            })),
        })
    }
    
    async fn execute_get_project_statistics(&self) -> Result<IntentResult> {
        println!("📈 Getting project statistics");
        
        match self.web3_service.get_project_statistics().await {
            Ok((total, active, completed, total_raised, investors)) => {
                Ok(IntentResult {
                    intent_id: Uuid::new_v4().to_string(),
                    success: true,
                    message: "Project statistics retrieved".to_string(),
                    ai_message: String::new(),
                    transaction_data: None,
                    transaction_hash: None,
                    data: Some(serde_json::json!({
                        "statistics": {
                            "total_projects": total,
                            "active_projects": active,
                            "completed_projects": completed,
                            "total_raised": total_raised,
                            "total_investors": investors,
                        },
                        "action": "get_statistics",
                        "requires_signing": false,
                    })),
                })
            },
            Err(e) => {
                Err(anyhow!("Failed to get project statistics: {}", e))
            }
        }
    }
    
    async fn generate_ai_response(&self, intent_result: &IntentResult) -> Result<String> {
        let mut context = format!(
            "User intent result:\n\
             Success: {}\n\
             Message: {}\n",
            intent_result.success, intent_result.message
        );
        
        if let Some(data) = &intent_result.data {
            context += &format!("Data: {}\n", serde_json::to_string_pretty(data)?);
        }
        
        if let Some(tx_data) = &intent_result.transaction_data {
            context += &format!(
                "\nTransaction prepared:\n\
                 Description: {}\n\
                 To: {}\n\
                 Chain ID: {}\n\
                 Value: {} wei\n\
                 \nPlease provide a helpful response to the user explaining:\n\
                 1. What action will be performed\n\
                 2. That they need to sign the transaction in their wallet\n\
                 3. Any important details about the transaction\n\
                 4. Be friendly and encouraging!\n",
                tx_data.description, tx_data.to, tx_data.chain_id, tx_data.value
            );
        } else {
            context += "\nNo transaction required for this action. Please provide a helpful response to the user about the information they requested.\n";
        }
        
        if !intent_result.success {
            context += "\nThe operation failed. Please provide a helpful error message and suggest what the user can do next.\n";
        }
        
        let prompt = format!(
            "You are Teemah AI, a helpful Web3 assistant. Generate a friendly, informative response based on this context:\n\n{}\n\nResponse should be in markdown format, be concise, and helpful.",
            context
        );
        
        self.llm_client.generate_response(&prompt).await
    }
    
    pub async fn get_active_intents(&self) -> Vec<IntentResult> {
        self.active_intents.values().cloned().collect()
    }
    
    // Note: Commented out since users sign their own transactions
    // pub async fn set_wallet_signer(&mut self, private_key: &str, chain_id: u64) -> Result<()> {
    //     self.web3_service.set_wallet_signer(private_key, chain_id).await?;
    //     Ok(())
    // }
}