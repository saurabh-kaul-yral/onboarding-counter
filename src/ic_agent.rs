use anyhow::{anyhow, Result};
use candid::{Decode, Nat};
use ic_agent::{export::Principal, Agent};
use serde::{Deserialize, Serialize};

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

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

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

/// Test local canister connectivity with default canister IDs
pub async fn test_local_canisters() -> Result<()> {
    println!("🚀 Testing local canister connectivity...\n");

    // These are common local canister IDs from dfx
    let counter_id = "rdmx6-jaaaa-aaaaa-aaadq-cai"; // typical counter canister
    let caller_id = "rrkah-fqaaa-aaaaa-aaaaq-cai"; // typical caller canister

    match create_local_client(counter_id, caller_id).await {
        Ok(client) => {
            println!("✅ Successfully created IC client");
            println!("🔧 Agent principal: {:?}", client.get_principal()?);

            // Run comprehensive tests
            // client.run_comprehensive_tests().await?;

            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to create IC client: {}", e);
            println!("💡 Make sure your local IC replica is running (dfx start)");
            println!("💡 Check that your canister IDs are correct");
            Err(e)
        }
    }
}
