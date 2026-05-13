# Rue - Vue 3-like Reactive UI Framework in Rust/WASM

## Overview
Rue is a reactive UI framework for building web applications, implemented in pure Rust and compiled to WebAssembly. It follows Vue 3's Composition API patterns and provides a familiar developer experience through:

- **Signals** — reactive state management (like Vue 3's `ref()`)
- **Computed values** — derived reactive state (like `computed()`)
- **Effects** — auto-tracking side effects (like `watchEffect()`)
- **Component trait** — lifecycle-managed components with `init()`, `render()`, `mounted()`
- **Virtual DOM** — efficient diff/patch algorithm with keyed children reconciliation
- **`html!` macro** — declarative HTML templates (like Vue templates/JSX)
- **Builder API** — type-safe programmatic DOM construction

## Architecture

### Workspace Structure
```
rue/
├── Cargo.toml              # Workspace root (members: core, macros, examples/landing)
├── core/                   # rue-core library — compiles to WASM
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public API re-exports
│       ├── component.rs    # Component trait (init, render, mounted, should_update)
│       ├── reactive/       # Reactivity system
│       │   ├── mod.rs
│       │   ├── context.rs  # Global effect/signal tracking
│       │   ├── signal.rs   # Signal<T> (Vue 3 ref) — Cloneable
│       │   ├── computed.rs # Computed<T> (Vue 3 computed)
│       │   └── effect.rs   # Effect (Vue 3 watchEffect)
│       ├── node/           # Virtual DOM types
│       │   ├── mod.rs      # VNode enum, VElement, VElementBuilder
│       │   ├── mount.rs    # mount_to_dom() — create DOM from VNode
│       │   ├── patch.rs    # Core diffing/patching algorithm
│       │   └── children.rs # Keyed children reconciliation (LIS)
│       └── app.rs          # App creation, mounting, virtual-DOM updates
├── macros/                 # rue-macros proc-macro crate
│   ├── Cargo.toml
│   └── src/lib.rs          # html! macro (with {vnode: } support)
└── examples/
    └── landing/            # Landing page (full Vue 3 example port)
        ├── Cargo.toml
        ├── index.html
        ├── pkg/            # Generated WASM package
        └── src/
            ├── lib.rs      # Entry point, Root component, update_app()
            └── components/
                ├── mod.rs
                ├── navbar.rs    # NavBar struct — has Signal<bool> for mobile menu
                ├── hero.rs      # HeroSection struct — stateless
                ├── features.rs  # FeaturesSection struct — stateless
                └── footer.rs    # FooterSection struct — stateless
```

## Core API

### Component Trait
```rust
/// A UI component with lifecycle hooks (like Vue 3 Options API).
pub trait Component: 'static {
    fn init(&mut self) {}      // Called once before first render (like setup/data)
    fn mounted(&self) {}       // Called after DOM insertion (like onMounted)
    fn render(&self) -> VNode; // Called on mount and every update (like template)
    fn should_update(&self) -> bool { true } // Skip re-render optimization
}
```

### Component Example
```rust
use rue_core::*;
use rue_macros::html;

pub struct Counter {
    count: Signal<i32>,
}

impl Counter {
    pub fn new() -> Self {
        Counter { count: Signal::new(0) }
    }
}

impl Component for Counter {
    fn mounted(&self) {
        web_sys::console::log_1(&"Counter mounted!".into());
    }

    fn render(&self) -> VNode {
        let value = self.count.get_clone();
        let sig = self.count.clone();

        let handle_click = move |_| {
            let current = sig.get_clone();
            sig.set(current + 1);
            crate::update_app(); // Trigger virtual-DOM patch
        };

        html! {
            <div>
                <p>{"Count: "}{value}</p>
                <button on:click={handle_click}>{"+"}</button>
            </div>
        }
    }
}

// Usage:
let mut app = App::from_component("#app", Counter::new());
app.mount()?;
```

### Reactivity
```rust
let count = rue_core::signal(0);
count.get();       // Read value (auto-tracks in effects)
count.set(5);      // Set value (triggers dependents)
count.update(|n| *n += 1); // Mutate in place

let doubled = rue_core::computed(move || count.get() * 2);
rue_core::effect(move || {
    web_sys::console::log_1(&count.get().to_string().into());
});

// Signal is Clone — share state with event handlers
let sig = count.clone();
let handler = move |_| { sig.set(42); };
```

### VNode Builder API
```rust
VNode::element("div")
    .class("container")
    .attr("id", "main")
    .on("click", move |_| { /* handler */ })
    .child(VNode::element("span").text("Hello").build())
    .build()
```

### html! Macro
```rust
html! {
    <div class="container">
        <h1>{"Title"}</h1>
        <button on:click={handler}>{"Click"}</button>
        {vnode: dynamic_content}
    </div>
}
```

### App Mounting
```rust
// With component (recommended)
let mut app = App::from_component("#app", MyComponent::new());
app.mount()?;

// With render closure (low-level)
let mut app = App::new("#app", || html! { <div>{"Hello"}</div> });
app.mount()?;

// Trigger patch update
app.update()?; // Re-renders via virtual-DOM diffing
```

## Component Lifecycle

```
App::from_component(selector, component)
  │
  ├── component.init()         ← Initialize Signals, Computed, Effects
  │
  └── app.mount()
        │
        ├── component.render() ← Get initial VNode tree
        ├── mount_to_dom()     ← Create real DOM nodes
        ├── component.mounted() ← DOM is live, access refs
        │
        └── ... user interactions ...
              │
              └── app.update()
                    │
                    ├── component.should_update() ← Optimization gate
                    ├── component.render()        ← New VNode tree
                    └── patch_node(old, new)      ← Minimal DOM updates
```

## Virtual DOM Implementation

The patch engine implements a three-level diffing strategy:

1. **`patch_node()`** — top-level dispatcher (same type → patch; different → replace)
2. **`patch_element()`** — attribute diffing (HashMap-based), event listener lifecycle (Closure storage/cleanup), children reconciliation
3. **`patch_children()`** — keyed (LIS algorithm, same as Vue 3) or un-keyed (position-based)

Key features:
- **In-place text updates** — `set_text_content()` on existing text nodes
- **Attribute diffing** — only set/remove changed attributes
- **Event listener lifecycle** — Closures stored in VElement, properly removed/added during patch
- **No full-DOM destruction** — scroll position, input focus, and form state preserved

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| wasm-bindgen | 0.2.100+ | JS/WASM interop |
| web-sys | 0.3 | DOM API bindings |
| js-sys | 0.3 | JS types |
| console_error_panic_hook | 0.1 | Better WASM panics |
| proc-macro2, quote, syn | 2.x | Proc macro infrastructure |

## Building & Running

```bash
# Build the landing example
cd rue/examples/landing
wasm-pack build --target web --out-dir pkg

# Serve with any HTTP server
python3 -m http.server 8080
# Open http://localhost:8080
```
