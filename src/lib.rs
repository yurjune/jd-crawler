pub mod client;
pub mod models;

pub use client::WantedClient;
pub use models::Job;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
