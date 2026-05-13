<script setup>
const features = [
  {
    title: 'Lightning Fast',
    description: 'Optimized for speed with instant page loads and real-time updates out of the box.',
    icon: '⚡',
  },
  {
    title: 'Fully Responsive',
    description: 'Looks perfect on every device — from mobile phones to large desktop screens.',
    icon: '📱',
  },
  {
    title: 'Secure by Default',
    description: 'Enterprise-grade security with encryption, authentication, and access controls built in.',
    icon: '🔒',
  },
  {
    title: 'Easy Integration',
    description: 'Seamlessly connect with your favourite tools and services through our API.',
    icon: '🔗',
  },
  {
    title: 'Developer Friendly',
    description: 'Clean APIs, thorough documentation, and SDKs for all major programming languages.',
    icon: '💻',
  },
  {
    title: '24/7 Support',
    description: 'Our dedicated support team is available around the clock to help you succeed.',
    icon: '🎧',
  },
]
</script>

<template>
  <section id="features" class="py-20 sm:py-28 bg-gray-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <!-- Section header -->
      <div class="text-center max-w-2xl mx-auto mb-16">
        <h2 class="text-3xl sm:text-4xl font-bold text-gray-900">
          Everything you need
        </h2>
        <p class="mt-4 text-lg text-gray-600">
          A comprehensive set of features to help you build and scale your projects with ease.
        </p>
      </div>

      <!-- Features grid -->
      <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-8">
        <div
          v-for="feature in features"
          :key="feature.title"
          class="bg-white rounded-xl p-6 border border-gray-100 hover:shadow-lg hover:border-indigo-100 transition-all duration-200"
        >
          <div class="w-12 h-12 bg-indigo-50 rounded-lg flex items-center justify-center text-2xl mb-4">
            {{ feature.icon }}
          </div>
          <h3 class="text-lg font-semibold text-gray-900 mb-2">
            {{ feature.title }}
          </h3>
          <p class="text-gray-600 leading-relaxed">
            {{ feature.description }}
          </p>
        </div>
      </div>
    </div>
  </section>
</template>
