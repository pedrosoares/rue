use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::*;

pub mod mount;
pub mod patch;
pub mod children;

// ---------------------------------------------------------------------------
// VNodeType — classification of a VNode for diffing
// ---------------------------------------------------------------------------

/// The "type" of a VNode, used to decide whether two nodes can be patched
/// in-place or must be replaced entirely.
#[derive(Clone, Debug, PartialEq)]
pub enum VNodeType {
    Element(String),       // tag name
    Text,
    Fragment,
    Dynamic,
    Empty,
}

// ---------------------------------------------------------------------------
// VNode
// ---------------------------------------------------------------------------

/// A virtual DOM node.
pub enum VNode {
    /// An HTML element with tag, attributes, event listeners, and children.
    Element(VElement),
    /// A text node.
    Text(String),
    /// A fragment containing multiple nodes.
    Fragment(Vec<VNode>),
    /// A dynamic node that re-renders when signals change.
    Dynamic(
        Rc<RefCell<Box<dyn Fn() -> VNode>>>,
        Rc<RefCell<Option<web_sys::Node>>>,
    ),
    /// A placeholder for empty/false conditions.
    Empty,
}

impl VNode {
    /// Create an element node.
    pub fn element(tag: &str) -> VElementBuilder {
        VElementBuilder::new(tag)
    }

    /// Create a text node.
    pub fn text(content: &str) -> Self {
        VNode::Text(content.to_string())
    }

    /// Create a fragment node.
    pub fn fragment(children: Vec<VNode>) -> Self {
        VNode::Fragment(children)
    }

    /// Create an empty node.
    pub fn empty() -> Self {
        VNode::Empty
    }

    // -----------------------------------------------------------------------
    // Virtual DOM helpers
    // -----------------------------------------------------------------------

    /// Return the `VNodeType` of this node (used for diffing).
    pub fn node_type(&self) -> VNodeType {
        match self {
            VNode::Element(el) => VNodeType::Element(el.tag.clone()),
            VNode::Text(_) => VNodeType::Text,
            VNode::Fragment(_) => VNodeType::Fragment,
            VNode::Dynamic(_, _) => VNodeType::Dynamic,
            VNode::Empty => VNodeType::Empty,
        }
    }

    /// Quick check: can we patch `self` (old) into `other` (new) in-place?
    /// Two nodes can be patched if they have the same VNodeType (same variant,
    /// same tag for elements).
    pub fn same_type(&self, other: &VNode) -> bool {
        match (self, other) {
            (VNode::Element(a), VNode::Element(b)) => a.tag == b.tag,
            (VNode::Text(_), VNode::Text(_)) => true,
            (VNode::Fragment(_), VNode::Fragment(_)) => true,
            (VNode::Dynamic(_, _), VNode::Dynamic(_, _)) => true,
            (VNode::Empty, VNode::Empty) => true,
            _ => false,
        }
    }

    /// Return the rendering key (`:key` attribute) for this node, if any.
    /// Only elements may have keys.
    pub fn key(&self) -> Option<&str> {
        match self {
            VNode::Element(el) => el.key.as_deref(),
            _ => None,
        }
    }

    /// Return a reference to the actual DOM node if this VNode has been
    /// mounted/rendered.
    pub fn dom_node(&self) -> Option<web_sys::Node> {
        match self {
            VNode::Element(el) => el.dom_ref.borrow().clone(),
            VNode::Dynamic(_, dom_ref) => dom_ref.borrow().clone(),
            VNode::Text(_) | VNode::Fragment(_) | VNode::Empty => None,
        }
    }

    /// Set the DOM node reference on this VNode (called during mount/patch).
    pub fn set_dom_node(&self, node: web_sys::Node) {
        match self {
            VNode::Element(el) => {
                *el.dom_ref.borrow_mut() = Some(node);
            }
            VNode::Dynamic(_, dom_ref) => {
                *dom_ref.borrow_mut() = Some(node);
            }
            VNode::Text(_) | VNode::Fragment(_) | VNode::Empty => {
                // Text nodes are created fresh each time; fragments don't store
                // a single DOM ref. This is intentionally a no-op.
            }
        }
    }

    /// Recursively collect all `(child_index, VNode)` pairs that have a key.
    /// Used by the keyed children reconciliation algorithm.
    pub fn keyed_children(&self) -> Vec<(usize, &VNode)> {
        match self {
            VNode::Element(el) => el
                .children
                .iter()
                .enumerate()
                .filter(|(_, c)| c.key().is_some())
                .collect(),
            _ => vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// VElement
// ---------------------------------------------------------------------------

/// An HTML element representation.
pub struct VElement {
    pub tag: String,
    pub attrs: Vec<(&'static str, String)>,
    pub events: Vec<(&'static str, EventHandler)>,
    pub children: Vec<VNode>,
    pub key: Option<String>,
    /// Reference to the actual DOM node (set during rendering).
    pub dom_ref: Rc<RefCell<Option<web_sys::Node>>>,
    /// Stored Closures for active event listeners, so they can be removed
    /// during patching.
    pub listener_closures: Rc<RefCell<Vec<(&'static str, Closure<dyn FnMut(web_sys::Event)>)>>>,
}

// ---------------------------------------------------------------------------
// EventHandler
// ---------------------------------------------------------------------------

/// A clonable event handler using Rc.
pub struct EventHandler(Rc<dyn Fn(web_sys::Event)>);

impl EventHandler {
    pub fn new<F: Fn(web_sys::Event) + 'static>(f: F) -> Self {
        EventHandler(Rc::new(f))
    }

    pub fn call(&self, event: web_sys::Event) {
        (self.0)(event)
    }
}

impl Clone for EventHandler {
    fn clone(&self) -> Self {
        EventHandler(self.0.clone())
    }
}

// ---------------------------------------------------------------------------
// VElementBuilder
// ---------------------------------------------------------------------------

/// Builder for creating element VNodes.
pub struct VElementBuilder {
    tag: String,
    attrs: Vec<(&'static str, String)>,
    events: Vec<(&'static str, EventHandler)>,
    children: Vec<VNode>,
    key: Option<String>,
}

impl VElementBuilder {
    fn new(tag: &str) -> Self {
        VElementBuilder {
            tag: tag.to_string(),
            attrs: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Set an attribute.
    pub fn attr(mut self, name: &'static str, value: &str) -> Self {
        self.attrs.push((name, value.to_string()));
        self
    }

    /// Set the "class" attribute.
    pub fn class(mut self, value: &str) -> Self {
        self.attrs.push(("class", value.to_string()));
        self
    }

    /// Set the "id" attribute.
    pub fn id(mut self, value: &str) -> Self {
        self.attrs.push(("id", value.to_string()));
        self
    }

    /// Set the rendering key (used for keyed children reconciliation).
    pub fn key(mut self, value: &str) -> Self {
        self.key = Some(value.to_string());
        self
    }

    /// Add an event listener.
    pub fn on<F: Fn(web_sys::Event) + 'static>(mut self, event: &'static str, handler: F) -> Self {
        self.events.push((event, EventHandler::new(handler)));
        self
    }

    /// Add a child node.
    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children.
    pub fn children(mut self, children: Vec<VNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Add text content.
    pub fn text(mut self, content: &str) -> Self {
        self.children.push(VNode::Text(content.to_string()));
        self
    }

    /// Build the element node.
    pub fn build(self) -> VNode {
        VNode::Element(VElement {
            tag: self.tag,
            attrs: self.attrs,
            events: self.events,
            children: self.children,
            key: self.key,
            dom_ref: Rc::new(RefCell::new(None)),
            listener_closures: Rc::new(RefCell::new(Vec::new())),
        })
    }
}

// ---------------------------------------------------------------------------
// Legacy render_to_dom — thin wrapper around mount::mount_to_dom
// ---------------------------------------------------------------------------

/// Render a VNode tree into actual DOM nodes and return the root node.
/// This is a simple wrapper around `mount::mount_to_dom` for backward
/// compatibility.
pub fn render_to_dom(vnode: &VNode) -> Option<web_sys::Node> {
    // We need a temporary parent to use mount_to_dom.
    let document = web_sys::window()?.document()?;
    let temp = document.create_element("div").ok()?;
    let parent: web_sys::Node = temp.into();
    mount::mount_to_dom(vnode, &parent, None)
}
