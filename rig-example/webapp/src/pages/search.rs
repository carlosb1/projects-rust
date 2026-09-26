use dioxus::prelude::*;

use crate::components::{
    navbar::Navbar,
    pagination::Pagination,
    result_card::{ResultCard},
};
use crate::models::search::SearchResult;

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResult>::new);
    let mut current_page = use_signal(|| 1usize);
    let mut total_pages = use_signal(|| 1usize);


    // Datos temporales hasta conectar Axum.
    let mut temp_results = vec![
        SearchResult {
            title: "Real Decreto 123/2025".into(),
            excerpt: "Este documento establece las condiciones aplicables a...".into(),
            source: "BOE · página 12".into(),
            score: 0.94,
        },
        SearchResult {
            title: "Ley 18/2024".into(),
            excerpt: "Las administraciones públicas deberán garantizar...".into(),
            source: "BOE · página 4".into(),
            score: 0.88,
        },
        SearchResult {
            title: "Resolución de 4 de marzo".into(),
            excerpt: "De acuerdo con lo establecido en el artículo...".into(),
            source: "BOE · página 27".into(),
            score: 0.82,
        },
    ];

    rsx! {
        div {
            class: "min-h-screen bg-gray-50",

            Navbar {}

            main {
                class: "max-w-5xl mx-auto px-6 py-12",

                h1 {
                    class: "text-3xl font-bold text-gray-900 mb-2",

                    "Search"
                }

                p {
                    class: "text-gray-500 mb-8",

                    "Search across your indexed documents."
                }

                form {
                    class: "flex gap-3 mb-10",

                    onsubmit: move |event| {
                        event.prevent_default();
                        let mut tmp_results = temp_results.clone();

                        let query_value = query();

                        spawn(async move {
                            match crate::api::search::search(
                                &query_value,
                                1,
                            ).await {
                                Ok(response) => {
                                    /*
                                    results.set(response.items);
                                    current_page.set(response.page);
                                    total_pages.set(response.total_pages);
                                     */
                                    results.set(tmp_results.clone());
                                    current_page.set(1);
                                    total_pages.set(1);
                                }

                                Err(err) => {
                                    results.set(tmp_results.clone());
                                    current_page.set(1);
                                    total_pages.set(1);
                                    println!("Search error: {err}");
                                }
                            }
                        });
                    },

                    input {
                        class: "
                            flex-1
                            bg-white
                            border border-gray-300
                            rounded-lg
                            px-4 py-3
                            text-gray-900
                            placeholder-gray-400
                            focus:outline-none
                            focus:ring-2
                            focus:ring-blue-500
                            focus:border-transparent
                        ",

                        placeholder: "Search documents...",

                        value: "{query}",

                        oninput: move |event| {
                            query.set(event.value());
                        }
                    }

                    button {
                        r#type: "submit",

                        class: "
                            bg-blue-600
                            hover:bg-blue-700
                            text-white
                            px-6 py-3
                            rounded-lg
                            font-medium
                            transition
                        ",

                        "Search"
                    }
                }

                div {
                    class: "flex items-center justify-between mb-5",

                    h2 {
                        class: "text-lg font-semibold text-gray-900",

                        "Results"
                    }

                    span {
                        class: "text-sm text-gray-500",

                        "{results.len()} results"
                    }
                }

                div {
                    class: "space-y-4",

                    for result in results() {
                        ResultCard {
                            result
                        }
                    }
                }

                Pagination {
                    current_page: 1,
                    total_pages: 5
                }
            }
        }
    }
}