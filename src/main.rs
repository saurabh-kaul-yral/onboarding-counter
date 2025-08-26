#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;

    use leptos::logging::log;
    use leptos::prelude::provide_context;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use onboarding_counter::app::*;
    use onboarding_counter::ic_agent::{create_client_from_config, load_env_config, ICConfig};

    let ic_config = ICConfig::default_mainnet();
    let canister_client = create_client_from_config(&ic_config).await?;

    println!("\n🌐 Starting Leptos web server...");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let canister_client = canister_client.clone();
                move || {
                    provide_context(canister_client.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // Start the server
    log!("🚀 Leptos server listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
