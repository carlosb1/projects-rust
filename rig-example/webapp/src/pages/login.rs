use dioxus::prelude::*;
use crate::app::Route;

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);

    let navigator = use_navigator();

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 flex items-center justify-center",

            form {
                class: "w-full max-w-sm bg-white border border-gray-200 rounded-xl shadow-sm p-8",

                onsubmit: move |event| {
                    event.prevent_default();

                    let username_value = username();
                    let password_value = password();
                    let navigator = navigator.clone();

                    spawn(async move {
                        match crate::api::auth::login(
                            username_value,
                            password_value,
                        ).await {
                            Ok(_) => {
                                navigator.push(Route::Search {});
                            }

                            Err(err) => {
                                error.set(Some(err));
                            }
                        }
                    });
                },

                h1 {
                    class: "text-2xl font-bold text-gray-900 mb-6",
                    "Sign in"
                }

                input {
                    class: "w-full border rounded-lg px-4 py-3 mb-4",
                    placeholder: "Username",
                    value: "{username}",
                    oninput: move |e| username.set(e.value())
                }

                input {
                    class: "w-full border rounded-lg px-4 py-3 mb-6",
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value())
                }

                button {
                    class: "w-full bg-blue-600 text-white rounded-lg py-3",
                    r#type: "submit",
                    "Sign in"
                }
            }
        }
    }
}