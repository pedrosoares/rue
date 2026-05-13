use rue_core::*;
use rue_macros::html;
use crate::update_app;

pub struct NavBar {
    is_open: Signal<bool>,
}

impl NavBar {
    pub fn new() -> Self {
        NavBar {
            is_open: Signal::new(false),
        }
    }
}

impl Component for NavBar {
    fn render(&self) -> VNode {
        let is_open = self.is_open.get_clone();
        let sig = self.is_open.clone();

        let handle_toggle = move |_| {
            let current = sig.get_clone();
            sig.set(!current);
            update_app();
        };

        // Mobile menu — only when open
        let mobile_menu = if is_open {
            html! {
                <div class="md:hidden bg-white border-b border-gray-100">
                    <div class="px-4 py-3 space-y-2">
                        <a href="#home" class="block px-3 py-2 text-gray-600 hover:text-indigo-600 hover:bg-gray-50 rounded-lg transition-colors font-medium">{"Home"}</a>
                        <a href="#features" class="block px-3 py-2 text-gray-600 hover:text-indigo-600 hover:bg-gray-50 rounded-lg transition-colors font-medium">{"Features"}</a>
                        <a href="#about" class="block px-3 py-2 text-gray-600 hover:text-indigo-600 hover:bg-gray-50 rounded-lg transition-colors font-medium">{"About"}</a>
                        <a href="#contact" class="block px-3 py-2 text-gray-600 hover:text-indigo-600 hover:bg-gray-50 rounded-lg transition-colors font-medium">{"Contact"}</a>
                        <a href="#" class="block px-3 py-2 text-center bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium">{"Get Started"}</a>
                    </div>
                </div>
            }
        } else {
            VNode::empty()
        };

        // Toggle icon — X when open, hamburger when closed
        let menu_icon = if is_open {
            html! {
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
            }
        } else {
            html! {
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                </svg>
            }
        };

        html! {
            <nav class="fixed top-0 left-0 w-full bg-white/80 backdrop-blur-md border-b border-gray-100 z-50">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex items-center justify-between h-16">
                        <a href="#" class="flex items-center space-x-2">
                            <div class="w-8 h-8 bg-indigo-600 rounded-lg flex items-center justify-center">
                                <span class="text-white font-bold text-sm">{"R"}</span>
                            </div>
                            <span class="text-xl font-bold text-gray-900">{"Rue"}</span>
                        </a>
                        <div class="hidden md:flex items-center space-x-8">
                            <a href="#home" class="text-gray-600 hover:text-indigo-600 transition-colors duration-200 font-medium">{"Home"}</a>
                            <a href="#features" class="text-gray-600 hover:text-indigo-600 transition-colors duration-200 font-medium">{"Features"}</a>
                            <a href="#about" class="text-gray-600 hover:text-indigo-600 transition-colors duration-200 font-medium">{"About"}</a>
                            <a href="#contact" class="text-gray-600 hover:text-indigo-600 transition-colors duration-200 font-medium">{"Contact"}</a>
                            <a href="#" class="bg-indigo-600 text-white px-5 py-2.5 rounded-lg hover:bg-indigo-700 transition-colors duration-200 font-medium">{"Get Started"}</a>
                        </div>
                        <button on:click={handle_toggle} class="md:hidden p-2 rounded-lg text-gray-600 hover:bg-gray-100 transition-colors">
                            {vnode: menu_icon}
                        </button>
                    </div>
                </div>
                {vnode: mobile_menu}
            </nav>
        }
    }
}
