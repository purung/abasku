use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Distance {
    text: String,
    value: u32,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Duration {
    text: String,
    value: u32,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Travel {
    distance: Distance,
    duration: Duration,
}

impl Travel {
    pub fn km(&self) -> f32 {
        (self.distance.value as f32 / 1000. * 10.).round() / 10.
    }
    pub fn minutes(&self) -> u32 {
        self.duration.value / 60
    }
}

type Destinations = HashMap<String, HashMap<String, Vec<Travel>>>;

static DESTINATIONS: Lazy<Option<Destinations>> = Lazy::new(|| {
    let file = include_bytes!("../data/matrix.json");
    let json: Destinations = serde_json::from_slice(file).ok()?;
    Some(json)
});

pub fn destinations() -> Vec<String> {
    DESTINATIONS
        .as_ref()
        .map_or_else(Vec::new, |x| x.keys().map(Clone::clone).collect())
}

pub fn travel(from: &str, to: &str) -> Option<Travel> {
    let travel = DESTINATIONS.as_ref()?.get(from)?.get(to)?.iter().next()?;

    Some(travel.clone())
}
