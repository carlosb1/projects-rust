use dioxus::prelude::*;
use crate::models::search::SearchResult;
use crate::pages::search::HighlightedText;

#[component]
pub fn ResultCard(result: SearchResult, query: String) -> Element {
    rsx! {
        article {
            class: "bg-white border border-gray-200 rounded-xl p-6 shadow-sm",

            h2 {
                class: "text-lg font-semibold",
                "{result.title}"
            }

            p {
                class: "text-gray-600",

                HighlightedText {
                    text: result.excerpt,
                    query,
                }
            }
        }
    }
}
/*
#[component]
pub fn ResultCard(result: SearchResult) -> Element {
    rsx! {
        article {
            class: "
                bg-white
                border border-gray-200
                rounded-xl
                p-6
                shadow-sm
                hover:shadow-md
                transition
            ",

            div {
                class: "flex justify-between gap-4 mb-3",

                h2 {
                    class: "text-lg font-semibold text-gray-900",

                    "{result.title}"
                }

                span {
                    class: "
                        shrink-0
                        text-xs
                        font-medium
                        bg-blue-50
                        text-blue-700
                        px-2.5 py-1
                        rounded-full
                    ",

                    "{result.score:.2}"
                }
            }

            p {
                class: "text-gray-600 leading-relaxed mb-4",

                "{result.excerpt}"
            }

            div {
                class: "text-sm text-gray-400",

                "{result.source}"
            }
        }
    }
}

 */