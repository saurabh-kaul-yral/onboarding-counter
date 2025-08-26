use leptos::prelude::*;
use serde::{Deserialize, Serialize};


use crate::ic_agent::{ICConfig,ICClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallerAction {
    Get,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerResult {
    pub value: String,
    pub success: bool,
    pub error: Option<String>,
    pub action: CallerAction,
}



#[server(ExecuteCallerAction, "/api")]
pub async fn execute_counter_action(
    action: CallerAction,
) -> Result<CallerResult, ServerFnError<String>> {
    #[cfg(feature = "ssr")]
    {
        let client = expect_context::<ICClient>();
        match action {
            CallerAction::Get => {
                let value = client
                    .caller_get()
                    .await
                    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
                Ok(CallerResult {
                    value: value,
                    success: true,
                    error: None,
                    action,
                })
            }
            CallerAction::Increment => {
                let value = client
                    .caller_increment()
                    .await
                    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
                Ok(CallerResult {
                    value: value,
                    success: true,
                    error: None,
                    action,
                })
            }
            CallerAction::Decrement => {
                let value = client
                    .caller_decrement()
                    .await
                    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
                Ok(CallerResult {
                    value: value,
                    success: true,
                    error: None,
                    action,
                })
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        // On client side, return a placeholder response
        Err(ServerFnError::ServerError(
            "Server function called on client side".to_string(),
        ))
    }
}
