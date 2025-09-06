include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

// Re-export reqwest for users of this crate
pub use reqwest;

// Re-export reqwest-middleware when tracing feature is enabled
#[cfg(feature = "tracing")]
pub use reqwest_middleware;
