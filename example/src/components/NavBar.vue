<script setup>
import { ref } from 'vue'

const isOpen = ref(false)

const links = [
  { label: 'Home', href: '#home' },
  { label: 'Features', href: '#features' },
  { label: 'About', href: '#about' },
  { label: 'Contact', href: '#contact' },
]
</script>

<template>
  <nav class="fixed top-0 left-0 w-full bg-white/80 backdrop-blur-md border-b border-gray-100 z-50">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <!-- Logo -->
        <a href="#" class="flex items-center space-x-2">
          <div class="w-8 h-8 bg-indigo-600 rounded-lg flex items-center justify-center">
            <span class="text-white font-bold text-sm">L</span>
          </div>
          <span class="text-xl font-bold text-gray-900">Landing</span>
        </a>

        <!-- Desktop links -->
        <div class="hidden md:flex items-center space-x-8">
          <a
            v-for="link in links"
            :key="link.label"
            :href="link.href"
            class="text-gray-600 hover:text-indigo-600 transition-colors duration-200 font-medium"
          >
            {{ link.label }}
          </a>
          <a
            href="#"
            class="bg-indigo-600 text-white px-5 py-2.5 rounded-lg hover:bg-indigo-700 transition-colors duration-200 font-medium"
          >
            Get Started
          </a>
        </div>

        <!-- Mobile hamburger -->
        <button
          @click="isOpen = !isOpen"
          class="md:hidden p-2 rounded-lg text-gray-600 hover:bg-gray-100 transition-colors"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              v-if="!isOpen"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h16M4 18h16"
            />
            <path
              v-else
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- Mobile menu -->
    <div
      v-show="isOpen"
      class="md:hidden bg-white border-b border-gray-100"
    >
      <div class="px-4 py-3 space-y-2">
        <a
          v-for="link in links"
          :key="link.label"
          :href="link.href"
          class="block px-3 py-2 text-gray-600 hover:text-indigo-600 hover:bg-gray-50 rounded-lg transition-colors font-medium"
          @click="isOpen = false"
        >
          {{ link.label }}
        </a>
        <a
          href="#"
          class="block px-3 py-2 text-center bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium"
          @click="isOpen = false"
        >
          Get Started
        </a>
      </div>
    </div>
  </nav>
</template>
