use crate::api;
use crate::appearance::{focus_control, use_appearance};
use crate::components::AppearanceMenu;
use crate::Route;
use dioxus::prelude::*;

/// Signal bumped when login/logout succeeds so the layout refetches user.
pub fn use_auth_invalidate() -> Signal<u32> {
    use_context::<Signal<u32>>()
}

#[derive(Clone, Copy)]
struct HomeBannerHidden(Signal<bool>);

pub fn use_home_banner_hidden() -> Signal<bool> {
    use_context::<HomeBannerHidden>().0
}

#[cfg(target_arch = "wasm32")]
const INTRODUCTION_STORAGE_KEY: &str = "arctos.introduction.hidden.v1";

fn home_banner_is_hidden() -> bool {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        return storage
            .get_item(INTRODUCTION_STORAGE_KEY)
            .ok()
            .flatten()
            .as_deref()
            == Some("true");
    }
    false
}

pub fn remember_home_banner(hidden: bool) {
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(
            INTRODUCTION_STORAGE_KEY,
            if hidden { "true" } else { "false" },
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = hidden;
}

fn focus_restored_home_banner() {
    #[cfg(target_arch = "wasm32")]
    spawn(async move {
        // Let the shared signal remove the banner's hidden attribute first.
        gloo_timers::future::TimeoutFuture::new(0).await;
        focus_control("arctos-introduction-heading");
    });
}

/// Returns a short page name for the tab title: "Page Name | Arctos".
fn page_title_for_route(route: &Route) -> String {
    match route {
        Route::Index { .. } => "Home".into(),
        Route::NotFound { .. } => "Page not found".into(),
        Route::Login { .. } => "Login".into(),
        Route::Register { .. } => "Register".into(),
        Route::RegisterPlayer { .. } => "Register as Player".into(),
        Route::RegisterTeam { .. } => "Register as Team".into(),
        Route::GoogleChooseAccountType { .. } => "Choose Account Type".into(),
        Route::GoogleCompleteProfile { .. } => "Complete Profile".into(),
        Route::TournamentHome { url } | Route::TournamentHomeWithTab { url, .. } => {
            format!("{url}")
        }
        Route::Schedule { url, .. } => format!("{url} Schedule"),
        Route::ScheduleEdit { url } => format!("{url} Edit Schedule"),
        Route::Results { url } => format!("{url} Results"),
        Route::Bracket { url } => format!("{url} Bracket"),
        Route::LegacyBracket { url } => format!("{url} Legacy Bracket"),
        Route::TournamentSettings { url } => format!("{url} Settings"),
        Route::TournamentRegister { url } => format!("{url} Register"),
        Route::Manage { url } => format!("{url} Manage"),
        Route::ManageFootage { url } => format!("{url} Footage"),
        Route::Invitations { url } => format!("{url} Roster"),
        Route::StartMatch { .. } => format!("Start Match").into(),
        Route::RunMatch { .. } => format!("Run Match").into(),
        Route::FinalizeMatch { .. } => format!("Finalize Match").into(),
        Route::Scoreboard { .. } => format!("Scoreboard"),
        Route::ScoreboardRender { .. } => "Scoreboard Render".into(),
        Route::MatchPage { .. } => format!("Match"),
        Route::MatchPageById { .. } => format!("Match").into(),
        Route::AddInjury { .. } => "Add Injury".into(),
        Route::EditInjury { .. } => "Edit Injury".into(),
        Route::EditPlayerProfile { .. } => "Edit Profile".into(),
        Route::EditTeamProfile { .. } => "Edit Profile".into(),
        Route::PlayersList { .. } => "Players".into(),
        Route::PlayerProfilePage { id } => format!("{id}"),
        Route::TeamsList { .. } => "Teams".into(),
        Route::TeamProfilePage { id } => format!("{id}"),
        Route::Stones { .. } => "Stones".into(),
        Route::About { .. } => "About".into(),
        Route::NewTournament { .. } => "Create Tournament".into(),
        Route::LeaguesIndex { .. } => "Leagues".into(),
        Route::CreateEvent { .. } => "Create event".into(),
        Route::NewLeague { .. } => "Create League".into(),
        Route::LeagueHome { league_url } => league_url.clone(),
        Route::LeagueRegister { .. } => "Register".into(),
        Route::LeagueResults { .. } => "Results".into(),
        Route::LeagueSettings { .. } => "Settings".into(),
        Route::LeagueNewTournament { league_url } => format!("Add Event | {league_url}"),
        Route::LeagueManage { league_url } => format!("{league_url} Manage"),
        Route::LeagueInvitations { league_url } => format!("{league_url} Roster"),
        Route::Docs { .. } => "User Docs".into(),
        Route::Privacy { .. } => "Privacy Policy".into(),
        Route::Terms { .. } => "Terms".into(),
        Route::Thanks { .. } => "Thanks".into(),
        Route::License { .. } => "License".into(),
        Route::ArctosScheduleScript { .. } => "Arctos Schedule Script".into(),
        Route::DataAccessibilityGuide { .. } => "Data Accessibility Guide".into(),
        Route::Developers { .. } => "For Developers".into(),
        Route::DeveloperApps { .. } => "Your Apps".into(),
        Route::OAuthAuthorize { .. } => "Sign in with Arctos".into(),
        Route::SideCompNew { url } => format!("{url} New Side Competition"),
        Route::SideCompDetail { url, .. } => format!("{url} Side Competition"),
        Route::SideCompEdit { url, .. } => format!("{url} Edit Side Competition"),
        Route::SideCompRegisterAsTo { url, .. } => format!("{url} Side Competition Quick Register"),
        Route::SideCompEnterResults { url, .. } => format!("{url} Enter Results"),
        Route::SideCompResults { url, .. } => format!("{url} Side Comp Results"),
        Route::SideCompManageResults { url, .. } => format!("{url} Manage Side Comp Results"),
    }
}

const BOOTSTRAP: Asset = asset!("/assets/vendor/bootstrap.min.css");
const ICONS: Asset = asset!("/assets/vendor/fontawesome");

#[component]
pub fn Layout() -> Element {
    let auth_version = use_signal(|| 0u32);
    provide_context(auth_version);
    let mut introduction_hidden = use_signal(home_banner_is_hidden);
    provide_context(HomeBannerHidden(introduction_hidden));
    let user = use_resource(move || {
        let _ = auth_version();
        async move { api::me().await }
    });
    let mut account_open = use_signal(|| false);
    let mut utility_open = use_signal(|| false);
    let mut appearance_open = use_signal(|| false);
    let appearance = use_appearance();
    let palette = appearance.preferences().palette.id();
    let color_scheme = appearance.scheme();
    let mut logging_out = use_signal(|| false);
    let mut auth_error = use_signal(|| None::<String>);
    let navigator = use_navigator();
    let route = use_route::<Route>();
    let page_title = format!("{} | Arctos", page_title_for_route(&route));
    let path = route.to_string();
    let current_user = user
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned();
    let account_label = current_user.as_ref().map_or_else(String::new, |user| {
        format!("Account: {} ({})", user.name, user.user_type)
    });
    use_effect(use_reactive(&route, move |_| {
        account_open.set(false);
        utility_open.set(false);
        appearance_open.set(false);
    }));

    // Exact-size offline frames and OBS overlays intentionally have no app theme.
    if matches!(route, Route::ScoreboardRender { .. }) {
        return rsx! { Outlet::<Route> {} };
    }
    if matches!(route, Route::Scoreboard { .. }) {
        return rsx! {
            Title { "{page_title}" }
            link { rel: "stylesheet", href: BOOTSTRAP }
            link { rel: "stylesheet", href: "{ICONS}/css/all.min.css" }
            Outlet::<Route> {}
        };
    }
    // Keep the side competition keypad's full-height, purpose-built mobile layout.
    if matches!(route, Route::SideCompEnterResults { .. }) {
        return rsx! {
            Title { "{page_title}" }
            link { rel: "stylesheet", href: BOOTSTRAP }
            link { rel: "stylesheet", href: "{ICONS}/css/all.min.css" }
            style { {include_str!("../theme.css")} }
            div {
                class: "arctos-app arctos-scoring-surface",
                "data-palette": palette, "data-bs-theme": color_scheme,
                Outlet::<Route> {}
            }
        };
    }

    rsx! {
        Title { "{page_title}" }
        link { rel: "stylesheet", href: BOOTSTRAP }
        link { rel: "stylesheet", href: "{ICONS}/css/all.min.css" }
        style { {include_str!("../theme.css")} }
        style { {include_str!("public_pages.css")} }
        style { {include_str!("operator.css")} }
        div {
            class: "arctos-app",
            "data-palette": palette, "data-bs-theme": color_scheme,
            onkeydown: move |event| {
                if event.key() == Key::Escape && (account_open() || utility_open() || appearance_open()) {
                    event.prevent_default();
                    let trigger = if account_open() { "account-trigger" } else { "utility-trigger" };
                    appearance_open.set(false);
                    utility_open.set(false);
                    account_open.set(false);
                    focus_control(trigger);
                }
            },
            a { class: "skip-link", href: "#page-content", "Skip to content" }
            header { class: "app-header",
                div { class: "app-header-inner",
                    Link { to: Route::Index {}, class: "app-brand", aria_label: "Arctos home",
                        span { class: "brand-mark", aria_hidden: "true", i { class: "fa-solid fa-trophy" } }
                        span { "Arctos" }
                    }
                    nav { class: "app-nav", aria_label: "Main navigation",
                        for (to, label, icon, active) in [
                            (Route::Index {}, "Home", "fa-house", path == "/"),
                            (Route::TeamsList {}, "Teams", "fa-users", path.starts_with("/teams")),
                            (Route::PlayersList {}, "Players", "fa-user-group", path.starts_with("/players")),
                            (Route::Stones {}, "Stones", "fa-stopwatch", path == "/stones"),
                            (Route::LeaguesIndex {}, "Leagues", "fa-trophy", path.starts_with("/leagues")),
                        ] {
                            Link {
                                to,
                                class: if active { "app-nav-link active" } else { "app-nav-link" },
                                aria_current: if active { "page" } else { "false" },
                                i { class: "fa-solid {icon}", aria_hidden: "true" }
                                span { "{label}" }
                            }
                        }
                    }
                    div { class: "app-header-actions",
                        Link { to: Route::CreateEvent {}, class: "btn btn-primary app-create", aria_label: "Create event",
                            i { class: "fa-solid fa-circle-plus", aria_hidden: "true" }
                            span { "Create event" }
                        }
                        if let Some(u) = current_user.as_ref() {
                            div { class: "account-menu",
                                button {
                                    class: "app-icon-button account-trigger",
                                    id: "account-trigger", r#type: "button",
                                    aria_label: "{account_label}", aria_expanded: "{account_open()}",
                                    aria_controls: "account-options",
                                    onclick: move |_| {
                                        utility_open.set(false);
                                        appearance_open.set(false);
                                        let was_open = account_open();
                                        account_open.set(!was_open);
                                    },
                                    i { class: "fa-regular fa-user", aria_hidden: "true" }
                                }
                                if account_open() {
                                    button {
                                        class: "account-dismiss", r#type: "button", tabindex: "-1",
                                        aria_label: "Dismiss account menu",
                                        onclick: move |_| {
                                            account_open.set(false);
                                            focus_control("account-trigger");
                                        }
                                    }
                                    div {
                                        class: "account-dropdown profile-dropdown", id: "account-options", tabindex: "-1",
                                        onmounted: move |_| focus_control("account-profile-link"),
                                        div { class: "account-identity",
                                            strong { "{u.name}" }
                                            span { class: "text-muted", "{u.user_type} account · @{u.id}" }
                                        }
                                        if u.user_type == "player" {
                                            Link { to: Route::PlayerProfilePage { id: u.id.clone() }, id: "account-profile-link", class: "dropdown-item", "My profile" }
                                        } else {
                                            Link { to: Route::TeamProfilePage { id: u.id.clone() }, id: "account-profile-link", class: "dropdown-item", "Team profile" }
                                        }
                                        Link { to: Route::DeveloperApps {}, class: "dropdown-item", "Developer apps" }
                                        button {
                                            r#type: "button", class: "dropdown-item", disabled: logging_out(),
                                            onclick: move |_| {
                                                if logging_out() { return; }
                                                logging_out.set(true);
                                                auth_error.set(None);
                                                let nav = navigator;
                                                let mut auth_version = auth_version;
                                                spawn(async move {
                                                    match api::logout().await {
                                                        Ok(_) => {
                                                            auth_version += 1;
                                                            account_open.set(false);
                                                            nav.push(Route::Index {});
                                                        }
                                                        Err(e) => auth_error.set(Some(e)),
                                                    }
                                                    logging_out.set(false);
                                                });
                                            },
                                            if logging_out() { "Logging out…" } else { "Log out" }
                                        }
                                    }
                                }
                            }
                        } else {
                            Link { to: Route::Login {}, class: "btn btn-outline-secondary app-login", "Log in" }
                        }
                        div { class: "account-menu utility-menu",
                            button {
                                class: "app-icon-button account-trigger utility-trigger",
                                id: "utility-trigger", r#type: "button", aria_label: "More options",
                                aria_expanded: "{utility_open() || appearance_open()}",
                                aria_controls: if appearance_open() { "appearance-options" } else { "utility-options" },
                                onclick: move |_| {
                                    let was_open = utility_open() || appearance_open();
                                    account_open.set(false);
                                    appearance_open.set(false);
                                    utility_open.set(!was_open);
                                },
                                i { class: "fa-solid fa-ellipsis", aria_hidden: "true" }
                            }
                            if utility_open() || appearance_open() {
                                button {
                                    class: "account-dismiss utility-dismiss", r#type: "button", tabindex: "-1",
                                    aria_label: "Dismiss More options",
                                    onclick: move |_| {
                                        utility_open.set(false);
                                        appearance_open.set(false);
                                        focus_control("utility-trigger");
                                    }
                                }
                            }
                            if utility_open() {
                                div {
                                    class: "account-dropdown utility-dropdown", id: "utility-options", tabindex: "-1",
                                    onmounted: move |_| focus_control("appearance-menu-item"),
                                    button {
                                        id: "appearance-menu-item", r#type: "button", class: "dropdown-item utility-menu-row",
                                        aria_haspopup: "dialog", aria_controls: "appearance-options",
                                        onclick: move |_| { utility_open.set(false); appearance_open.set(true); },
                                        i { class: "fa-solid fa-palette", aria_hidden: "true" }
                                        span { "Appearance" }
                                        i { class: "fa-solid fa-chevron-right menu-chevron", aria_hidden: "true" }
                                    }
                                    Link { to: Route::Docs {}, class: "dropdown-item", "Help & documentation" }
                                    Link { to: Route::About {}, class: "dropdown-item", "About Arctos" }
                                    if path == "/" && introduction_hidden() {
                                        button {
                                            id: "arctos-introduction-restore", r#type: "button", class: "dropdown-item",
                                            onclick: move |_| {
                                                introduction_hidden.set(false);
                                                remember_home_banner(false);
                                                utility_open.set(false);
                                                focus_restored_home_banner();
                                            },
                                            "Show home banner"
                                        }
                                    }
                                }
                            }
                            AppearanceMenu { open: appearance_open, utility_open, appearance }
                        }
                    }
                }
            }
            main { class: "container-wide app-main", id: "page-content", tabindex: "-1",
                if let Some(error) = auth_error() {
                    div { class: "alert alert-danger", role: "alert", "Could not log out. {error}" }
                }
                SuspenseBoundary {
                    fallback: |_| rsx! { div { class: "loading-state", role: "status", "Loading…" } },
                    Outlet::<Route> {}
                }
            }
            footer { class: "app-footer",
                div { class: "container-wide",
                    div { class: "footer-top",
                        nav { class: "footer-links", aria_label: "Help and information",
                            Link { to: Route::About {}, "About" }
                            Link { to: Route::Docs {}, "Documentation" }
                            Link { to: Route::Developers {}, "Developers" }
                            Link { to: Route::Thanks {}, "Thanks" }
                            Link { to: Route::Privacy {}, "Privacy" }
                            Link { to: Route::Terms {}, "Terms" }
                            Link { to: Route::License {}, "License" }
                        }
                    }
                    div { class: "footer-bottom",
                        p { a { href: "https://github.com/reid23/arctos", "Source code" } }
                        p { "An independent project, not affiliated with CAJA or the US NJA." }
                    }
                }
            }
        }
    }
}
