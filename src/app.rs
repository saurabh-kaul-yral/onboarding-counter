use crate::ic_agent::{create_local_client, ICClient};
use crate::server_functions::{CallerAction, ExecuteCallerAction};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
fn ServerCallerButtons(set_text: WriteSignal<String>) -> impl IntoView {
    let action = ServerAction::<ExecuteCallerAction>::new();

    Effect::new(move || {
        if let Some(result) = action.value().get() {
            match result {
                Ok(counter_result) => {
                    if counter_result.success {
                        set_text(format!("Current Value: {}", counter_result.value))
                    } else {
                        set_text(format!("{:#?}", counter_result))
                    }
                }
                Err(e) => set_text(format!("Server Error: {}", e)),
            }
        } else {
            set_text("Click Get to retrieve value".to_string())
        }
    });

    view! {
        <div class="button-group">
        <h3>"Server-Side Buttons"</h3>
            <button
                class="counter-btn get-btn"
                on:click=move |_| {
                    action.dispatch(ExecuteCallerAction {
                        action: CallerAction::Get
                    });
                }
                disabled=move || action.pending().get()
            >
                 "Server Get"
            </button>

            <button
                class="counter-btn increment-btn"
                on:click=move |_| {
                    action.dispatch(ExecuteCallerAction {
                        action: CallerAction::Increment
                    });
                }
                disabled=move || action.pending().get()
            >
                "Server Increment"
            </button>

            <button
                class="counter-btn decrement-btn"
                on:click=move |_| {
                    action.dispatch(ExecuteCallerAction {
                        action: CallerAction::Decrement
                    });
                }
                disabled=move || action.pending().get()
            >
                "Server Decrement"
            </button>
        </div>
    }
}

#[component]
fn ClientCallerButtons(set_text: WriteSignal<String>) -> impl IntoView {
    // Get the ICClient signal from context
    let ic_client_signal = use_context::<ReadSignal<Option<ICClient>>>();

    view! {
        <div class="button-group client-buttons">
            <h3>"Client-Side Buttons"</h3>
            <Show
                when=move || ic_client_signal.map(|sig| sig.get().is_some()).unwrap_or(false)
                fallback=move || view! {
                    <button class="counter-btn get-btn" disabled=true>"Client Get (Loading...)"</button>
                    <button class="counter-btn increment-btn" disabled=true>"Client Increment (Loading...)"</button>
                    <button class="counter-btn decrement-btn" disabled=true>"Client Decrement (Loading...)"</button>
                }
            >
                {move || {
                    let ic_client = ic_client_signal.unwrap().get().unwrap();
                    view! {
                        <button
                            class="counter-btn get-btn"
                            on:click={
                                let ic_client = ic_client.clone();
                                move |_| {
                                    let ic_client = ic_client.clone();
                                    spawn_local(async move {
                                        match ic_client.caller_get().await {
                                            Ok(value) => set_text(format!("Current Value: {}", value)),
                                            Err(e) => set_text(format!("Client Error: {}", e)),
                                        }
                                    });
                                }
                            }
                        >
                            "Client Get"
                        </button>

                        <button
                            class="counter-btn increment-btn"
                            on:click={
                                let ic_client = ic_client.clone();
                                move |_| {
                                    let ic_client = ic_client.clone();
                                    spawn_local(async move {
                                        match ic_client.caller_increment().await {
                                            Ok(value) => set_text(format!("Current Value: {}", value)),
                                            Err(e) => set_text(format!("Client Error: {}", e)),
                                        }
                                    });
                                }
                            }
                        >
                            "Client Increment"
                        </button>

                        <button
                            class="counter-btn decrement-btn"
                            on:click={
                                let ic_client = ic_client.clone();
                                move |_| {
                                    let ic_client = ic_client.clone();
                                    spawn_local(async move {
                                        match ic_client.caller_decrement().await {
                                            Ok(value) => set_text(format!("Current Value: {}", value)),
                                            Err(e) => set_text(format!("Client Error: {}", e)),
                                        }
                                    });
                                }
                            }
                        >
                            "Client Decrement"
                        </button>
                    }
                }}
            </Show>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create a signal to hold the ICClient
    let (ic_client, set_ic_client) = signal::<Option<ICClient>>(None);

    // Initialize the client on startup
    Effect::new(move || {
        spawn_local(async move {
            match create_local_client("u6s2n-gx777-77774-qaaba-cai", "uxrrr-q7777-77774-qaaaq-cai")
                .await
            {
                Ok(client) => set_ic_client(Some(client)),
                Err(_) => set_ic_client(None),
            }
        });
    });

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/onboarding-counter.css"/>

        // sets the document title
        <Title text="Counter App Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=move || {
                        // Always provide the ICClient signal context for consistent hydration
                        provide_context(ic_client);
                        view! { <HomePage/> }
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (text, set_text) = signal("Click Get to retrieve value".to_string());

    view! {
        <h1>"Welcome to Saurabh's Onboarding Project"</h1>
        <div class="button-container">
            <h4>These Buttons call the same canister from our axum webserver</h4>
            <ServerCallerButtons set_text/>
            <h4>These Buttons call the same canister directly from the browser</h4>
            <ClientCallerButtons set_text/>
        </div>
        <p class="counter-result">
            {move || text.get()}
        </p>
    }
}
