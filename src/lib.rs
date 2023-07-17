#![allow(clippy::missing_errors_doc)]

mod apps;
mod io;
mod backend_panel;
mod frame_history;
mod wrap_app;

pub use wrap_app::WrapApp;

// ----------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;
