use dioxus::prelude::*;
use crate::{
    app::Route,
    components::navbar::Navbar,
};

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-gray-50",

            Navbar {}

            main {
                class: "max-w-6xl mx-auto px-6 py-20",

                div {
                    class: "max-w-2xl",

                    h1 {
                        class: "text-5xl font-bold tracking-tight text-gray-900 mb-6",

                        "Search your documents"
                    }

                    p {
                        class: "text-xl text-gray-600 mb-8",

                        "Upload documents and search through their contents."
                    }

                    div {
                        class: "flex gap-4",

                        Link {
                            to: Route::Search {},
                            class: "
                                bg-blue-600
                                hover:bg-blue-700
                                text-white
                                px-6 py-3
                                rounded-lg
                                font-medium
                                transition
                            ",

                            "Start searching"
                        }

                        Link {
                            to: Route::Upload {},
                            class: "
                                bg-white
                                border border-gray-300
                                hover:bg-gray-50
                                text-gray-700
                                px-6 py-3
                                rounded-lg
                                font-medium
                                transition
                            ",

                            "Upload document"
                        }
                    }
                }
            }
        }
    }
}