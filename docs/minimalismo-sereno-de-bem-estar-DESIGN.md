# Design System: Minimalismo Sereno de Bem-Estar

## 1. Definição do Estilo

- **Nome:** Minimalismo Sereno de Bem-Estar
- **Tipo:** Calm, Minimalist, Serene
- **Keywords:** meditation, wellness, calm, minimalist, serene, peaceful, intuitive, mindful, health, balanced
- **Era:** 2026+ Bem-Estar Digital
- **Light/Dark:** ✓ Full / ✗ No

## 2. Paleta de Cores

- **Primárias:** Branco #FFFFFF, Bege Claro #FDF6E3, Verde Água #A8DADC, Cinza Suave #E0E0E0
- **Secundárias:** Azul Céu #87CEEB, Lavanda #E6E6FA, Verde Menta #98FB98, Marrom Claro #D2B48C

## 3. Efeitos Visuais

Layouts arejados com muito espaço em branco, tipografia suave e arredondada, ilustrações minimalistas de natureza, animações de fundo sutis (ondas, nuvens), micro-interações de feedback tátil, transições fluidas e relaxantes.

## 4. AI Prompt Keywords

Design a calm and minimalist landing page for a meditation and wellness app. Use: airy layouts, abundant white space, soft rounded typography, minimalist nature illustrations, subtle background animations (waves, clouds), tactile feedback micro-interactions, fluid and relaxing transitions, serene and balanced feel.

## 5. CSS Technical

```css
background: #FFFFFF, color: #333333, box-shadow: 0 1px 3px rgba(0,0,0,0.03), border-radius: 8px, font-family: "Nunito, sans-serif", transition: all 0.4s ease-in-out, .hero-animation-subtle, .illustration-minimal, .testimonial-card-soft.
```

## 6. Design System Variables

```css
--white-bg: #FFFFFF, --light-beige-bg: #FDF6E3, --aqua-green: #A8DADC, --soft-grey: #E0E0E0, --border-radius-md: 8px, --font-main: "Nunito, sans-serif", --shadow-calm: 0 1px 3px rgba(0,0,0,0.03).
```

## 7. Checklist de Implementação

- ☐ Layouts arejados
- ☐ Tipografia suave
- ☐ Ilustrações minimalistas
- ☐ Animações de fundo sutis
- ☐ Micro-interações táteis
- ☐ Transições relaxantes.

## 8. Visual Theme & Atmosphere

Minimalismo Sereno de Bem-Estar — Design minimalism com meditation, wellness, calm. Template e prompt pronto para IA. Estilo Minimalismo Sereno de Bem-Estar representa uma tendência moderna em design UI/UX web com foco em minimalism.

- Density: 3/10 — Airy
- Variance: 2/10 — Structured
- Motion: 4/10 — Subtle

## 9. Color Palette & Roles

- **Branco** (#FFFFFF) — Light surface, card backgrounds
- **Bege Claro** (#FDF6E3) — Secondary surface or text color
- **Verde Água** (#A8DADC) — Supporting palette color
- **Cinza Suave** (#E0E0E0) — Secondary text, borders, muted elements
- **Azul Céu** (#87CEEB) — Secondary accent
- **Lavanda** (#E6E6FA) — Extended palette, decorative use
- **Verde Menta** (#98FB98) — Success states, positive indicators
- **Marrom Claro** (#D2B48C) — Extended palette, decorative use

## 10. Typography Rules

- **Display / Hero:** Nunito — Weight 700, tight tracking, used for headline impact
- **Body:** Nunito — Weight 400, 16px/1.6 line-height, max 72ch per line
- **UI Labels / Captions:** Nunito — 0.875rem, weight 500, slight letter-spacing
- **Monospace:** JetBrains Mono — Used for code, metadata, and technical values

Scale:
- Hero: clamp(2.5rem, 5vw, 4rem)
- H1: 2.25rem
- H2: 1.5rem
- Body: 1rem / 1.6
- Small: 0.875rem

## 11. Component Stylings

- **Primary Button:** Rounded (8px) shape. Accent color fill. Hover: 8% darken + subtle lift shadow. Active: -1px translate tactile press. Font weight 600. No outer glows.
- **Secondary / Ghost Button:** Outline variant. 1.5px border in muted color. Text in primary color. Hover: subtle background fill.
- **Cards:** Rounded (8px) corners. Surface background. Subtle shadow (0 2px 12px rgba(0,0,0,0.06)). 1px border stroke.
- **Inputs:** Label above input. 1px border stroke. Focus ring: 2px accent color offset 2px. Error text below in semantic red. No floating labels.
- **Navigation:** Primary surface background. Active item: accent color indicator. Font weight 500 when active.
- **Skeletons:** Shimmer animation matching component dimensions. No circular spinners.
- **Empty States:** Icon-based composition with descriptive text and action button.

## 12. Layout Principles

- **Grid:** CSS Grid primary. Max-width containment: 1280px centered with 1.5rem side padding.
- **Spacing rhythm:** Balanced. Base unit: 0.5rem (8px).
- **Section vertical gaps:** clamp(4rem, 8vw, 8rem).
- **Hero layout:** Split-screen (text left, visual right).
- **Feature sections:** Zig-zag alternating text+image rows. No 3-equal-columns.
- **Mobile collapse:** All multi-column layouts collapse below 768px. No horizontal overflow.
- **z-index contract:** base (0) / sticky-nav (100) / overlay (200) / modal (300) / toast (500).

## 13. Motion & Interaction

- **Physics:** Ease-out curves, 200-300ms duration. Smooth and predictable.
- **Entry animations:** Fade + translate-Y (16px → 0) over 420ms ease-out. Staggered cascades for lists: 80ms between items.
- **Hover states:** Subtle color shift + shadow adjustment over 200ms.
- **Page transitions:** Fade only (200ms).
- **Performance:** Only transform and opacity animated. No layout-triggering properties.

## 14. Anti-Patterns (Banned)

- No emojis in UI — use icon system only (Lucide, Heroicons)
- No decorative gradients — flat color only
- No shadows heavier than 0 2px 8px rgba(0,0,0,0.08)
- No pure black (#000000) — use off-black or charcoal variants
- No oversaturated accent colors (saturation cap: 80%)
- No 3-column equal-width feature layouts — use zig-zag or asymmetric grid
- No `h-screen` — use `min-h-[100dvh]`
- No AI copywriting clichés: "Elevate", "Seamless", "Unleash", "Next-Gen"
- No broken external image links — use picsum.photos or inline SVG
- No generic lorem ipsum in demos

## Contexto Histórico

Estilo Minimalismo Sereno de Bem-Estar representa uma tendência moderna em design UI/UX web com foco em minimalism.

## Caso de Uso

Landing pages, Websites modernas
