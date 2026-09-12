use dioxus::logger::tracing;
use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");


#[cfg(feature = "local-api")]
const API_BASE_URL: &str = "http://localhost:3000";

#[cfg(not(feature = "local-api"))]
const API_BASE_URL: &str = "";

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
    tracing::debug!("search() query={:?}", query);
    let pairs: Vec<(&str, String)> = query.into_iter().map(|s| ("q", s)).collect();
    let qs = serde_urlencoded::to_string(&pairs).unwrap();


    let url = format!("{API_BASE_URL}/search?{qs}");

    tracing::info!("Calling API: {}", url);

    match Request::get(url.as_str()).send().await {
        Ok(res) => {
            if res.ok() {
                match res.json::<Vec<SearchResult>>().await {
                    Ok(json) => {
                        tracing::info!("API response: {:?}", json.clone());
                        *state.write() = FetchState::Loaded(json)
                    },
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
        div { class: "max-w-6xl  mx-auto space-y-4",
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
    let mut items: Vec<EntryResult> = Vec::new();

    for result in results {
        items.extend(result.results);
    }

    rsx! {
        div { class: "space-y-4",
            for item in items {
                ResultCard { result: item }
            }
        }
    }
}

fn clean_value(value: &str) -> String {
    match serde_json::from_str::<Value>(value) {
        Ok(Value::String(s)) => s,
        Ok(Value::Null) => String::new(),
        Ok(Value::Number(n)) => n.to_string(),
        Ok(Value::Bool(b)) => b.to_string(),
        Ok(other) => other.to_string(),
        Err(_) => value.to_string(),
    }
}

fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!(
            "{}/{}/{}",
            &date[6..8],
            &date[4..6],
            &date[0..4],
        )
    } else {
        date.to_string()
    }
}

#[component]
fn ResultCard(result: EntryResult) -> Element {
    let get = |key: &str| -> Option<String> {
        result
            .payload
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| clean_value(v))
    };

    let title = get("item_titulo")
        .unwrap_or_else(|| "Documento sin título".to_string());

    let department = get("departamento_nombre");
    let section = get("seccion_nombre");
    let epigraph = get("epigrafe_nombre");

    let document_id = get("item_identificador");
    let publication = get("publicacion");
    let date = get("fecha_publicacion")
        .or_else(|| get("date"));

    let page = get("page");
    let chunk_idx = get("chunk_idx");

    let text = get("text");
    let url = get("source_url");

    rsx! {
    div {
        class: "bg-gray-500 border border-gray-200 rounded-xl p-4",

        // Primera línea
        div {
            class: "flex justify-between items-start gap-4",

            div {
                class: "min-w-0",

                h2 {
                    class: "text-base font-semibold text-gray-900",
                    "{title}"
                }

                if let Some(department) = department {
                    div {
                        class: "text-lg text-gray-700 mt-1 font-bold",
                        "{department}"

                    }

                }
            }

            span {
                class: "shrink-0 text-xs text-gray-500 bg-gray-100 rounded-md px-2 py-1",
                "{result.score:.3}"
            }
        }


        // Metadata principal
        div {
            class: "flex flex-wrap items-center mt-3 text-xs text-gray-700 font-bold",
            style: "column-gap: 1.5rem; row-gap: 0.5rem;",

            if let Some(publication) = publication {
                span { "{publication}" }
            }

            if let Some(date) = date {
                span { "{format_date(&date)}" }
            }

            if let Some(page) = page {
                span { "Página {page}" }
            }

            if let Some(chunk_idx) = chunk_idx {
                span { "Chunk {chunk_idx}" }
            }
        }


        if let Some(text) = text {
            if !text.trim().is_empty() {
                div {
                    class: "mt-4 rounded-xl border border-gray-200 bg-gray-100 p-4 shadow-sm",

                    p {
                        class: "text-sm text-gray-700 leading-relaxed",
                        "{text}"
                    }
                }
            }
        }

        // Footer
        div {
        class: "mt-4 pt-3 border-t border-gray-100",
        style: "display: flex; width: 100%; justify-content: space-between; align-items: center;",

            if let Some(document_id) = document_id {
                span {
                    class: "text-xs text-gray-700",
                    "{document_id}"
                }
            }

            if let Some(url) = url {
                a {
                    href: "{url}",
                    target: "_blank",
                    class: "text-sm text-blue-600 hover:underline",
                    "Abrir BOE ↗"
                }
            }
        }
    }
}

}
