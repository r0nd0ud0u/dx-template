use dioxus::prelude::*;
use dioxus_primitives::popover::{
    self, PopoverContentProps, PopoverRootProps, PopoverTriggerProps,
};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

#[component]
pub fn PopoverRoot(props: PopoverRootProps) -> Element {
    rsx! {
        popover::PopoverRoot {
            class: "popover",
            is_modal: props.is_modal,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn PopoverTrigger(props: PopoverTriggerProps) -> Element {
    rsx! {
        popover::PopoverTrigger { class: "popover-trigger", attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    rsx! {
        popover::PopoverContent {
            class: "popover-content",
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: props.attributes,
            {props.children}
        }
    }
}
