//! Self-service registration of OAuth apps ("Sign in with Arctos").

use crate::api;
use crate::components::{Modal, ModalKind};
use crate::types::OAuthClient;
use crate::Route;
use dioxus::prelude::*;

fn redirect_uri_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
struct AppCreateAttempt {
    before_ids: Vec<String>,
    name: String,
    homepage: String,
    redirect_uris: Vec<String>,
    confidential: bool,
    failure: String,
}

impl AppCreateAttempt {
    fn matches_new(&self, client: &OAuthClient) -> bool {
        !self.before_ids.contains(&client.client_id)
            && client.name == self.name.trim()
            && client.homepage_url.as_deref().unwrap_or_default() == self.homepage.trim()
            && client.redirect_uris == self.redirect_uris
            && client.confidential == self.confidential
    }
}

#[component]
pub fn DeveloperApps() -> Element {
    let me = use_resource(api::me);
    let mut refresh = use_signal(|| 0u32);
    let clients = use_resource(move || async move {
        let _ = refresh();
        api::oauth_clients().await
    });
    let mut new_secret = use_signal(|| None::<(String, String)>);
    let create_pending = use_signal(|| false);

    if let Some(Err(_)) = &*me.read() {
        return rsx! {
            div { class: "alert alert-info",
                "Log in to register apps. "
                Link { to: Route::Login {}, "Log in" }
            }
        };
    }

    rsx! {
        h1 { "Your apps" }
        p {
            "Register an app to let people sign in to it with their Arctos account and use the "
            Link { to: Route::Developers {}, "Arctos API" }
            "."
        }
        if let Some((name, secret)) = new_secret() {
            div { class: "alert alert-warning",
                strong { "Client secret for {name}: " }
                code { "{secret}" }
                div { class: "small mt-1", "Copy it now. Arctos only stores a hash, so it can't be shown again." }
                button { class: "btn btn-sm btn-outline-secondary mt-2", onclick: move |_| new_secret.set(None), "Done" }
            }
        }
        match &*clients.read() {
            None => rsx! { p { "Loading…" } },
            Some(Err(message)) => rsx! { div { class: "alert alert-danger", "{message}" } },
            Some(Ok(clients)) => rsx! {
                for client in clients.iter() {
                    AppCard {
                        key: "{client.client_id}",
                        client: client.clone(),
                        secret_pending: new_secret().is_some() || create_pending(),
                        on_change: move |_| refresh += 1,
                        on_secret: move |secret| new_secret.set(Some(secret)),
                    }
                }
                if clients.is_empty() {
                    p { class: "text-muted", "You haven't registered any apps yet." }
                }
            },
        }
        NewAppCard {
            secret_pending: new_secret().is_some(),
            pending: create_pending,
            on_created: move |secret: Option<(String, String)>| {
                refresh += 1;
                if secret.is_some() {
                    new_secret.set(secret);
                }
            },
        }
    }
}

#[component]
fn AppCard(
    client: OAuthClient,
    secret_pending: bool,
    on_change: EventHandler<()>,
    on_secret: EventHandler<(String, String)>,
) -> Element {
    let mut editing = use_signal(|| false);
    let mut name = use_signal(|| client.name.clone());
    let mut homepage = use_signal(|| client.homepage_url.clone().unwrap_or_default());
    let mut redirect_uris = use_signal(|| client.redirect_uris.join("\n"));
    let mut error = use_signal(|| None::<String>);
    let mut pending = use_signal(|| false);
    let mut confirm = use_signal(|| None::<bool>); // true: rotate, false: delete
    let mut confirmation_error = use_signal(|| None::<String>);
    let client_id = client.client_id.clone();
    let original = client.clone();
    let dialog_client = client.clone();
    let confirm_id = format!("app-confirm-{}", client.client_id);
    let title_id = format!("app-confirm-title-{}", client.client_id);
    let description_id = format!("app-confirm-description-{}", client.client_id);
    let cancel_id = format!("app-confirm-cancel-{}", client.client_id);

    let save = {
        let client_id = client_id.clone();
        move |_| {
            if pending() {
                return;
            }
            pending.set(true);
            error.set(None);
            let client_id = client_id.clone();
            let draft = (name(), homepage(), redirect_uri_lines(&redirect_uris()));
            spawn(async move {
                match api::update_oauth_client(&client_id, &draft.0, &draft.1, &draft.2).await {
                    Ok(_) => {
                        error.set(None);
                        editing.set(false);
                        on_change.call(());
                    }
                    Err(message) => error.set(Some(message)),
                }
                pending.set(false);
            });
        }
    };
    let perform = move |_| {
        if pending() {
            return;
        }
        let Some(rotate) = confirm() else {
            return;
        };
        pending.set(true);
        confirmation_error.set(None);
        let client = dialog_client.clone();
        spawn(async move {
            if rotate {
                match api::rotate_oauth_client_secret(&client.client_id).await {
                    Ok(secret) => {
                        on_secret.call((client.name, secret));
                        confirm.set(None);
                    }
                    Err(message) => confirmation_error.set(Some(format!("Could not confirm the new secret. The old secret may no longer work. Check your integration before choosing Replace secret again. {message}"))),
                }
            } else {
                match api::delete_oauth_client(&client.client_id).await {
                    Ok(()) => {
                        confirm.set(None);
                        on_change.call(());
                    }
                    Err(message) => {
                        // A lost delete response is safe to reconcile with the owner's list.
                        match api::oauth_clients().await {
                            Ok(clients)
                                if !clients.iter().any(|c| c.client_id == client.client_id) =>
                            {
                                confirm.set(None);
                                on_change.call(());
                            }
                            _ => confirmation_error.set(Some(message)),
                        }
                    }
                }
            }
            pending.set(false);
        });
    };

    rsx! {
        div { id: "app-client-{client.client_id}", class: if editing() { "edit-section mb-3" } else { "card mb-3" },
            div { class: if editing() { "edit-section-heading d-flex justify-content-between align-items-center" } else { "card-header d-flex justify-content-between align-items-center" },
                h5 { class: "mb-0", "{client.name}" }
                span { class: "badge bg-secondary", if client.confidential { "Confidential" } else { "Public" } }
            }
            div { class: if editing() { "edit-section-body" } else { "card-body" },
                if let Some(message) = error() {
                    div { class: "alert alert-danger py-2", role: "alert", "{message}" }
                }
                p { class: "mb-1", "Client ID: " code { "{client.client_id}" } }
                if editing() {
                    AppFields { name, homepage, redirect_uris, prefix: client.client_id.clone(), disabled: pending() }
                    div { class: "d-flex gap-2",
                        button { class: "btn btn-primary btn-sm", disabled: pending(), onclick: save, if pending() { "Saving…" } else { "Save" } }
                        button { class: "btn btn-outline-secondary btn-sm", disabled: pending(), onclick: move |_| {
                            name.set(original.name.clone());
                            homepage.set(original.homepage_url.clone().unwrap_or_default());
                            redirect_uris.set(original.redirect_uris.join("\n"));
                            error.set(None);
                            editing.set(false);
                        }, "Cancel" }
                    }
                } else {
                    if let Some(homepage) = client.homepage_url.as_ref() {
                        p { class: "mb-1", "Homepage: " a { href: "{homepage}", target: "_blank", "{homepage}" } }
                    }
                    p { class: "mb-1", "Redirect URIs:" }
                    ul {
                        for uri in client.redirect_uris.iter() {
                            li { key: "{uri}", code { "{uri}" } }
                        }
                    }
                    div { class: "d-flex gap-2",
                        button { class: "btn btn-outline-primary btn-sm", disabled: pending(), onclick: move |_| editing.set(true), "Edit" }
                        if client.confidential {
                            button { class: "btn btn-outline-warning btn-sm", id: "app-secret-{client.client_id}", disabled: pending() || secret_pending, onclick: move |_| { confirmation_error.set(None); confirm.set(Some(true)); }, "New secret" }
                        }
                        button { class: "btn btn-outline-danger btn-sm", id: "app-delete-{client.client_id}", disabled: pending() || secret_pending, onclick: move |_| { confirmation_error.set(None); confirm.set(Some(false)); }, "Delete" }
                    }
                }
            }
        }
        if let Some(rotate) = confirm() {
            Modal {
                id: confirm_id,
                labelled_by: title_id.clone(),
                described_by: Some(description_id.clone()),
                initial_focus_id: Some(cancel_id.clone()),
                return_focus_id: Some(format!("app-{}-{}", if rotate { "secret" } else { "delete" }, client.client_id)),
                kind: ModalKind::AlertDialog,
                dismissible: !pending(),
                on_close: move |_| confirm.set(None),
                div { class: "modal-header",
                    h2 { class: "modal-title h5", id: title_id, tabindex: "-1", if rotate { "Replace secret for {client.name}?" } else { "Delete {client.name}?" } }
                }
                div { class: "modal-body",
                    p { id: description_id,
                        if rotate { "The current secret will stop working immediately. Update your integration with the new secret, which is shown only once." }
                        else { "This permanently deletes this app registration and its authorizations. People will no longer be able to use it to sign in with Arctos. This cannot be undone." }
                    }
                    if let Some(message) = confirmation_error() { p { class: "alert alert-danger", role: "alert", "{message}" } }
                }
                div { class: "modal-footer",
                    button { id: cancel_id, r#type: "button", class: "btn btn-secondary", disabled: pending(), onclick: move |_| confirm.set(None), "Cancel" }
                    button { r#type: "button", class: "btn btn-danger", disabled: pending(), onclick: perform,
                        if pending() { "Working…" } else if rotate { "Replace secret" } else { "Delete app" }
                    }
                }
            }
        }
    }
}

#[component]
fn AppFields(
    name: Signal<String>,
    homepage: Signal<String>,
    redirect_uris: Signal<String>,
    prefix: String,
    disabled: bool,
) -> Element {
    rsx! {
        div { class: "mb-2",
            label { class: "form-label", r#for: "{prefix}-name", "Name" }
            input { id: "{prefix}-name", disabled, class: "form-control", value: "{name}", oninput: move |e| name.set(e.value()) }
            div { class: "form-text", "Shown to people when they sign in." }
        }
        div { class: "mb-2",
            label { class: "form-label", r#for: "{prefix}-homepage", "Homepage (optional)" }
            input { id: "{prefix}-homepage", disabled, class: "form-control", value: "{homepage}", oninput: move |e| homepage.set(e.value()) }
        }
        div { class: "mb-2",
            label { class: "form-label", r#for: "{prefix}-redirects", "Redirect URIs" }
            textarea {
                id: "{prefix}-redirects", disabled,
                class: "form-control font-monospace",
                rows: "3",
                value: "{redirect_uris}",
                oninput: move |e| redirect_uris.set(e.value()),
                placeholder: "https://example.com/callback\nhttp://127.0.0.1/callback",
            }
            div { class: "form-text",
                "One per line. Use https, or http://127.0.0.1/… for apps running on the user's computer (any port is accepted)."
            }
        }
    }
}

#[component]
fn NewAppCard(
    secret_pending: bool,
    mut pending: Signal<bool>,
    on_created: EventHandler<Option<(String, String)>>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut homepage = use_signal(String::new);
    let mut redirect_uris = use_signal(String::new);
    let mut confidential = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut uncertain = use_signal(|| None::<AppCreateAttempt>);
    let mut recovered = use_signal(Vec::<OAuthClient>::new);

    let check_saved = EventHandler::new(move |_| {
        if pending() {
            return;
        }
        let Some(attempt) = uncertain() else {
            return;
        };
        pending.set(true);
        error.set(None);
        spawn(async move {
            match api::oauth_clients().await {
                Ok(clients) => {
                    let matches: Vec<_> = clients.into_iter()
                        .filter(|client| attempt.matches_new(client)).collect();
                    uncertain.set(None);
                    on_created.call(None);
                    if matches.is_empty() {
                        error.set(Some(format!("{} No new matching app was found. Your entries are kept; review your apps before registering again.", attempt.failure)));
                    } else {
                        // List responses never contain a secret. Replacing a lost secret
                        // must go through the app's existing explicit confirmation.
                        recovered.set(matches);
                        name.set(String::new());
                        homepage.set(String::new());
                        redirect_uris.set(String::new());
                    }
                }
                Err(message) => error.set(Some(format!("Registration could not be confirmed, and the app list could not be checked. Your entries are kept. Check saved apps before trying to register again. {message}"))),
            }
            pending.set(false);
        });
    });

    rsx! {
        div { class: "edit-section mb-3",
            div { class: "edit-section-heading", h5 { class: "mb-0", "Register a new app" } }
            div { class: "edit-section-body",
                if let Some(message) = error() {
                    div { class: "alert alert-danger py-2", role: "alert", "{message}" }
                }
                if !recovered().is_empty() {
                    div { id: "app-create-recovery", class: "alert alert-warning", role: "status",
                        p { "Matching new app registrations were found. Nothing was registered again." }
                        ul { class: "mb-0",
                            for client in recovered() {
                                li { key: "{client.client_id}",
                                    strong { "{client.name}" }
                                    " was created. "
                                    if client.confidential {
                                        "Its one-time secret is unavailable because the response was lost. "
                                        a { href: "#app-secret-{client.client_id}", "Choose New secret for this app" }
                                        " to review and confirm a replacement."
                                    } else {
                                        "This public app does not use a client secret. "
                                        a { href: "#app-client-{client.client_id}", "Review app" }
                                    }
                                }
                            }
                        }
                    }
                }
                AppFields { name, homepage, redirect_uris, prefix: "new-app", disabled: pending() || uncertain().is_some() }
                div { class: "mb-3",
                    div { class: "form-check",
                        input {
                            class: "form-check-input",
                            r#type: "radio",
                            id: "app-public", name: "app-kind", disabled: pending() || uncertain().is_some(),
                            checked: !confidential(),
                            onchange: move |_| confidential.set(false),
                        }
                        label { class: "form-check-label", r#for: "app-public",
                            "Public: runs on the user's device (desktop, mobile, or browser app) and can't keep a secret. Uses PKCE."
                        }
                    }
                    div { class: "form-check",
                        input {
                            class: "form-check-input",
                            r#type: "radio",
                            id: "app-confidential", name: "app-kind", disabled: pending() || uncertain().is_some(),
                            checked: confidential(),
                            onchange: move |_| confidential.set(true),
                        }
                        label { class: "form-check-label", r#for: "app-confidential",
                            "Confidential: runs on a server you control and can keep a client secret."
                        }
                    }
                }
                button {
                    class: "btn btn-primary",
                    disabled: pending() || secret_pending || uncertain().is_some(),
                    onclick: move |_| {
                        if pending() || secret_pending || uncertain().is_some() { return; }
                        pending.set(true);
                        error.set(None);
                        recovered.set(Vec::new());
                        let mut attempt = AppCreateAttempt {
                            before_ids: Vec::new(), name: name(), homepage: homepage(),
                            redirect_uris: redirect_uri_lines(&redirect_uris()), confidential: confidential(),
                            failure: String::new(),
                        };
                        spawn(async move {
                            // Fresh IDs distinguish this attempt from identical old apps.
                            match api::oauth_clients().await {
                                Ok(clients) => attempt.before_ids = clients.into_iter().map(|client| client.client_id).collect(),
                                Err(message) => {
                                    error.set(Some(format!("Could not check your apps. Nothing was submitted. Try again when the app list is available. {message}")));
                                    pending.set(false);
                                    return;
                                }
                            }
                            match api::create_oauth_client(&attempt.name, &attempt.homepage, &attempt.redirect_uris, attempt.confidential).await {
                                Ok(created) => {
                                    error.set(None);
                                    name.set(String::new());
                                    homepage.set(String::new());
                                    redirect_uris.set(String::new());
                                    on_created.call(created.client_secret.map(|secret| (created.client.name, secret)));
                                }
                                Err(message) => {
                                    attempt.failure = message;
                                    uncertain.set(Some(attempt));
                                    pending.set(false);
                                    check_saved.call(());
                                    return;
                                }
                            }
                            pending.set(false);
                        });
                    },
                    if pending() { "Checking…" } else { "Register app" }
                }
                if uncertain().is_some() {
                    button { class: "btn btn-outline-secondary ms-2", disabled: pending(), onclick: move |_| check_saved.call(()), "Check saved apps" }
                }
            }
        }
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    fn attempt() -> AppCreateAttempt {
        AppCreateAttempt {
            before_ids: vec!["old".into()],
            name: "  Match recorder  ".into(),
            homepage: "  https://example.com  ".into(),
            redirect_uris: redirect_uri_lines(
                " https://example.com/callback\n\nhttp://127.0.0.1/callback ",
            ),
            confidential: true,
            failure: "response lost".into(),
        }
    }

    fn client(id: &str) -> OAuthClient {
        OAuthClient {
            client_id: id.into(),
            name: "Match recorder".into(),
            homepage_url: Some("https://example.com".into()),
            redirect_uris: vec![
                "https://example.com/callback".into(),
                "http://127.0.0.1/callback".into(),
            ],
            confidential: true,
            created_at: None,
        }
    }

    #[test]
    fn app_creation_reconciliation_excludes_existing_identical_app() {
        assert!(!attempt().matches_new(&client("old")));
        assert!(attempt().matches_new(&client("new")));
    }

    #[test]
    fn app_creation_reconciliation_requires_all_submitted_fields_and_kind() {
        let attempt = attempt();
        let mut changed = client("new");
        changed.name = "Another app".into();
        assert!(!attempt.matches_new(&changed));
        changed = client("new");
        changed.homepage_url = None;
        assert!(!attempt.matches_new(&changed));
        changed = client("new");
        changed.redirect_uris.pop();
        assert!(!attempt.matches_new(&changed));
        changed = client("new");
        changed.confidential = false;
        assert!(!attempt.matches_new(&changed));
    }

    #[test]
    fn app_creation_reconciliation_accepts_empty_homepage_and_keeps_all_matches() {
        let mut attempt = attempt();
        attempt.homepage = "  ".into();
        let mut first = client("new-one");
        first.homepage_url = None;
        let mut second = first.clone();
        second.client_id = "new-two".into();
        let clients = [client("old"), first, second];
        let found: Vec<_> = clients
            .iter()
            .filter(|client| attempt.matches_new(client))
            .map(|client| client.client_id.as_str())
            .collect();
        assert_eq!(found, ["new-one", "new-two"]);
    }
}
