//! Pure link resolution for code and evidence references.
//!
//! Turns references such as `UseCase.php:153-156` into git host blob URLs
//! with line anchors when a remote is resolvable, or relative file links
//! otherwise. No IO except [`detect_remote_url`].

mod annotate;
mod code_ref;
mod evidence;
mod remote;

pub use evidence::{EvidenceRef, InlineRef, LinkResolver};
pub use remote::detect_remote_url;
