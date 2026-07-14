//! Assembles Provenance state into an intermediate wiki page model.
//!
//! The model is pure data: pages with semantic sections, typed cross-page
//! links, and first-class gap and orphan notices. Rendering (HTML, themes,
//! serving) is a separate concern layered on top of [`model::WikiCorpus`].

pub mod assemble;
pub mod links;
pub mod model;
mod publish;
pub mod render;
pub mod site;
pub mod theme;
