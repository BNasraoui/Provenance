//! Pure link resolution for code and evidence references.
//!
//! Turns references such as `UseCase.php:153-156` into commit-pinned git host
//! blob URLs with line anchors when both a remote and revision are known.
//! Otherwise the reference remains plain text. No IO except [`detect_remote_url`].

mod annotate;
mod code_ref;
mod evidence;
mod remote;

pub use code_ref::{parse_code_ref, parse_code_refs, CodeRef, LineRange};
pub use evidence::{EvidenceRef, EvidenceSnippet, InlineRef, LinkResolver};
pub use remote::detect_remote_url;
