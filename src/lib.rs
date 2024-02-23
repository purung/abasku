#![feature(iter_map_windows)]

use std::collections::HashMap;

use chrono::NaiveDate;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Modules
mod components;
mod destinations;
mod pages;

use crate::pages::checkpoint::{AddCheckpoint, CheckpointSummary, Checkpoints};
// Top-Level pages
use crate::pages::home::Home;
use crate::pages::food;
use crate::pages::not_found::NotFound;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Trips {
    trips: Vec<Trip>,
}

impl Trips {
    fn add(&mut self, trip: Trip) {
        self.trips.push(trip);
        self.trips.sort_by_cached_key(|k| k.date);
    }
    fn remove(mut self, uuid: Uuid) -> Self {
        self.trips = self.trips.into_iter().filter(|x| x.uuid != uuid).collect();
        self
    }
    fn favorites(&self) -> Vec<Trip> {
        let mut counts = HashMap::new();
        for t in self.trips.iter() {
            *counts.entry((&t.from, &t.to)).or_insert(0_usize) += 1;
        }
        let mut counts: Vec<_> = counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        let mut finish = Vec::new();
        for ((f, t), _) in counts.iter().take(5) {
            if let Some(found) = self.trips.iter().find(|x| x.from == **f && x.to == **t) {
                finish.push(found.clone());
            }
        }
        finish
    }
    fn recent(&self, n: usize) -> Vec<Trip> {
        self.trips.iter().rev().take(n).cloned().collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Trip {
    #[serde(default = "Uuid::new_v4")]
    uuid: Uuid,
    date: NaiveDate,
    from: String,
    to: String,
    distance: f32,
    time: u32,
    reason: String,
    #[serde(default)]
    returning: bool,
}

impl Trip {
    fn calculate_distance(&self) -> f32 {
        if self.returning {
            self.distance * 2.
        } else {
            self.distance
        }
    }
    fn distance_for_human(&self) -> String {
        format!("{} km", self.calculate_distance()).replace(".", ",")
    }
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html lang="sv" dir="ltr" attr:data-theme="light"/>

        // sets the document title
        <Title text="Welcome to Leptos CSR"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <ErrorBoundary fallback=|errors| {
            view! {
                <h1 class="">"Uh oh! Something went wrong!"</h1>

                <p>"Errors: "</p>
                // Render a list of errors as strings - good for development purposes
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}

                </ul>
            }
        }>
            <Router>
                <Nav/>
                <Routes>
                    <Route path="/" view=Home/>
                    <Route path="/checkpoint" view=Checkpoints>
                        <Route path="" view=CheckpointSummary/>
                        <Route path="ny" view=AddCheckpoint/>
                    </Route>
                    <Route path="/mat" view=food::Calendar>
                        <Route path="" view=food::Overview />
                    </Route>
                    <Route path="/*" view=NotFound/>
                </Routes>
            </Router>

        </ErrorBoundary>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    view! {
        <nav class="navbar bg-secondary text-base-100">
            <h1 class="flex-1">Självservice</h1>
            <div class="flex-none">
                <ul class="menu menu-horizontal px-1">
                    <li>
                        <A href="/">Resa</A>
                    </li>
                    <li>
                        <A href="/checkpoint">Avstämning</A>
                    </li>
                    <li>
                        <A href="/mat">Måltider</A>
                    </li>
                </ul>
            </div>
        </nav>
    }
}
