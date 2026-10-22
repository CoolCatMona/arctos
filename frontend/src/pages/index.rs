use crate::api;
use crate::types::{Tournament, UserRegStatus};
use crate::Route;
use chrono::{Local, NaiveDate};
use dioxus::prelude::*;

fn format_date(iso: &str) -> String {
    let date = iso.split('T').next().unwrap_or(iso);
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|date| date.format("%b %-d, %Y").to_string())
        .unwrap_or_else(|_| date.to_string())
}

fn format_date_display(start: &str, end: Option<&String>) -> String {
    let start_fmt = format_date(start);
    match end {
        Some(end) if format_date(end) != start_fmt => {
            format!("{} – {}", start_fmt, format_date(end))
        }
        _ => start_fmt,
    }
}

/// An event stays upcoming until its final day has passed in the user's timezone.
fn is_past_in_local_tz(t: &Tournament) -> bool {
    let end_str = t.end_date.as_deref().unwrap_or(&t.start_date);
    let date_str = end_str.get(..10).unwrap_or(end_str);
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(end_date) => Local::now().date_naive() > end_date,
        Err(_) => false,
    }
}

fn split_upcoming_past(tournaments: &[Tournament]) -> (Vec<Tournament>, Vec<Tournament>) {
    let (mut past, mut upcoming): (Vec<_>, Vec<_>) =
        tournaments.iter().cloned().partition(is_past_in_local_tz);
    upcoming.sort_by(|a, b| a.start_date.cmp(&b.start_date));
    past.sort_by(|a, b| {
        let ea = a.end_date.as_deref().unwrap_or(&a.start_date);
        let eb = b.end_date.as_deref().unwrap_or(&b.start_date);
        eb.cmp(ea)
    });
    (upcoming, past)
}

#[component]
pub fn Index() -> Element {
    let mut tournaments =
        use_resource(move || async move { api::tournaments().await.map_err(|e| e.to_string()) });
    let mut show_past = use_signal(|| false);
    let mut search = use_signal(String::new);
    let val = tournaments.value();

    rsx! {
        section { class: "arctos-discovery-hero",
            div { class: "arctos-brand-mark", aria_hidden: "true", i { class: "fas fa-trophy" } }
            h1 { "Jugger Tournament Operations" }
            p { "Find your next event, follow the action, and keep match day running smoothly. Registration, schedules, live scores, and results in one place." }
            div { class: "arctos-hero-actions",
                Link { to: Route::CreateEvent {}, class: "btn btn-primary", i { class: "fas fa-plus-circle", aria_hidden: "true" } "Create event" }
                Link { to: Route::LeaguesIndex {}, class: "btn btn-outline-secondary", i { class: "fas fa-trophy", aria_hidden: "true" } "Browse leagues" }
            }
        }
        section { class: "arctos-discovery", aria_label: "Find an event",
            if let Some(Ok(data)) = val.read().as_ref() {
                {
                    let (upcoming, past) = split_upcoming_past(&data.tournaments);
                    let active = if show_past() { &past } else { &upcoming };
                    let query = search().trim().to_lowercase();
                    let filtered: Vec<_> = active.iter().filter(|t| {
                        t.name.to_lowercase().contains(&query)
                            || t.location.as_deref().unwrap_or("").to_lowercase().contains(&query)
                            || t.league.as_ref().is_some_and(|league| league.name.to_lowercase().contains(&query))
                    }).collect();
                    rsx! {
                        div { class: "arctos-discovery-toolbar",
                            div { class: "arctos-segmented", role: "group", aria_label: "Event dates",
                                button {
                                    r#type: "button",
                                    class: if !show_past() { "arctos-segment is-active" } else { "arctos-segment" },
                                    aria_pressed: "{!show_past()}",
                                    onclick: move |_| show_past.set(false),
                                    "Upcoming " span { class: "arctos-count", "{upcoming.len()}" }
                                }
                                button {
                                    r#type: "button",
                                    class: if show_past() { "arctos-segment is-active" } else { "arctos-segment" },
                                    aria_pressed: "{show_past()}",
                                    onclick: move |_| show_past.set(true),
                                    "Past " span { class: "arctos-count", "{past.len()}" }
                                }
                            }
                            div { class: "arctos-event-search",
                                label { class: "visually-hidden", r#for: "event-search", "Search event names, locations, or leagues" }
                                i { class: "fas fa-search", aria_hidden: "true" }
                                input {
                                    id: "event-search", r#type: "search", class: "form-control",
                                    placeholder: "Find an event or location…", value: "{search()}",
                                    oninput: move |event| search.set(event.value()),
                                }
                            }
                        }
                        h2 { class: "visually-hidden", if show_past() { "Past events" } else { "Upcoming events" } }
                        if filtered.is_empty() {
                            div { class: "arctos-empty-state", role: "status",
                                i { class: "fas fa-calendar-alt", aria_hidden: "true" }
                                h3 { if !query.is_empty() { "No matching events" } else if show_past() { "No past events yet" } else { "No upcoming events yet" } }
                                p { if !query.is_empty() { "Try another event name or location." } else if show_past() { "Completed events will appear here after their final day." } else { "Check back for new events, or explore the leagues." } }
                                if !query.is_empty() {
                                    button { class: "btn btn-outline-secondary", r#type: "button", onclick: move |_| search.set(String::new()), "Clear search" }
                                } else if !show_past() {
                                    Link { to: Route::LeaguesIndex {}, class: "btn btn-outline-secondary", "Browse leagues" }
                                }
                            }
                        } else {
                            div { class: "arctos-card-grid",
                                for t in filtered {
                                    TournamentCard {
                                        key: "{t.url}",
                                        tournament: t.clone(),
                                        team_count: data.team_counts.get(&t.url).copied().unwrap_or(0),
                                        user_reg_status: data.user_reg_status.get(&t.url).cloned(),
                                        past: show_past(),
                                    }
                                }
                            }
                        }
                    }
                }
            } else if let Some(Err(error)) = val.read().as_ref() {
                div { class: "arctos-empty-state", role: "alert",
                    h2 { "Events could not be loaded" }
                    p { "{error}" }
                    button { class: "btn btn-primary", r#type: "button", onclick: move |_| tournaments.restart(), "Try again" }
                }
            } else {
                div { class: "arctos-empty-state", role: "status", "Loading events…" }
            }
        }
    }
}

#[component]
fn TournamentCard(
    tournament: Tournament,
    team_count: u32,
    user_reg_status: Option<UserRegStatus>,
    past: bool,
) -> Element {
    let (teams_open, players_open) = tournament
        .league
        .as_ref()
        .map(|league| {
            (
                league.team_registration_open,
                league.player_registration_open,
            )
        })
        .unwrap_or((
            tournament.team_registration_open,
            tournament.player_registration_open,
        ));
    let registration_label = match (teams_open, players_open) {
        (true, true) => "Registration open",
        (true, false) => "Team registration open",
        (false, true) => "Player registration open",
        _ => "Registration closed",
    };
    rsx! {
        article { class: "card arctos-event-card",
            div { class: "card-body",
                div { class: "arctos-card-statuses",
                    if !tournament.published { span { class: "badge bg-warning text-dark", "Unpublished" } }
                    if past {
                        span { class: "badge arctos-badge-neutral", "Past event" }
                    } else {
                        span { class: if teams_open || players_open { "badge arctos-badge-success" } else { "badge arctos-badge-neutral" }, "{registration_label}" }
                    }
                }
                h3 { class: "arctos-card-title",
                    Link { to: Route::TournamentHome { url: tournament.url.clone() }, "{tournament.name}" }
                }
                if let Some(ref league) = tournament.league {
                    Link {
                        to: Route::LeagueHome { league_url: league.league_url.clone() },
                        class: "arctos-league-link",
                        i { class: "fas fa-trophy", aria_hidden: "true" }
                        "{league.name}"
                    }
                }
                ul { class: "arctos-meta-list",
                    li { i { class: "fas fa-map-marker-alt", aria_hidden: "true" } "{tournament.location.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or(\"Location to be announced\")}" }
                    li { i { class: "far fa-calendar-alt", aria_hidden: "true" } "{format_date_display(&tournament.start_date, tournament.end_date.as_ref())}" }
                    li {
                        i { class: "fas fa-users", aria_hidden: "true" }
                        if let Some(max) = tournament.n_max_teams {
                            "{team_count} / {max} teams registered"
                        } else { "{team_count} teams registered" }
                    }
                }
                if let Some(urs) = user_reg_status {
                    div { class: "arctos-registration-summary", aria_label: "Your registration", UserRegBadges { urs } }
                }
            }
            div { class: "arctos-card-footer",
                Link { to: Route::TournamentHome { url: tournament.url }, class: "btn btn-outline-secondary w-100", "View event" i { class: "fas fa-arrow-right", aria_hidden: "true" } }
            }
        }
    }
}

#[component]
pub(super) fn UserRegBadges(urs: UserRegStatus) -> Element {
    let (status_class, status_label) = match urs.status.as_str() {
        "CONFIRMED" => ("arctos-badge-success", "Confirmed"),
        "PENDING_TEAM_APPROVAL" => ("arctos-badge-warning", "Awaiting team approval"),
        "REJECTED" => ("arctos-badge-danger", "Declined"),
        "CANCELLED" => ("arctos-badge-neutral", "Cancelled"),
        _ => ("arctos-badge-neutral", "Status unavailable"),
    };
    let (waiver_class, waiver_label) = match urs.waiver_status.as_deref().unwrap_or("NOT_SIGNED") {
        "VALID" => ("arctos-badge-success", "Waiver valid"),
        "OUT_OF_DATE" => ("arctos-badge-warning", "Waiver needs re-signing"),
        "NOT_SIGNED" => ("arctos-badge-danger", "Waiver not signed"),
        _ => ("arctos-badge-neutral", "Waiver status unknown"),
    };
    let account_label = if urs.reg_type == "team" {
        "Team"
    } else {
        "Player"
    };
    rsx! {
        span { class: "badge {status_class}", "{account_label}: {status_label}" }
        span { class: if urs.paid { "badge arctos-badge-success" } else { "badge arctos-badge-warning" }, if urs.paid { "Paid" } else { "Unpaid" } }
        if urs.waiver_required { span { class: "badge {waiver_class}", "{waiver_label}" } }
    }
}
