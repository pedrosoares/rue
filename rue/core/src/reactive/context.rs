use std::cell::RefCell;

thread_local! {
    static CURRENT_EFFECT: RefCell<Option<EffectId>> = const { RefCell::new(None) };
    static EFFECTS: RefCell<Vec<Box<dyn Fn()>>> = const { RefCell::new(Vec::new()) };
    static TRACKING: RefCell<Vec<Vec<EffectId>>> = const { RefCell::new(Vec::new()) };
}

type EffectId = usize;

/// Register an effect to be re-run when its dependencies change.
pub fn run_effect(f: Box<dyn Fn()>) -> EffectId {
    EFFECTS.with(|e| {
        let mut effects = e.borrow_mut();
        let id = effects.len();
        effects.push(f);
        id
    })
}

/// Execute a function while tracking which signals it reads.
pub fn track_effect(effect_id: EffectId, f: impl Fn()) {
    // Set current effect
    let prev = CURRENT_EFFECT.with(|c| c.replace(Some(effect_id)));
    TRACKING.with(|t| {
        // Ensure we have a tracking list slot for this effect
        let mut tracking = t.borrow_mut();
        while tracking.len() <= effect_id {
            tracking.push(Vec::new());
        }
        tracking[effect_id].clear();
    });

    // Run the function (signals will register themselves via track_signal)
    f();

    // Reset current effect
    CURRENT_EFFECT.with(|c| {
        c.replace(prev);
    });
}

/// Called by Signal::get() to register the current effect as a subscriber.
pub fn track_signal(signal_id: usize) {
    CURRENT_EFFECT.with(|c| {
        if let Some(effect_id) = *c.borrow() {
            TRACKING.with(|t| {
                let mut tracking = t.borrow_mut();
                if effect_id < tracking.len() {
                    if !tracking[effect_id].contains(&signal_id) {
                        tracking[effect_id].push(signal_id);
                    }
                }
            });
        }
    });
}

/// Get all effect IDs that depend on a given signal.
pub fn get_dependent_effects(signal_id: usize) -> Vec<EffectId> {
    TRACKING.with(|t| {
        let tracking = t.borrow();
        let mut deps = Vec::new();
        for (effect_id, signals) in tracking.iter().enumerate() {
            if signals.contains(&signal_id) {
                deps.push(effect_id);
            }
        }
        deps
    })
}

/// Schedule effects that depend on the given signal to be re-run.
/// Returns the effect IDs that need re-running (caller can execute them).
pub fn trigger_effects(signal_id: usize) {
    let deps = get_dependent_effects(signal_id);
    EFFECTS.with(|e| {
        let effects = e.borrow();
        for dep_id in deps {
            if dep_id < effects.len() {
                let effect = &effects[dep_id];
                track_effect(dep_id, effect);
            }
        }
    });
}
