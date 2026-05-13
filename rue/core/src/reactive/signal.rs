use std::cell::RefCell;
use std::rc::Rc;
use crate::reactive::context;

/// A reactive value (like Vue 3's `ref()`).
/// Wraps a value and notifies dependents when the value changes.
#[derive(Clone)]
pub struct Signal<T> {
    value: Rc<RefCell<T>>,
    id: usize,
}

static SIGNAL_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl<T> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(value: T) -> Self {
        let id = SIGNAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Signal {
            value: Rc::new(RefCell::new(value)),
            id,
        }
    }

    /// Get the current value. Registers the current effect as a dependency.
    pub fn get(&self) -> std::cell::Ref<'_, T> {
        context::track_signal(self.id);
        self.value.borrow()
    }

    /// Set a new value and notify dependents.
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        context::trigger_effects(self.id);
    }

    /// Update the value in place using a closure.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut *self.value.borrow_mut());
        context::trigger_effects(self.id);
    }

    /// Get the signal ID (used internally for dependency tracking).
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: Clone> Signal<T> {
    /// Get a cloned copy of the value.
    pub fn get_clone(&self) -> T {
        self.get().clone()
    }
}

impl<T: Clone + 'static> Signal<T> {
    /// Create a signal from a reference, tracking it as a dependency.
    /// This is useful inside computed values.
    pub fn from_ref(signal: &Signal<T>) -> Self {
        let val = signal.get_clone();
        let new = Signal::new(val);
        new
    }
}

/// Create a new signal (convenience function).
pub fn signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}
