use crate::appearance::{focus_control, Appearance, ColorMode, Palette};
use dioxus::prelude::*;

#[component]
pub fn AppearanceMenu(
    mut open: Signal<bool>,
    mut account_open: Signal<bool>,
    appearance: Appearance,
) -> Element {
    let preferences = appearance.preferences();
    rsx! {
        div { class: "appearance-menu",
            button {
                id: "appearance-trigger", class: "app-icon-button appearance-trigger",
                r#type: "button", aria_label: "Appearance", title: "Appearance",
                aria_expanded: "{open()}", aria_controls: "appearance-options", aria_haspopup: "dialog",
                onclick: move |_| {
                    account_open.set(false);
                    open.toggle();
                },
                i { class: "fa-solid fa-palette", aria_hidden: "true" }
            }
            if open() {
                button {
                    class: "appearance-dismiss", r#type: "button", tabindex: "-1",
                    aria_label: "Close appearance settings",
                    onclick: move |_| { open.set(false); focus_control("appearance-trigger"); }
                }
                div {
                    class: "appearance-dropdown", id: "appearance-options", role: "dialog",
                    aria_labelledby: "appearance-heading",
                    onmounted: move |_| focus_control(&format!("appearance-palette-{}", preferences.palette.id())),
                    h2 { id: "appearance-heading", "Appearance" }
                    fieldset { class: "appearance-section",
                        legend { "Color palette" }
                        div { class: "appearance-options",
                            for palette in Palette::ALL {
                                label { class: if preferences.palette == palette { "appearance-option is-selected" } else { "appearance-option" },
                                    input {
                                        id: "appearance-palette-{palette.id()}", r#type: "radio", name: "arctos-palette",
                                        value: palette.id(), aria_label: "{palette.name()} palette", checked: preferences.palette == palette,
                                        onchange: move |_| appearance.set_palette(palette),
                                    }
                                    span { class: "appearance-swatch", "data-palette": palette.id(), aria_hidden: "true" }
                                    span { "{palette.name()}" }
                                }
                            }
                        }
                    }
                    fieldset { class: "appearance-section",
                        legend { "Display mode" }
                        div { class: "appearance-mode-options",
                            for mode in ColorMode::ALL {
                                label { class: if preferences.mode == mode { "appearance-option is-selected" } else { "appearance-option" },
                                    input {
                                        r#type: "radio", name: "arctos-color-mode", value: mode.id(),
                                        aria_label: "{mode.name()} mode", checked: preferences.mode == mode,
                                        onchange: move |_| appearance.set_mode(mode),
                                    }
                                    span { "{mode.name()}" }
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
