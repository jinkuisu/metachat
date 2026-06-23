# Design System: Rococó Delicado

## 1. Definição do Estilo

- **Nome:** Rococó Delicado
- **Tipo:** Ornate, Playful, Elegant
- **Keywords:** rococo, delicate, ornate, playful, elegant, pastel, gilded, asymmetrical, floral, charming
- **Era:** 18th Century, French Rococo
- **Light/Dark:** ✓ Full / ✗ No

## 2. Paleta de Cores

- **Primárias:** Pastel Pink #FADADD, Powder Blue #B0E0E6, Mint Green #98FF98, Gilded Gold #FFD700
- **Secundárias:** Cream #FFFDD0, Lavender #E6E6FA, White #FFFFFF, Light Grey #D3D3D3

## 3. Efeitos Visuais

Gilded scrollwork, delicate floral patterns, asymmetrical layouts, soft pastel colors, ornate mirrors, shell motifs, playful cherubs, elegant typography, subtle animations

## 4. AI Prompt Keywords

Design a delicate Rococo landing page. Use: pastel pink and powder blue, gilded scrollwork, delicate floral patterns, asymmetrical layouts, soft pastel colors, ornate mirrors, shell motifs, playful cherubs, elegant typography.

## 5. CSS Technical

```css
background: #FFFDD0, color: #5C4033, font-family: 'Great Vibes', cursive, border: 1px solid #FFD700, box-shadow: 0 4px 8px rgba(0,0,0,0.1), text-shadow: 1px 1px 0 #FFFFFF, animation: shimmer-effect 3s infinite, background-image: url('floral-rococo.png'), background-blend-mode: soft-light
```

## 6. Design System Variables

```css
--pastel-pink-rococo: #FADADD, --powder-blue-rococo: #B0E0E6, --mint-green-rococo: #98FF98, --gilded-gold-rococo: #FFD700, --scrollwork-thickness: 1px, --font-rococo: 'Great Vibes', cursive
```

## 7. Checklist de Implementação

- ☐ Gilded scrollwork
- ☐ Delicate floral patterns
- ☐ Asymmetrical layouts
- ☐ Soft pastel colors
- ☐ Shell motifs
- ☐ Elegant typography

## 8. Visual Theme & Atmosphere

Rococó Delicado — Design general com rococo, delicate, ornate. Template e prompt pronto para IA. Estilo Rococó Delicado representa uma tendência moderna em design UI/UX web com foco em general.

- Density: 5/10 — Balanced
- Variance: 7/10 — Dynamic
- Motion: 6/10 — Expressive

## 9. Color Palette & Roles

- **Pastel Pink** (#FADADD) — Primary text color
- **Powder Blue** (#B0E0E6) — Accent highlight, links and focus states
- **Mint Green** (#98FF98) — Supporting palette color
- **Gilded Gold** (#FFD700) — Premium accent, decorative highlights
- **Cream** (#FFFDD0) — Secondary surface
- **Lavender** (#E6E6FA) — Extended palette, decorative use
- **White** (#FFFFFF) — Secondary surface
- **Light Grey** (#D3D3D3) — Secondary text, borders, muted elements

## 10. Typography Rules

- **Display / Hero:** Great Vibes — Weight 700, tight tracking, used for headline impact
- **Body:** Great Vibes — Weight 400, 16px/1.6 line-height, max 72ch per line
- **UI Labels / Captions:** Great Vibes — 0.875rem, weight 500, slight letter-spacing
- **Monospace:** JetBrains Mono — Used for code, metadata, and technical values

Scale:
- Hero: clamp(2.5rem, 5vw, 4rem)
- H1: 2.25rem
- H2: 1.5rem
- Body: 1rem / 1.6
- Small: 0.875rem

## 11. Component Stylings

- **Primary Button:** Moderately rounded (0.75rem) shape. Accent color fill. Hover: 8% darken + subtle lift shadow. Active: -1px translate tactile press. Font weight 600. No outer glows.
- **Secondary / Ghost Button:** Outline variant. 1.5px border in muted color. Text in primary color. Hover: subtle background fill.
- **Cards:** Moderately rounded (0.75rem) corners. Surface background. Subtle shadow (0 2px 12px rgba(0,0,0,0.06)). 1px border stroke.
- **Inputs:** Label above input. 1px border stroke. Focus ring: 2px accent color offset 2px. Error text below in semantic red. No floating labels.
- **Navigation:** Primary surface background. Active item: accent color indicator. Font weight 500 when active.
- **Skeletons:** Shimmer animation matching component dimensions. No circular spinners.
- **Empty States:** Icon-based composition with descriptive text and action button.

## 12. Layout Principles

- **Grid:** CSS Grid primary. Max-width containment: 1280px centered with 1.5rem side padding.
- **Spacing rhythm:** Balanced. Base unit: 0.5rem (8px).
- **Section vertical gaps:** clamp(4rem, 8vw, 8rem).
- **Hero layout:** Asymmetric composition.
- **Feature sections:** Asymmetric grid with varied card sizes. No 3-equal-columns.
- **Mobile collapse:** All multi-column layouts collapse below 768px. No horizontal overflow.
- **z-index contract:** base (0) / sticky-nav (100) / overlay (200) / modal (300) / toast (500).

## 13. Motion & Interaction

- **Physics:** Spring — stiffness 120, damping 20. Confident, weighted transitions.
- **Entry animations:** Fade + translate-Y (16px → 0) over 480ms ease-out. Staggered cascades for lists: 100ms between items.
- **Hover states:** Scale(1.03) + shadow lift over 200ms.
- **Page transitions:** Fade + slide (300ms).
- **Performance:** Only transform and opacity animated. No layout-triggering properties.

## 14. Anti-Patterns (Banned)

- No emojis in UI — use icon system only (Lucide, Heroicons)
- No pure black (#000000) — use off-black or charcoal variants
- No oversaturated accent colors (saturation cap: 80%)
- No 3-column equal-width feature layouts — use zig-zag or asymmetric grid
- No `h-screen` — use `min-h-[100dvh]`
- No AI copywriting clichés: "Elevate", "Seamless", "Unleash", "Next-Gen"
- No broken external image links — use picsum.photos or inline SVG
- No generic lorem ipsum in demos

## Contexto Histórico

Estilo Rococó Delicado representa uma tendência moderna em design UI/UX web com foco em general.

## Caso de Uso

Landing pages, SaaS
