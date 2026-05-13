use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

use crate::component::Component;
use crate::node::{mount::mount_to_dom, patch::patch_node, VNode};

/// The Rue application.
pub struct App {
    /// The root VNode render function.
    root_fn: Box<dyn Fn() -> VNode>,
    /// The mount point selector (CSS selector).
    selector: String,
    /// The mount point DOM element.
    mount_element: Option<web_sys::Element>,
    /// The previously rendered VNode tree (used for diffing).
    old_vnode: Option<VNode>,
    /// Shared component reference for lifecycle hooks.
    component_ctrl: Option<Rc<RefCell<Box<dyn Component>>>>,
}

impl App {
    /// Create a new application from a render closure.
    ///
    /// This is the low-level constructor. Prefer [`App::from_component`] for
    /// lifecycle-managed components.
    pub fn new<F: Fn() -> VNode + 'static>(selector: &str, root_fn: F) -> Self {
        App {
            root_fn: Box::new(root_fn),
            selector: selector.to_string(),
            mount_element: None,
            old_vnode: None,
            component_ctrl: None,
        }
    }

    /// Create a new application from a [`Component`].
    ///
    /// This will call [`Component::init()`] once, then use [`Component::render()`]
    /// for every render cycle. After the first mount, [`Component::mounted()`]
    /// is called. Before each update, [`Component::should_update()`] is checked.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let app = App::from_component("#app", MyComponent::new());
    /// app.mount()?;
    /// ```
    pub fn from_component<T: Component>(selector: &str, mut component: T) -> Self {
        // 1. Call init() lifecycle
        component.init();

        // 2. Box and share for closure + lifecycle access
        let comp: Rc<RefCell<Box<dyn Component>>> =
            Rc::new(RefCell::new(Box::new(component)));

        // 3. Create render closure that calls component.render()
        let comp_clone = comp.clone();
        let root_fn = Box::new(move || -> VNode {
            comp_clone.borrow().render()
        });

        App {
            root_fn,
            selector: selector.to_string(),
            mount_element: None,
            old_vnode: None,
            component_ctrl: Some(comp),
        }
    }

    /// Mount the application to the DOM.
    ///
    /// Renders the initial VNode tree, inserts it into the mount point,
    /// and calls [`Component::mounted()`] if a component was provided.
    pub fn mount(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let mount_point = document
            .query_selector(&self.selector)
            .ok()
            .flatten()
            .ok_or("mount point not found")?;

        // Clear the mount point
        mount_point.set_inner_html("");

        // Render the root VNode
        let vnode = (self.root_fn)();
        let parent: web_sys::Node = mount_point.clone().into();
        let _ = mount_to_dom(&vnode, &parent, None);

        // Store state for future updates
        self.mount_element = Some(mount_point);
        self.old_vnode = Some(vnode);

        // Call mounted() lifecycle
        if let Some(ref comp) = self.component_ctrl {
            let borrowed = comp.borrow();
            borrowed.mounted();
        }

        Ok(())
    }

    /// Update the application by diffing the old and new VNode trees and
    /// applying only the necessary changes to the DOM.
    ///
    /// Before re-rendering, calls [`Component::should_update()`] if a component
    /// was provided. Skips the update if it returns `false`.
    ///
    /// Unlike the previous naive implementation (which destroyed and recreated
    /// all DOM nodes), this preserves scroll position, input focus, and
    /// form state for unchanged elements.
    pub fn update(&mut self) -> Result<(), JsValue> {
        // Check should_update() lifecycle
        if let Some(ref comp) = self.component_ctrl {
            let borrowed = comp.borrow();
            if !borrowed.should_update() {
                return Ok(());
            }
        }

        let mount_point = self
            .mount_element
            .as_ref()
            .ok_or_else(|| JsValue::from_str("App not mounted"))?;

        // Render new VNode tree
        let new_vnode = (self.root_fn)();

        // If we have a previous tree, patch in-place
        if let Some(ref old_vnode) = self.old_vnode {
            let parent: web_sys::Node = mount_point.clone().into();
            patch_node(old_vnode, &new_vnode, &parent, None);
        } else {
            // No previous tree — mount fresh
            let parent: web_sys::Node = mount_point.clone().into();
            mount_to_dom(&new_vnode, &parent, None);
        }

        // Store new tree for next diff
        self.old_vnode = Some(new_vnode);

        Ok(())
    }

    /// Get a reference to the mount element, if mounted.
    pub fn mount_element(&self) -> Option<&web_sys::Element> {
        self.mount_element.as_ref()
    }
}

/// Mount an application to the DOM (convenience function).
///
/// Creates an `App` from a render closure, mounts it, and returns the `App`.
pub fn mount<F: Fn() -> VNode + 'static>(selector: &str, root_fn: F) -> App {
    let mut app = App::new(selector, root_fn);
    let _ = app.mount();
    app
}
