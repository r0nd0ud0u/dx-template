use dioxus::prelude::*;
use dioxus_primitives::label::{self, LabelProps};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

#[component]
pub fn Label(props: LabelProps) -> Element {
    rsx! {
        label::Label {
            class: "label",
            html_for: props.html_for,
            attributes: props.attributes,
            {props.children}
        }
    }
}
