use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

use super::mount::mount_to_dom;
use super::children::patch_children;
use super::{EventHandler, VElement, VNode};

/// SVG namespace URI
const SVG_NS: &str = "http://www.w3.org/2000/svg";

// ---------------------------------------------------------------------------
// Top-level patch dispatcher
// ---------------------------------------------------------------------------

/// Patch `old` VNode tree into `new` VNode tree in-place within `parent`.
///
/// Returns the (possibly new) root DOM node for the patched subtree.
/// When `old` and `new` are the same type, the existing DOM node is reused
/// and only the differences are applied (attributes, events, children).
/// When they differ, the old DOM node is replaced entirely.
pub fn patch_node(
    old: &VNode,
    new: &VNode,
    parent: &web_sys::Node,
    anchor: Option<&web_sys::Node>,
) -> Option<web_sys::Node> {
    // If both are empty, nothing to do
    if matches!(old, VNode::Empty) && matches!(new, VNode::Empty) {
        return None;
    }

    // If old is empty but new is not → mount new
    if matches!(old, VNode::Empty) {
        return mount_to_dom(new, parent, anchor);
    }

    // If new is empty but old is not → remove old
    if matches!(new, VNode::Empty) {
        if let Some(old_node) = old.dom_node() {
            let _ = parent.remove_child(&old_node);
        }
        return None;
    }

    // If types differ → replace
    if !old.same_type(new) {
        return replace_node(old, new, parent, anchor);
    }

    // Same type — dispatch to specialized patch
    match (old, new) {
        (VNode::Element(old_el), VNode::Element(new_el)) => {
            Some(patch_element(old_el, new_el, parent, anchor))
        }
        (VNode::Text(old_text), VNode::Text(new_text)) => {
            patch_text_node(old_text, new_text, old)
        }
        (VNode::Fragment(old_children), VNode::Fragment(new_children)) => {
            // A fragment is transparent — delegate to children patching
            // The "old DOM node" for a fragment is tricky; we patch children directly.
            // We pass old.children and new.children.
            patch_children(old_children, new_children, parent);
            // Return the first child of the new fragment as the "root"
            new_children.first().and_then(|c| c.dom_node())
        }
        (VNode::Dynamic(_old_fn, _old_dom), VNode::Dynamic(new_fn, new_dom)) => {
            // Dynamic nodes: re-run the render function and patch the result.
            // We use the new render function.
            let f = new_fn.borrow();
            let new_inner = f();
            drop(f);

            // Remove old dynamic content if it exists
            if let Some(old_node) = old.dom_node() {
                let _ = parent.remove_child(&old_node);
            }

            // Mount the new inner VNode
            if let Some(node) = mount_to_dom(&new_inner, parent, anchor) {
                // Update the new Dynamic node's DOM reference
                *new_dom.borrow_mut() = Some(node.clone());
                Some(node)
            } else {
                None
            }
        }
        _ => {
            // Fallback: replace
            replace_node(old, new, parent, anchor)
        }
    }
}

// ---------------------------------------------------------------------------
// Replace: old and new have different types
// ---------------------------------------------------------------------------

/// Remove the old DOM node and mount the new one in its place.
fn replace_node(
    old: &VNode,
    new: &VNode,
    parent: &web_sys::Node,
    anchor: Option<&web_sys::Node>,
) -> Option<web_sys::Node> {
    // Remove old DOM node
    if let Some(old_node) = old.dom_node() {
        let _ = parent.remove_child(&old_node);
    }

    // Mount new node (insert at the old node's position if we know it)
    mount_to_dom(new, parent, anchor)
}

// ---------------------------------------------------------------------------
// Patch text node
// ---------------------------------------------------------------------------

/// Update a text node's content if it changed.
fn patch_text_node(old_text: &str, new_text: &str, old_vnode: &VNode) -> Option<web_sys::Node> {
    if let Some(node) = old_vnode.dom_node() {
        if old_text != new_text {
            let _ = node.set_text_content(Some(new_text));
        }
        Some(node)
    } else {
        // No existing DOM node — should not happen, but handle gracefully
        None
    }
}

// ---------------------------------------------------------------------------
// Patch element
// ---------------------------------------------------------------------------

/// Patch an old `VElement` into a new one, reusing the existing DOM element.
fn patch_element(
    old: &VElement,
    new: &VElement,
    _parent: &web_sys::Node,
    _anchor: Option<&web_sys::Node>,
) -> web_sys::Node {
    // Get the existing DOM element from old
    let dom_elem: web_sys::Element = match *old.dom_ref.borrow() {
        Some(ref node) => {
            // Safety: we know it's an Element because we created it that way
            node.clone().dyn_into::<web_sys::Element>().unwrap_or_else(|_| {
                // Fallback: create a new element with correct namespace
                create_element_ns(&new.tag)
            })
        }
        None => {
            // No existing element — create a new one with correct namespace
            create_element_ns(&new.tag)
        }
    };

    // Patch attributes
    patch_attributes(old, new, &dom_elem);

    // Patch event listeners
    patch_event_listeners(old, new, &dom_elem);

    // Update the new VElement's dom_ref to reuse the existing DOM element
    let node: web_sys::Node = dom_elem.clone().into();
    *new.dom_ref.borrow_mut() = Some(node.clone());

    // Patch children
    patch_children(&old.children, &new.children, &node);

    node
}

/// Create a DOM element with the correct namespace.
/// Uses SVG namespace for SVG elements, HTML namespace otherwise.
fn create_element_ns(tag: &str) -> web_sys::Element {
    let doc = web_sys::window().unwrap().document().unwrap();
    if tag == "svg" {
        doc.create_element_ns(Some(SVG_NS), tag)
    } else {
        doc.create_element(tag)
    }
    .unwrap()
}

// ---------------------------------------------------------------------------
// Patch attributes
// ---------------------------------------------------------------------------

/// Compare old and new attributes, applying only the changes.
fn patch_attributes(old: &VElement, new: &VElement, el: &web_sys::Element) {
    // Build lookup maps: name → value
    let old_map: std::collections::HashMap<&str, &str> =
        old.attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let new_map: std::collections::HashMap<&str, &str> =
        new.attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();

    // Attributes in new but not in old, or with different values → set
    for (name, new_val) in &new.attrs {
        match old_map.get(name) {
            Some(old_val) if *old_val == new_val.as_str() => {
                // Same value — skip
            }
            _ => {
                let _ = el.set_attribute(name, new_val);
            }
        }
    }

    // Attributes in old but not in new → remove
    for (name, _) in &old.attrs {
        if !new_map.contains_key(name) {
            let _ = el.remove_attribute(name);
        }
    }
}

// ---------------------------------------------------------------------------
// Patch event listeners
// ---------------------------------------------------------------------------

/// Compare old and new event listeners, adding/removing as needed.
fn patch_event_listeners(old: &VElement, new: &VElement, el: &web_sys::Element) {
    // Build lookup: event_name → handler reference (from old)
    let old_events: std::collections::HashMap<&str, &EventHandler> =
        old.events.iter().map(|(k, v)| (*k, v)).collect();
    let new_events: std::collections::HashMap<&str, &EventHandler> =
        new.events.iter().map(|(k, v)| (*k, v)).collect();

    // Remove event listeners that are no longer present or have changed
    let mut closures_to_keep: Vec<(&'static str, Closure<dyn FnMut(web_sys::Event)>)> = Vec::new();

    for (event_name, handler) in &old.events {
        match new_events.get(event_name) {
            Some(new_handler) => {
                // Event still exists — check if handler changed
                // Since EventHandler stores Rc, we compare ptr equality
                let changed = !Rc::ptr_eq(&handler.0, &new_handler.0);
                if changed {
                    // Remove old listener
                    if let Some(old_closure) = find_and_remove_closure(&old, event_name) {
                        let js_func: &js_sys::Function = old_closure.as_ref().unchecked_ref();
                        let _ = el.remove_event_listener_with_callback(event_name, js_func);
                        // old_closure dropped here → JS callback released
                    }
                    // Add new listener
                    let new_closure = create_and_add_listener(el, event_name, new_handler);
                    closures_to_keep.push((event_name, new_closure));
                } else {
                    // Same handler — keep the old closure
                    if let Some(old_closure) = take_closure(&old, event_name) {
                        closures_to_keep.push((event_name, old_closure));
                    } else {
                        // No old closure found — create new one
                        let new_closure = create_and_add_listener(el, event_name, new_handler);
                        closures_to_keep.push((event_name, new_closure));
                    }
                }
            }
            None => {
                // Event removed — remove old listener
                if let Some(old_closure) = find_and_remove_closure(&old, event_name) {
                    let js_func: &js_sys::Function = old_closure.as_ref().unchecked_ref();
                    let _ = el.remove_event_listener_with_callback(event_name, js_func);
                    // old_closure dropped here
                }
            }
        }
    }

    // Add brand-new events (in new but not in old)
    for (event_name, handler) in &new.events {
        if !old_events.contains_key(event_name) {
            let new_closure = create_and_add_listener(el, event_name, handler);
            closures_to_keep.push((event_name, new_closure));
        }
    }

    // Update the new VElement's listener_closures
    *new.listener_closures.borrow_mut() = closures_to_keep;
}

/// Find and remove (by taking) a closure from the old VElement's stored list.
fn take_closure(
    el: &VElement,
    event_name: &str,
) -> Option<Closure<dyn FnMut(web_sys::Event)>> {
    let mut closures = el.listener_closures.borrow_mut();
    if let Some(pos) = closures.iter().position(|(name, _)| *name == event_name) {
        let (_, closure) = closures.remove(pos);
        Some(closure)
    } else {
        None
    }
}

/// Find and remove a closure from the old VElement's stored list (dropping it).
fn find_and_remove_closure(el: &VElement, event_name: &str) -> Option<Closure<dyn FnMut(web_sys::Event)>> {
    take_closure(el, event_name)
}

/// Create a Closure for an event handler, add it to the element, and return it.
fn create_and_add_listener(
    el: &web_sys::Element,
    event_name: &str,
    handler: &EventHandler,
) -> Closure<dyn FnMut(web_sys::Event)> {
    let handler = handler.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        handler.call(event);
    }) as Box<dyn FnMut(web_sys::Event)>);

    let js_func: &js_sys::Function = closure.as_ref().unchecked_ref();
    let _ = el.add_event_listener_with_callback(event_name, js_func);

    closure
}
