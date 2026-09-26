mod app;
mod components;
mod pages;

pub mod models;
pub mod api;

use app::App;

fn main() {
    dioxus::launch(App);
}