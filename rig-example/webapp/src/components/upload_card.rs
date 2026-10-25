use dioxus::prelude::*;

use crate::{
    app::Route,
    models::upload::{UploadItem, UploadStatus},
};

#[component]
pub fn UploadCard(upload: UploadItem) -> Element {
    let status = match upload.status {
        UploadStatus::Queued => "Queued",
        UploadStatus::Processing => "Processing",
        UploadStatus::Completed => "Completed",
        UploadStatus::Failed => "Failed",
    };

    rsx! {
        article {
            class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",

            div {
                class: "flex justify-between items-start",

                div {
                    h3 {
                        class: "font-semibold text-gray-900",
                        "{upload.filename}"
                    }

                    p {
                        class: "text-sm text-gray-500 mt-1",
                        "{status}"
                    }
                }

            }


            div {
                class: "flex justify-end mt-5",

                Link {
                    to: Route::UploadReport {
                        id: upload.id.clone().unwrap_or(uuid::Uuid::nil()),
                    },

                    class: "
                        border border-gray-300
                        bg-white
                        hover:bg-gray-50
                        text-gray-700
                        px-4 py-2
                        rounded-lg
                        text-sm
                        font-medium
                    ",

                    "View report"
                }
            }
        }
    }
}