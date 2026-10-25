use dioxus::prelude::*;

use crate::{
    api::uploads::get_upload_report,
    components::navbar::Navbar,
};

#[component]
pub fn UploadReport(id: String) -> Element {
    let mut report = use_signal(|| None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| true);

    {
        let id = id.clone();

        use_effect(move || {
            let id = id.clone();

            spawn(async move {
                match get_upload_report(&id).await {
                    Ok(value) => {
                        report.set(Some(value));
                    }

                    Err(err) => {
                        error.set(Some(err));
                    }
                }

                loading.set(false);
            });
        });
    }

    rsx! {
        div {
            class: "min-h-screen bg-gray-50",

            Navbar {}

            main {
                class: "max-w-4xl mx-auto px-6 py-12",

                h1 {
                    class: "text-3xl font-bold text-gray-900 mb-8",
                    "Upload report"
                }

                if loading() {
                    p {
                        class: "text-gray-500",
                        "Loading report..."
                    }
                } else if let Some(message) = error() {
                    p {
                        class: "text-red-600",
                        "{message}"
                    }
                } else if let Some(report) = report() {
                    div {
                        class: "
                            bg-white
                            border border-gray-200
                            rounded-xl
                            shadow-sm
                            p-6
                            space-y-5
                        ",

                        div {
                            h2 {
                                class: "text-xl font-semibold text-gray-900",
                                "Job {report.id}"
                            }

                            p {
                                class: "text-gray-500 mt-1",
                                "Status: {report.status}"
                            }
                        }

                        /*
                        div {
                            p {
                                class: "text-sm text-gray-500 mb-2",
                                "Progress"
                            }

                            div {
                                class: "w-full bg-gray-100 rounded-full h-3",

                                div {
                                    class: "bg-blue-600 h-3 rounded-full",
                                    style: "width: {report.progress}%;"
                                }
                            }

                            p {
                                class: "text-sm text-gray-500 mt-2",
                                "{report.progress}%"
                            }
                        }



                        div {
                            class: "grid grid-cols-2 gap-4",

                            div {
                                class: "bg-gray-50 rounded-lg p-4",

                                p {
                                    class: "text-sm text-gray-500",
                                    "Processed"
                                }

                                p {
                                    class: "text-2xl font-semibold text-gray-900",
                                    "{report.processed}"
                                }
                            }

                            div {
                                class: "bg-gray-50 rounded-lg p-4",

                                p {
                                    class: "text-sm text-gray-500",
                                    "Failed"
                                }

                                p {
                                    class: "text-2xl font-semibold text-gray-900",
                                    "{report.failed}"
                                }
                            }
                        }
*/
                        /*
                        if let Some(message) = report.message {
                            div {
                                class: "border-t border-gray-200 pt-4",

                                p {
                                    class: "text-gray-700",
                                    "{message}"
                                }
                            }
                        }
                         */
                    }
                }
            }
        }
    }
}