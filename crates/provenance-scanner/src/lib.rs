mod binding_lexer;

pub mod parser;
pub mod validate;
pub mod walker;

pub use parser::{
    parse_annotations, Annotation, CoverageLevel, ParseResult, ParseWarning, Verification,
};
pub use validate::{validate_annotations, validate_bindings, ValidationWarning};
pub use walker::{scan_file, scan_path, AnnotationLocation, AttributeBinding, FileScan, Language};
