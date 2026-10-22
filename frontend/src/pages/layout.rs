use crate::api;
use crate::appearance::{focus_control, use_appearance};
use crate::components::AppearanceMenu;
use crate::Route;
use dioxus::prelude::*;

/// Signal bumped when login/logout succeeds so the layout refetches user.
pub fn use_auth_invalidate() -> Signal<u32> {
    use_context::<Signal<u32>>()
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
    let user = use_resource(move || {
        let _ = auth_version();
        async move { api::me().await }
    });
    let mut account_open = use_signal(|| false);
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
    let menu_label = current_user.as_ref().map_or_else(
        || "More options".to_string(),
        |user| format!("Account: {} ({})", user.name, user.user_type),
    );
    use_effect(use_reactive(&route, move |_| {
        account_open.set(false);
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
                if event.key() == Key::Escape && (account_open() || appearance_open()) {
                    event.prevent_default();
                    appearance_open.set(false);
                    account_open.set(false);
                    focus_control("account-trigger");
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
                        if current_user.is_none() {
                            Link { to: Route::Login {}, class: "btn btn-outline-secondary app-login", "Log in" }
                        }
                        div { class: "account-menu",
                            button {
                                class: "app-icon-button account-trigger",
                                id: "account-trigger",
                                r#type: "button",
                                aria_label: "{menu_label}",
                                aria_expanded: "{account_open() || appearance_open()}",
                                aria_controls: if appearance_open() { "appearance-options" } else { "account-options" },
                                onclick: move |_| {
                                    let was_open = account_open() || appearance_open();
                                    appearance_open.set(false);
                                    account_open.set(!was_open);
                                },
                                if current_user.is_some() {
                                    i { class: "fa-regular fa-user", aria_hidden: "true" }
                                } else {
                                    i { class: "fa-solid fa-ellipsis", aria_hidden: "true" }
                                }
                            }
                            if account_open() || appearance_open() {
                                button {
                                    class: "account-dismiss", r#type: "button", tabindex: "-1", aria_label: "Dismiss menu",
                                    onclick: move |_| {
                                        account_open.set(false);
                                        appearance_open.set(false);
                                        focus_control("account-trigger");
                                    }
                                }
                            }
                            if account_open() {
                                div {
                                    class: "account-dropdown", id: "account-options",
                                    onmounted: move |_| focus_control("appearance-menu-item"),
                                    if let Some(u) = current_user.as_ref() {
                                        div { class: "account-identity",
                                            strong { "{u.name}" }
                                            span { class: "text-muted", "{u.user_type} account · @{u.id}" }
                                        }
                                        if u.user_type == "player" {
                                            Link { to: Route::PlayerProfilePage { id: u.id.clone() }, class: "dropdown-item", "My profile" }
                                        } else {
                                            Link { to: Route::TeamProfilePage { id: u.id.clone() }, class: "dropdown-item", "Team profile" }
                                        }
                                        Link { to: Route::DeveloperApps {}, class: "dropdown-item", "Developer apps" }
                                    }
                                    button {
                                        id: "appearance-menu-item", r#type: "button", class: "dropdown-item utility-menu-row",
                                        aria_haspopup: "dialog", aria_controls: "appearance-options",
                                        onclick: move |_| { account_open.set(false); appearance_open.set(true); },
                                        i { class: "fa-solid fa-palette", aria_hidden: "true" }
                                        span { "Appearance" }
                                        i { class: "fa-solid fa-chevron-right menu-chevron", aria_hidden: "true" }
                                    }
                                    Link { to: Route::Docs {}, class: "dropdown-item", "Help & documentation" }
                                    Link { to: Route::About {}, class: "dropdown-item", "About Arctos" }
                                    if current_user.is_some() {
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
                            AppearanceMenu { open: appearance_open, account_open, appearance }
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
                        div { class: "footer-brand",
                            span { class: "brand-mark", aria_hidden: "true", i { class: "fa-solid fa-trophy" } }
                            span { "Arctos Tournament Operations" }
                        }
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
                        p { "Made for the Jugger community. " a { href: "https://github.com/reid23/arctos", "Open source" } "." }
                        p { "An independent project, not affiliated with CAJA or the US NJA." }
                    }
                }
            }
        }
    }
}
