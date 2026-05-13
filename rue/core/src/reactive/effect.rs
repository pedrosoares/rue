use crate::reactive::context;

/// An effect (like Vue 3's `watchEffect()`).
/// Runs a closure and automatically tracks any signals read inside it.
/// Re-runs whenever a tracked signal changes.
pub struct Effect {
    effect_id: usize,
}

impl Effect {
    /// Create a new effect from the given closure.
    pub fn new<F: Fn() + Clone + 'static>(f: F) -> Self {
        let effect_id = context::run_effect(Box::new(f.clone()));
        // Run immediately to establish dependencies
        context::track_effect(effect_id, f);
        Effect { effect_id }
    }

    /// Get the effect ID.
    pub fn id(&self) -> usize {
        self.effect_id
    }
}

/// Create a new effect (convenience function).
pub fn effect<F: Fn() + Clone + 'static>(f: F) -> Effect {
    Effect::new(f)
}
