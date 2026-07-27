use dioxus::prelude::*;
use dioxus_primitives::separator::{self, SeparatorProps};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    rsx! {
        separator::Separator {
            class: "separator",
            horizontal: props.horizontal,
            decorative: props.decorative,
            attributes: props.attributes,
            {props.children}
        }
    }
}
