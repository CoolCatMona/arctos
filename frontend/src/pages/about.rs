use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn About() -> Element {
    let features = [
        ("fa-calendar-alt", "Flexible scheduling", "Publish schedules that fit your tournament structure, with fields, referees, and dynamic match dependencies."),
        ("fa-users", "Registration and rosters", "Bring team and player registrations together. Track payments, waivers, roster requests, and capacity."),
        ("fa-stopwatch", "Live match operations", "Select field rosters, record points and penalties, track stones, and finalize matches with captain signatures."),
        ("fa-video", "Footage and broadcasts", "View and upload match footage, connect field cameras, and show live scoreboards in OBS."),
        ("fa-trophy", "Leagues and results", "Follow event results or organize a season with shared registration and collective standings."),
        ("fa-award", "Side competitions", "Manage registrations and live results for individual competitions alongside the main tournament."),
    ];
    rsx! {
        div { class: "arctos-about",
            header { class: "arctos-discovery-hero",
                div { class: "arctos-brand-mark", aria_hidden: "true", i { class: "fas fa-trophy" } }
                h1 { "About Arctos" }
                p { "A tournament operations tool built by organizers, for organizers—and everyone who makes a Jugger event happen." }
            }
            section { class: "card arctos-about-story",
                h2 { "Less administration. More time on the field." }
                p { "Arctos helps organizers publish events, manage registration, run matches, and share results. Players, teams, referees, and spectators can find what they need in one place." }
                p {
                    "Arctos, Latin for " em { "bear" } ", is a "
                    a { href: "https://en.wikipedia.org/wiki/Recursive_acronym", "recursive" } " "
                    a { href: "https://en.wikipedia.org/wiki/Backronym", "backronym" }
                    " that stands for " strong { "Arctos: Reid’s Comprehensive Tournament Organization System." }
                }
            }
            section { class: "arctos-about-features", aria_labelledby: "about-features-heading",
                h2 { id: "about-features-heading", "From registration to the final score" }
                div { class: "arctos-card-grid",
                    for (icon, title, description) in features {
                        article { class: "card arctos-feature-card",
                            div { class: "arctos-feature-icon", aria_hidden: "true", i { class: "fas {icon}" } }
                            h3 { "{title}" }
                            p { "{description}" }
                        }
                    }
                }
            }
            section { class: "card arctos-about-story",
                h2 { "Built around the way you organize" }
                dl { class: "arctos-principles",
                    div { dt { "Capable and accessible" } dd { "Give all juggers easy access to useful tools and event data, from a phone on the sideline or a desktop at the organizer’s table." } }
                    div { dt { "Unopinionated" } dd { "Support different event formats without imposing unnecessary structure. Organizers stay in control of their schedules and brackets." } }
                    div { dt { "Minimal" } dd { "Collect the data needed for the event and make everyday tasks clear." } }
                    div { dt { "Open source and independent" } dd { "Arctos is an independent community project. It does not belong to or represent CAJA or the US NJA." } }
                }
            }
            div { class: "arctos-about-actions",
                Link { to: Route::Docs {}, class: "btn btn-primary", i { class: "fas fa-book", aria_hidden: "true" } "Read the user guide" }
                a { href: "https://github.com/reid23/arctos", class: "btn btn-outline-secondary", i { class: "fab fa-github", aria_hidden: "true" } "Explore the source" }
            }
        }
    }
}
