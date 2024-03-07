use std::collections::BTreeSet;

use chrono::{Days, Local, NaiveDate};
use leptos::{logging::log, *};
use leptos_icons::Icon;
use leptos_router::{use_params_map, ActionForm, Form, FromFormData, Outlet, A};
use leptos_use::{storage::use_local_storage, use_clipboard, utils::JsonCodec, UseClipboardReturn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::{MouseEvent, SubmitEvent};

use crate::{
    components::{InputWrap, Modal}, Meals, Trip, Trips
};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Killring {
    marked: BTreeSet<Uuid>,
}

impl Killring {
    fn new() -> Self {
        Self {
            marked: BTreeSet::new(),
        }
    }

    fn add(&mut self, trip: Uuid) {
        self.marked.insert(trip);
    }
    fn remove(&mut self, uuid: &Uuid) {
        self.marked.remove(uuid);
    }
    fn contains(&self, trip: &Uuid) -> bool {
        self.marked.contains(trip)
    }
    fn clear(&mut self) {
        self.marked.clear();
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
    let (r_meals, w_meals, _) = use_local_storage::<Meals, JsonCodec>("my-meals");
    let (r_checkpoints, w_checkpoints, _) =
        use_local_storage::<Checkpoints, JsonCodec>("my-checkpoints");
    let kill_ring = RwSignal::new(Killring::new());
    provide_context((w_trips, kill_ring));
    let cp_pairs = r_checkpoints
        .get_untracked()
        .points
        .iter()
        .rev()
        .map_windows(|[&x, &y]| {
            let (x, y) = (x.clone(), y.clone());
            view! { <Interval start=y end=x pool=r_trips food=r_meals/> }
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
            <Interval start=last_checkpoint end=Local::now().date_naive() pool=r_trips food=r_meals/>
            {cp_pairs}
            <Interval start=first_trip_date end=first_checkpoint pool=r_trips food=r_meals/>
        </div>
    }
}

#[component]
pub fn Interval(start: NaiveDate, end: NaiveDate, pool: Signal<Trips>, food: Signal<Meals>) -> impl IntoView {
    let date_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();
    let filtered: Signal<Vec<Trip>> = Signal::derive(move || {
        pool()
            .trips
            .iter()
            .rev()
            .filter(|&t| {
                let greater = t.date > start && t.date <= end;
                greater
            })
            .cloned()
            .collect()
    });
    let distance = Signal::derive(move || {
        let dist = filtered()
            .iter()
            .map(|t| t.calculate_distance())
            .sum::<f32>();
        format!("{dist:.1}").replace(".", ",")
    });
    let time = Signal::derive(move || {
        let tim = filtered().iter().map(|t| t.time).sum::<u32>() as f32 / 60.;
        format!("{tim:.1}").replace(".", ",")
    });
    let trip_views = filtered
        .get_untracked()
        .into_iter()
        .map(|t| {
            view! { <TripRow trip=t/> }
        })
        .collect_view();
    let href = format!("report/{end_str}");
    let meals_count = food.get_untracked().in_period(&start, &end);
    view! {
        <div class="collapse bg-base-200 collapse-open">
            <input type="checkbox" class="h-full w-full"/>
            <div class="collapse-title flex justify-between">

                <div class=" text-xl font-medium flex gap-3">
                    {date_str} <Icon icon=icondata::BiArrowFromLeftSolid class="h-full text-2xl"/>
                    {end_str.clone()}
                </div>
                <div class="flex text-xl place-self-center gap-3">
                    <div class="place-self-center flex gap-1">
                        <Icon icon=icondata::TbSum/>
                        <span class="place-self-center text-sm ">{distance} km</span>
                    </div>

                    <div class="place-self-center flex gap-1">
                        <Icon icon=icondata::CgTimer/>
                        <span class="place-self-center text-sm ">{time} h</span>
                    </div>

                    <div class="place-self-center flex gap-1">
                        <Icon icon=icondata::TbPizza/>
                        <span class="place-self-center text-sm ">{meals_count}</span>
                    </div>
                </div>
            </div>
            <div class="collapse-content">
                <ul role="list" class="divide-y divide-gray-100">
                    <li class="flex justify-around pb-4">
                        <button class="btn btn-sm btn-outline btn-primary">
                            Radera checkpoint
                        </button>
                        <A href=href class="btn btn-sm btn-outline btn-primary">
                            Generera rapport
                        </A>
                    </li>
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
    let (w_trips, killring) = expect_context::<(WriteSignal<Trips>, RwSignal<Killring>)>();
    let delete = move |_| {
        killring.update(|k| {
            k.add(trip.uuid.clone());
        });
        w_trips.update(|tr| {
            tr.remove(&trip.uuid);
        });
    };
    view! {
        <li
            class="flex justify-between gap-x-6 py-5"
            class=("opacity-20", move || killring().contains(&trip.uuid))
        >
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
            <div class="flex gap-2">
                <div class="hidden shrink-0 sm:flex sm:flex-col sm:items-end">
                    <p class="text-sm leading-6 text-gray-900">{for_humans}</p>
                    <p class="mt-1 text-xs leading-5 text-gray-500">{trip.time} min</p>
                </div>
                <div class="dropdown dropdown-top dropdown-left">
                    <div
                        tabindex="0"
                        role="button"
                        class="btn btn-ghost btn-circle text-secondary"
                        class=("invisible", move || killring().contains(&trip.uuid))
                    >
                        <Icon class="size-6" icon=icondata::TiDeleteOutline/>
                    </div>
                    <ul
                        tabindex="0"
                        class="dropdown-content z-40 grid menu shadow bg-base-100 rounded-box w-52"
                    >
                        <li>
                            <button
                                on:click=delete
                                class="btn btn-warning place-self-center w-full flex justify-center content-center"
                            >
                                <p class="place-self-center">Ja, radera resa</p>
                            </button>
                        </li>
                    </ul>
                </div>
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

#[component]
pub fn Report() -> impl IntoView {
    let (r_trips, _, _) = use_local_storage::<Trips, JsonCodec>("my-trips");
    let (r_checkpoints, _, _) = use_local_storage::<Checkpoints, JsonCodec>("my-checkpoints");
    let selected_checkpoint = use_params_map()
        .get_untracked()
        .get("checkpoint")
        .and_then(|t| t.parse::<NaiveDate>().ok())
        .unwrap_or_else(move || {
            r_checkpoints
                .get_untracked()
                .points
                .iter()
                .last()
                .cloned()
                .unwrap_or_else(|| Local::now().date_naive())
        });
    let start = r_checkpoints
        .get_untracked()
        .points
        .iter()
        .rev()
        .filter(|f| **f < selected_checkpoint)
        .next()
        .cloned()
        .unwrap_or_else(|| {
            r_trips
                .get_untracked()
                .trips
                .iter()
                .map(|t| &t.date)
                .min()
                .cloned()
                .unwrap_or_else(|| Local::now().date_naive())
                .checked_sub_days(Days::new(1))
                .unwrap()
        });
    let date_str = start.format("%Y-%m-%d").to_string();
    let end_str = selected_checkpoint.format("%Y-%m-%d").to_string();
    let filtered: Vec<Trip> = r_trips
        .get_untracked()
        .trips
        .iter()
        .rev()
        .filter(|&t| {
            let greater = t.date > start && t.date <= selected_checkpoint;
            greater
        })
        .cloned()
        .collect();
    let distance = filtered.iter().map(|t| t.calculate_distance()).sum::<f32>();
    let distance = format!("{distance:.1}").replace(".", ",");
    let time: f32 = (filtered.iter().map(|t| t.time).sum::<u32>() as f32) / 60.;
    let time = format!("{time:.1}").replace(".", ",");
    let longest_trip = filtered
        .iter()
        .map(|t| t.from_to().chars().count())
        .max()
        .unwrap_or(0);
    let longest_distance = filtered
        .iter()
        .map(|t| t.distance_for_human().chars().count())
        .max()
        .unwrap_or(0);
    let trip_views = filtered
        .into_iter()
        .map(|t| t.report_row(longest_trip, longest_distance));
    let for_clipboard = trip_views.clone().collect::<Vec<String>>().join("\n");
    let for_view = trip_views.map(|t| view! { <p>{t}</p> }).collect_view();
    let UseClipboardReturn {
        is_supported, copy, ..
    } = use_clipboard();
    view! {
        <div class="bg-base-200 h-fit p-8 rounded-lg">
            <div class="flex justify-between">

                <div class=" text-xl font-medium flex gap-3">
                    {date_str} <Icon icon=icondata::BiArrowFromLeftSolid class="h-full text-2xl"/>
                    {end_str.clone()}
                </div>
                <div class="flex text-xl place-self-center gap-3">
                    <div class="place-self-center flex items-center gap-2">
                        <Icon icon=icondata::TbSum/>
                        <span class="place-self-center text-sm ">{distance} kms</span>
                    </div>

                    <div class="place-self-center flex items-center gap-2">
                        <Icon icon=icondata::CgTimer/>
                        <span class="place-self-center text-sm ">{time} h</span>
                    </div>

                </div>
            </div>
            <div class="">
                <Show when=is_supported>
                    <button
                        on:click={
                            let copy = copy.clone();
                            let to_copy = for_clipboard.clone();
                            move |_| copy(&to_copy)
                        }

                        class="btn btn-sm btn-outline btn-primary my-6"
                    >
                        "Kopiera logg"
                    </button>
                </Show>
                <ul role="list" class="font-mono">
                    <li class="flex justify-around pb-4"></li>
                    {for_view}
                </ul>
            </div>
        </div>
    }
}
