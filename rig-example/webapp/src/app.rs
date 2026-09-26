use dioxus::prelude::*;

use crate::pages::{
    login::Login,
    home::Home,
    search::Search,
    upload::Upload,
    upload_report::UploadReport,
};

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
//    #[route("/")]
//    Login {},

    #[route("/")]
    Home{},

    #[route("/search")]
    Search {},

    #[route("/upload")]
    Upload {},


    #[route("/uploads/:id")]
    UploadReport { id: String },
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        }

        Router::<Route> {}
    }
}