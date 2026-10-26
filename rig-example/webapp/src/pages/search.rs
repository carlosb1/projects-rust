use dioxus::prelude::*;

use crate::components::{
    navbar::Navbar,
    pagination::Pagination,
    result_card::{ResultCard},
};
use crate::models::search::SearchResult;

#[component]
pub fn HighlightedText(text: String, query: String) -> Element {
    if query.is_empty() {
        return rsx! { "{text}" };
    }

    rsx! {
        for (index, part) in text.split(&query).enumerate() {
            if index > 0 {
                mark {
                    class: "bg-yellow-200 text-gray-900 rounded px-0.5",
                    "{query}"
                }
            }

            "{part}"
        }
    }
}

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResult>::new);
    let mut current_page = use_signal(|| 1usize);
    let mut total_pages = use_signal(|| 1usize);

    let mut submitted_query = use_signal(String::new);

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
                        //let mut tmp_results = temp_results.clone();
                        //let mut tmp_results = Vec::new();

                        let query_value = query();
                        submitted_query.set(query_value.clone());


                        spawn(async move {
                            match crate::api::search::search(
                                &query_value,
                                1,
                            ).await {
                                Ok(response) => {
                                    results.set(response.items);
                                    current_page.set(response.page);
                                    total_pages.set(response.total_pages);
                                    //results.set(tmp_results.clone());
                                    //current_page.set(1);
                                    //total_pages.set(1);
                                }
                                Err(err) => {
                                    //results.set(tmp_results.clone());
                                    //current_page.set(1);
                                    //total_pages.set(1);
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
                            result,
                            query: submitted_query(),
                        }
                    }
                }

            Pagination {
                current_page: current_page(),
                total_pages: total_pages(),
                on_page_change: move |page| {
                    let query_value = submitted_query();

                    spawn(async move {
                        match crate::api::search::search(&query_value, page).await {
                            Ok(response) => {
                                results.set(response.items);
                                current_page.set(response.page);
                                total_pages.set(response.total_pages);
                            }
                            Err(err) => {
                                println!("Search error: {err}");
                            }
                        }
                    });
                },
}
            }
        }
    }
}