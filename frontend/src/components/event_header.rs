//! Shared event identity, metadata, and registration actions.

use crate::Route;
use crate::appearance::focus_control;
use crate::pages::layout::{page_banner_key, set_page_banner_hidden, use_page_banner_preferences};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// A real event destination; an unavailable destination has no href.
#[derive(Clone, PartialEq)]
pub struct EventTab {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub href: Option<String>,
}

fn reveal_event_tab(id: &str) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(link) = document.get_element_by_id(id) else {
        return;
    };
    let Some(nav) = link.parent_element() else {
        return;
    };
    let link_bounds = link.get_bounding_client_rect();
    let nav_bounds = nav.get_bounding_client_rect();
    let delta = if link_bounds.left() < nav_bounds.left() + 8.0 {
        (link_bounds.left() - nav_bounds.left() - 8.0).floor()
    } else if link_bounds.right() > nav_bounds.right() - 8.0 {
        (link_bounds.right() - nav_bounds.right() + 8.0).ceil()
    } else {
        0.0
    };
    // Reveal the destination without moving the page vertically.
    nav.set_scroll_left(nav.scroll_left() + delta as i32);
}

#[component]
pub fn EventTabs(prefix: String, label: String, active: String, tabs: Vec<EventTab>) -> Element {
    let navigator = use_navigator();
    let active_id = format!("{prefix}-tab-{active}");
    use_effect(use_reactive(&active_id, move |active_id| {
        reveal_event_tab(&active_id);
        #[cfg(target_arch = "wasm32")]
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            if let (Some(link), Ok(fonts_ready)) = (
                document.get_element_by_id(&active_id),
                document.fonts().ready(),
            ) {
                spawn(async move {
                    if wasm_bindgen_futures::JsFuture::from(fonts_ready).await.is_ok()
                        && link.is_connected()
                        && link.get_attribute("aria-current").as_deref() == Some("page")
                    {
                        // Font loading can change tab widths after the first reveal.
                        // Measuring again flushes final layout; a stale tab cannot move the nav.
                        reveal_event_tab(&active_id);
                    }
                });
            }
        }
    }));
    rsx! {
        div { class: "event-route-tabs-bar",
            nav { class: "container-wide event-route-tabs", aria_label: "{label}",
                for tab in tabs {
                    if let Some(href) = tab.href {
                        a {
                            key: "{tab.id}", id: "{prefix}-tab-{tab.id}",
                            href: href.clone(), class: "event-route-tab",
                            aria_current: (active == tab.id).then_some("page"),
                            onfocus: {
                                let id = format!("{prefix}-tab-{}", tab.id);
                                move |_| reveal_event_tab(&id)
                            },
                            onclick: move |event: MouseEvent| {
                                // Keep modified clicks and other mouse buttons native, like Link.
                                if !event.modifiers().is_empty()
                                    || event.trigger_button() != Some(MouseButton::Primary)
                                {
                                    return;
                                }
                                event.prevent_default();
                                navigator.push(href.clone());
                            },
                            i { class: "{tab.icon}", aria_hidden: "true" }
                            "{tab.label}"
                        }
                    } else {
                        button {
                            key: "{tab.id}", id: "{prefix}-tab-{tab.id}",
                            r#type: "button", class: "event-route-tab", disabled: true,
                            aria_current: (active == tab.id).then_some("page"),
                            i { class: "{tab.icon}", aria_hidden: "true" }
                            "{tab.label}"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct EventHeaderProps {
    pub title: String,
    #[props(default)]
    pub banner: Option<String>,
    pub kind: String,
    pub date: Option<String>,
    pub location: Option<String>,
    pub team_count: usize,
    pub max_teams: Option<u32>,
    pub event_count: Option<usize>,
    pub registration_open: bool,
    pub badge_league_url: Option<String>,
    pub badge_name: Option<String>,
    pub children: Element,
}

#[component]
pub fn EventHeader(props: EventHeaderProps) -> Element {
    let preferences = use_page_banner_preferences();
    let key = page_banner_key(&use_route::<Route>()).unwrap_or_else(|| "event".into());
    let collapsed = preferences.read().contains(&key);
    rsx! {
        style { {include_str!("event_details.css")} }
        header { id: "page-banner", class: if collapsed { "event-detail-hero is-collapsed" } else { "event-detail-hero" },
            if !collapsed {
                if let Some(path) = props.banner {
                    div { class: "event-banner-image",
                        img { src: "{crate::api::base_url()}/static/{path}", alt: "" }
                    }
                }
            }
            div { class: "container-wide event-hero-inner",
            if !collapsed {
                button {
                    id: "page-banner-dismiss", class: "arctos-intro-dismiss event-banner-dismiss", r#type: "button",
                    aria_label: "Hide event banner", title: "Hide event banner",
                    onclick: move |_| {
                        set_page_banner_hidden(preferences, &key, true);
                        focus_control("page-banner-heading");
                    },
                    i { class: "fa-solid fa-xmark", aria_hidden: "true" }
                }
            }
            div { class: "event-hero-main",
                div { class: "event-hero-badges", hidden: collapsed,
                    span { class: "event-kind-badge", i { class: "fas fa-trophy", aria_hidden: "true" } "{props.kind}" }
                    if props.registration_open {
                        span { class: "event-registration-badge", i { class: "fas fa-circle-check", aria_hidden: "true" } "Registration open" }
                    }
                    if let (Some(url), Some(name)) = (props.badge_league_url, props.badge_name) {
                        Link { to: Route::LeagueHome { league_url: url }, class: "arctos-event-league-badge", "Part of {name}" }
                    }
                }
                h1 { id: "page-banner-heading", tabindex: "-1", "{props.title}" }
                ul { class: "event-hero-meta", hidden: collapsed,
                    if let Some(date) = props.date {
                        li { i { class: "far fa-calendar-alt", aria_hidden: "true" } span { "{date}" } }
                    }
                    if let Some(location) = props.location {
                        li { i { class: "fas fa-map-marker-alt", aria_hidden: "true" } span { "{location}" } }
                    }
                    li { i { class: "fas fa-users", aria_hidden: "true" }
                        span {
                            "{props.team_count}"
                            if let Some(max) = props.max_teams { " / {max}" }
                            " teams"
                        }
                    }
                    if let Some(count) = props.event_count {
                        li { i { class: "far fa-calendar-check", aria_hidden: "true" } span { "{count} " if count == 1 { "event" } else { "events" } } }
                    }
                }
            }
            div { class: "event-hero-actions", {props.children} }
            }
        }
    }
}
