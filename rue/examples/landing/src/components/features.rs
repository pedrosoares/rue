use rue_core::*;
use rue_macros::html;

pub struct FeaturesSection;

impl Component for FeaturesSection {
    fn render(&self) -> VNode {
        html! {
            <section id="features" class="py-20 sm:py-28 bg-gray-50">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="text-center max-w-2xl mx-auto mb-16">
                        <h2 class="text-3xl sm:text-4xl font-bold text-gray-900">
                            {"Everything you need"}
                        </h2>
                        <p class="mt-4 text-lg text-gray-600">
                            {"A comprehensive set of features to help you build and scale your projects with ease."}
                        </p>
                    </div>
                    <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-8">
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"⚡"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"Lightning Fast"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Optimized for speed with instant page loads and real-time updates out of the box."}</p>
                        </div>
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"📱"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"Fully Responsive"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Looks perfect on every device — from mobile phones to large desktop screens."}</p>
                        </div>
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"🔒"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"Secure by Default"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Enterprise-grade security with encryption, authentication, and access controls built in."}</p>
                        </div>
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"🔗"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"Easy Integration"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Seamlessly connect with your favourite tools and services through our API."}</p>
                        </div>
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"💻"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"Developer Friendly"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Clean APIs, thorough documentation, and SDKs for all major programming languages."}</p>
                        </div>
                        <div class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200">
                            <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">{"🎧"}</div>
                            <h3 class="text-lg font-semibold text-gray-900 mb-2">{"24/7 Support"}</h3>
                            <p class="text-gray-600 leading-relaxed">{"Our dedicated support team is available around the clock to help you succeed."}</p>
                        </div>
                    </div>
                </div>
            </section>
        }
    }
}
