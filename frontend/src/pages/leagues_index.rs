use super::index::UserRegBadges;
use crate::api;
use crate::types::{LeagueInfo, UserRegStatus};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn LeaguesIndex() -> Element {
    let mut data = use_resource(move || async move {
        api::leagues_list().await.map_err(|error| error.to_string())
    });
    let val = data.value();
    rsx! {
        header { class: "arctos-page-intro",
            h1 { "Leagues" }
            p { "Event series with shared registration and standings." }
        }
        if let Some(Ok(response)) = val.read().as_ref() {
            if response.leagues.is_empty() {
                div { class: "arctos-empty-state", role: "status",
                    i { class: "fas fa-trophy", aria_hidden: "true" }
                    h2 { "No leagues yet" }
                    p { "Create a league for events with shared registration." }
                    Link { to: Route::NewLeague {}, class: "btn btn-primary", "Create a league" }
                }
            } else {
                div { class: "arctos-card-grid arctos-league-grid",
                    for league in response.leagues.iter() {
                        LeagueCard {
                            key: "{league.league_url}",
                            league: league.clone(),
                            team_count: response.team_counts.get(&league.league_url).copied().unwrap_or(0),
                            user_reg_status: response.user_reg_status.get(&league.league_url).cloned(),
                        }
                    }
                }
            }
        } else if let Some(Err(error)) = val.read().as_ref() {
            div { class: "arctos-empty-state", role: "alert",
                h2 { "Leagues could not be loaded" }
                p { "{error}" }
                button { class: "btn btn-primary", r#type: "button", onclick: move |_| data.restart(), "Try again" }
            }
        } else {
            div { class: "arctos-empty-state", role: "status", "Loading leagues…" }
        }
    }
}

#[component]
fn LeagueCard(
    league: LeagueInfo,
    team_count: u32,
    user_reg_status: Option<UserRegStatus>,
) -> Element {
    let registration_label = match (
        league.team_registration_open,
        league.player_registration_open,
    ) {
        (true, true) => "Registration open",
        (true, false) => "Team registration open",
        (false, true) => "Player registration open",
        _ => "Registration closed",
    };
    rsx! {
        article { class: "card arctos-event-card",
            div { class: "card-body",
                div { class: "arctos-card-statuses",
                    if !league.published { span { class: "badge arctos-badge-warning", "Unpublished" } }
                    span { class: if league.team_registration_open || league.player_registration_open { "badge arctos-badge-success" } else { "badge arctos-badge-neutral" }, "{registration_label}" }
                }
                div { class: "arctos-league-identity",
                    div { class: "arctos-brand-mark", aria_hidden: "true", i { class: "fas fa-trophy" } }
                    div {
                        h2 { class: "arctos-card-title", Link { to: Route::LeagueHome { league_url: league.league_url.clone() }, "{league.name}" } }
                        p { class: "arctos-league-caption", "Shared registration" }
                    }
                }
                ul { class: "arctos-meta-list",
                    li { i { class: "fas fa-users", aria_hidden: "true" }
                        if let Some(max) = league.n_max_teams { "{team_count} / {max} teams registered" } else { "{team_count} teams registered" }
                    }
                }
                if let Some(urs) = user_reg_status {
                    div { class: "arctos-registration-summary", aria_label: "Your registration", UserRegBadges { urs } }
                }
            }
            div { class: "arctos-card-footer",
                Link { to: Route::LeagueHome { league_url: league.league_url }, class: "btn btn-outline-secondary w-100", "View league" i { class: "fas fa-arrow-right", aria_hidden: "true" } }
            }
        }
    }
}
