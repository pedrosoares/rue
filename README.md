<div align="center">
  <h1>🌿 Rue</h1>
  <p><strong>A Vue 3-like Reactive UI Framework in Rust/WASM</strong></p>
  <p>
    <a href="#features">Features</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#core-concepts">Core Concepts</a> •
    <a href="#api-summary">API</a> •
    <a href="#project-structure">Structure</a>
  </p>
  <p>
    <img src="https://img.shields.io/badge/rust-stable-orange" alt="Rust">
    <img src="https://img.shields.io/badge/webassembly-target-purple" alt="WASM">
    <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
  </p>
</div>

---

**Rue** is a reactive UI framework for building web applications, written in pure Rust and compiled to WebAssembly. It draws heavy inspiration from **Vue 3's Composition API**, offering a familiar developer experience:

- **Signals** — reactive state management (like `ref()`)
- **Computed values** — derived reactive state (like `computed()`)
- **Effects** — auto-tracking side effects (like `watchEffect()`)
- **Virtual DOM** — efficient diff/patch algorithm with keyed reconciliation
- **`html!` macro** — declarative templates in Rust (like Vue SFC templates)
- **Component trait** — lifecycle-managed components with `init()`, `render()`, `mounted()`

## Features

| Feature | Description |
|---------|-------------|
| ⚡ **Reactive System** | Signal-based reactivity with automatic dependency tracking |
| 🖼️ **Virtual DOM** | Efficient diffing/patching with in-place text/attribute updates |
| 🔑 **Keyed Reconciliation** | Longest-Increasing-Subsequence algorithm (same as Vue 3) |
| 📦 **`html!` Macro** | Compile-time HTML-to-VNode conversion with event support |
| 🧩 **Component Trait** | Lifecycle hooks: `init()`, `render()`, `mounted()`, `should_update()` |
| 🛠️ **Builder API** | Type-safe programmatic DOM construction |
| 🌐 **WASM Target** | Compiles to WebAssembly via `wasm-bindgen` |
| 🎨 **Tailwind CSS Compatible** | Works seamlessly with utility CSS frameworks |

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- Python 3 (for local dev server) or any HTTP server

### Clone & Build

```bash
git clone https://github.com/yourusername/rue.git
cd rue

# Build the landing page example
cd examples/landing
wasm-pack build --target web --out-dir pkg

# Serve it
python3 -m http.server 8080
# Open http://localhost:8080
```

### Your First App

```rust
use rue_core::*;
use rue_macros::html;

// Define a component
struct Counter {
    count: Signal<i32>,
}

impl Counter {
    pub fn new() -> Self {
        Counter { count: Signal::new(0) }
    }
}

impl Component for Counter {
    fn render(&self) -> VNode {
        let value = self.count.get_clone();
        let sig = self.count.clone();
        let handle_click = move |_| {
            sig.update(|n| *n += 1);
            crate::update_app(); // trigger re-render
        };

        html! {
            <div>
                <p>{"Count: "}{value}</p>
                <button on:click={handle_click}>{"+"}</button>
            </div>
        }
    }
}

// Mount it
fn main() -> Result<(), JsValue> {
    rue_core::init();
    let mut app = App::from_component("#app", Counter::new());
    app.mount()
}
```

## Core Concepts

### Reactivity (Vue 3 Composition API Pattern)

```rust
// Signal = ref()
let count = signal(0);
count.get();          // Read value (auto-tracks in effects)
count.set(5);         // Set value (triggers dependents)
count.update(|n| *n += 1); // Mutate in place

// Computed = computed()
let doubled = computed(move || count.get() * 2);

// Effect = watchEffect()
effect(move || {
    web_sys::console::log_1(&count.get().to_string().into());
});
```

### Component Model

Components are **structs implementing the `Component` trait** (similar to Vue 3 Options API or `<script setup>`):

```rust
pub trait Component: 'static {
    fn init(&mut self) {}       // Once before first render
    fn mounted(&self) {}        // After DOM insertion (like onMounted)
    fn render(&self) -> VNode;  // Returns VNode tree (like template)
    fn should_update(&self) -> bool { true } // Optimization gate
}
```

You can also use the **functional component style** (like `<script setup>`):

```rust
thread_local! {
    static COUNT: Signal<i32> = Signal::new(0);
}

pub fn MyComponent() -> VNode {
    let count = COUNT.with(|s| s.get_clone());
    let handle = move |_| {
        COUNT.with(|s| s.update(|n| *n += 1));
        update_app();
    };

    html! {
        <button on:click={handle}>{"Clicked: "}{count}</button>
    }
}
```

### Virtual DOM

Re-renders go through a three-level diff/patch engine:

1. **`patch_node()`** — compare node types (same type → patch; different → replace)
2. **`patch_element()`** — attribute diffing, event listener lifecycle
3. **`patch_children()`** — keyed (LIS algorithm) or un-keyed reconciliation

This ensures **minimal DOM mutations** — text nodes update in-place, attributes are diffed, event listeners are properly cleaned up, and scroll position / focus is preserved.

### `html!` Macro Syntax

| Syntax | Description |
|--------|-------------|
| `<div>...</div>` | Element with children |
| `<br />` | Self-closing element |
| `<div class="foo">` | Static attribute |
| `<div class={expr}>` | Dynamic attribute (auto `.to_string()`) |
| `<button on:click={handler}>` | Event listener |
| `{"text"}` | Text content (recommended) |
| `{vnode: expr}` | Embed a VNode directly |
| `<>...</>` | Fragment |

> **Note:** Always wrap text in `{"..."}` expressions — the proc-macro tokenizer handles Rust tokens, not general unicode text.

### Builder API (Alternative to `html!`)

```rust
VNode::element("div")
    .class("container")
    .attr("id", "main")
    .on("click", move |_| { /* handler */ })
    .child(VNode::element("span").text("Hello").build())
    .children(vec![VNode::text("Item 1"), VNode::text("Item 2")])
    .build()
```

### Triggering Updates

After mutating component state, call `update_app()` to trigger the virtual-DOM patch cycle:

```rust
fn handle_click() {
    // 1. Mutate signal
    COUNT.with(|s| s.set(new_value));
    // 2. Trigger re-render & patch
    update_app();
}
```

## API Summary

| Function / Type | Description | Vue 3 Equivalent |
|----------------|-------------|-------------------|
| `signal(value)` | Create a reactive `Signal<T>` | `ref()` |
| `computed(|| ...)` | Create a derived `Computed<T>` | `computed()` |
| `effect(|| ...)` | Create an auto-tracking side effect | `watchEffect()` |
| `Signal<T>` | Cloneable reactive value with `get()`, `set()`, `update()` | `Ref<T>` |
| `App::new(selector, render_fn)` | Create app with render closure | `createApp()` |
| `App::from_component(selector, component)` | Create app from `Component` trait | `defineComponent()` |
| `app.mount()` | Initial render and DOM mounting | `mount()` |
| `app.update()` | Diff/patch re-render cycle | `nextTick()` + re-render |
| `VNode` | Virtual DOM node enum | `VNode` |
| `html! { ... }` | HTML template macro | Vue SFC `<template>` |

## Project Structure

```
rue/
├── Cargo.toml                    # Workspace root
├── core/                         # rue-core library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Public API re-exports
│       ├── component.rs          # Component trait
│       ├── app.rs                # App creation & mounting
│       ├── reactive/
│       │   ├── mod.rs
│       │   ├── context.rs        # Global tracking context
│       │   ├── signal.rs         # Signal<T> implementation
│       │   ├── computed.rs       # Computed<T> implementation
│       │   └── effect.rs         # Effect implementation
│       └── node/
│           ├── mod.rs            # VNode, VElement, VElementBuilder
│           ├── mount.rs          # DOM mounting
│           ├── patch.rs          # Diff/patch algorithm
│           └── children.rs       # Keyed reconciliation (LIS)
├── macros/                       # rue-macros proc-macro crate
│   ├── Cargo.toml
│   └── src/lib.rs               # html! macro implementation
└── examples/
    └── landing/                  # Full landing page example
        ├── Cargo.toml
        ├── index.html
        └── src/
            ├── lib.rs            # Entry point
            └── components/
                ├── mod.rs
                ├── navbar.rs
                ├── hero.rs
                ├── features.rs
                └── footer.rs
```

### Data Flow

```
User clicks button
  → event handler fires
    → Signal.set(new_value)
      → triggers dependents
    → update_app()
      → app.update()
        → root_render() composes component functions
          → Each component reads its Signals
          → Returns new VNode tree via html! macro
        → patch_node(old_vnode, new_vnode)
          → patch_element (attributes, events, children)
            → patch_attributes (HashMap diff)
            → patch_event_listeners (Closure lifecycle)
            → patch_children (keyed via LIS)
              → mount_to_dom (new nodes)
              → patch_node recursively
          → Real DOM updated efficiently
```

## Building & Running

```bash
# Build the core library (WASM)
cd rue
wasm-pack build --target web

# Build the landing example
cd examples/landing
wasm-pack build --target web --out-dir pkg

# Serve locally
python3 -m http.server 8080
# → http://localhost:8080

# Or use any HTTP server
npx http-server .
```

### Development within a larger project

Add `rue-core` and `rue-macros` to your `Cargo.toml`:

```toml
[dependencies]
rue-core = { path = "../rue/core" }
rue-macros = { path = "../rue/macros" }
wasm-bindgen = "0.2"
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `wasm-bindgen` | 0.2.100+ | JS/WASM interop |
| `web-sys` | 0.3 | DOM API bindings |
| `js-sys` | 0.3 | JavaScript types |
| `console_error_panic_hook` | 0.1 | Better WASM panic messages |
| `proc-macro2` / `quote` / `syn` | 2.x | Procedural macro infrastructure |

## Comparison with Vue 3

| Concept | Vue 3 | Rue |
|---------|-------|-----|
| Reactive value | `ref()` | `Signal::new()` |
| Derived value | `computed()` | `Computed::new()` |
| Side effect | `watchEffect()` | `Effect::new()` |
| Template | SFC `<template>` | `html!` macro |
| Component state | `setup()` with `ref` | `thread_local!` or struct fields |
| State mutation | `value++` | `s.set(new_value)` |
| Trigger update | Automatic | Manual `update_app()` |
| Virtual DOM | Automatic | `patch_node()` on `app.update()` |
| Lifecycle | `onMounted` etc. | `mounted()` trait method |

## Example

The [landing example](./rue/examples/landing) is a port of a Vue 3 landing page, showcasing:

- **NavBar** — responsive navigation with mobile menu toggle (local state via `Signal<bool>`)
- **HeroSection** — stateless hero banner with gradient background
- **FeaturesSection** — feature cards with icons and descriptions
- **FooterSection** — footer with links and social icons

## Built With [Code Architect](https://code-architect-frontend.developer-71b.workers.dev/)

This project was built using **Code Architect** — an AI-powered development assistant that helps design, implement, and evolve software projects.

The project's architecture, conventions, and design decisions are documented in the integrated knowledge base at [`.coder/docs/`](.coder/docs/). This documentation serves as the source of truth for the project and guides all development work.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <sub>Built with ❤️ using Rust & WebAssembly & <a href="https://code-architect-frontend.developer-71b.workers.dev/">Code Architect</a></sub>
</div>
