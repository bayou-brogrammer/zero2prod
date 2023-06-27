#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

pub mod configuration;
pub mod db;
pub mod routes;
pub mod startup;
pub mod telemetry;
