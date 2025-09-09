use dioxus::logger::tracing;
use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub time: f64,
    pub results: Vec<EntryResult>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntryResult {
    pub score: f32,
    pub payload: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
enum FetchState {
    Idle,
    Loading,
    Loaded(Vec<SearchResult>),
    Error(String),
}
async fn search(query: Vec<String>, mut state: Signal<FetchState>) {
    let pairs: Vec<(&str, String)> = query.into_iter().map(|s| ("q", s)).collect();
    let qs = serde_urlencoded::to_string(&pairs).unwrap();

    let url = format!("/search?{qs}");

    match Request::get(url.as_str()).send().await {
        Ok(res) => {
            if res.ok() {
                match res.json::<Vec<SearchResult>>().await {
                    Ok(json) => *state.write() = FetchState::Loaded(json),
                    Err(e) => *state.write() = FetchState::Error(format!("Error parseando JSON: {}", e)),
                }
            } else {
                *state.write() = FetchState::Error(format!("HTTP {}", res.status()));
            }
        }
        Err(e) => *state.write() = FetchState::Error(format!("Fetch error: {}", e)),
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        link { rel: "stylesheet", href: TAILWIND_CSS }
        link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "min-h-screen w-full bg-gray-50 p-6",
        div { class: "max-w-3xl  mx-auto space-y-4",
        h1 { class: "text-2xl font-bold text-black", "Buscador" }
        p { class: "text-sm text-gray-600", "Introduce consultas y pulsa Enter para añadir."
            }
                SearchView {}
            }
        }
    }
}

#[component]
fn SearchView() -> Element {
    let mut queries = use_signal(|| Vec::<String>::new());
    let mut input = use_signal(|| String::new());
    let mut state = use_signal(|| FetchState::Idle);

    let mut on_add = move || {
        let val = input.read().trim().to_string();
        if !val.is_empty() {
            queries.write().push(val);
            *input.write() = String::new();
        }
    };

    let do_search = move |_| {
        let q = queries.read().clone();
        if q.is_empty() {
            *state.write() = FetchState::Error("Añade al menos una query".into());
            return;
        }
        *state.write() = FetchState::Loading;
        spawn(search(q, state));
    };

    rsx! {
        div { class: "space-y-3",
                // Entrada + chips de queries
                div { class: "flex flex-wrap gap-2 items-center",
                // Chips
                for (idx, q) in queries.read().iter().cloned().enumerate() {
                Chip { label: q, on_remove: move |_| {
                    queries.write().remove(idx);
                    }}
                }
                // Input
                input {
                    class: "flex-1 min-w-[220px] border rounded-xl px-3 py-2 shadow-sm text-black",
                    placeholder: "Escribe una consulta y Enter…",
                    value: "{input.read()}",
                    oninput: move |e| *input.write() = e.value().to_string(),
                    onkeydown: move |e| {
                      if e.key() == Key::Enter { on_add(); }
                    }
                }
                button {
                    class: "px-4 py-2 rounded-xl bg-black text-white",
                    onclick: do_search,
                    "Buscar"
                }
            }
            match &*state.read() {
                FetchState::Idle => rsx!( p { class: "text-gray-500", "Esperando consulta…" } ),
                FetchState::Loading => rsx!( p { class: "animate-pulse", "Buscando…" } ),
                FetchState::Error(msg) => rsx!( p { class: "text-red-600", "{msg}" } ),
                FetchState::Loaded(res) => rsx!( ResultsList { results: res.clone() } ),
                }
        }
    }
}

#[component]
fn Chip(label: String, on_remove: EventHandler<MouseEvent>) -> Element {
    rsx! {
        span { class: "inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white border shadow-sm",
            span { class: "text-sm text-black", "{label}" }
            button { class: "text-xs text-gray-500 hover:text-black", onclick: on_remove, "✕" }
        }
    }
}

#[component]
fn ResultsList(results: Vec<SearchResult>) -> Element {
    // let total = result.total.unwrap_or_default();
    // let items = result.results.unwrap_or_default();

    let mut items: Vec<(f32, String)> = Vec::new();
    for result in results {
        for e in &result.results {
            items.push( (e.score, format!("{:?}", e.payload)));
        }
    }
    rsx! {
        div { class: "space-y-2",
        for item in items.iter() {
            div { class: "p-4 bg-white border rounded-xl shadow-sm space-y-1",
                    p { class: "text-sm ", "Score: {item.0}"}
                    p { class: "text-sm text-gray-700", "{item.1}" }
            }
        }
        }
    }
}
