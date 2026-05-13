# Rue Landing Example — Component Architecture

## Overview

The landing example demonstrates Rue's key features:
- **`html!` macro** — declarative HTML templates (like Vue/JSX)
- **Component-local state** — each component owns its reactive state via Signals
- **Virtual DOM** — efficient patch-based updates via the diff engine
- **Composition** — components are just functions returning VNode

## File Structure

```
examples/landing/
├── Cargo.toml
├── index.html              # HTML shell, imports WASM
├── pkg/                    # Built WASM output
└── src/
    ├── lib.rs              # Entry point, app setup, update_app()
    └── components/
        ├── mod.rs          # Re-exports
        ├── navbar.rs       # NavBar component (has local state)
        ├── hero.rs         # HeroSection component (stateless)
        ├── features.rs     # FeaturesSection component (stateless)
        └── footer.rs       # FooterSection component (stateless)
```

## Component Pattern

### Stateless Component
```rust
use rue_core::VNode;
use rue_macros::html;

pub fn HeroSection() -> VNode {
    html! {
        <section id="home" class="...">
            <h1>{"Build something amazing today"}</h1>
        </section>
    }
}
```

### Stateful Component
```rust
use rue_core::*;
use rue_macros::html;
use crate::update_app;

// Component-local state — persists across renders
thread_local! {
    static IS_OPEN: Signal<bool> = Signal::new(false);
}

pub fn NavBar() -> VNode {
    let is_open = IS_OPEN.with(|s| s.get_clone());

    let handle_toggle = move |_| {
        IS_OPEN.with(|s| {
            let current = s.get_clone();
            s.set(!current);
        });
        update_app(); // triggers virtual-DOM patch
    };

    let mobile_menu = if is_open {
        html! { <div class="md:hidden">...</div> }
    } else {
        VNode::empty()
    };

    html! {
        <nav class="...">
            <button on:click={handle_toggle}>{"Toggle"}</button>
            {vnode: mobile_menu}
        </nav>
    }
}
```

## Key Patterns

### 1. Component-Local State
Each component declares its own `thread_local! { static STATE: Signal<T> }`. The Signal persists for the lifetime of the thread (the WASM instance). Components read their state on every render and mutate it in event handlers.

### 2. Triggering Updates
After mutating state, call `update_app()` (defined in `lib.rs`) which calls `app.update()` — this triggers the virtual-DOM diff/patch cycle.

### 3. `html!` Macro Text Content
All text content is wrapped in `{"..."}` expressions:
```rust
// ❌ This may fail with special characters:
<p>Hello ⚡ world</p>

// ✅ This always works:
<p>{"Hello ⚡ world"}</p>
```
This is because the macro works at the Rust token level, and raw text content containing non-identifier characters (emoji, special unicode, etc.) can't be tokenized by Rust's proc-macro tokenizer.

### 4. Embedding VNode Expressions
Use `{vnode: expr}` to embed a VNode directly (not wrapped in VNode::text):
```rust
{vnode: if condition { html! { <div>...</div> } } else { VNode::empty() }}
{vnode: some_vnode_variable}
```

### 5. Composition
Components are functions that return `VNode`. Compose them naturally:
```rust
fn root_render() -> VNode {
    VNode::element("div")
        .child(NavBar())
        .child(HeroSection())
        .build()
}
```

## `html!` Macro Syntax Reference

| Syntax | Description |
|--------|-------------|
| `<div>...</div>` | Element with children |
| `<br />` | Self-closing element |
| `<div class="foo">` | Static attribute |
| `<div class={expr}>` | Dynamic attribute (converted to string) |
| `<button on:click={handler}>` | Event listener |
| `{"text"}` | Text content (recommended over raw text) |
| `{vnode: expr}` | Embed a VNode expression |
| `<>...</>` | Fragment |

## Data Flow

```
User clicks button
  → handle_toggle closure fires
    → IS_OPEN.signal.set(!old_value)
      → Signal triggers dependents
    → update_app()
      → app.update()
        → root_render() called
          → NavBar() reads IS_OPEN signal
          → Returns new VNode tree
        → patch_node(old_vnode, new_vnode)
          → Minimal DOM updates applied
```

## Compared to Vue 3

| Concept | Vue 3 | Rue |
|---------|-------|-----|
| Reactive value | `ref()` | `Signal::new()` |
| Derived value | `computed()` | `Computed::new()` |
| Side effect | `watchEffect()` | `Effect::new()` |
| Template | Vue SFC `<template>` | `html!` macro |
| Component state | `setup()` with `ref` | `thread_local! { static STATE: Signal }` |
| State mutation | `value++` | `s.set(new_value)` |
| Trigger update | Automatic (reactivity) | Manual `update_app()` |
| Virtual DOM | Automatic | `patch_node()` on `app.update()` |
