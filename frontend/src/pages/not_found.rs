use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        section { class: "arctos-empty-state",
            i { class: "fa-solid fa-compass", aria_hidden: "true" }
            h1 { "Page not found" }
            p { "This link may have changed. Find your event or browse the documentation to get back on track." }
            div { class: "d-flex flex-wrap gap-2 justify-content-center",
                Link { to: Route::Index {}, class: "btn btn-primary", "Find an event" }
                Link { to: Route::Docs {}, class: "btn btn-outline-secondary", "Documentation" }
            }
        }
    }
}
