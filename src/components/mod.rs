use leptos::*;
#[component]
pub fn InputWrap(
    label: &'static str,
    #[prop(optional, into)] error: Option<MaybeSignal<String>>,
    #[prop(optional, into)] explanation: Option<MaybeSignal<String>>,
    #[prop(optional, into)] extra: Option<MaybeSignal<String>>,
    children: Children,
) -> impl IntoView {
    let top_right = extra.map(|x| view! { <span class="label-text-alt">{x}</span> });
    let bottom_left = explanation.map(|x| view! { <span class="label-text-alt">{x}</span> });
    let bottom_right =
        error.map(|x| view! { <span class="label-text-alt text-error italic">{x}</span> });

    view! {
        <label class="form-control w-full max-w-xs">
            <div class="label">
                <span class="label-text">{label}</span>
                {top_right}
            </div>
            {children()}
            <div class="label">{bottom_left} {bottom_right}</div>
        </label>
    }
}
