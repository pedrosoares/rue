<script setup>
import NavBar from './components/NavBar.vue'
import HeroSection from './components/HeroSection.vue'
import FeaturesSection from './components/FeaturesSection.vue'
import FooterSection from './components/FooterSection.vue'
</script>

<template>
  <div class="min-h-screen bg-white">
    <NavBar />
    <main>
      <HeroSection />
      <FeaturesSection />
    </main>
    <FooterSection />
  </div>
</template>
