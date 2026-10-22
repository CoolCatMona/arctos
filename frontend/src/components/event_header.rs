//! Shared event identity, with an optional link to its parent league.

use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct EventHeaderProps {
    pub title: String,
    pub subtitle: String,
    /// When set, show a "Part of {name}" badge linking to the league.
    pub badge_league_url: Option<String>,
    pub badge_season: Option<String>,
    pub badge_name: Option<String>,
}

#[component]
pub fn EventHeader(props: EventHeaderProps) -> Element {
    let EventHeaderProps {
        title,
        subtitle,
        badge_league_url,
        badge_season: _,
        badge_name,
    } = props;
    rsx! {
        header { class: "arctos-event-hero",
            div { class: "arctos-event-hero-mark", aria_hidden: "true", i { class: "fas fa-trophy" } }
            div { class: "arctos-event-hero-content",
                if let (Some(url), Some(name)) = (badge_league_url, badge_name) {
                    Link { to: Route::LeagueHome { league_url: url }, class: "arctos-event-league-badge", i { class: "fas fa-trophy", aria_hidden: "true" } "Part of {name}" }
                }
                h1 { "{title}" }
                if !subtitle.trim().is_empty() { p { "{subtitle}" } }
            }
        }
    }
}
