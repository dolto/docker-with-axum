use dioxus::prelude::*;

use crate::front::page::component::login::Login;

#[component]
pub fn Home() -> Element {
    rsx! {
        Login  {}
        h1{"Hello World"}
    }
}
