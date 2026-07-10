mod artifacts;
mod collaboration;
mod graph;
mod ideation;
mod services;
mod shaping;

pub use artifacts::*;
pub use collaboration::*;
pub use graph::*;
pub use ideation::*;
pub use services::*;
pub use shaping::*;

fn normalize_enum_value(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_lowercase()
}
