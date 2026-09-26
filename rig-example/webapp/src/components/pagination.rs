use dioxus::prelude::*;

#[component]
pub fn Pagination(
    current_page: usize,
    total_pages: usize,
) -> Element {
    rsx! {
        nav {
            class: "flex justify-center items-center gap-2 mt-10",

            button {
                disabled: current_page <= 1,

                class: "
                    px-3 py-2
                    border border-gray-300
                    rounded-lg
                    text-sm text-gray-600
                    bg-white
                    hover:bg-gray-50
                    disabled:opacity-40
                    disabled:cursor-not-allowed
                ",

                "Previous"
            }

            for page in 1..=total_pages {

                button {
                    class: if page == current_page {
                        "
                        px-3 py-2
                        rounded-lg
                        text-sm
                        bg-blue-600
                        text-white
                        "
                    } else {
                        "
                        px-3 py-2
                        rounded-lg
                        text-sm
                        bg-white
                        border border-gray-300
                        text-gray-600
                        hover:bg-gray-50
                        "
                    },

                    "{page}"
                }
            }

            button {
                disabled: current_page >= total_pages,

                class: "
                    px-3 py-2
                    border border-gray-300
                    rounded-lg
                    text-sm text-gray-600
                    bg-white
                    hover:bg-gray-50
                    disabled:opacity-40
                    disabled:cursor-not-allowed
                ",

                "Next"
            }
        }
    }
}