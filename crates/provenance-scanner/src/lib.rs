pub mod parser;
pub mod validate;
pub mod walker;

pub use parser::{parse_annotations, Annotation, CoverageLevel, ParseResult, ParseWarning};
pub use validate::{validate_annotations, ValidationWarning};
pub use walker::{scan_file, scan_path, AnnotationLocation, FileScan, Language};
