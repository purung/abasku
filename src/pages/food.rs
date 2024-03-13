use std::collections::BTreeSet;

use chrono::{Datelike, Days, Local, Months, NaiveDate};
use leptos::*;

use leptos_router::Outlet;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};

use web_sys::MouseEvent;

use crate::Meals;

#[component]
pub fn Calendar() -> impl IntoView {
    view! {
        <div class="grid min-h-svh">
            <div class="w-11/12 place-self-center flex flex-col items-center gap-y-12 py-12">
                <h2 class="text-2xl">Pedagogiska måltider</h2>
                <Outlet/>
            </div>
        </div>
    }
}

fn get_days_for_calendar_short(date: NaiveDate) -> BTreeSet<NaiveDate> {
    let first = date
        .checked_sub_days(Days::new(date.day0() as u64))
        .unwrap();
    let offset_from_monday = first.weekday().number_from_monday() as u64 - 1;
    let offset = match offset_from_monday {
        0 => 7,
        _ => offset_from_monday
    };
    let first = first.checked_sub_days(Days::new(offset)).unwrap();
    first.iter_days().take(6 * 7).collect()
}

#[component]
pub fn Overview() -> impl IntoView {
    let (r_meals, w_meals, _) = use_local_storage::<Meals, JsonCodec>("my-meals");
    let (r_current, w_current) = create_signal(Local::now().date_naive());
    let days = move || get_days_for_calendar_short(r_current.get_untracked());
    let (r_days, w_days) =
        create_signal(Vec::from_iter(days().iter().map(|d| create_rw_signal(*d))));
    create_effect(move |_| {
        r_current.track();
        let new_days = days();
        w_days.update(|dt| {
            for (d, nd) in dt.iter_mut().zip(new_days.into_iter()) {
                d.set(nd);
            }
        });
    });
    let forward = move || {
        w_current.update(|d| *d = d.checked_add_months(Months::new(1)).unwrap());
    };
    let backward = move || {
        w_current.update(|d| *d = d.checked_sub_months(Months::new(1)).unwrap());
    };
    let cells = move || {
        r_days
            .get_untracked()
            .iter()
            .cloned()
            .map(|d| {
                view! {
                    <CalendarCell
                        date=d.into()
                        current=r_current
                        selected=Signal::derive(move || r_meals().has(d()))
                        toggle=move |_| w_meals.update(|m| m.toggle(d()))
                    />
                }
            })
            .collect_view()
    };
    view! {
        <div class="w-full max-w-xl flex gap-3 justify-center">
            <CalendarView forward=move |_| forward() backward=move |_| backward() current=r_current>
                {cells}
            </CalendarView>
        </div>
    }
}

#[component]
pub fn CalendarCell<F>(
    date: Signal<NaiveDate>,
    current: ReadSignal<NaiveDate>,
    selected: Signal<bool>,
    toggle: F,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    let iso = Signal::derive(move || date().to_string());
    let same_month = Signal::derive(move || {
        with!(|date, current| date.month0() == current.month0() && date.year() == current.year())
    });
    let not_same_month = Signal::derive(move || !same_month());
    let display_date = Signal::derive(move || date.with(|d| d.day0() + 1));
    view! {
        <button
            type="button"
            class="py-1 cursor-pointer hover:bg-primary"
            class=("bg-base-100", same_month)
            class=("bg-base-200", not_same_month)
            on:click=toggle
        >
            <time
                datetime=iso
                class="rounded-full justify-center items-center size-7 flex mx-auto"
                class=("bg-accent", selected)
                class=("text-base-100", selected)
            >
                {display_date}
            </time>
        </button>
    }
}

#[component]
pub fn CalendarView<F, FF>(
    forward: F,
    backward: FF,
    current: ReadSignal<NaiveDate>,
    children: Children,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
    FF: Fn(MouseEvent) + 'static,
{
    let current_display = Signal::derive(move || current().format("%b %Y").to_string());
    view! {
        <div class="max-w-96 w-full aspect-square">
            <div class="text-center items-center flex justify-between">
                <button
                    type="button"
                    on:click=backward
                    class="p-[0.375rem] justify-center items-center flex-none flex m-[-0.375rem] btn btn-ghost btn-circle"
                >
                    <span class="sr-only">Förra månaden</span>
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                        class="size-5"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"
                            clip-rule="evenodd"
                        ></path>
                    </svg>
                </button>
                <div class="font-semibold text-sm leading-5 select-none ">{current_display}</div>
                <button
                    on:click=forward
                    type="button"
                    class="p-[0.375rem] justify-center items-center flex-none flex m-[-0.375rem] btn btn-ghost btn-circle"
                >
                    <span class="sr-only">Nästa månad</span>
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                        class="size-5"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                            clip-rule="evenodd"
                        ></path>
                    </svg>
                </button>
            </div>
            <div class="text-primary leading-6 text-xs text-center grid-cols-7 grid mt-6">
                <div>M</div>
                <div>T</div>
                <div>O</div>
                <div>T</div>
                <div>F</div>
                <div>L</div>
                <div>S</div>
            </div>
            <div class="bg-base-300 rounded-lg ring-1 ring-base-300 shadow-sm text-sm leading-5 border-2 gap-[1px] grid grid-cols-7 mt-2 isolate">
                {children()}
            </div>
        </div>
    }
}
