use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;

use crate::components::{
    scroll_area::ScrollArea,
    sheet::{Sheet, SheetContent, SheetHeader, SheetSide, SheetTitle},
};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

/// Hamburger button that opens a [`Sidebar`]. Presentational only — visibility
/// per breakpoint (mobile-only) is controlled by the caller's CSS, not here.
#[component]
pub fn SidebarTrigger(open: Signal<bool>, #[props(default)] label: Option<String>) -> Element {
    let aria_label = label.unwrap_or_else(|| "Menu".to_owned());

    rsx! {
        button {
            class: "sidebar-trigger",
            "aria-label": "{aria_label}",
            r#type: "button",
            onclick: move |_| open.set(true),
            svg {
                class: "sidebar-trigger-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M3 6h18" }
                path { d: "M3 12h18" }
                path { d: "M3 18h18" }
            }
        }
    }
}

/// Generic slide-in navigation drawer, built on top of the existing [`Sheet`]
/// primitive (`SheetSide::Left`). Knows nothing about routes, auth, or game
/// state — callers pass their own nav items (buttons/links) as `children`.
#[component]
pub fn Sidebar(
    open: Signal<bool>,
    #[props(default)] title: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            SheetContent {
                side: SheetSide::Left,
                class: Some("sidebar-content".to_owned()),
                if let Some(t) = title {
                    SheetHeader {
                        SheetTitle { "{t}" }
                    }
                }
                ScrollArea {
                    direction: ScrollDirection::Vertical,
                    width: "100%",
                    height: "100%",
                    div { class: "sidebar-nav", {children} }
                }
            }
        }
    }
}
