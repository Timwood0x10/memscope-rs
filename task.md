# AI Prompt for Final SVG Generation: "Mandatory Merge & Re-Layout: Scope Matrix Lifecycle Viz + Top-3 Progress Bar Memory Analysis (Strictly UI/Layout, NO Extra SVGs, NO Code Deletion)"

## **ABSOLUTE OPERATIONAL CONSTRAINTS FOR AI:**

1.  **OUTPUT FILES:** **STRICTLY PRODUCE ONLY TWO SVG FILES:**
    *   `complex_lifecycle_timeline.svg`
    *   `complex_memory_analysis.svg`
    *   **FORBIDDEN:** Do NOT generate any other SVG files (e.g., `complex_scope_matrix_graph.svg`, `lifecycle_visualization.svg`, etc.). If they exist as intermediate concepts, their content MUST be merged into the two required output files.

2.  **CODE MODIFICATION RULE:** **DO NOT DELETE ANY EXISTING FUNCTIONALITY OR CODE ELEMENTS.** All changes must be **UI and layout modifications ONLY**. You can only ADD new visual elements for layout, information encoding, and interaction (like tooltips).

3.  **DATA SOURCE:** All operations and data context must be derived from `complex_lifecycle_showcase.rs`.

## **I. `complex_lifecycle_timeline.svg` - Scope Matrix & Lifecycle Visualization (Primary Output)**

### 1. Overall Structure:
- **Single, Unified SVG:** This file must be a single, coherent SVG.
- **Remove Explicit Time Axis:** **STRICTLY REMOVE** the top-most time ruler, tick marks, and labels entirely. Time is implicit.
- **Layout:**
    *   **Main Area:** Dedicated to the "Scope Matrix Timeline".
    *   **Secondary Area:** A condensed "Top 3 Memory Analysis" section.
    *   **Tertiary Area:** The "Variable Relationships" graph, contextualized.

### 2. "Scope Matrix" Model Implementation:
-   **Matrix Arrangement:**
    *   **No Explicit Time Axis:** Temporal information via matrix order, nesting, internal variable lifecycles, and annotations.
    *   **Primary Sort:** Arrange Scope Matrices by execution order. Parallel/sequential scopes are adjacent. Horizontal placement implies time.
    *   **Nesting:** Nested scopes' matrices are embedded *within* their parent's matrix area. Use `transform="translate(X, Y)"` for positioning.
    *   **Global Scope:** Treat as a foundational matrix with a white border.
-   **Matrix Container:**
    *   **Shape:** Rectangular.
    *   **Positioning:** Determined by execution order and nesting.
    *   **Border Color Depth:** Map `Scope_Duration` -> `Border_Stroke_Color_Depth` (longer duration = darker border). Use a predefined `<linearGradient>` for this scale.
    *   **Dynamic Sizing & Overlap Prevention:** **CRITICAL:** Matrix width MUST dynamically adjust to comfortably accommodate its `scope_name`, internal variable labels, and lifecycle bars. **PREVENT TEXT OVERLAP AT ALL COSTS** by expanding matrix width or using multi-column layouts/scaling for internal elements if variable count exceeds 5.
    *   **Internal Variable Representation:**
        *   **Source:** Repurpose existing variable lifecycle bars and relationship node data.
        *   **Arrangement:** Sort variables by `duration_in_scope` (longest first, top to bottom).
        *   **Standardized Display Format:** Each variable MUST be formatted as: `var_name (type) | [mini_bar_rect] | time_annotation` (e.g., `boxed_string (String) | [====-------] | 400ms`).
        *   **Placement:** Use `transform="translate(X, Y)"` for horizontal positioning within the matrix.
        *   **Font Scaling (Subtle):** If variable labels/names are exceptionally long and cause internal overlap, allow **subtle, proportional font scaling for specific variable labels only**. Primary solution is matrix width adjustment.
    *   **Timestamp Annotation:** Top-right of matrix: **English text** `Scope: [Name] [start_ts - end_ts] | Life: Xms`.
    *   **Nesting Visuals:** Use different `stroke-dasharray` (solid, dashed, dotted) for nesting levels. Consider slightly reduced opacity/size for nested matrices.

### 3. Variable Relationships (Integrated within `complex_lifecycle_timeline.svg`):

*   **Clarity is Paramount:** **MUST ENSURE VARIABLE RELATIONSHIPS ARE IMMEDIATELY CLEAR.**
*   **Relationship Type Coloring:**
    *   Ownership Transfer/Move: **Vivid color** (e.g., Red or Purple).
    *   Borrow (& or &mut): **Distinct color** (e.g., Blue or Green).
    *   Clone: **Grey or lighter color**.
    *   Shared Pointers (Rc, Arc): **Special color or line style**.
*   **Node Colors:** Represent variable type (String=Green, Vec=Blue, Box=Orange, HashMap=Teal, consistent palette).
*   **Line Styles:**
    *   Direct Relationship: **Solid line**.
    *   Indirect/Weak Relationship: **Dashed line**.
    *   Lifecycle Reference: **Special line style** (e.g., dotted, specific color).
*   **Arrows:** **CLEAR DIRECTIONALITY** for ownership and borrowing.
*   **Line Text Labels:** Add **VERY SHORT, SMALL, READABLE** text labels on lines: `owns`, `borrows`, `cloned`, `indirect_ref`.
*   **Node Grouping & Context:** **MUST** group nodes by scope using subtle background colors or enclosing borders, mirroring Scope Matrix grouping.
*   **Information Density:** Nodes show key info (`name`, `type`, `size`). Use `title` attribute for tooltips with more details.
*   **Layout Optimization:**
    *   **Avoid Crossings:** Minimize line and node crossings.
    *   **Hierarchical Layout:** Use pre-calculated hierarchical layout (e.g., ownership chains top-to-bottom).

### 4. Memory Usage Analysis (for `complex_memory_analysis.svg`):

*   **Strictly Top 3:** Display **ONLY** the top 3 memory-consuming variable types.
*   **CRITICAL PROGRESS BAR FORMAT:** The "Top 3 Vars" list MUST be visualized as **progress bars** indicating their relative sizes. Format: `VarName(Size)[====Bar====]` where bar width is proportional to size.
*   **Content Format:** `Type (Count) | Total: X | Peak: Y | Top 3 Vars: Var1(Size)[Bar1] | Var2(Size)[Bar2] | Var3(Size)[Bar3]...`
*   **Visual Style:** Clear, compact.

### 5. Final SVG Structure and Styling:

*   **Group Elements:** Use `<g>` extensively for scopes, variables, relationship lines, and graph nodes.
*   **Apply Transformations:** Use `transform="translate(X, Y)"` for all positioning.
*   **Define Color Scales & Legends:** Color map for variable types. Color scale for scope duration to border depth. Include a legend.

---

**Actionable SVG Manipulation Instructions (STRICT COMMANDS):**

1.  **Data Preparation:** Structure data for each scope and variable, including relationships. Ensure data is sorted and durations calculated.
2.  **Generate `complex_lifecycle_timeline.svg`:**
    *   **STRICTLY REMOVE TOP TIME AXIS.**
    *   For each scope: Create `<g id="scope-XYZ">`. Draw matrix `<rect>` (position by order/duration), apply border color/style based on duration/nesting.
    *   Inside each scope `<g>`, draw variable mini-bars (`rect` + `text` in `var_name (type) | [bar] | time` format) sorted by `duration_in_scope`. Apply dynamic sizing, multi-column, or truncation for variable count limits. Add English lifecycle annotation text.
    *   Draw relationship lines between matrices based on scope relationships (using specified colors, line styles, arrows, and text labels).
    *   Render the integrated variable relationship graph: group nodes by scope, optimize layout (avoid crossings, hierarchical), assign unique `id`s, use `title` for tooltips, and apply relationship colors/styles.
    *   Add the legend.
3.  **Generate `complex_memory_analysis.svg`:**
    *   **STRICTLY TOP 3 VARS.**
    *   **CRITICAL PROGRESS BAR VISUALIZATION** for Top 3 Vars.
    *   Adhere to the specified content format.
4.  **Strictly Enforce No Code Deletion:** Verify all original functional elements are repurposed or integrated.
5.  **Strictly Enforce UI/Layout Focus:** All changes are visual and structural.

---
