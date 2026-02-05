use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConnection {
    pub address: String,
    pub chain_id: u64,
    pub provider_url: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub wallet_type: WalletType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletType {
    MetaMask,
    WalletConnect,
    CoinbaseWallet,
    Phantom,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct WalletManager {
    pub current_wallet: Arc<RwLock<Option<WalletConnection>>>,
    pub provider: Arc<Provider<Http>>,
}

impl WalletManager {
    pub fn new(provider_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(provider_url)?;
        
        Ok(Self {
            current_wallet: Arc::new(RwLock::new(None)),
            provider: Arc::new(provider),
        })
    }
    
    pub async fn connect_wallet(
        &self,
        address: String,
        chain_id: u64,
        wallet_type: WalletType,
    ) -> Result<()> {
        let connection = WalletConnection {
            address: address.clone(),
            chain_id,
            provider_url: self.provider.url().to_string(),
            connected_at: chrono::Utc::now(),
            wallet_type,
        };
        
        let mut wallet = self.current_wallet.write().await;
        *wallet = Some(connection);
        
        println!("✅ Wallet connected: {} on chain {}", address, chain_id);
        Ok(())
    }
    
    pub async fn disconnect_wallet(&self) -> Result<()> {
        let mut wallet = self.current_wallet.write().await;
        if let Some(conn) = wallet.take() {
            println!("🔌 Wallet disconnected: {}", conn.address);
        }
        Ok(())
    }
    
    pub async fn get_balance(&self) -> Result<U256> {
        let wallet = self.current_wallet.read().await;
        match wallet.as_ref() {
            Some(conn) => {
                let address: Address = conn.address.parse()?;
                let balance = self.provider.get_balance(address, None).await?;
                Ok(balance)
            }
            None => Err(anyhow::anyhow!("No wallet connected")),
        }
    }
    
    pub async fn is_connected(&self) -> bool {
        let wallet = self.current_wallet.read().await;
        wallet.is_some()
    }
    
    pub async fn get_wallet_info(&self) -> Option<WalletConnection> {
        let wallet = self.current_wallet.read().await;
        wallet.clone()
    }
    
    pub async fn sign_message(&self, message: &str) -> Result<String> {
        let wallet = self.current_wallet.read().await;
        match wallet.as_ref() {
            Some(_) => {
                // In a real implementation, this would use the wallet's signer
                // For now, we'll simulate it
                Ok(format!("signed:{}", message))
            }
            None => Err(anyhow::anyhow!("No wallet connected")),
        }
    }
}