use ethers::types::{U256, Address};
use ethers::providers::{Provider, Http};
use ethers::signers::{Wallet, Signer};
use ethers::middleware::SignerMiddleware;
use std::str::FromStr;
use anyhow::Result;
use crate::rand;
use std::sync::Arc;
use anyhow::anyhow;
use crate::launchpad_services::{LaunchpadClient, MechaLaunchpad, Project, ProjectDetails, UserInfo};

#[derive(Debug, Clone)]
pub struct Web3Service {
    launchpad_client: LaunchpadClient,
    chain_id: u64,
    is_read_only: bool,
    provider_url: String, // Store provider URL for reconnection
}

#[derive(Debug, Clone)]
pub struct Investment {
    pub investor: Address,
    pub project_id: Address,
    pub amount: U256,
    pub token_amount: U256,
    pub timestamp: U256,
    pub refunded: bool,
    pub tokens_claimed: bool,
}

#[derive(Debug, Clone)]
pub struct TokenData {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub token_price: U256,
    pub total_raised: U256,
    pub hard_cap: U256,
    pub remaining_cap: U256,
    pub liquidity_percent: U256,
    pub investor_count: u64,
    pub verified: bool,
    pub risk_score: U256,
    pub user_balance: U256,
}

impl Web3Service {
    pub fn new(
        provider_url: &str,
        contract_address: &str,
        private_key: &str,
        chain_id: u64,
    ) -> Result<Self> {
        let contract_addr = Address::from_str(contract_address)?;
        let launchpad_client = LaunchpadClient::new(provider_url, contract_addr, private_key, chain_id)?;
        
        Ok(Self {
            launchpad_client,
            chain_id,
            is_read_only: false,
            provider_url: provider_url.to_string(),
        })
    }
    
    pub fn new_without_signer(
        provider_url: &str,
        contract_address: &str,
    ) -> Result<Self> {
        // Create a dummy wallet for read-only operations
        // The wallet won't actually sign anything
        let dummy_key = ethers::core::k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let dummy_wallet = Wallet::from(dummy_key);
        
        let provider = Provider::<Http>::try_from(provider_url)?;
        let signer_middleware = SignerMiddleware::new(provider, dummy_wallet);
        let client = Arc::new(signer_middleware);
        
        let contract_addr = Address::from_str(contract_address)?;
        let contract = MechaLaunchpad::new(contract_addr, client.clone());
        
        // Create LaunchpadClient
        let launchpad_client = LaunchpadClient {
            client,
            contract,
        };
        
        Ok(Self {
            launchpad_client,
            chain_id: 97,
            is_read_only: true,
            provider_url: provider_url.to_string(),
        })
    }
    pub async fn get_contract_address(&self) -> Result<String> {
        Ok(format!("{:?}", self.launchpad_client.contract.address()))
    }
    pub async fn set_wallet_signer(&mut self, private_key: &str, chain_id: u64) -> Result<()> {
        // Create new wallet
        let wallet: Wallet<ethers::core::k256::ecdsa::SigningKey> = private_key.parse()?;
        let wallet_with_chain = wallet.with_chain_id(chain_id);
        
        // Create new provider (can't extract from existing client easily)
        let provider = Provider::<Http>::try_from(&self.provider_url)?;
        
        // Create new signer middleware
        let signer_middleware = SignerMiddleware::new(provider, wallet_with_chain);
        let client = Arc::new(signer_middleware);
        
        // Create new contract instance
        let contract_addr = self.launchpad_client.contract.address();
        let contract = MechaLaunchpad::new(contract_addr, client.clone());
        
        // Update launchpad client
        self.launchpad_client = LaunchpadClient {
            client,
            contract,
        };
        
        self.chain_id = chain_id;
        self.is_read_only = false;
        
        Ok(())
    }
    
    // Update provider URL if needed
    pub fn set_provider_url(&mut self, provider_url: &str) {
        self.provider_url = provider_url.to_string();
    }
    
    pub async fn get_all_projects(&self) -> Result<Vec<Project>> {
        let project_addresses = self.launchpad_client.get_all_projects().await?;
        
        let mut projects = Vec::new();
        for addr in project_addresses {
            if let Ok(project) = self.launchpad_client.get_project_details(addr).await {
                projects.push(project);
            }
        }
        Ok(projects)
    }
    
    pub async fn get_trending_projects(&self, limit: u64) -> Result<Vec<Project>> {
        let project_addresses = self.launchpad_client.get_trending_projects(limit).await?;
        
        let mut projects = Vec::new();
        for addr in project_addresses {
            if let Ok(project) = self.launchpad_client.get_project_details(addr).await {
                projects.push(project);
            }
        }
        Ok(projects)
    }
    
    pub async fn get_newly_launched_projects(&self, limit: u64) -> Result<Vec<Project>> {
        let project_addresses = self.launchpad_client.get_newly_launched_projects(limit).await?;
        
        let mut projects = Vec::new();
        for addr in project_addresses {
            if let Ok(project) = self.launchpad_client.get_project_details(addr).await {
                projects.push(project);
            }
        }
        Ok(projects)
    }
    
    pub async fn get_projects_by_status(&self, status: u8, page: u64, page_size: u64) -> Result<Vec<Project>> {
        let project_addresses = self.launchpad_client.get_projects_by_status(status, page, page_size).await?;
        
        let mut projects = Vec::new();
        for addr in project_addresses {
            if let Ok(project) = self.launchpad_client.get_project_details(addr).await {
                projects.push(project);
            }
        }
        Ok(projects)
    }
    
    pub async fn get_project_statistics(&self) -> Result<(u64, u64, u64, u64, u64)> {
        let stats = self.launchpad_client.get_project_statistics().await?;
        Ok((
            stats.0,
            stats.1,
            stats.2,
            stats.3.as_u64(),
            stats.4,
        ))
    }
    
    pub async fn get_active_projects(&self) -> Result<Vec<Project>> {
        self.get_projects_by_status(3, 1, 50).await
    }
    
    pub async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let target_addr = Address::from_str(project_id)?;
        match self.launchpad_client.get_project_details(target_addr).await {
            Ok(project) => Ok(Some(project)),
            Err(_) => Ok(None),
        }
    }
    
    pub async fn get_project_full_details(&self, project_id: &str) -> Result<Option<(Project, ProjectDetails)>> {
        let project_addr = Address::from_str(project_id)?;
        
        match self.launchpad_client.get_project_details(project_addr).await {
            Ok(project) => {
                let project_details = ProjectDetails {
                    description: String::new(),
                    website: String::new(),
                    whitepaper: String::new(),
                    telegram_group: String::new(),
                };
                
                Ok(Some((project, project_details)))
            }
            Err(_) => Ok(None),
        }
    }
    
    pub async fn get_project_marketing_info(&self, project_id: &str) -> Result<Option<(U256, U256, U256)>> {
        let project_addr = Address::from_str(project_id)?;
        match self.launchpad_client.get_project_details(project_addr).await {
            Ok(project) => {
                let marketing_amount = (project.tokens_for_sale * U256::from(project.marketing_percent)) / U256::from(10000);
                Ok(Some((
                    U256::from(project.marketing_percent),
                    project.marketing_telegram_id,
                    marketing_amount,
                )))
            }
            Err(_) => Ok(None),
        }
    }
    
    pub async fn invest_in_project(&self, project_id: &str, amount_eth: f64) -> Result<String> {
        if self.is_read_only {
            return Err(anyhow!("Cannot invest: Web3Service is in read-only mode. Connect wallet first."));
        }
        
        let amount_wei = (amount_eth * 1e18) as u128;
        let amount = U256::from(amount_wei);
        
        let project_addr = project_id.parse::<Address>()
            .map_err(|e| anyhow::anyhow!("Invalid project address: {}", e))?;
        
        match self.launchpad_client.invest(project_addr, amount).await {
            Ok(receipt) => {
                Ok(format!("{:?}", receipt.transaction_hash))
            }
            Err(e) => Err(anyhow::anyhow!("Investment failed: {}", e)),
        }
    }

    pub async fn claim_tokens(&self, project_id: &str) -> Result<String> {
        if self.is_read_only {
            return Err(anyhow!("Cannot claim tokens: Web3Service is in read-only mode. Connect wallet first."));
        }
        
        let project_addr = ethers::types::Address::from_str(project_id)?;
        match self.launchpad_client.claim_tokens(project_addr).await {
            Ok(receipt) => Ok(format!("0x{:x}", receipt.transaction_hash)),
            Err(e) => Err(anyhow::anyhow!("Token claim failed: {}", e)),
        }
    }
    
    pub async fn create_project(
        &self,
        name: String,
        symbol: String,
        soft_cap: U256,
        hard_cap: U256,
        end_time: U256,
     ) -> Result<String> {
        if self.is_read_only {
            return Err(anyhow!("Cannot create project: Web3Service is in read-only mode. Connect wallet first."));
        }
        
        use chrono::Utc;
        
        // For now, use zero address as creator
        // In a real implementation, you'd get this from the connected wallet
        let creator = Address::zero();
        
        let project_name = name.clone();
        let token_name = format!("{} Token", name);
        let token_symbol = symbol.clone();
        let token_decimals = 18;
        let initial_supply = hard_cap * U256::from(2);
        
        // Use current time + 5 minutes as start time
        let now = Utc::now().timestamp() as u64;
        let start_time = U256::from(now + 300);
        
        // Ensure end_time is in seconds
        let project_end_time = if end_time < U256::from(1_000_000_000) {
            // Likely provided in hours, convert to seconds
            end_time * U256::from(3600)
        } else {
            end_time
        };
        
        let token_price = U256::from(1_000_000_000_000_000u64); // 0.001 ETH
        let tokens_for_sale = initial_supply * U256::from(70) / U256::from(100);
        let liquidity_percent: u16 = 3000; // 30%
        let marketing_percent: u16 = 500; // 5%
        let marketing_telegram_id = U256::from(12345678u64);
        
        let project_id = self.launchpad_client.create_project(
            creator,
            project_name,
            token_name.clone(),
            token_symbol.clone(),
            token_decimals,
            initial_supply,
            soft_cap,
            hard_cap,
            start_time,
            project_end_time,
            token_price,
            tokens_for_sale,
            liquidity_percent,
            marketing_percent,
            marketing_telegram_id,
        ).await?;
        
        Ok(format!(
            "✅ Project created successfully!\n\
             📋 Project ID: {}\n\
             💰 Token Address: {}\n\
             📈 Project Name: {}\n\
             🏷️ Token Name: {}\n\
             🔠 Token Symbol: {}\n\
             💵 Soft Cap: {} ETH\n\
             🎯 Hard Cap: {} ETH\n\
             ⏰ Duration: {} days\n\
             📊 Liquidity: {}%\n\
             📢 Marketing: {}%",
            project_id,
            project_id,
            name,
            token_name,
            token_symbol,
            ethers::utils::format_units(soft_cap, "ether").unwrap_or_default(),
            ethers::utils::format_units(hard_cap, "ether").unwrap_or_default(),
            project_end_time.as_u64() / 86400,
            liquidity_percent / 100,
            marketing_percent / 100
        ))
    }
    
    pub async fn register_user(&self, _telegram_id: u64, _telegram_username: &str) -> Result<String> {
        log::warn!("register_user not implemented in LaunchpadClient");
        Ok("User registration not implemented".to_string())
    }
    
    pub async fn get_user_by_telegram_id(&self, telegram_id: u64) -> Result<Option<(UserInfo, Address)>> {
        match self.launchpad_client.get_user_by_telegram_id(telegram_id).await {
            Ok((user_info, address)) => Ok(Some((user_info, address))),
            Err(_) => Ok(None),
        }
    }
    
    pub async fn listen_project_created(&self) -> Result<()> {
        self.launchpad_client.listen_project_created().await
    }
    
    pub async fn listen_investments(&self) -> Result<()> {
        self.launchpad_client.listen_investments().await
    }
    
    pub async fn check_connection(&self) -> Result<u64> {
        match self.get_project_statistics().await {
            Ok(_) => Ok(1),
            Err(e) => Err(anyhow::anyhow!("Connection failed: {}", e)),
        }
    }
    
    pub async fn get_project_investment(
        &self,
        _project_id: &str,
        _user_address: &str,
     ) -> Result<Option<Investment>> {
        log::warn!("get_project_investment not implemented");
        Ok(None)
    }
    
    pub async fn get_user_investments(&self, _user_address: &str) -> Result<Vec<Investment>> {
        log::warn!("get_user_investments not implemented");
        Ok(vec![])
    }
    
    pub async fn get_token_data(&self, address: &str) -> Result<Option<TokenData>> {
        if let Some(project) = self.get_project(address).await? {
            Ok(Some(TokenData {
                address: address.to_string(),
                name: project.name,
                symbol: project.symbol,
                token_price: project.token_price,
                total_raised: project.tokens_for_sale,
                hard_cap: project.hard_cap,
                remaining_cap: project.hard_cap - project.soft_cap,
                liquidity_percent: U256::from(project.liquidity_percent),
                investor_count: project.tokens_for_sale.as_u64(),
                verified: true,
                risk_score: U256::from(3),
                user_balance: U256::zero(),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }
    
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
    
    pub fn provider_url(&self) -> &str {
        &self.provider_url
    }
    
    // Get client address for read-only mode (returns None)
    pub async fn get_client_address(&self) -> Result<Address> {
        if self.is_read_only {
            Err(anyhow!("Client is read-only, no address available"))
        } else {
            // Try to extract address from signer middleware
            // This is a simplified version - in real code you'd need proper downcasting
            Err(anyhow!("Cannot get client address in current implementation"))
        }
    }
}

// Utility functions
pub fn web3_to_ethers_address(addr: ethers::types::Address) -> ethers::types::Address {
    ethers::types::Address::from_slice(addr.as_bytes())
}

pub fn ethers_to_web3_address(addr: ethers::types::Address) -> ethers::types::Address {
    ethers::types::Address::from_slice(addr.as_bytes())
}

pub fn web3_to_ethers_u256(val: ethers::types::U256) -> ethers::types::U256 {
    let mut bytes = [0u8; 32];
    val.to_little_endian(&mut bytes);
    ethers::types::U256::from_little_endian(&bytes)
}

pub fn ethers_to_web3_u256(val: ethers::types::U256) -> U256 {
    let mut bytes = [0u8; 32];
    val.to_little_endian(&mut bytes);
    U256::from_little_endian(&bytes)
}