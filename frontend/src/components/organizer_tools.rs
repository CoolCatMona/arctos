use crate::appearance::focus_control;
use dioxus::prelude::*;

#[component]
pub fn OrganizerTools(children: Element) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        style { {include_str!("organizer_tools.css")} }
        div {
            class: if open() { "organizer-tools is-open" } else { "organizer-tools" },
            onkeydown: move |event| {
                if open() && event.key() == Key::Escape {
                    event.prevent_default();
                    event.stop_propagation();
                    open.set(false);
                    focus_control("organizer-tools-trigger");
                }
            },
            button {
                id: "organizer-tools-trigger", class: "organizer-tools-trigger", r#type: "button",
                aria_expanded: "{open()}", aria_controls: "organizer-tools-options",
                onclick: move |_| open.set(!open()),
                "Organizer tools"
                i { class: "fas fa-chevron-down", aria_hidden: "true" }
            }
            if open() {
                button {
                    class: "organizer-tools-dismiss", r#type: "button", tabindex: "-1",
                    aria_label: "Dismiss organizer tools",
                    onclick: move |_| { open.set(false); focus_control("organizer-tools-trigger"); },
                }
                div {
                    id: "organizer-tools-options", class: "organizer-tools-dropdown",
                    onclick: move |_| open.set(false),
                    {children}
                }
            }
        }
    }
}
