# Rue - Vue 3-like Reactive UI Framework in Rust/WASM

## Overview
Rue is a reactive UI framework for building web applications, implemented in pure Rust and compiled to WebAssembly. It follows Vue 3's Composition API patterns and provides a familiar developer experience.

## Architecture

### Workspace Structure
```
rue/
├── Cargo.toml              # Workspace root
├── core/                   # rue-core library (compiles to wasm)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Public API re-exports
│       ├── reactive/       # Reactivity system
│       │   ├── mod.rs
│       │   ├── context.rs  # Global effect/signal tracking
│       │   ├── signal.rs   # Signal<T> (like Vue 3's ref)
│       │   ├── computed.rs # Computed<T> (like Vue 3's computed)
│       │   └── effect.rs   # Effect (like Vue 3's watchEffect)
│       ├── node/           # Virtual Node types & DOM manipulation
│       │   ├── mod.rs      # VNode enum, VElement, VElementBuilder
│       │   ├── mount.rs    # mount_to_dom() — create DOM from VNode
│       │   ├── patch.rs    # Core diffing/patching algorithm
│       │   ├── children.rs # Keyed children reconciliation (LIS)
│       │   └── dom_utils.rs# Low-level DOM helpers
│       └── app.rs          # Application creation and mounting
├── macros/                 # rue-macros proc-macro crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # html! and component macros (with {vnode: } support)
└── examples/
    └── landing/            # Landing page (full Vue 3 example port)
        ├── Cargo.toml
        ├── index.html
        └── src/
            ├── lib.rs      # Entry point with WASM start + update_app()
            └── components/
                ├── mod.rs  # Re-exports all component functions
                ├── navbar.rs   # NavBar — has local Signal<bool> for mobile menu
                ├── hero.rs     # HeroSection — stateless, pure html! macro
                ├── features.rs # FeaturesSection — stateless, pure html! macro
                └── footer.rs   # FooterSection — stateless, pure html! macro
```

### Data Flow
```
User clicks button
  → component event handler fires
    → component-local Signal.set(new_value)
      → Signal triggers dependents
    → update_app() (global helper)
      → app.update()
        → root_render() composes component functions
          → Each component reads its Signals
          → Returns new VNode tree via html! macro
        → patch_node(old_vnode, new_vnode)
          → patch_element (attributes, events, children)
            → patch_attributes (HashMap diff)
            → patch_event_listeners (Closure lifecycle)
            → patch_children (keyed via LIS or un-keyed)
              → mount_to_dom (for new nodes)
              → patch_node recursively
          → Real DOM updated efficiently
```

## Key Concepts

### Reactivity (like Vue 3 Composition API)
- **Signal<T>** = `ref()` — single reactive value
- **Computed<T>** = `computed()` — derived reactive value
- **Effect** = `watchEffect()` — auto-running side effects
- All based on a global subscriber/dependency tracking system

### Component Model
- Components are **functions returning VNode** (like Vue 3 `<script setup>`)
- Component-local state via `thread_local! { static STATE: Signal<T> }`
- State mutations call `update_app()` to trigger virtual-DOM patch
- No special trait or struct needed — just functions

### Virtual DOM
- **Diffing** — Compare old and new VNode trees to find minimal changes
- **Patching** — Apply only the changed parts to the real DOM
- **Keyed Reconciliation** — Efficient list reordering using longest-increasing-subsequence algorithm
- **In-place Updates** — Text nodes, attributes, and event listeners update without DOM replacement
- **Event Listener Lifecycle** — Closures stored and properly cleaned up during patching

### `html!` Macro
- Compile-time HTML-to-VNode conversion
- Supports: elements, text (`{"..."}`), attributes, events (`on:click={}`), VNode expressions (`{vnode: }`)
- Hyphenated attributes (e.g., `stroke-linecap`) handled via token sequence parsing
- Falls back to `VNode::empty()` for conditional rendering

## Core API

### Reactivity
```rust
let count = rue_core::signal(0);
count.get();       // Read value (auto-tracks in effects)
count.set(5);      // Set value (triggers dependents)
count.update(|n| *n += 1);

let doubled = rue_core::computed(move || count.get() * 2);
rue_core::effect(move || { web_sys::console::log_1(&count.get().to_string().into()); });
```

### VNode Builder API
```rust
VNode::element("div")
    .class("container")
    .attr("id", "main")
    .on("click", move |_| { /* handler */ })
    .child(VNode::element("span").text("Hello").build())
    .children(vec![VNode::text("Item 1"), VNode::text("Item 2")])
    .build()
```

### `html!` Macro
```rust
html! {
    <div class="container">
        <h1>{"Title"}</h1>
        <p>{"Static text here"}</p>
        <button on:click={move |_| handle_click()}>{"Click"}</button>
        {vnode: dynamic_content}
    </div>
}
```

### App + Virtual DOM
```rust
let mut app = rue_core::App::new("#app", || {
    html! { <div>{"Hello World"}</div> }
});
app.mount()?;   // Initial render
app.update()?;  // Patch-based re-render (does NOT destroy DOM)
```

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
cd rue/examples/landing
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
# Open http://localhost:8080
```
