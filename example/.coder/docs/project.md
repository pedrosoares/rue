# Vue Landing Page with Tailwind CSS

## Overview
A minimal Vue 3 landing page built with Vite and Tailwind CSS.

## Tech Stack
- **Vue 3** (Composition API with `<script setup>`)
- **Vite** (build tool)
- **Tailwind CSS v3** (utility-first CSS)
- **PostCSS + Autoprefixer**

## Project Structure
```
example/
├── index.html             # Entry HTML
├── package.json
├── vite.config.js
├── tailwind.config.js
├── postcss.config.js
├── src/
│   ├── main.js            # Vue app bootstrap
│   ├── index.css          # Tailwind directives (@tailwind base/components/utilities)
│   ├── App.vue            # Root component, composes all sections
│   └── components/
│       ├── NavBar.vue         # Fixed top nav with mobile hamburger menu
│       ├── HeroSection.vue    # Hero with headline, CTA, stats
│       ├── FeaturesSection.vue # 6-feature grid with icons
│       └── FooterSection.vue  # Dark footer with links
```

## Components

| Component | Description |
|-----------|-------------|
| `NavBar` | Fixed top navigation with responsive mobile menu (hamburger toggle) |
| `HeroSection` | Hero area with badge, headline, subtitle, dual CTA buttons, and stats row |
| `FeaturesSection` | Light gray section with 6 feature cards in a responsive grid |
| `FooterSection` | Dark footer with branding, description, and link columns |

## Commands
- `npm run dev` — start dev server
- `npm run build` — production build to `dist/`
- `npm run preview` — preview production build

## Tailwind Configuration
- Content paths: `index.html` + all files in `src/`
- Uses default Tailwind theme (can be extended in `tailwind.config.js`)
