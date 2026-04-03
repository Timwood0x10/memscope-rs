// Dashboard rendering plan based on existing template design

## Design Philosophy

Based on `aim/html/DASHBOARD_REFACTORING_PLAN_FINAL.md`:

1. **Template Engine**: Tera (Jinja2-like, Rust-friendly)
2. **Page Structure**: Single-page Dashboard
3. **Tech Stack**: Tera + Tailwind CSS + Chart.js + D3.js
4. **Core Features**:
   - 🔒 Unsafe/FFI Memory Passport (D3.js force graph + timeline)
   - 📋 Async Task Analysis (Chart.js + task flow)
   - 🔗 Variable Relationship Graph (D3.js force graph)

## Current Data Types

1. Memory Analysis: allocations, memory_analysis.json
2. Thread Analysis: thread_analysis.json  
3. Variable Relationships: variable_relationships.json
4. Memory Passports: memory_passports.json
5. Leak Detection: leak_detection.json
6. Unsafe/FFI: unsafe_ffi.json
7. System Resources: system_resources.json
8. Lifetime: lifetime.json

## Rendering Strategy

### Option 1: Unified Dashboard + Conditional Rendering

Auto-detect data types and render relevant components:

```rust
pub struct DashboardComponents {
    // Always show
    header: bool,           // System resources + KPI cards
    memory_analysis: bool,  // Memory distribution + allocations table
    
    // Conditional
    thread_analysis: bool,   // Thread resource comparison
    relationship_graph: bool, // D3.js force graph
    passport_timeline: bool,   // Memory passport tracking
    unsafe_ffi: bool,        // Rust/FFI swimlane
    leak_detection: bool,    // Leaked memory table
}
```

### Option 2: Mode-based Dashboard + Dynamic Components

```rust
pub enum DashboardMode {
    Memory,      // Focus on memory analysis
    Thread,       // Focus on thread analysis  
    Relationship,  // Focus on variable relationships
    Passport,     // Focus on memory passports
    UnsafeFFI,    // Focus on Unsafe/FFI
    Comprehensive // All features
}
```

### Recommended: Unified Dashboard + Smart Component Layout

**Advantages**:
1. Users don't need to switch modes
2. Simpler code - only one render function
3. Can reuse old template CSS/JS from `templates/clean_dashboard.html`
4. Progressive enhancement - show more data as available

## Implementation Plan

### Phase 1: Reuse Old Template (Recommended)

1. **Copy CSS/JS from old templates**:
   - `templates/clean_dashboard.html` → `src/render_engine/dashboard/templates/`
   - `templates/script.js` → `src/render_engine/dashboard/templates/`
   - `templates/styles.css` → `src/render_engine/dashboard/templates/`

2. **Adapt old template structure**:
   - Keep the KPI cards
   - Keep the variable relationship graph (D3.js)
   - Keep the memory passport sections
   - Keep the unsafe/FFI sections

3. **Modify data injection**:
   - Keep using Handlebars (easier than Tera for now)
   - Inject JSON data via `window.dashboardData`
   - Reuse existing Chart.js and D3.js code

### Phase 2: Component Architecture

Create modular components that can be enabled/disabled:

```rust
pub struct DashboardComponents {
    pub header: HeaderComponent,
    pub kpi_cards: KPICardsComponent,
    pub memory_section: MemorySection,
    pub thread_section: ThreadSection,
    pub relationship_section: RelationshipSection,
    pub passport_section: PassportSection,
    pub unsafe_ffi_section: UnsafeFFISection,
    pub leak_section: LeakSection,
}
```

## Rendering Flow

```
Tracker Data
    ↓
DashboardRenderer::render_from_tracker()
    ↓
Build DashboardContext (all data)
    ↓
Template Engine (Handlebars/Tera)
    ↓
HTML Output
```

## Next Steps

Should I:
1. **Reuse old template** - Copy and adapt `templates/clean_dashboard.html` (recommended)
2. **Continue with current approach** - Fix the current D3.js issues
3. **Start from scratch** - Build new template based on plan

Recommendation: **Option 1** - Reuse old template because:
- It's already tested and proven
- Has all the core features (D3.js, Chart.js, Tailwind)
- Better design and user experience
- Less risk of bugs
- Can incrementally improve

Let me know which approach you prefer!