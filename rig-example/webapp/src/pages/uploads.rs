use dioxus::prelude::*;

use crate::{
    api::uploads::list_reports,
    app::Route,
    components::navbar::Navbar,
};

#[component]
pub fn Uploads() -> Element {
    let reports = use_resource(move || async move {
        list_reports().await
    });

    rsx! {
        div {
            class: "min-h-screen bg-gray-50",

            Navbar {}

            main {
                class: "max-w-6xl mx-auto px-6 py-12",

                h1 {
                    class: "text-3xl font-bold text-gray-900 mb-8",
                    "My reports"
                }

                match &*reports.read_unchecked() {
                    None => rsx! {
                        p {
                            class: "text-gray-500",
                            "Loading reports..."
                        }
                    },

                    Some(Err(error)) => rsx! {
                        p {
                            class: "text-red-600",
                            "Failed to load reports: {error}"
                        }
                    },

                    Some(Ok(items)) => rsx! {
                        if items.is_empty() {
                            p {
                                class: "text-gray-500",
                                "No reports yet."
                            }

                            Link {
                                to: Route::Upload {},
                                class: "inline-block mt-4 text-blue-600 hover:underline",
                                "Upload a document"
                            }
                        } else {
                            div {
                                class: "grid gap-4",

                                for report in items {
                                    div {
                                        key: "{report.report_id}",
                                        class: "bg-white border border-gray-200 rounded-xl p-6",

                                        h2 {
                                            class: "font-semibold text-gray-900 break-all",
                                            "Report {report.report_id}"
                                        }

                                        p {
                                            class: "text-gray-600 mt-2",
                                            "Status: {report.status}"
                                        }

                                        p {
                                            class: "text-sm text-gray-500 mt-1",
                                            {
                                                format!(
                                                    "Created: {}",
                                                    report.created_at.format("%d/%m/%Y %H:%M UTC")
                                                )
                                            }
                                        }

                                        Link {
                                            to: Route::UploadReport {
                                                id: report.report_id,
                                            },
                                            class: "inline-flex mt-4 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition",
                                            "View report"
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}