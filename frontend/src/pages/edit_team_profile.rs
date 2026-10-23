use crate::api;
use crate::components::{BannerAccountKind, BannerUpload, PhotoUpload, ChangePasswordCard, ConnectedAppsCard};
use crate::types::{UpdateTeamProfileRequest};
use dioxus::prelude::*;
use dioxus::prelude::use_navigator;
use crate::Route;

#[component]
pub fn EditTeamProfile(team_id: String) -> Element {
    let nav = use_navigator();
    let mut name = use_signal(|| "".to_string());
    let mut location = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut website = use_signal(|| "".to_string());
    let mut about = use_signal(|| "".to_string());
    let mut profile_photo = use_signal(|| None::<String>);
    let mut banner_photo = use_signal(|| None::<String>);
    let banner_busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let photo_busy = use_signal(|| false);

    let me = use_resource(move || async move { api::me().await });

    let _fetch = use_resource(use_reactive(&team_id, move |id| async move {
        loading.set(true);
        match api::team_profile(&id).await {
            Ok(res) => {
                let t = res.team;
                name.set(t.name);
                location.set(t.location.unwrap_or_default());
                email.set(t.email.unwrap_or_default());
                website.set(t.website.unwrap_or_default());
                about.set(t.about.unwrap_or_default());
                profile_photo.set(t.profile_photo);
                banner_photo.set(t.banner_photo);
            }
            Err(e) => error.set(Some(e)),
        }
        loading.set(false);
    }));

    let team_id_for_submit = team_id.clone();
    let onsubmit = move |_evt: Event<FormData>| {
        let team_id = team_id_for_submit.clone();
        async move {
            if banner_busy() || photo_busy() { return; }
            loading.set(true);
            error.set(None);
            
            let req = UpdateTeamProfileRequest {
                name: Some(name()),
                location: Some(location()),
                email: Some(email()),
                website: Some(website()),
                about: Some(about()),
            };

            match api::update_team_profile(&team_id, &req).await {
                Ok(_) => {
                    nav.push(Route::TeamProfilePage { id: team_id.clone() });
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        }
    };

    let own_profile = me.read().as_ref().and_then(|r| r.as_ref().ok()).map(|u| u.user_type == "team" && u.id == team_id).unwrap_or(false);

    rsx! {
        div { class: "row",
                div { class: "col-12",
                    h1 { "Edit Team Profile" }
                    nav { "aria-label": "breadcrumb",
                        ol { class: "breadcrumb",
                            li { class: "breadcrumb-item",
                                Link { to: Route::TeamProfilePage { id: team_id.clone() }, "{name}" }
                            }
                            li { class: "breadcrumb-item active", "Edit Profile" }
                        }
                    }
                }
            }
            
        if loading() {
            div { class: "d-flex justify-content-center",
                div { class: "spinner-border", role: "status",
                    span { class: "visually-hidden", "Loading..." }
                }
            }
        } else {
            div { class: "row justify-content-center",
                div { class: "col-md-8",
                    div { class: "card",
                        div { class: "card-header",
                            h5 { class: "mb-0", "Team Information" }
                        }
                        div { class: "card-body",
                            if let Some(err) = error() {
                                div { class: "alert alert-danger", "{err}" }
                            }
                            if own_profile {
                                PhotoUpload { kind: BannerAccountKind::Team, account_id: team_id.clone(), photo: profile_photo, busy: photo_busy }
                            }
                            if own_profile {
                                BannerUpload { kind: BannerAccountKind::Team, account_id: team_id.clone(), photo: banner_photo, busy: banner_busy }
                            }
                            form {
                                onsubmit: onsubmit,
                                div { class: "row",
                                        div { class: "col-md-6",
                                            div { class: "mb-3",
                                                label { class: "form-label", "Team Name" }
                                                input {
                                                    class: "form-control",
                                                    "type": "text",
                                                    value: "{name}",
                                                    oninput: move |e| name.set(e.value()),
                                                    required: true
                                                }
                                            }
                                        }
                                        div { class: "col-md-6",
                                            div { class: "mb-3",
                                                label { class: "form-label", "Location" }
                                                input {
                                                    class: "form-control",
                                                    "type": "text",
                                                    value: "{location}",
                                                    oninput: move |e| location.set(e.value())
                                                }
                                            }
                                        }
                                    }
                                    
                                    div { class: "row",
                                        div { class: "col-md-6",
                                            div { class: "mb-3",
                                                label { class: "form-label", "Email" }
                                                input {
                                                    class: "form-control",
                                                    "type": "email",
                                                    value: "{email}",
                                                    oninput: move |e| email.set(e.value())
                                                }
                                            }
                                        }
                                        div { class: "col-md-6",
                                            div { class: "mb-3",
                                                label { class: "form-label", "Website" }
                                                input {
                                                    class: "form-control",
                                                    "type": "url",
                                                    value: "{website}",
                                                    oninput: move |e| website.set(e.value())
                                                }
                                            }
                                        }
                                    }
                                    
                                    div { class: "mb-3",
                                        label { class: "form-label", "About" }
                                        textarea {
                                            class: "form-control",
                                            rows: "4",
                                            value: "{about}",
                                            oninput: move |e| about.set(e.value()),
                                            placeholder: "Tell people about your team"
                                        }
                                    }
                                    
                                    div { class: "d-grid gap-2",
                                        button { class: "btn btn-primary", "type": "submit", disabled: banner_busy() || photo_busy(), "Update Team Profile" }
                                        Link {
                                            class: "btn btn-outline-secondary",
                                            to: Route::TeamProfilePage { id: team_id.clone() },
                                            "Cancel"
                                        }
                                    }
                            }
                        }
                    }

                    if me.read().as_ref().and_then(|r| r.as_ref().ok()).map(|u| u.has_password).unwrap_or(false) {
                        ChangePasswordCard {}
                    }
                    if own_profile {
                        ConnectedAppsCard {}
                    }

                }
            }
        }
    }
}
