//! The `athena` binary's library half: wiring, displays, and hook entry points. `main.rs` parses the
//! command line and calls into these and the core.

pub mod display;
pub mod hooks;
pub mod wiring;
