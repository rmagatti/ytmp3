#![feature(let_chains)]
#![recursion_limit = "256"]

pub mod app;
pub mod converter;

pub mod components;
pub mod domain;
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
