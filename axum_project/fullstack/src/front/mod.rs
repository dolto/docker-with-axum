pub mod page;
pub mod util;

use crate::front::page::home::Home;
use crate::resources::style::TACIT;
use dioxus::fullstack::FullstackContext;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(ErrorLayout)]
    #[route("/")]
    Home {},
}
#[component]
pub fn app() -> Element {
    rsx! {
        document::Title{"Dolto's Blog"}
        document::Link { rel: "stylesheet", href: TACIT}
        Router::<Route> {}
    }
}

// And then our Outlet is wrapped in a fallback UI
#[component]
fn ErrorLayout() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: move |err: ErrorContext| {
                let http_error = FullstackContext::commit_error_status(err.error().unwrap());
                match http_error.status {
                    StatusCode::NOT_FOUND => rsx! { div { "404 - Page not found" } },
                    _ => rsx! { div { "An unknown error occurred" } },
                }
            },
            Outlet::<Route> {}
        }
    }
}
