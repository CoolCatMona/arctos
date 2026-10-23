use crate::api;
use crate::components::{BannerAccountKind, BannerUpload, PhotoUpload, ChangePasswordCard, ConnectedAppsCard};
use crate::types::{UpdatePlayerProfileRequest};
use dioxus::prelude::*;
use dioxus::prelude::use_navigator;
use crate::Route;

#[component]
pub fn EditPlayerProfile(player_id: String) -> Element {
    let nav = use_navigator();
    let mut name = use_signal(|| "".to_string());
    let mut phone = use_signal(|| "".to_string());
    let mut location = use_signal(|| "".to_string());
    let mut bio = use_signal(|| "".to_string());
    let mut profile_photo = use_signal(|| None::<String>);
    let mut banner_photo = use_signal(|| None::<String>);
    let banner_busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let photo_busy = use_signal(|| false);

    let me = use_resource(move || async move { api::me().await });

    let _fetch = use_resource(use_reactive(&player_id, move |id| async move {
        loading.set(true);
        match api::player_profile(&id).await {
            Ok(res) => {
                let p = res.player;
                name.set(p.name);
                phone.set(p.phone.unwrap_or_default());
                location.set(p.location.unwrap_or_default());
                bio.set(p.bio.unwrap_or_default());
                profile_photo.set(p.profile_photo);
                banner_photo.set(p.banner_photo);
            }
            Err(e) => error.set(Some(e)),
        }
        loading.set(false);
    }));

    let player_id_for_submit = player_id.clone();
    let onsubmit = move |_evt: Event<FormData>| {
        let player_id = player_id_for_submit.clone();
        async move {
            if banner_busy() || photo_busy() { return; }
            loading.set(true);
            error.set(None);
            
            let req = UpdatePlayerProfileRequest {
                name: Some(name()),
                phone: Some(phone()),
                location: Some(location()),
                bio: Some(bio()),
            };

            match api::update_player_profile(&player_id, &req).await {
                Ok(_) => {
                    nav.push(Route::PlayerProfilePage { id: player_id.clone() });
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        }
    };

    let own_profile = me.read().as_ref().and_then(|r| r.as_ref().ok()).map(|u| u.user_type == "player" && u.id == player_id).unwrap_or(false);

    rsx! {
        div { class: "row",
                div { class: "col-12",
                    h1 { "Edit Player Profile" }
                    nav { "aria-label": "breadcrumb",
                        ol { class: "breadcrumb",
                            li { class: "breadcrumb-item",
                                Link { to: Route::PlayerProfilePage { id: player_id.clone() }, "{name}" }
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
                div { class: "col-md-6",
                    div { class: "card",
                        div { class: "card-header",
                            h5 { class: "mb-0", "Player Information" }
                        }
                        div { class: "card-body",
                            if let Some(err) = error() {
                                div { class: "alert alert-danger", "{err}" }
                            }
                            if own_profile {
                                PhotoUpload { kind: BannerAccountKind::Player, account_id: player_id.clone(), photo: profile_photo, busy: photo_busy }
                            }
                            if own_profile {
                                BannerUpload { kind: BannerAccountKind::Player, account_id: player_id.clone(), photo: banner_photo, busy: banner_busy }
                            }
                            form {
                                onsubmit: onsubmit,
                                div { class: "mb-3",
                                        label { class: "form-label", "Display Name" }
                                        input {
                                            class: "form-control",
                                            "type": "text",
                                            value: "{name}",
                                            oninput: move |e| name.set(e.value()),
                                            required: true
                                        }
                                    }
                                    
                                    div { class: "mb-3",
                                        label { class: "form-label", "Phone Number" }
                                        input {
                                            class: "form-control",
                                            "type": "text",
                                            value: "{phone}",
                                            oninput: move |e| phone.set(e.value())
                                        }
                                        div { class: "form-text", "Optional - for notifications (not public)" }
                                    }
                                    
                                    div { class: "mb-3",
                                        label { class: "form-label", "Location" }
                                        input {
                                            class: "form-control",
                                            "type": "text",
                                            value: "{location}",
                                            oninput: move |e| location.set(e.value()),
                                            placeholder: "e.g., Seattle, WA"
                                        }
                                        div { class: "form-text", "Optional - your general location" }
                                    }
                                    
                                    div { class: "mb-3",
                                        label { class: "form-label", "Bio" }
                                        textarea {
                                            class: "form-control",
                                            rows: "3",
                                            value: "{bio}",
                                            oninput: move |e| bio.set(e.value()),
                                            placeholder: "Tell us about yourself..."
                                        }
                                        div { class: "form-text", "Optional - a brief description about yourself" }
                                    }
                                    
                                    div { class: "d-grid gap-2",
                                        button { class: "btn btn-primary", "type": "submit", disabled: banner_busy() || photo_busy(), "Update Profile" }
                                        Link {
                                            class: "btn btn-outline-secondary",
                                            to: Route::PlayerProfilePage { id: player_id.clone() },
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
