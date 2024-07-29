use std::{
    collections::{BTreeSet, HashMap},
    num::ParseIntError,
    string::ParseError,
};

use chrono::{Datelike, Days, Local, Months, NaiveDate};
use itertools::Itertools;
use leptos::{logging::log, *};
use leptos_icons::Icon;
use leptos_router::{use_params_map, Form, FromFormData, Outlet, A};
use leptos_use::{storage::use_local_storage, use_clipboard, utils::JsonCodec, UseClipboardReturn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::SubmitEvent;

use crate::{components::InputWrap, Trip, Trips};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct Killring {
    marked: BTreeSet<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Month {
    year: i32,
    month: u32,
}

impl TryFrom<(&String, &String)> for Month {
    type Error = ParseIntError;

    fn try_from((y, m): (&String, &String)) -> Result<Self, Self::Error> {
        Ok(Self::new(y.parse()?, m.parse()?))
    }
}

impl Month {
    fn new(year: i32, month: u32) -> Self {
        Self { year, month }
    }

    pub fn first_of(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap()
    }
    pub fn last_of(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month + 1, 1)
            .or_else(|| NaiveDate::from_ymd_opt(self.year + 1, 1, 1))
            .unwrap()
            .pred_opt()
            .unwrap()
    }
    fn fmt_human(&self) -> String {
        format!("{} {}", self.human_month_name(), self.year)
    }
    fn human_month_name(&self) -> &'static str {
        match self.month {
            1 => "januari",
            2 => "februari",
            3 => "mars",
            4 => "april",
            5 => "maj",
            6 => "juni",
            7 => "juli",
            8 => "augusti",
            9 => "september",
            10 => "oktober",
            11 => "november",
            12 => "december",
            _ => unreachable!(),
        }
    }
}

impl From<(i32, u32)> for Month {
    fn from(value: (i32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
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
    fn contains(&self, trip: &Uuid) -> bool {
        self.marked.contains(trip)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonthStatistic {
    distance: f32,
    time: u32,
}

impl MonthStatistic {
    fn new(distance: f32, time: u32) -> Self {
        Self {
            distance,
            time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct WEvent {
    date: NaiveDate,
}

#[component]
pub fn Checkpoints() -> impl IntoView {
    view! {
        <div class="grid min-h-svh">
            <div class="w-11/12 flex justify-center flex-col lg:flex-row items-center lg:items-start gap-12 py-12">
                <Outlet/>
            </div>
        </div>
    }
}

#[component]
pub fn CheckpointSummary() -> impl IntoView {
    let (r_trips, w_trips, _) = use_local_storage::<Trips, JsonCodec>("my-trips");
    let kill_ring = RwSignal::new(Killring::new());
    provide_context((w_trips, kill_ring));
    let months: Signal<Vec<(Month, Vec<Trip>)>> = Signal::derive(move || {
        let groups = &r_trips()
            .trips
            .into_iter()
            .rev()
            .group_by(|t| (t.date.year(), t.date.month()));
        groups
            .into_iter()
            .map(|(ym, tr)| (ym.into(), tr.into_iter().collect_vec()))
            .collect_vec()
    });
    let statistics: Signal<HashMap<Month, MonthStatistic>> = Signal::derive(move || {
        with!(|months| {
            HashMap::from_iter(months.iter().map(|(ym, dt)| {
                (
                    ym.to_owned(),
                    MonthStatistic::new(
                        dt.iter().map(|d| d.calculate_distance()).sum(),
                        dt.iter().map(|d| d.calculate_time()).sum(),
                    ),
                )
            }))
        })
    });
    view! {
        <div class="w-full max-w-xl flex flex-col gap-3">
            <For each=months key=|(ym, _)| ym.to_owned() let:iva>
                <Interval
                    month=iva.0.to_owned()
                    statistics=Signal::derive(move || {
                        statistics.with(|s| s.get(&iva.0).cloned().unwrap())
                    })

                    trips=iva.1
                />
            </For>
        </div>
    }
}

#[component]
pub fn Interval(
    month: Month,
    statistics: Signal<MonthStatistic>,
    trips: Vec<Trip>,
) -> impl IntoView {
    let date_str = month.fmt_human();
    let distance = Signal::derive(move || {
        let dist = statistics.with(|s| s.distance);
        format!("{dist:.1}").replace('.', ",")
    });
    let time = Signal::derive(move || {
        let tim = statistics.with(|s| s.time as f32 / 60.);
        format!("{tim:.1}").replace('.', ",")
    });
    let trip_views = trips
        .into_iter()
        .map(|t| {
            view! { <TripRow trip=t/> }
        })
        .collect_view();
    let href = format!("report/{}/{}", month.year, month.month);
    view! {
        <div class="collapse bg-base-200">
            <input type="checkbox" class="h-full w-full"/>
            <div class="collapse-title flex justify-between">

                <div class="text-xl font-medium flex gap-3 capitalize">{date_str}</div>
                <div class="flex text-xl gap-3 pt-1 h-min">
                    <div class="place-self-center flex gap-1">
                        <Icon icon=icondata::FaCarSideSolid/>
                        <span class="place-self-center text-sm ">{distance} km</span>
                    </div>

                    <div class="place-self-center flex gap-1">
                        <Icon icon=icondata::CgTimer/>
                        <span class="place-self-center text-sm ">{time} h</span>
                    </div>

                </div>
            </div>
            <div class="collapse-content">
                <ul role="list" class="divide-y divide-gray-100">
                    <li class="flex justify-around pb-4">
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
            k.add(trip.uuid);
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
#[component]
pub fn Report() -> impl IntoView {
    let (r_trips, _, _) = use_local_storage::<Trips, JsonCodec>("my-trips");
    let checkpoints = use_params_map().get_untracked();
    let month = checkpoints
        .get("year")
        .zip(checkpoints.get("month"))
        .and_then(|ym| ym.try_into().ok())
        .unwrap_or_else(|| {
            log!("Kunde inte hitta eller konvertera");
            let now = Local::now()
                .date_naive()
                .checked_sub_months(Months::new(1))
                .unwrap();
            Month::new(now.year(), now.month())
        });
    let start = month.first_of();
    let last = month.last_of();
    let date_str = start.format("%Y-%m-%d").to_string();
    let end_str = last.format("%Y-%m-%d").to_string();
    let filtered: Vec<Trip> = r_trips
        .get_untracked()
        .trips
        .iter()
        .rev()
        .filter(|&t| t.date >= start && t.date <= last)
        .cloned()
        .collect();
    let distance = filtered.iter().map(|t| t.calculate_distance()).sum::<f32>();
    let distance = format!("{distance:.1}").replace('.', ",");
    let time: f32 = (filtered.iter().map(|t| t.time).sum::<u32>() as f32) / 60.;
    let time = format!("{time:.1}").replace('.', ",");
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
                        <span class="place-self-center text-sm ">{distance} km</span>
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

