use crate::appearance::focus_control;
use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum OrganizeContext {
    Events,
    Leagues,
}

#[component]
pub fn OrganizeMenu(context: OrganizeContext) -> Element {
    let mut open = use_signal(|| false);
    let (destination, label, description) = match context {
        OrganizeContext::Events => (Route::NewTournament {}, "Create a tournament", "One competition"),
        OrganizeContext::Leagues => (Route::NewLeague {}, "Create a league", "A season of events"),
    };

    rsx! {
        div {
            class: if open() { "organize-menu is-open" } else { "organize-menu" },
            onkeydown: move |event| {
                if open() && event.key() == Key::Escape {
                    event.prevent_default();
                    event.stop_propagation();
                    open.set(false);
                    focus_control("organize-trigger");
                }
            },
            button {
                id: "organize-trigger", class: "organize-trigger", r#type: "button",
                aria_expanded: "{open()}", aria_controls: "organize-options",
                onclick: move |_| open.set(!open()),
                "Organize"
                i { class: "fas fa-chevron-down", aria_hidden: "true" }
            }
            if open() {
                button {
                    class: "organize-dismiss", r#type: "button", tabindex: "-1",
                    aria_label: "Dismiss organize menu",
                    onclick: move |_| {
                        open.set(false);
                        focus_control("organize-trigger");
                    },
                }
                div { id: "organize-options", class: "organize-dropdown",
                    Link { to: destination, class: "dropdown-item",
                        aria_label: "{label}", aria_describedby: "organize-description",
                        strong { "{label}" }
                        small { id: "organize-description", "{description}" }
                    }
                }
            }
        }
    }
}
