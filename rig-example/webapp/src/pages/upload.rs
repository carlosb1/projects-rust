use dioxus::prelude::*;

use crate::{
    components::{
        navbar::Navbar,
        upload_card::UploadCard,
    },
    models::upload::{
        UploadItem,
        UploadStatus,
    },
};

#[component]
pub fn Upload() -> Element {
    let mut selected_file = use_signal(|| Option::<String>::None);
    let mut file_data = use_signal(|| Option::<Vec<u8>>::None);

    let mut uploads = use_signal(Vec::<UploadItem>::new);

    rsx! {
        div {
            class: "min-h-screen bg-gray-50",

            Navbar {}

            main {
                class: "max-w-4xl mx-auto px-6 py-12",

                h1 {
                    class: "text-3xl font-bold text-gray-900 mb-2",
                    "Upload ZIP"
                }

                p {
                    class: "text-gray-500 mb-8",
                    "Upload a ZIP file to process its documents."
                }

                // Upload box
                div {
                    class: "
                        bg-white
                        border border-gray-200
                        rounded-xl
                        p-6
                        shadow-sm
                        mb-10
                    ",

                    div {
                        class: "
                            border-2
                            border-dashed
                            border-gray-300
                            rounded-xl
                            p-10
                            text-center
                        ",

                        input {
                            id: "zip-upload",
                            r#type: "file",
                            accept: ".zip,application/zip",
                            class: "hidden",

                            onchange: move |event| {
                                let files = event.files();

                                let Some(file) = files.first() else {
                                    return;
                                };

                                selected_file.set(Some(file.name()));

                                let file = file.clone();

                                spawn(async move {
                                    match file.read_bytes().await {
                                        Ok(bytes) => {
                                            file_data.set(Some(bytes.to_vec()));
                                        }

                                        Err(err) => {
                                            println!("Could not read file: {err:?}");
                                        }
                                    }
                                });
                            }
                        }

                        label {
                            r#for: "zip-upload",
                            class: "
                                inline-block
                                cursor-pointer
                                bg-blue-600
                                hover:bg-blue-700
                                text-white
                                px-6 py-3
                                rounded-lg
                                font-medium
                            ",

                            "Choose ZIP"
                        }

                        p {
                            class: "mt-3 text-sm text-gray-500",
                            "Select a .zip file"
                        }
                    }

                    if let Some(filename) = selected_file() {
                        p {
                            class: "mt-5 text-sm text-gray-700",

                            "Selected: "

                            span {
                                class: "font-medium",
                                "{filename}"
                            }
                        }
                    }

                    button {
                        disabled: file_data().is_none(),

                        class: "
                            mt-6
                            bg-blue-600
                            hover:bg-blue-700
                            disabled:bg-gray-300
                            disabled:cursor-not-allowed
                            text-white
                            px-6 py-3
                            rounded-lg
                        ",

                        onclick: move |_| {
                            let Some(filename) = selected_file() else {
                                return;
                            };

                            let Some(data) = file_data() else {
                                return;
                            };

                            // Añadimos card inmediatamente
                            let upload_id = filename.clone();

                            uploads.write().insert(
                                0,
                                UploadItem {
                                    id: upload_id.clone(),
                                    filename: filename.clone(),
                                    status: UploadStatus::Processing,
                                    progress: 20,
                                    message: Some("Uploading...".into()),
                                },
                            );

                            spawn(async move {
                                match crate::api::uploads::upload_zip(
                                    filename.clone(),
                                    data,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        let mut uploads = uploads.write();

                                        if let Some(upload) = uploads
                                            .iter_mut()
                                            .find(|u| u.id == upload_id)
                                        {
                                            upload.status = UploadStatus::Completed;
                                            upload.progress = 100;
                                            upload.message =
                                                Some("Upload completed".into());
                                        }
                                    }

                                    Err(err) => {
                                        let mut uploads = uploads.write();

                                        if let Some(upload) = uploads
                                            .iter_mut()
                                            .find(|u| u.id == upload_id)
                                        {
                                            upload.status = UploadStatus::Failed;
                                            upload.progress = 100;
                                            upload.message = Some(err);
                                        }
                                    }
                                }
                            });
                        },

                        "Upload"
                    }
                }

                // Upload history
                div {
                    class: "flex items-center justify-between mb-5",

                    h2 {
                        class: "text-xl font-semibold text-gray-900",
                        "Uploads"
                    }

                    span {
                        class: "text-sm text-gray-500",
                        "{uploads().len()} uploads"
                    }
                }

                if uploads().is_empty() {
                    div {
                        class: "
                            bg-white
                            border border-gray-200
                            rounded-xl
                            p-8
                            text-center
                            text-gray-500
                        ",

                        "No uploads yet."
                    }
                } else {
                    div {
                        class: "space-y-4",

                        for upload in uploads() {
                            UploadCard { upload }
                        }
                    }
                }
            }
        }
    }
}