pub mod page;
pub mod util;

use crate::front::page::home::Home;
use crate::front::util::ErrorLayout;
#[cfg(feature = "server")]
use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::style::TACIT;
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

#[cfg(feature = "server")]
pub fn init_router(aex: AppExtension) -> axum::Router {
    use crate::front::page::component::login;

    let login_router = login::init_router(aex.clone());
    let util_router = util::init_router();

    axum::Router::new()
        .nest("/front", login_router)
        .nest("/front", util_router)
}
