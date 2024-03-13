#![feature(iter_map_windows)]

use std::collections::{BTreeSet, HashMap};

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

use crate::pages::checkpoint::{CheckpointSummary, Checkpoints, Report};
// Top-Level pages
use crate::pages::food;
use crate::pages::home::Home;
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
    fn remove(&mut self, uuid: &Uuid) {
        self.trips.retain(|x| x.uuid != *uuid);
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
        format!("{} km", self.calculate_distance()).replace('.', ",")
    }
    fn from_to(&self) -> String {
        format!(
            "{}-{}{}",
            self.from,
            self.to,
            if self.returning { " ToR" } else { "" }
        )
    }
    fn report_row(&self, longest_trip: usize, longest_distance: usize) -> String {
        let Self {
            date,
            reason,
            
            ..
        } = &self;
        let points = self.from_to();
        let date = date.format("%d/%m").to_string();
        let distance = self.distance_for_human();
        let p_1 = ".".repeat(3 + longest_trip - points.chars().count());
        let p_2 = ".".repeat(3 + longest_distance - distance.chars().count());
        format!("{date}: {points}{p_1}{distance}{p_2}{reason}")
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Meals {
    points: BTreeSet<NaiveDate>,
}

impl Meals {
    fn add(&mut self, new: NaiveDate) {
        self.points.insert(new);
    }
    fn remove(&mut self, point: NaiveDate) {
        self.points.remove(&point);
    }
    fn has(&self, date: NaiveDate) -> bool {
        self.points.contains(&date)
    }
    fn toggle(&mut self, date: NaiveDate) {
        if self.has(date) {
            self.remove(date)
        } else {
            self.add(date)
        }
    }
    fn in_period(&self, after: &NaiveDate, before_inclusive: &NaiveDate) -> usize {
        self.points
            .iter()
            .filter(|f| *f > after && *f <= before_inclusive)
            .count()
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
        <Title text="Loggbok"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router>
            <Nav/>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/checkpoint" view=Checkpoints>
                    <Route path="" view=CheckpointSummary/>
                    <Route path="report/:checkpoint" view=Report/>
                </Route>
                <Route path="/mat" view=food::Calendar>
                    <Route path="" view=food::Overview/>
                </Route>
                <Route path="/*" view=NotFound/>
            </Routes>
        </Router>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    view! {
        <nav class="navbar bg-secondary text-base-100">
            <h1 class="flex-1"><A href="/">Självservice</A></h1>
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
