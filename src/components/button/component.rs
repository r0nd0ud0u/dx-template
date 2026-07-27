use dioxus::prelude::*;

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonVariant {
    AtkVigorType,
    AtkManaType,
    AtkBerserkType,
    AtkDefaultType,
    AtkName,
    AtkNameBlocked,
    AtkAutoMenu,
    AtkMenu,
    CharacterName,
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    GreenType,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            // Button attaks type
            ButtonVariant::AtkDefaultType => "atk-default-type",
            ButtonVariant::AtkVigorType => "atk-vigor-type",
            ButtonVariant::AtkManaType => "atk-mana-type",
            ButtonVariant::AtkBerserkType => "atk-berserk-type",
            ButtonVariant::AtkMenu => "atk-menu",
            ButtonVariant::AtkAutoMenu => "auto-atk-menu",
            ButtonVariant::CharacterName => "character-name",
            // Button atk name
            ButtonVariant::AtkName => "atk-name",
            ButtonVariant::AtkNameBlocked => "atk-name-blocked",
            // other buttons
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Destructive => "destructive",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::GreenType => "green-type",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "button",
            "data-style": variant.class(),
            onclick: move |event| {
                if let Some(f) = &onclick {
                    f.call(event);
                }
            },
            onmousedown: move |event| {
                if let Some(f) = &onmousedown {
                    f.call(event);
                }
            },
            onmouseup: move |event| {
                if let Some(f) = &onmouseup {
                    f.call(event);
                }
            },
            ..attributes,
            {children}
        }
    }
}
