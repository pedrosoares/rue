pub mod reactive;
pub mod node;
pub mod app;
pub mod component;

// Re-export the most commonly used types
pub use reactive::{Signal, signal, Computed, computed, Effect, effect};
pub use node::VNode;
pub use component::Component;
pub use app::{App, mount};

/// Initialize the framework (call this at the start of your main).
pub fn init() {
    console_error_panic_hook::set_once();
}

// Re-export JsValue for convenience
pub use wasm_bindgen::JsValue;
