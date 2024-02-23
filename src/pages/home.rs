use chrono::Local;

use leptos::{logging::log, *};
use leptos_icons::Icon;
use leptos_router::*;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use web_sys::SubmitEvent;

use crate::{
    components::InputWrap,
    destinations::{destinations, travel},
    Trip, Trips,
};

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let (r_trips, w_trips, _) = use_local_storage::<Trips, JsonCodec>("my-trips");

    view! {
        <div class="min-h-svh py-12">
            <div class="w-11/12 flex justify-around">
                <QuickChoice trips=r_trips/>
                <div class="form-control w-full max-w-sm outline my-6 p-6 outline-1 outline-primary rounded-xl">
                    <DestinationDataList/>
                // <AddTravel write_to=w_trips/>
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
        <div class="flex flex-col gap-4 justify-center">
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
        <li class="flex justify-between gap-x-6 py-5">
            <div class="flex min-w-0 gap-x-4">
                <div class="min-w-0 flex-auto">
                    <p class="text-sm font-semibold leading-6 text-gray-900 flex gap-x-2 content-center">
                        {trip.from.clone()} <Icon class="h-full place-self-center" icon=icon/>
                        {trip.to.clone()}
                    </p>
                </div>
            </div>
            <div class="hidden shrink-0 sm:flex sm:flex-col sm:items-end">
                <p class="text-sm leading-6 text-gray-900"></p>
                <p class="mt-1 text-xs leading-5 text-gray-500">{trip.time} min</p>
            </div>
        </li>
    }
}

#[component]
pub fn AddTravel(write_to: WriteSignal<Trips>) -> impl IntoView {
    let (r_from, w_from) = create_query_signal::<String>("from");
    let (r_to, w_to) = create_query_signal::<String>("to");
    let (r_distance, w_distance) = create_signal(0.);
    let (r_time, w_time) = create_signal(0);
    let today = Local::now().date_naive().to_string();
    let zero_out = move || {
        w_distance(0.);
        w_time(0)
    };
    let autopilot = move || {
        with!(move |r_from, r_to| {
            if let (Some(f), Some(t)) = (r_from, r_to) {
                if !f.is_empty() && !t.is_empty() {
                    if let Some(travel_data) = travel(f, t) {
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
    create_effect(move |_| {
        // r_from();
        // r_to();
        // autopilot();
    });
    let switch = move || {
        let (ind_t, ind_f) = (r_to(), r_from());
        w_to(None);
        w_from(None);
        w_to(ind_f);
        w_from(ind_t);
    };
    let new = move |ev: SubmitEvent| {
        ev.prevent_default();
        let t = Trip::from_event(&ev);
        if let Ok(t) = t {
            write_to.update(|tr: &mut Trips| {
                tr.add(t);
            });
            switch();
        };
    };

    view! {
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
                        log!("Input 1");
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
                        log!("Input 2");
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
                <label class="label cursor-pointer justify-start align-center gap-3">
                    <input
                        name="returning"
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
            <button type="reset" class="btn btn-secondary btn-outline">
                Nollställ
            </button>
        </form>
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