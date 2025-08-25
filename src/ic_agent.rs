use anyhow::{anyhow, Result};
use candid::{Decode, Nat};
use ic_agent::{export::Principal, Agent};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
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

// =============================================================================
// ENVIRONMENT CONFIGURATION
// =============================================================================

/// Load environment variables and return configuration (server-side only)
#[cfg(feature = "ssr")]
fn load_env_config() -> Result<ICConfig> {
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

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/// Create an IC client based on environment configuration (server-side only)
#[cfg(feature = "ssr")]
pub async fn create_client() -> Result<ICClient> {
    let config = load_env_config()?;
    create_client_with_config(
        &config.deployment_env,
        &config.counter_canister_id,
        &config.caller_canister_id,
    )
    .await
}

/// Get IC configuration from environment variables (server-side only)
#[cfg(feature = "ssr")]
pub fn get_ic_config() -> Result<ICConfig> {
    load_env_config()
}

/// Create a client with specific configuration (works on both server and client)
pub async fn create_client_with_config(
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

/// Client-side fallback that uses hardcoded values
/// This is used when running in WASM where env vars aren't available
#[cfg(not(feature = "ssr"))]
pub async fn create_client() -> Result<ICClient> {
    let config = ICConfig::default_local();
    create_client_with_config(
        &config.deployment_env,
        &config.counter_canister_id,
        &config.caller_canister_id,
    )
    .await
}

/// Create client with provided configuration (works on both server and client)
pub async fn create_client_from_config(config: &ICConfig) -> Result<ICClient> {
    create_client_with_config(
        &config.deployment_env,
        &config.counter_canister_id,
        &config.caller_canister_id,
    )
    .await
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

/// Test canister connectivity using environment configuration (server-side only)
#[cfg(feature = "ssr")]
pub async fn test_canisters() -> Result<()> {
    let config = load_env_config()?;

    println!(
        "🚀 Testing canister connectivity for {} environment...\n",
        config.deployment_env
    );

    match create_client().await {
        Ok(client) => {
            println!(
                "✅ Successfully created IC client for {} environment",
                config.deployment_env
            );
            println!("🔧 Agent principal: {:?}", client.get_principal()?);
            let (counter_id, caller_id) = client.get_canister_ids();
            println!("📦 Counter canister: {}", counter_id);
            println!("📞 Caller canister: {}", caller_id);

            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to create IC client: {}", e);
            if config.deployment_env == "local" {
                println!("💡 Make sure your local IC replica is running (dfx start)");
            }
            println!("💡 Check that your canister IDs are correct in .env file");
            println!("💡 Verify DEPLOYMENT_ENV is set to 'local' or 'prod'");
            Err(e)
        }
    }
}

/// Client-side fallback for testing (always uses local)
#[cfg(not(feature = "ssr"))]
pub async fn test_canisters() -> Result<()> {
    println!("🚀 Testing canister connectivity for client-side (local)...\n");

    match create_client().await {
        Ok(client) => {
            println!("✅ Successfully created IC client for client-side");
            let (counter_id, caller_id) = client.get_canister_ids();
            println!("📦 Counter canister: {}", counter_id);
            println!("📞 Caller canister: {}", caller_id);
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to create IC client: {}", e);
            println!("💡 Check that your hardcoded canister IDs are correct");
            Err(e)
        }
    }
}

/// Test local canister connectivity (deprecated - use test_canisters instead)
#[deprecated(note = "Use test_canisters() instead which uses environment configuration")]
pub async fn test_local_canisters() -> Result<()> {
    test_canisters().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_env_config_loading() {
        // Set test environment variables
        env::set_var("DEPLOYMENT_ENV", "local");
        env::set_var("COUNTER_CANISTER_ID", "rdmx6-jaaaa-aaaaa-aaadq-cai");
        env::set_var("CALLER_CANISTER_ID", "rrkah-fqaaa-aaaaa-aaaaq-cai");

        let result = load_env_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.deployment_env, "local");
        assert_eq!(config.counter_canister_id, "rdmx6-jaaaa-aaaaa-aaadq-cai");
        assert_eq!(config.caller_canister_id, "rrkah-fqaaa-aaaaa-aaaaq-cai");

        // Clean up
        env::remove_var("DEPLOYMENT_ENV");
        env::remove_var("COUNTER_CANISTER_ID");
        env::remove_var("CALLER_CANISTER_ID");
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_invalid_deployment_env() {
        // Set invalid environment
        env::set_var("DEPLOYMENT_ENV", "invalid");
        env::set_var("COUNTER_CANISTER_ID", "rdmx6-jaaaa-aaaaa-aaadq-cai");
        env::set_var("CALLER_CANISTER_ID", "rrkah-fqaaa-aaaaa-aaaaq-cai");

        let result = create_client().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid DEPLOYMENT_ENV"));

        // Clean up
        env::remove_var("DEPLOYMENT_ENV");
        env::remove_var("COUNTER_CANISTER_ID");
        env::remove_var("CALLER_CANISTER_ID");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_missing_env_vars() {
        // Ensure env vars are not set
        env::remove_var("COUNTER_CANISTER_ID");
        env::remove_var("CALLER_CANISTER_ID");

        let result = load_env_config();
        assert!(result.is_err());
    }
}
