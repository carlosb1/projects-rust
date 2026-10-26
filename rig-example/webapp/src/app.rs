use dioxus::prelude::*;
use uuid::Uuid;
use crate::pages::{
    login::Login,
    home::Home,
    search::Search,
    upload::Upload,
    upload_report::UploadReport,
    uploads::Uploads,
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
    UploadReport { id: Uuid },

    #[route("/uploads")]
    Uploads {},


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