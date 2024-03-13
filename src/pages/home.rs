use std::collections::HashMap;

use chrono::Local;

use leptos::{*};
use leptos_icons::Icon;
use leptos_router::*;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};
use web_sys::SubmitEvent;

use crate::{
    components::InputWrap,
    destinations::{destinations, travel, Travel},
    Trip, Trips,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
struct CustomTrips {
    trips: HashMap<String, HashMap<String, Travel>>,
}

impl CustomTrips {
    fn get(&self, from: &str, to: &str) -> Option<Travel> {
        self.trips.get(from)?.get(to).cloned()
    }
    fn add(&mut self, trip: &Trip) {
        let fr = trip.from.to_owned();
        let to = trip.to.to_owned();
        let tra: Travel = trip.into();
        self.trips
            .entry(fr)
            .or_default()
            .insert(to, tra);
    }
}

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let (r_trips, w_trips, _) = use_local_storage::<Trips, JsonCodec>("my-trips");

    view! {
        <div class="min-h-svh py-12">
            <div class="w-11/12 flex justify-center gap-20">
                <QuickChoice trips=r_trips/>
                <div class="form-control w-full max-w-sm outline my-6 p-6 outline-1 outline-primary rounded-xl h-fit">
                    <DestinationDataList/>
                    <AddTravel write_to=w_trips/>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn QuickChoice(trips: Signal<Trips>) -> impl IntoView {
    let favs = trips
        .with_untracked(|tr| tr.favorites())
        .into_iter()
        .map(|f| {
            view! { <QuickChoiceRow trip=f/> }
        })
        .collect_view();
    let recents = trips
        .with_untracked(|tr| tr.recent(5))
        .into_iter()
        .map(|f| view! { <QuickChoiceRow trip=f/> })
        .collect_view();
    view! {
        <div class="flex flex-col xl:flex-row items-center xl:justify-around gap-12 justify-center">
            <div class="flex flex-col gap-3">
                <h2 class="text-2xl text-center">Favoriter</h2>
                <div class="flex flex-col divide-y-2 ">{favs}</div>
            </div>
            <div class="flex flex-col gap-3">
                <h2 class="text-2xl text-center">Senaste</h2>
                <div class="flex flex-col divide-y-2 ">{recents}</div>
            </div>
        </div>
    }
}

#[component]
pub fn QuickChoiceRow(trip: Trip) -> impl IntoView {
    let icon = if trip.returning {
        icondata::BsArrowLeftRight
    } else {
        icondata::BsArrowRight
    };
    view! {
        <li class="flex justify-between items-center gap-x-6 py-5">
            <div class="flex min-w-0 gap-x-4">
                <div class="min-w-0 flex-auto">
                    <p class="text-sm leading-6 text-gray-900 flex gap-x-2 content-center">
                        {trip.from.clone()} <Icon class="h-full place-self-center" icon=icon/>
                        {trip.to.clone()}
                    </p>
                </div>
            </div>
            <Form action="" method="GET" class="hidden shrink-0 sm:flex sm:flex-col sm:items-end">
                <input type="hidden" name="to" value=trip.to/>
                <input type="hidden" name="from" value=trip.from/>
                <input type="hidden" name="returning" value=trip.returning.to_string()/>
                <button type="submit" class="btn btn-ghost btn-circle text-secondary">
                    <Icon icon=icondata::IoAddCircleOutline class="size-6"/>
                </button>
            </Form>
        </li>
    }
}

#[component]
pub fn AddTravel(write_to: WriteSignal<Trips>) -> impl IntoView {
    let (r_custom, w_custom, _) = use_local_storage::<CustomTrips, JsonCodec>("my-custom-trips");
    let (r_from, w_from) = create_query_signal::<String>("from");
    let (r_to, w_to) = create_query_signal::<String>("to");
    let (r_returning, w_returning) = create_query_signal::<bool>("returning");
    let (r_distance, w_distance) = create_signal(0.);
    let (r_time, w_time) = create_signal(0);
    let today = Local::now().date_naive().to_string();
    let zero_out = move || {
        w_distance(0.);
        w_time(0)
    };
    let autopilot = move || {
        with!(move |r_from, r_to, r_custom| {
            if let (Some(f), Some(t)) = (r_from, r_to) {
                if !f.is_empty() && !t.is_empty() {
                    if let Some(travel_data) = r_custom
                        .get(f, t)
                        .or_else(|| r_custom.get(t, f))
                        .or_else(|| travel(f, t))
                    {
                        w_distance(travel_data.km());
                        w_time(travel_data.minutes());
                    } else {
                        zero_out()
                    };
                } else {
                    zero_out();
                }
            } else {
                zero_out();
            }
        })
    };
    let cleaner = move || {
        if r_from.get_untracked() == r_to.get_untracked() {
            w_to(None)
        }
    };
    let reset_returning =
        move || {
            let tim = gloo::timers::callback::Timeout::new(40, move || w_returning(Some(false)));
            tim.forget();
        };
    create_effect(move |_| {
        r_from.track();
        r_to.track();
        autopilot();
        cleaner();
    });
    let new = move |ev: SubmitEvent| {
        ev.prevent_default();
        let t = Trip::from_event(&ev);
        if let Ok(t) = t {
            if travel(&t.from, &t.to).is_none() {
                w_custom.update(|ct| ct.add(&t));
            };
            write_to.update(|tr: &mut Trips| {
                tr.add(t);
            });
            zero_out();
            reset_returning();
            w_to(r_from.get_untracked());
        };
    };

    view! {
        <div class="h-fit">
            <form on:submit=new class="flex flex-col gap-3">
                <InputWrap label="Datum">
                    <input
                        name="date"
                        type="date"
                        value=today.clone()
                        max=today
                        class="input input-bordered w-full max-w-xs"
                        required
                    />

                </InputWrap>
                <InputWrap label="Utgångspunkt">
                    <input
                        name="from"
                        prop:value=move || r_from().unwrap_or_default()
                        list="destination-choices"
                        class="input input-bordered w-full max-w-xs"
                        required
                        on:input=move |ev| {
                            w_from(Some(event_target_value(&ev)));
                        }
                    />

                </InputWrap>
                <InputWrap label="Resmål">
                    <input
                        name="to"
                        class="input input-bordered w-full max-w-xs"
                        value=""
                        prop:value=move || r_to().unwrap_or_default()
                        list="destination-choices"
                        required
                        on:input=move |ev| {
                            w_to(Some(event_target_value(&ev)));
                        }
                    />

                </InputWrap>
                <div class="flex gap-2">
                    <InputWrap label="Avstånd" explanation="kilometer">
                        <input
                            name="distance"
                            type="number"
                            required
                            min=0.1
                            max=1000
                            inputmode="decimal"
                            step=0.1
                            class="input input-bordered w-full max-w-xs"
                            value=r_distance
                            on:input=move |ev| {
                                let s: String = event_target_value(&ev);
                                let d = s.parse::<f32>();
                                if let Ok(d) = d {
                                    w_distance(d)
                                }
                            }
                        />

                    </InputWrap>
                    <InputWrap label="Restid" explanation="minuter">
                        <input
                            type="number"
                            name="time"
                            min=1
                            max=1000
                            inputmode="numeric"
                            step=1
                            class="input input-bordered w-full max-w-xs"
                            value=r_time
                            required
                            on:input=move |ev| {
                                let s: String = event_target_value(&ev);
                                let d = s.parse::<u32>();
                                if let Ok(d) = d {
                                    w_time(d)
                                }
                            }
                        />

                    </InputWrap>
                </div>
                <InputWrap label="Anledning">
                    <input
                        name="reason"
                        value="Möte"
                        class="input input-bordered w-full max-w-xs"
                        required
                    />
                </InputWrap>

                <div class="form-control">
                    <label
                        on:click=move |_e| {
                            let switch = r_returning.get().map(|r| !r).or(Some(true));
                            w_returning.set(switch);
                        }

                        class="label cursor-pointer justify-start align-center gap-3"
                    >
                        <input
                            name="returning"
                            checked=r_returning
                            type="checkbox"
                            value="true"
                            class="checkbox bg-base-100 checkbox-primary"
                        />
                        <span class="label-text">Tur och retur</span>
                    </label>
                </div>

                <button type="submit" class="btn btn-secondary btn-outline">
                    Lägg in
                </button>
            </form>
        </div>
    }
}

#[component]
fn DestinationDataList() -> impl IntoView {
    let welcome_package = destinations()
        .into_iter()
        .map(|x| view! { <option value=x></option> })
        .collect_view();
    view! {
        <datalist id="destination-choices">

            {welcome_package}

        </datalist>
    }
}
