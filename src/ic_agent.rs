use anyhow::{anyhow, Result};
use candid::{Decode, Nat};
use ic_agent::{export::Principal, Agent};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ICConfig {
    pub deployment_env: String,
    pub counter_canister_id: String,
    pub caller_canister_id: String,
}

impl ICConfig {
    pub fn new(
        deployment_env: String,
        counter_canister_id: String,
        caller_canister_id: String,
    ) -> Self {
        Self {
            deployment_env,
            counter_canister_id,
            caller_canister_id,
        }
    }

    pub fn default_local() -> Self {
        Self {
            deployment_env: "local".to_string(),
            counter_canister_id: "u6s2n-gx777-77774-qaaba-cai".to_string(),
            caller_canister_id: "uxrrr-q7777-77774-qaaaq-cai".to_string(),
        }
    }

    pub fn default_mainnet() -> Self {
        Self {
            deployment_env: "prod".to_string(),
            counter_canister_id: "qmgff-sqaaa-aaaad-qhowa-cai".to_string(),
            caller_canister_id: "qzbui-tyaaa-aaaad-qhovq-cai".to_string(),
        }
    }
}

/// IC Agent client for interacting with counter and caller canisters
#[derive(Clone, Serialize, Deserialize)]
pub struct ICClient {
    #[serde(skip)]
    agent: Option<Agent>,
    counter_canister_id: Principal,
    caller_canister_id: Principal,
}

impl ICClient {
    /// Create a new IC client
    pub async fn new(
        replica_url: &str,
        counter_canister_id: &str,
        caller_canister_id: &str,
    ) -> Result<Self> {
        // Create agent
        let agent = Agent::builder()
            .with_url(replica_url)
            .build()
            .map_err(|e| anyhow!("Failed to create agent: {}", e))?;

        // For local development, fetch root key
        if replica_url.contains("127.0.0.1") || replica_url.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|e| anyhow!("Failed to fetch root key: {}", e))?;
        }

        let counter_principal = Principal::from_text(counter_canister_id)
            .map_err(|e| anyhow!("Invalid counter canister ID: {}", e))?;
        let caller_principal = Principal::from_text(caller_canister_id)
            .map_err(|e| anyhow!("Invalid caller canister ID: {}", e))?;

        Ok(ICClient {
            agent: Some(agent),
            counter_canister_id: counter_principal,
            caller_canister_id: caller_principal,
        })
    }

    pub async fn caller_get(&self) -> Result<String> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| anyhow!("Agent not available"))?;
        let response = agent
            .update(&self.caller_canister_id, "call_get")
            .with_arg(candid::encode_args((&self.counter_canister_id,))?)
            .call_and_wait()
            .await
            .map_err(|e| anyhow!("Update failed: {}", e))?;

        let result = Decode!(&response, Result<Nat, String>)
            .map_err(|e| anyhow!("Failed to decode response: {}", e))?;

        match result {
            Ok(value) => Ok(value.to_string()),
            Err(err) => Err(anyhow!("Error: {}", err)),
        }
    }

    /// Increment counter via caller canister
    pub async fn caller_increment(&self) -> Result<String> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| anyhow!("Agent not available"))?;
        let response = agent
            .update(&self.caller_canister_id, "call_increment")
            .with_arg(candid::encode_args((&self.counter_canister_id,))?)
            .call_and_wait()
            .await
            .map_err(|e| anyhow!("Update failed: {}", e))?;

        let result = Decode!(&response, Result<Nat, String>)
            .map_err(|e| anyhow!("Failed to decode response: {}", e))?;

        match result {
            Ok(value) => Ok(value.to_string()),
            Err(err) => Err(anyhow!("Error: {}", err)),
        }
    }

    /// Decrement counter via caller canister
    pub async fn caller_decrement(&self) -> Result<String> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| anyhow!("Agent not available"))?;
        let response = agent
            .update(&self.caller_canister_id, "call_decrement")
            .with_arg(candid::encode_args((&self.counter_canister_id,))?)
            .call_and_wait()
            .await
            .map_err(|e| anyhow!("Update failed: {}", e))?;

        let result = Decode!(&response, Result<Nat, String>)
            .map_err(|e| anyhow!("Failed to decode response: {}", e))?;

        match result {
            Ok(value) => Ok(value.to_string()),
            Err(err) => Err(anyhow!("Error: {}", err)),
        }
    }

    // =============================================================================
    // UTILITY METHODS
    // =============================================================================

    /// Get both canister IDs
    pub fn get_canister_ids(&self) -> (Principal, Principal) {
        (self.counter_canister_id, self.caller_canister_id)
    }

    /// Get agent principal (your identity)
    pub fn get_principal(&self) -> Result<Principal> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| anyhow!("Agent not available"))?;
        Ok(agent
            .get_principal()
            .map_err(|e| anyhow!("Failed to get principal: {}", e))?)
    }
}

pub fn load_env_config() -> Result<ICConfig> {
    let deployment_env = env::var("DEPLOYMENT_ENV").unwrap_or_else(|_| "local".to_string());

    let counter_canister_id = env::var("COUNTER_CANISTER_ID")
        .map_err(|_| anyhow!("COUNTER_CANISTER_ID environment variable not set"))?;

    let caller_canister_id = env::var("CALLER_CANISTER_ID")
        .map_err(|_| anyhow!("CALLER_CANISTER_ID environment variable not set"))?;

    Ok(ICConfig::new(
        deployment_env,
        counter_canister_id,
        caller_canister_id,
    ))
}

async fn create_client() -> Result<ICClient> {
    let config = load_env_config()?;
    create_client_with_config(
        &config.deployment_env,
        &config.counter_canister_id,
        &config.caller_canister_id,
    )
    .await
}

pub async fn create_client_from_config(config: &ICConfig) -> Result<ICClient> {
    create_client_with_config(
        &config.deployment_env,
        &config.counter_canister_id,
        &config.caller_canister_id,
    )
    .await
}

async fn create_client_with_config(
    deployment_env: &str,
    counter_canister_id: &str,
    caller_canister_id: &str,
) -> Result<ICClient> {
    match deployment_env {
        "local" => create_local_client(counter_canister_id, caller_canister_id).await,
        "prod" => create_mainnet_client(counter_canister_id, caller_canister_id).await,
        _ => Err(anyhow!(
            "Invalid DEPLOYMENT_ENV: {}. Must be 'local' or 'prod'",
            deployment_env
        )),
    }
}

/// Create an IC client for local development
pub async fn create_local_client(
    counter_canister_id: &str,
    caller_canister_id: &str,
) -> Result<ICClient> {
    ICClient::new(
        "http://127.0.0.1:4943",
        counter_canister_id,
        caller_canister_id,
    )
    .await
}

/// Create an IC client for mainnet
pub async fn create_mainnet_client(
    counter_canister_id: &str,
    caller_canister_id: &str,
) -> Result<ICClient> {
    ICClient::new("https://ic0.app", counter_canister_id, caller_canister_id).await
}
