use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "bg-white border-b border-gray-200",

            div {
                class: "max-w-6xl mx-auto px-6 h-16 flex items-center justify-between",

                Link {
                    to: Route::Home {},
                    class: "text-xl font-semibold text-gray-900",

                    "Rust Search"
                }

                div {
                    class: "flex items-center gap-6",

                    Link {
                        to: Route::Search {},
                        class: "text-gray-600 hover:text-blue-600 transition",
                        "Search"
                    }

                    Link {
                        to: Route::Upload {},
                        class: "text-gray-600 hover:text-blue-600 transition",
                        "Upload"
                    }

                    Link {
                        to: Route::Uploads {},
                        class: "text-gray-600 hover:text-blue-600 transition",
                        "My reports"
                    }
                }
            }
        }
    }
}