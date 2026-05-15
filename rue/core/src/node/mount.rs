use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

use super::VNode;

/// SVG namespace URI
const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// Mount a VNode tree into the DOM by creating real DOM nodes.
///
/// * `vnode` — the virtual node tree to mount
/// * `parent` — the DOM node to attach children to
/// * `anchor` — optional child node to insert before (if None, append)
///
/// Returns the first DOM node created (the "root" of the rendered subtree).
/// For elements this is the element itself; for text nodes it's the text node;
/// for fragments it's the first child; for empty it's None.
pub fn mount_to_dom(
    vnode: &VNode,
    parent: &web_sys::Node,
    anchor: Option<&web_sys::Node>,
) -> Option<web_sys::Node> {
    mount_to_dom_inner(vnode, parent, anchor, None)
}

/// Internal helper that carries namespace context for SVG support.
fn mount_to_dom_inner(
    vnode: &VNode,
    parent: &web_sys::Node,
    anchor: Option<&web_sys::Node>,
    namespace: Option<&str>,
) -> Option<web_sys::Node> {
    match vnode {
        VNode::Element(el) => {
            let document = web_sys::window()?.document()?;

            // Determine the namespace for this element.
            // <svg> always uses the SVG namespace. Other elements inherit
            // from their parent (SVG children stay in SVG namespace).
            let ns = if el.tag == "svg" {
                Some(SVG_NS)
            } else {
                namespace
            };

            // Create the element with the correct namespace
            let elem: web_sys::Element = match ns {
                Some(ns_str) => document.create_element_ns(Some(ns_str), &el.tag).ok()?,
                None => document.create_element(&el.tag).ok()?,
            };

            // Set attributes
            for (name, value) in &el.attrs {
                let _ = elem.set_attribute(name, value);
            }

            // Set event listeners using Closure — store for cleanup
            for (event_name, handler) in &el.events {
                let handler_cls = {
                    let handler = handler.clone();
                    Closure::wrap(Box::new(move |event: web_sys::Event| {
                        handler.call(event);
                    }) as Box<dyn FnMut(web_sys::Event)>)
                };

                let js_func: &js_sys::Function = handler_cls.as_ref().unchecked_ref();
                let _ = elem.add_event_listener_with_callback(event_name, js_func);

                // Store the closure so we can remove it later during patching
                el.listener_closures
                    .borrow_mut()
                    .push((event_name, handler_cls));
            }

            // Mount children — propagate the namespace
            for child in &el.children {
                if let Some(child_node) = mount_to_dom_inner(child, &elem, None, ns) {
                    let _ = elem.append_child(&child_node);
                }
            }

            let node: web_sys::Node = elem.into();
            *el.dom_ref.borrow_mut() = Some(node.clone());

            // Insert into parent at the right position
            if let Some(ref anchor_node) = anchor {
                let _ = parent.insert_before(&node, Some(anchor_node));
            } else {
                let _ = parent.append_child(&node);
            }

            Some(node)
        }

        VNode::Text(text) => {
            let document = web_sys::window()?.document()?;
            let text_node = document.create_text_node(text);
            let node: web_sys::Node = text_node.into();

            if let Some(ref anchor_node) = anchor {
                let _ = parent.insert_before(&node, Some(anchor_node));
            } else {
                let _ = parent.append_child(&node);
            }

            Some(node)
        }

        VNode::Fragment(children) => {
            let mut first_child = None;
            for child in children {
                if let Some(child_node) = mount_to_dom_inner(child, parent, anchor, namespace) {
                    if first_child.is_none() {
                        first_child = Some(child_node);
                    }
                }
            }
            first_child
        }

        VNode::Dynamic(render_fn, dom_ref) => {
            let f = render_fn.borrow();
            let new_vnode = f();
            drop(f);

            if let Some(node) = mount_to_dom_inner(&new_vnode, parent, anchor, namespace) {
                *dom_ref.borrow_mut() = Some(node.clone());
                Some(node)
            } else {
                None
            }
        }

        VNode::Empty => None,
    }
}
