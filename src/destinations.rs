use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::Trip;

#[derive(Serialize, Debug, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Distance {
    text: String,
    value: u32,
}

impl Distance {
    pub fn new(text: String, value: u32) -> Self { Self { text, value } }
}

#[derive(Serialize, Debug, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Duration {
    text: String,
    value: u32,
}

impl Duration {
    pub fn new(text: String, value: u32) -> Self { Self { text, value } }
}

#[derive(Serialize, Debug, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Travel {
    distance: Distance,
    duration: Duration,
}

impl Travel {
    pub fn new(distance: Distance, duration: Duration) -> Self { Self { distance, duration } }

    pub fn km(&self) -> f32 {
        (((self.distance.value as f32 / 1000.) * 10.).round()) / 10.
    }
    pub fn minutes(&self) -> u32 {
        self.duration.value / 60
    }
}

impl From<&Trip> for Travel {
    fn from(val: &Trip) -> Self {
        let dur = Duration::new(format!("{} min", val.time), val.time * 60);
        let dis = Distance::new(val.distance_for_human(), (val.distance * 1000.) as u32);
        Travel::new(dis, dur)
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
