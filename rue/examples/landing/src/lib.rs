use wasm_bindgen::prelude::*;
use rue_core::*;
use std::cell::RefCell;

mod components;
use components::*;

// ── Global app reference for component-triggered updates ──────────────

thread_local! {
    static APP: RefCell<Option<rue_core::App>> = const { RefCell::new(None) };
}

/// Called by any component after mutating its own state.
/// Triggers a virtual-DOM patch update.
pub fn update_app() {
    APP.with(|a| {
        if let Some(ref mut app) = *a.borrow_mut() {
            let _ = app.update();
        }
    });
}

// ── Root component ────────────────────────────────────────────────────

struct Root {
    navbar: NavBar,
    hero: HeroSection,
    features: FeaturesSection,
    footer: FooterSection,
}

impl Component for Root {
    fn render(&self) -> VNode {
        VNode::element("div")
            .class("min-h-screen bg-white")
            .child(self.navbar.render())
            .child(
                VNode::element("main")
                    .child(self.hero.render())
                    .child(self.features.render())
                    .build(),
            )
            .child(self.footer.render())
            .build()
    }
}

// ── Entry point ───────────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    rue_core::init();

    let mut app = App::from_component(
        "#app",
        Root {
            navbar: NavBar::new(),
            hero: HeroSection,
            features: FeaturesSection,
            footer: FooterSection,
        },
    );
    app.mount()?;
    APP.with(|a| *a.borrow_mut() = Some(app));

    Ok(())
}
