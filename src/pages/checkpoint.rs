use std::collections::BTreeSet;

use chrono::{Days, Local, NaiveDate};
use leptos::{logging::log, *};
use leptos_icons::Icon;
use leptos_router::{ActionForm, Form, FromFormData, Outlet};
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::{MouseEvent, SubmitEvent};

use crate::{components::InputWrap, Trip, Trips};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Killring {
    marked: Vec<Uuid>,
}

impl Killring {
    fn new() -> Self {
        Self { marked: Vec::new() }
    }

    fn add(mut self, trip: Uuid) -> Self {
        self.marked.push(trip);
        self
    }
    fn remove(mut self, uuid: Uuid) -> Self {
        self.marked = self.marked.into_iter().filter(|x| *x != uuid).collect();
        self
    }
    fn clear(mut self) -> Self {
        self.marked.clear();
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct WEvent {
    date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Checkpoints {
    points: BTreeSet<NaiveDate>,
}

impl Checkpoints {
    fn add(&mut self, new: NaiveDate) {
        self.points.insert(new);
    }
    fn remove(&mut self, point: NaiveDate) {
        self.points.remove(&point);
    }
}

#[component]
pub fn Checkpoints() -> impl IntoView {
    view! {
        <div class="grid min-h-svh">
            <div class="w-11/12 flex justify-center py-12">
                <Outlet/>
            </div>
        </div>
    }
}

#[component]
pub fn CheckpointSummary() -> impl IntoView {
    let (r_trips, w_trips, _) = use_local_storage::<Trips, JsonCodec>("my-trips");
    let (r_checkpoints, w_checkpoints, _) =
        use_local_storage::<Checkpoints, JsonCodec>("my-checkpoints");
    let kill_ring = RwSignal::new(Killring::new());
    provide_context(kill_ring);
    let cp_pairs = r_checkpoints
        .get_untracked()
        .points
        .iter()
        .rev()
        .map_windows(|[&x, &y]| {
            let (x, y) = (x.clone(), y.clone());
            view! { <Interval start=y end=x pool=r_trips/> }
        })
        .collect_view();
    let first_trip_date = r_trips.with_untracked(|tr| {
        tr.trips
            .iter()
            .next()
            .map(|t| t.date.checked_sub_days(Days::new(1)))
            .unwrap()
            .unwrap()
    });
    let first_checkpoint =
        r_checkpoints.with_untracked(|ch| ch.points.iter().min().unwrap().to_owned());
    let last_checkpoint =
        r_checkpoints.with_untracked(|ch| ch.points.iter().max().unwrap().to_owned());
    view! {
        <div class="w-full max-w-xl flex flex-col gap-3">
            <Interval start=last_checkpoint end=Local::now().date_naive() pool=r_trips/>
            {cp_pairs}
            <Interval start=first_trip_date end=first_checkpoint pool=r_trips/>
        </div>
    }
}

#[component]
pub fn Interval(start: NaiveDate, end: NaiveDate, pool: Signal<Trips>) -> impl IntoView {
    let date_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();
    let filtered: Vec<Trip> = pool
        .get_untracked()
        .trips
        .iter()
        .rev()
        .filter(|&t| {
            let greater = t.date > start && t.date <= end;
            greater
        })
        .cloned()
        .collect();
    let distance =
        (filtered.iter().map(|t| t.calculate_distance()).sum::<f32>() * 10.).round() / 10.;
    let time: f32 =
        ((filtered.iter().map(|t| t.time).sum::<u32>() as f32) / 60. * 10.).round() / 10.;
    let trip_views = filtered
        .into_iter()
        .map(|t| {
            view! { <TripRow trip=t/> }
        })
        .collect_view();
    view! {
        <div class="collapse bg-base-200 collapse-open">
            <input type="checkbox" class="h-full w-full"/>
            <div class="collapse-title flex justify-between">
                <div class=" text-xl font-medium flex gap-3">
                    {date_str} <Icon icon=icondata::BiArrowFromLeftSolid class="h-full text-2xl"/>
                    {end_str}
                </div>
                <div class="flex text-xl place-self-center gap-3">
                    <div class="place-self-center flex">
                        <Icon icon=icondata::TbSum/>
                        <span class="place-self-center text-sm ">{distance} km</span>
                    </div>

                    <div class="place-self-center flex">
                        <Icon icon=icondata::CgTimer/>
                        <span class="place-self-center text-sm ">{time} h</span>
                    </div>

                </div>
            </div>
            <div class="collapse-content">
                <ul role="list" class="divide-y divide-gray-100">
                    {trip_views}
                </ul>
            </div>
        </div>
    }
}

#[component]
pub fn TripRow(trip: Trip) -> impl IntoView {
    let icon = if trip.returning {
        icondata::BsArrowLeftRight
    } else {
        icondata::BsArrowRight
    };
    let for_humans = trip.distance_for_human();
    let date = trip.date.format("%d %b").to_string();
    view! {
        <li class="flex justify-between gap-x-6 py-5">
            <div class="flex min-w-0 gap-x-4">
                <div class="min-w-0 flex-auto">
                    <p class="text-sm font-semibold leading-6 text-gray-900 flex gap-x-2 content-center">
                        {trip.from.clone()} <Icon class="h-full place-self-center" icon=icon/>
                        {trip.to.clone()}
                    </p>
                    <div class="flex gap-3 divide-x-2 mt-1  text-xs leading-5 text-gray-500">
                        <p>{date}</p>
                        <p class="truncate">{trip.reason.clone()}</p>
                    </div>
                </div>
            </div>
            <div class="hidden shrink-0 sm:flex sm:flex-col sm:items-end">
                <p class="text-sm leading-6 text-gray-900">{for_humans}</p>
                <p class="mt-1 text-xs leading-5 text-gray-500">{trip.time} min</p>
            </div>
        </li>
    }
}
/// Default Checkpoint Page
#[component]
pub fn AddCheckpoint() -> impl IntoView {
    let (r_checkpoints, w_checkpoints, _) =
        use_local_storage::<Checkpoints, JsonCodec>("my-checkpoints");
    // let delete_checkpoint = move ||

    // let form_ref = create_node_ref::<Form>();
    let new = move |ev: SubmitEvent| {
        // ev.prevent_default();
        let t = WEvent::from_event(&ev);
        log!("{t:?}");
        if let Ok(t) = t {
            w_checkpoints.update(|cp| cp.add(t.date));
        };
    };
    let today = Local::now().date_naive().to_string();
    view! {
        <div class="form-control w-full max-w-xs outline my-6 p-6 outline-1 outline-primary rounded-xl">
            <Form action="/checkpoint" method="GET" on:submit=new class="flex flex-col gap-3">
                <InputWrap label="Ny checkpoint">
                    <input
                        name="date"
                        type="date"
                        value=today.clone()
                        max=today
                        class="input input-bordered w-full max-w-xs"
                        required
                    />

                </InputWrap>
                <button type="submit" class="btn btn-secondary btn-outline">
                    Lägg till
                </button>
            </Form>
        </div>
    }
}
