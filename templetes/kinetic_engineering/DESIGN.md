---
name: Kinetic Engineering
colors:
  surface: '#0e1511'
  surface-dim: '#0e1511'
  surface-bright: '#343b36'
  surface-container-lowest: '#09100c'
  surface-container-low: '#161d19'
  surface-container: '#1a211d'
  surface-container-high: '#242c27'
  surface-container-highest: '#2f3632'
  on-surface: '#dde4dd'
  on-surface-variant: '#bbcabf'
  inverse-surface: '#dde4dd'
  inverse-on-surface: '#2b322d'
  outline: '#86948a'
  outline-variant: '#3c4a42'
  surface-tint: '#4edea3'
  primary: '#4edea3'
  on-primary: '#003824'
  primary-container: '#10b981'
  on-primary-container: '#00422b'
  inverse-primary: '#006c49'
  secondary: '#ffb690'
  on-secondary: '#552100'
  secondary-container: '#ec6a06'
  on-secondary-container: '#4a1c00'
  tertiary: '#ffb3af'
  on-tertiary: '#650911'
  tertiary-container: '#fc7c78'
  on-tertiary-container: '#711419'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#6ffbbe'
  primary-fixed-dim: '#4edea3'
  on-primary-fixed: '#002113'
  on-primary-fixed-variant: '#005236'
  secondary-fixed: '#ffdbca'
  secondary-fixed-dim: '#ffb690'
  on-secondary-fixed: '#341100'
  on-secondary-fixed-variant: '#783200'
  tertiary-fixed: '#ffdad7'
  tertiary-fixed-dim: '#ffb3af'
  on-tertiary-fixed: '#410005'
  on-tertiary-fixed-variant: '#842225'
  background: '#0e1511'
  on-background: '#dde4dd'
  surface-variant: '#2f3632'
typography:
  display-lg:
    fontFamily: Geist
    fontSize: 32px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Geist
    fontSize: 20px
    fontWeight: '500'
    lineHeight: '1.4'
    letterSpacing: -0.01em
  body-base:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.6'
    letterSpacing: 0em
  data-mono:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: '1.5'
    letterSpacing: 0.02em
  label-caps:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '700'
    lineHeight: '1'
    letterSpacing: 0.1em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  gutter: 16px
  margin-sm: 12px
  margin-md: 24px
  container-max: 1440px
---

## Brand & Style

The design system is engineered for high-precision environments, evoking a sense of technical rigor, stability, and industrial efficiency. It targets technical operators and engineers who require a signal-to-noise ratio optimized for rapid data parsing. 

The aesthetic is a hybrid of **Minimalism** and **Modern Corporate**, stripped of decorative flourishes to focus on structural integrity. Every element is intentional, utilizing a "Machine UI" approach: high-contrast data points against a deep, void-like background. The emotional response is one of calm authority and absolute accuracy.

Visual pillars:
- **Rigid Structure:** Strict adherence to grid lines and 1px borders.
- **Luminance Hierarchy:** Use of light purely as a carrier of information.
- **Industrial Precision:** Sharp corners and tight spacing to maximize information density.

## Colors

This design system utilizes a high-contrast dark palette to reduce eye strain and highlight critical status indicators.

- **Foundational Neutrals:** The background is set to a deep Zinc 950 (#09090B). Surfaces utilize Zinc 925 (#121215) to create subtle separation.
- **Ice Emerald (#10B981):** This is the primary functional accent. It is never used for large surfaces; instead, it appears as 2px vertical "status lines" on the left edge of active components, or as high-precision text/icon accents.
- **Status Semantic Palette:**
  - **PINNED:** Deep Blue background with Light Blue text for persistent state.
  - **HOT:** Dark Orange background with #F97316 text for high-activity warnings.
  - **IDLE:** Dark Gray background with Zinc 300 text for inactive states.
- **Typography Colors:** Pure white (#FFFFFF) is reserved strictly for primary data and headings. Zinc 400 (#A1A1AA) is used for all metadata and labels to create a clear visual hierarchy.

## Typography

The typography system prioritizes legibility and technical character. 

- **Primary Typeface:** **Geist** is used for the interface and headlines. Its clean, geometric sans-serif terminals provide a modern, developer-centric feel.
- **Secondary Typeface:** **JetBrains Mono** is utilized for all data values, status tags, and labels. The monospaced nature ensures that columns of numbers align perfectly, aiding in rapid data comparison.
- **Styling Note:** Use `label-caps` (Uppercase JetBrains Mono) for all form labels and section headers to reinforce the industrial documentation aesthetic.

## Layout & Spacing

The design system employs a **Fixed Grid** philosophy based on a 4px base unit. 

- **The Grid:** A 12-column layout on desktop with a 16px gutter. Columns are used to strictly align technical readouts and modular cards.
- **Density:** High-density padding is preferred. Use 12px (3 units) for internal card padding and 16px (4 units) for component spacing.
- **Breakpoints:** 
  - **Desktop (1280px+):** Full 12-column visibility.
  - **Tablet (768px - 1279px):** 6-column layout, reducing side margins to 24px.
  - **Mobile (<768px):** 2-column layout, primary data prioritized vertically. 
- **Alignment:** All elements must snap to the 4px grid. Centering is rare; left-aligned data is the standard for scanability.

## Elevation & Depth

This system rejects shadows in favor of **Tonal Layering and Border Definition**.

- **Surfaces:** Depth is communicated by the contrast between the #09090B background and #121215 card surfaces.
- **Borders:** Every interactive or container element is defined by a 1px border using `rgba(255, 255, 255, 0.07)`. This "ghost border" provides structure without adding visual weight.
- **Focus States:** When an element is active or focused, the border color shifts to the primary Ice Emerald (#10B981) or gains a 2px left-hand accent line.
- **Absence of Shadows:** No drop shadows or ambient occlusions are permitted. The UI should appear as a single, flat, multi-layered glass instrument.

## Shapes

The shape language is sharp and disciplined. 

- **Standard Radius:** All containers, buttons, and input fields use a 4px (`0.25rem`) corner radius. 
- **Large Components:** Larger cards or modals may scale up to 6px, but never exceed this. 
- **Status Tags:** Tags are strictly rectangular or use the same 4px radius; pill-shaped (fully rounded) elements are prohibited as they conflict with the industrial aesthetic.

## Components

- **Cards:** Background #121215, 1px border `rgba(255,255,255,0.07)`. Cards containing active processes feature a 2px vertical Ice Emerald line on the far left edge.
- **Buttons:** 
  - **Primary:** Outline-only with Ice Emerald border and text. 
  - **Secondary:** Zinc 800 background with Zinc 300 text.
  - **Interaction:** On hover, buttons gain a subtle `rgba(16, 185, 129, 0.1)` background tint.
- **Status Tags:** 
  - **Text:** All-caps JetBrains Mono, 11px. 
  - **PINNED:** Background `rgba(30, 58, 138, 0.4)`, Text #60A5FA.
  - **HOT:** Background `rgba(124, 45, 18, 0.4)`, Text #F97316.
  - **IDLE:** Background `rgba(39, 39, 42, 0.5)`, Text #D4D4D8.
- **Inputs:** Background same as surface (#121215), 1px border. On focus, the border transitions to Ice Emerald.
- **Data Tables:** No vertical dividers. Horizontal dividers use the standard 1px border. Header row uses `label-caps` typography in Zinc 400.
- **Indicators:** Small 6px circles are used for real-time connection status, utilizing the primary Ice Emerald for "Online" and Zinc 600 for "Offline".