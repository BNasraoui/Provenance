//! Pure link resolution for code and evidence references.
//!
//! Turns references such as `UseCase.php:153-156` into git host blob URLs
//! with line anchors when a remote is resolvable, or relative file links
//! otherwise. No IO except [`detect_remote_url`].

mod annotate;
mod code_ref;
mod evidence;
mod remote;

#[allow(unused_imports)]
pub use code_ref::{parse_code_ref, CodeRef, LineRange};
pub use evidence::{EvidenceRef, InlineRef, LinkResolver};
#[allow(unused_imports)]
pub use remote::{blob_url, detect_remote_url, parse_git_remote, GitHost, GitRemote};
