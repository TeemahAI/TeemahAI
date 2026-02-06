use ethers::{
    prelude::*,
    types::{Address, U256, Filter, ValueOrArray},
    abi::{encode, decode, ParamType, Token, Detokenize, InvalidOutputType},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::Result;

abigen!(
    MechaLaunchpad,
    r#"[
        function createProjectWithTokenViaTelegram(address, string, string,uint8, uint256) external returns (address)
        function invest(address) external payable
        function claimTokens(address) external
        function completeProject(address) external
        function claimRefund(address ) external
        function getProject(address ) external view returns (address ,address,string , string,uint256 ,uint256 ,uint256 ,uint256 ,uint256 ,uint8 )
        function getProjectStatistics() external view returns (uint256, uint256 , uint256 , uint256 , uint256 )
        function getAllProjects() external view returns (address[] memory)
        function getTrendingProjects(uint256) external view returns (address[] memory)
        function getNewlyLaunchedProjects(uint256) external view returns (address[] memory)
        function getProjectsByStatus(uint8 , uint256 , uint256 ) external view returns (address[] memory)
        function searchProjects(string ) external view returns (address[] memory)
        function registerUser(uint256 , string ) external
        function getUserByTelegramId(uint256 ) external view returns (uint8, uint256 , uint256 , uint256 , uint256 , string , address )
        function getProjectMarketingInfo(address ) external view returns (uint256 , address , uint256 , bool , uint256 )
        
        event ProjectCreated(address indexed , address indexed , uint256 , string )
        event Invested(address indexed , address indexed , uint256 , uint256 , uint256 , uint8 )
        event ProjectStatusChanged(address indexed , uint8 )
    ]"#,
);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub creator: Address,
    pub name: String,
    pub symbol: String,
    pub decimal: u8,
    pub initial_supply: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetails {
    pub description: String,
    pub website: String,
    pub whitepaper: String,
    pub telegram_group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub tier: u8,
    pub total_invested: U256,
    pub total_projects_invested: U256,
    pub join_date: U256,
    pub telegram_id: U256,
    pub telegram_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    pub amount: U256,
    pub token_amount: U256,
    pub timestamp: U256,
    pub investor_telegram_id: U256,
    pub refunded: bool,
    pub tokens_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Address,
    pub name: String,
    pub total_raised: U256,
    pub hard_cap: U256,
   // pub status: u8,
    pub progress: u64,
}

#[derive(Debug, Clone)]
pub struct LaunchpadClient {
   pub client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
   pub contract: MechaLaunchpad<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl LaunchpadClient {
    pub fn new(
        provider_url: &str,
        contract_address: Address,
        private_key: &str,
        chain_id: u64,
     ) -> Result<Self> {
        let provider = Provider::<Http>::try_from(provider_url)?;
        
        let wallet_result: Result<LocalWallet, _> = private_key.parse();
        let wallet = wallet_result?.with_chain_id(chain_id);
        
        let client = Arc::new(SignerMiddleware::new(provider, wallet));
        let contract = MechaLaunchpad::new(contract_address, client.clone());
        
        Ok(Self { client, contract })
    }
    
    pub async fn register_user(&self, telegram_id: u64, telegram_username: String) -> Result<TransactionReceipt> {
      let call = self.contract
        .register_user(U256::from(telegram_id), telegram_username);
    
       let tx = call.send().await?;
      let pending_tx = tx;
       let receipt = pending_tx.await?;
      receipt.ok_or_else(|| anyhow::anyhow!("Transaction failed"))
    }
    
    pub async fn get_user_by_telegram_id(&self, telegram_id: u64) -> Result<(UserInfo, Address)> {
        let result = self.contract
            .get_user_by_telegram_id(U256::from(telegram_id))
            .call()
            .await?;
            
        let user_info = UserInfo {
            tier: result.0,
            total_invested: result.1,
            total_projects_invested: result.2,
            join_date: result.3,
            telegram_id: result.4,
            telegram_username: result.5,
        };
        
        Ok((user_info, result.6))
    }

    pub async fn get_project_details(&self, project_id: Address) -> Result<Project> {
     let result = self.contract
        .get_project(project_id)
        .call()
        .await?;
     let (
        creator,
        offering_token,
        name,
        symbol,
        soft_cap,
        hard_cap,
        total_raised,
        start_time,
        end_time,
        status
     ) = result;

     Ok(Project {
        creator,
        name: name.to_string(),
        symbol: symbol.to_string(),
        decimal: 9,
        initial_supply: U256::zero(),
      })
    }
        
    pub async fn get_all_projects(&self) -> Result<Vec<Address>> {
        let projects = self.contract
            .get_all_projects()
            .call()
            .await?;
        Ok(projects)
    }
    
    pub async fn get_trending_projects(&self, limit: u64) -> Result<Vec<Address>> {
        let projects = self.contract
            .get_trending_projects(U256::from(limit))
            .call()
            .await?;
        Ok(projects)
    }
    
    pub async fn get_newly_launched_projects(&self, limit: u64) -> Result<Vec<Address>> {
        let projects = self.contract
            .get_newly_launched_projects(U256::from(limit))
            .call()
            .await?;
        Ok(projects)
    }
    
    pub async fn get_projects_by_status(&self, status: u8, page: u64, page_size: u64) -> Result<Vec<Address>> {
        let projects = self.contract
            .get_projects_by_status(status, U256::from(page), U256::from(page_size))
            .call()
            .await?;
        Ok(projects)
    }
    
    pub async fn search_projects(&self, search_term: String) -> Result<Vec<Address>> {
        let projects = self.contract
            .search_projects(search_term)
            .call()
            .await?;
        Ok(projects)
    }
    
    pub async fn get_project_statistics(&self) -> Result<(u64, u64, u64, U256, u64)> {
        let stats = self.contract
            .get_project_statistics()
            .call()
            .await?;
        Ok((
            stats.0.as_u64(),
            stats.1.as_u64(),
            stats.2.as_u64(),
            stats.3,
            stats.4.as_u64(),
        ))
    }
    
    
    pub async fn get_project(&self, project_id: Address) -> Result<Project> {
     self.get_project_details(project_id).await
    }
    
    pub async fn get_project_marketing_info(&self, project_id: Address) -> Result<(U256, Address, U256, bool, U256)> {
        let info = self.contract
            .get_project_marketing_info(project_id)
            .call()
            .await?;
        Ok(info)
    }
    
    pub async fn create_project(
     &self,
     creator: Address,
     token_name: String,
     token_symbol: String,
     token_decimals: u8,
     initial_supply: U256,
    ) -> Result<Address> {
     let create_call = self.contract.create_project_with_token_via_telegram(
            creator,
            token_name,
            token_symbol,
            token_decimals,
            initial_supply
        );

        let gas_estimate = create_call.estimate_gas().await.unwrap_or(U256::from(300_000_000));
        println!("Estimated Gas {:?}", gas_estimate);
        let gas_with_buffer = gas_estimate * U256::from(120) / U256::from(100);
        
        let gas_call = create_call.gas(gas_with_buffer);
        
        // Estimate and set appropriate gas price
        let gas_price = self.client.get_gas_price().await?;
        let gas_call_with_price = gas_call.gas_price(gas_price);
        
        let tx = gas_call_with_price.send().await.map_err(|e| {
            anyhow::anyhow!("Send failed: {:?}", e)
        })?;
        
        let receipt = tx.await?.ok_or_else(|| anyhow::anyhow!("Transaction failed - no receipt"))?;
        
        let logs = receipt.logs;
        for log in logs {
            if let Ok(decoded) = self.contract.decode_event::<(Address, Address, U256, String)>(
                "ProjectCreated",
                log.topics.clone(),
                log.data.clone(),
            ) {
                let (project_address, project_creator, _, _) = decoded;
                println!("Project created at: {:?} by: {:?}", project_address, project_creator);
                return Ok(project_address);
            }
        }
        
        Err(anyhow::anyhow!("Failed to parse project address from transaction receipt"))
    }
    pub async fn invest(&self, project_id: Address, amount: U256) -> Result<TransactionReceipt> {
     let call = self.contract.invest(project_id).value(amount);
     let tx = call.send().await?;
     let pending_tx = tx;
     let receipt = pending_tx.await?;
     receipt.ok_or_else(|| anyhow::anyhow!("Transaction failed"))
    }
    
    pub async fn claim_tokens(&self, project_id: Address) -> Result<TransactionReceipt> {
     let call = self.contract.claim_tokens(project_id);
     let tx = call.send().await?;
     let pending_tx = tx;
     let receipt = pending_tx.await?;
     receipt.ok_or_else(|| anyhow::anyhow!("Transaction failed"))
    }
    
    pub async fn complete_project(&self, project_id: Address) -> Result<TransactionReceipt> {
     let call = self.contract.complete_project(project_id);
     let tx = call.send().await?;
     let pending_tx = tx;
     let receipt = pending_tx.await?;
     receipt.ok_or_else(|| anyhow::anyhow!("Transaction failed"))
    }
    
    pub async fn claim_refund(&self, project_id: Address) -> Result<TransactionReceipt> {
     let call = self.contract.claim_refund(project_id);
     let tx = call.send().await?;
     let pending_tx = tx;
     let receipt = pending_tx.await?;
     receipt.ok_or_else(|| anyhow::anyhow!("Transaction failed"))
    }
        
    pub async fn get_projects_with_details(&self, project_addresses: Vec<Address>) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        
        for address in project_addresses {
            if let Ok(project) = self.get_project(address).await {
                projects.push(project);
            }
        }
        
        Ok(projects)
    }
    
    pub async fn get_projects_summary(&self, project_addresses: Vec<Address>) -> Result<Vec<ProjectSummary>> {
        let mut summaries = Vec::new();
        
        for address in project_addresses {
            if let Ok(project) = self.get_project(address).await {
                let progress = if project.initial_supply > U256::zero() {
                    (project.initial_supply * U256::from(100)) / project.initial_supply
                } else {
                    U256::zero()
                };
                
                summaries.push(ProjectSummary {
                    id: project.creator,
                    name: project.name,
                    total_raised: project.initial_supply,
                    hard_cap: project.initial_supply,
                    progress: progress.as_u64(),
                });
            }
        }
        
        Ok(summaries)
    }    
    pub async fn listen_project_created(&self) -> Result<()> {
        let filter = Filter::new()
            .address(ValueOrArray::Value(self.contract.address()))
            .event("ProjectCreated");
            
        let mut stream = self.client.watch(&filter).await?;
        
        while let Some(log) = stream.next().await {
      //      println!("New project created: {:?}", log);
        }
        
        Ok(())
    }
    
    pub async fn listen_investments(&self) -> Result<()> {
        let filter = Filter::new()
            .address(ValueOrArray::Value(self.contract.address()))
            .event("Invested");
            
        let mut stream = self.client.watch(&filter).await?;
        
        while let Some(_log) = stream.next().await {
            println!("New investment: {:?}", _log);
        }
        
        Ok(())
    }
    
    pub async fn listen_user_registrations(&self) -> Result<()> {
        let filter = Filter::new()
            .address(ValueOrArray::Value(self.contract.address()))
            .event("UserRegistered");
            
        let mut stream = self.client.watch(&filter).await?;
        
        while let Some(log) = stream.next().await {
            println!("New user registered: {:?}", log);
        }
        
        Ok(())
    }
    
    // Utility Functions
    
    pub fn contract_address(&self) -> Address {
        self.contract.address()
    }
}
