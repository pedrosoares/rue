use std::cell::RefCell;
use std::rc::Rc;
use crate::reactive::context;

/// A computed value (like Vue 3's `computed()`).
/// Lazily evaluates a closure, caching the result and re-evaluating
/// when its dependencies change.
pub struct Computed<T: Clone> {
    value: Rc<RefCell<T>>,
    dirty: Rc<RefCell<bool>>,
    compute_fn: Rc<dyn Fn() -> T>,
    effect_id: usize,
}

impl<T: Clone + 'static> Computed<T> {
    /// Create a new computed value from the given computation function.
    pub fn new<F: Fn() -> T + 'static>(f: F) -> Self {
        let compute_fn: Rc<dyn Fn() -> T> = Rc::new(f);
        let initial = (compute_fn)();
        let value = Rc::new(RefCell::new(initial));
        let dirty = Rc::new(RefCell::new(true));

        let value_clone = value.clone();
        let dirty_clone = dirty.clone();
        let compute_clone = compute_fn.clone();

        let effect_id = context::run_effect(Box::new(move || {
            // Re-compute
            let result = (compute_clone)();
            *value_clone.borrow_mut() = result;
            *dirty_clone.borrow_mut() = false;
        }));

        // Run the effect once to establish dependencies
        let compute_for_effect = compute_fn.clone();
        context::track_effect(effect_id, move || {
            let _ = (compute_for_effect)();
        });

        Computed {
            value,
            dirty,
            compute_fn,
            effect_id,
        }
    }

    /// Get the computed value. This will re-compute if dependencies have changed.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        context::track_signal(self.effect_id);
        if *self.dirty.borrow() {
            let result = (self.compute_fn)();
            *self.value.borrow_mut() = result;
            *self.dirty.borrow_mut() = false;
        }
        self.value.borrow()
    }
}

/// Create a new computed value (convenience function).
pub fn computed<T: Clone + 'static, F: Fn() -> T + 'static>(f: F) -> Computed<T> {
    Computed::new(f)
}
