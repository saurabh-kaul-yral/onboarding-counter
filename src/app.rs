use crate::server_functions::{CallerAction, ExecuteCallerAction};
use leptos::prelude::*;
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
fn ServerCallerButtons() -> impl IntoView {
    let action = ServerAction::<ExecuteCallerAction>::new();
    let (text, set_text) = signal("Click Get to retrieve value".to_string());
    
    Effect::new(move || {
        if let Some(result) = action.value().get() {
            match result {
                Ok(counter_result) => {
                    if counter_result.success {
                        set_text(format!("Current Value: {}", counter_result.value))
                    } else {
                        set_text(format!("{:#?}",counter_result))
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
            <button
                class="counter-btn get-btn"
                on:click=move |_| {
                    action.dispatch(ExecuteCallerAction {
                        action: CallerAction::Get
                    });
                }
                disabled=move || action.pending().get()
            >
                {move || if action.pending().get() { "Getting..." } else { "Get" }}
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
                {move || if action.pending().get() { "Incrementing..." } else { "Increment" }}
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
                {move || if action.pending().get() { "Decrementing..." } else { "Decrement" }}
            </button>
        </div>

        // Show the result directly
        <p class="counter-result">
            {move || {
                text.get()
            }}
        </p>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

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
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to Saurabh's Onboarding Project"</h1>

        <ServerCallerButtons/>
    }
}
