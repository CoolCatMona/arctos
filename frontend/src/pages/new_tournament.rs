use crate::api;
use crate::url_slug::{is_valid_url_slug, URL_SLUG_ALLOWED_HINT};
use crate::Route;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

fn get_form_value(id: &str) -> String {
    let doc = web_sys::window().and_then(|w| w.document()).unwrap();
    doc.get_element_by_id(id)
        .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|e| e.value())
        .unwrap_or_default()
}

#[component]
pub fn NewTournament() -> Element {
    let navigator = use_navigator();
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let user = use_resource(move || async move { api::me().await });

    if user.read().is_none() {
        return rsx! { div { class: "loading-state", role: "status", "Checking your account…" } };
    }
    if !matches!(user.read().as_ref(), Some(Ok(_))) {
        return rsx! {
            div { class: "row justify-content-center form-page",
                div { class: "col-12 form-column",
                    div { class: "card",
                        div { class: "card-body",
                            h1 { "Create a tournament" }
                            p { "Log in to create and manage your tournament. Both player and team accounts can organize events." }
                            a { class: "btn btn-primary", href: "/login?next=/new-tournament", "Log in to continue" }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "row justify-content-center form-page",
            div { class: "col-12 form-column",
                div { class: "card",
                    div { class: "card-header",
                        p { class: "auth-eyebrow", "BRING EVERYONE TOGETHER" }
                        h1 { class: "mb-2", "Create a tournament" }
                        p { class: "text-muted mb-0", "Start with a name and link. Your tournament stays private until you publish it." }
                    }
                    div { class: "card-body",
                        if let Some(ref err) = error() {
                            div { class: "alert alert-danger mb-3", role: "alert", "{err}" }
                        }
                        p { class: "form-text mb-4", "After creating your event, set its dates, location, fees, and registration options in Settings before sharing it." }
                        form {
                            onsubmit: move |ev| {
                                ev.prevent_default();
                                if busy() { return; }
                                error.set(None);
                                let name = get_form_value("name").trim().to_string();
                                let url_slug = get_form_value("url").trim().to_string();
                                if name.is_empty() || url_slug.is_empty() {
                                    error.set(Some("Name and URL slug are required.".to_string()));
                                    return;
                                }
                                if !is_valid_url_slug(&url_slug) {
                                    error.set(Some(URL_SLUG_ALLOWED_HINT.to_string()));
                                    return;
                                }
                                busy.set(true);
                                let nav = navigator.clone();
                                spawn(async move {
                                    match api::create_tournament(&name, &url_slug, None).await {
                                        Ok(res) if res.success => {
                                            if let Some(url) = res.url {
                                                nav.push(Route::TournamentHome { url });
                                            } else {
                                                error.set(Some("Tournament created but no URL returned.".to_string()));
                                            }
                                        }
                                        Ok(res) => {
                                            error.set(Some(res.error.unwrap_or_else(|| "Creation failed.".to_string())));
                                        }
                                        Err(e) => {
                                            error.set(Some(e));
                                        }
                                    }
                                    busy.set(false);
                                });
                            },
                            div { class: "mb-3",
                                label { r#for: "name", class: "form-label", "Tournament Name" }
                                input { r#type: "text", class: "form-control", id: "name", name: "name", required: true }
                            }
                            div { class: "mb-3",
                                label { r#for: "url", class: "form-label", "Event URL" }
                                input {
                                    r#type: "text",
                                    class: "form-control",
                                    id: "url",
                                    name: "url",
                                    autocomplete: "off",
                                    autocapitalize: "none",
                                    spellcheck: "false",
                                    required: true,
                                    pattern: r"[A-Za-z0-9._~\-]+",
                                    title: URL_SLUG_ALLOWED_HINT,
                                }
                                div { class: "form-text", "Choose a unique, shareable link, such as /summer-cup. {URL_SLUG_ALLOWED_HINT}" }
                            }
                            div { class: "d-grid",
                                button { r#type: "submit", class: "btn btn-primary", disabled: busy(), if busy() { "Creating tournament…" } else { "Create tournament" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
