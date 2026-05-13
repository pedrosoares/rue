use crate::node::VNode;

/// A UI component with lifecycle hooks, similar to Vue 3's Options API.
///
/// # Lifecycle
///
/// 1. **`init()`** — called once before the first render. Initialize Signals,
///    Computed values, and Effects here. Equivalent to Vue 3's `setup()` or
///    the `data()` option.
///
/// 2. **`render()`** — called to produce the VNode tree. Called on initial
///    mount and on every subsequent update. This is the component's template.
///
/// 3. **`mounted()`** — called after the component's DOM nodes have been
///    inserted into the document. Use this to access DOM properties,
///    integrate with third-party JS libraries, or start timers.
///    Equivalent to Vue 3's `onMounted()`.
///
/// 4. **`should_update()`** — called before each re-render. Return `false`
///    to skip the update entirely (useful for optimization).
///
/// # Example
///
/// ```ignore
/// use rue_core::*;
/// use rue_macros::html;
///
/// struct Counter {
///     count: Signal<i32>,
/// }
///
/// impl Component for Counter {
///     fn init(&mut self) {
///         self.count = signal(0);
///     }
///
///     fn render(&self) -> VNode {
///         let value = self.count.get_clone();
///         html! {
///             <div>
///                 <p>{"Count: "}{value}</p>
///                 <button on:click={move |_| {
///                     self.count.set(self.count.get_clone() + 1);
///                 }}>{"+"}</button>
///             </div>
///         }
///     }
/// }
/// ```
pub trait Component: 'static {
    /// Initialize component state. Called once before first render.
    /// Create your Signals, Computed values, and side-effect subscriptions here.
    fn init(&mut self) {}

    /// Called after the component is mounted to the DOM and the initial
    /// VNode tree has been rendered. The DOM nodes are accessible via
    /// [`VNode::dom_node()`] on the tree returned by the first `render()` call.
    fn mounted(&self) {}

    /// Render the component's UI. Called on mount and on every update.
    /// Must return a valid [`VNode`] tree.
    fn render(&self) -> VNode;

    /// Called before each re-render. Return `false` to skip the update.
    /// Default implementation returns `true`.
    fn should_update(&self) -> bool {
        true
    }
}
