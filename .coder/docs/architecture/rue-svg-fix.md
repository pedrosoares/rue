# SVG Support Fix

## Problem
SVG elements (like `<svg>`, `<path>`, `<circle>`, etc.) were not being rendered because they were created with `document.createElement()` (HTML namespace) instead of `document.createElementNS("http://www.w3.org/2000/svg", tag)` (SVG namespace). This caused the browser to treat them as unknown HTML elements rather than SVG elements.

## Fix

### `core/src/node/mount.rs`
- Added `const SVG_NS: &str = "http://www.w3.org/2000/svg"`
- Refactored `mount_to_dom()` into a public-facing wrapper and internal `mount_to_dom_inner()` that carries a `namespace: Option<&str>` parameter
- When creating an element with tag `"svg"`, the namespace is set to `SVG_NS`. Child elements inherit the namespace from their parent, so SVG child elements (like `<path>`, `<circle>`, `<g>`) are also created in the correct SVG namespace
- Uses `document.create_element_ns(Some(ns_str), tag)` for SVG elements, falls back to `document.create_element(tag)` for HTML elements

### `core/src/node/patch.rs`
- Added same `SVG_NS` constant
- Added `create_element_ns()` helper function that detects `"svg"` tag and uses the SVG namespace
- Updated `patch_element()` fallback element creation paths to use `create_element_ns()` instead of raw `document.create_element()`

## How it works
When the VNode tree is being mounted:
1. `<svg>` tag triggers `SVG_NS` namespace
2. All children of `<svg>` inherit the SVG namespace
3. Elements are created with `createElementNS("http://www.w3.org/2000/svg", "tag")`
4. During patching, if a new element needs to be created as fallback, it also uses the correct namespace

## Files changed
- `rue/core/src/node/mount.rs` — namespace-aware element creation, recursive namespace propagation
- `rue/core/src/node/patch.rs` — namespace-aware fallback element creation
