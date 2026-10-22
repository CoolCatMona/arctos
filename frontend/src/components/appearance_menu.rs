use crate::appearance::{focus_control, Appearance, ColorMode, Palette};
use dioxus::prelude::*;

/// The Appearance form shares the account/More menu's anchor and dismiss control.
#[component]
pub fn AppearanceMenu(
    mut open: Signal<bool>,
    mut account_open: Signal<bool>,
    appearance: Appearance,
) -> Element {
    let preferences = appearance.preferences();
    rsx! {
        style { {include_str!("appearance_menu.css")} }
        if open() {
            div {
                class: "appearance-panel", id: "appearance-options", role: "dialog",
                aria_labelledby: "appearance-heading",
                onmounted: move |_| focus_control(&format!("appearance-mode-{}", preferences.mode.id())),
                div { class: "appearance-panel-header",
                    button {
                        class: "appearance-back", r#type: "button", aria_label: "Back to menu",
                        onclick: move |_| { open.set(false); account_open.set(true); },
                        i { class: "fa-solid fa-chevron-left", aria_hidden: "true" }
                        span { "Menu" }
                    }
                    h2 { id: "appearance-heading", "Appearance" }
                    button {
                        class: "app-icon-button appearance-close", r#type: "button",
                        aria_label: "Close appearance settings",
                        onclick: move |_| {
                            open.set(false);
                            account_open.set(false);
                            focus_control("account-trigger");
                        },
                        i { class: "fa-solid fa-xmark", aria_hidden: "true" }
                    }
                }
                div { class: "appearance-panel-body",
                    fieldset { class: "appearance-section",
                        legend { "Display mode" }
                        div { class: "appearance-mode-options",
                            for mode in ColorMode::ALL {
                                label { class: if preferences.mode == mode { "appearance-mode-option is-selected" } else { "appearance-mode-option" },
                                    input {
                                        id: "appearance-mode-{mode.id()}", r#type: "radio",
                                        name: "arctos-color-mode", value: mode.id(),
                                        aria_label: "{mode.name()} mode", checked: preferences.mode == mode,
                                        onchange: move |_| appearance.set_mode(mode),
                                    }
                                    span { "{mode.name()}" }
                                }
                            }
                        }
                    }
                    fieldset { class: "appearance-section",
                        legend { "Theme" }
                        div { class: "appearance-theme-options",
                            for palette in Palette::PRIMARY {
                                ThemeOption { palette, appearance }
                            }
                        }
                        details { class: "appearance-additional", open: preferences.palette.is_additional(),
                            summary { "Additional themes" }
                            div { class: "appearance-theme-options",
                                for palette in Palette::ADDITIONAL {
                                    ThemeOption { palette, appearance }
                                }
                            }
                        }
                    }
                    p { class: "appearance-note",
                        if appearance.storage_available() { "Preferences stay on this browser. System follows your device’s light or dark setting." }
                        else { "Browser storage is unavailable. Changes apply for this visit. System follows your device’s light or dark setting." }
                    }
                }
            }
        }
    }
}

#[component]
fn ThemeOption(palette: Palette, appearance: Appearance) -> Element {
    let selected = appearance.preferences().palette == palette;
    let scheme = appearance.scheme();
    rsx! {
        label { class: if selected { "appearance-theme-option is-selected" } else { "appearance-theme-option" },
            input {
                id: "appearance-palette-{palette.id()}", r#type: "radio", name: "arctos-palette",
                value: palette.id(), aria_label: "{palette.name()} theme", checked: selected,
                onchange: move |_| appearance.set_palette(palette),
            }
            span { class: "appearance-theme-copy",
                strong { "{palette.name()}" }
                span { "{palette.description()}" }
            }
            span {
                class: "theme-preview", "data-palette": palette.id(), "data-bs-theme": scheme,
                aria_hidden: "true",
                span { class: "theme-preview-page" }
                span { class: "theme-preview-surface" }
                span { class: "theme-preview-accent" }
                span { class: "theme-preview-secondary" }
            }
        }
    }
}
