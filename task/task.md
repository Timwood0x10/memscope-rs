
## **ENHANCED REQUIREMENTS - LARGE PROJECT SCALABILITY & UI IMPROVEMENTS (NEGOTIATED):**

### 7. **LANGUAGE CONSTRAINTS:**
*   **NO CHINESE TEXT:** All SVG content MUST use English only. Replace any Chinese text with English equivalents:
    *   Chinese -> "Memory"
    *   Chinese -> "Variables" 
    *   Chinese -> "Lifetime"
    *   Chinese -> "Progress Bar"

### 8. **INTELLIGENT MATRIX LAYOUT FOR LARGE PROJECTS:**

#### **8.1 Scope Display Limitation:**
*   **Maximum 15 Scope Matrices:** SVG MUST display at most 15 scope matrices simultaneously to maintain readability
*   **Prioritization Algorithm:** Implement intelligent scope selection based on:
    *   **Critical Scopes:** main(), error handlers, core business logic (Priority: MUST display)
    *   **High Priority:** Primary algorithms, data processing functions (Priority: HIGH)
    *   **Medium Priority:** Utility functions, helpers (Priority: MEDIUM)
    *   **Low Priority:** Generated code, macro expansions (Priority: LOW)

#### **8.2 Dynamic Layout Grid System:**
```
struct LayoutGrid {
    max_scope_matrices: usize = 15,    // Maximum matrices in SVG
    max_columns: usize = 3,            // 3 matrices per row
    max_rows: usize = 5,               // 5 rows maximum
    matrix_min_width: i32 = 280,       // Minimum matrix width
    matrix_max_width: i32 = 400,       // Maximum matrix width
    vertical_spacing: i32 = 60,        // Space between rows
    horizontal_spacing: i32 = 40,      // Space between columns
    max_svg_dimensions: (i32, i32) = (1600, 3000), // SVG size limits
}
```

### 9. **ENHANCED VARIABLE DISPLAY:**

#### **9.1 Per-Scope Variable Limitation:**
*   **Maximum 5 Variables per Matrix:** Each scope matrix displays at most 5 variables
*   **Selection Criteria:** Show variables with largest memory footprint first
*   **Overflow Indicator:** Display "+ N more variables" for remaining variables

#### **9.2 Modern Card-Based Variable Design:**
*   **Progress Bar Enhancement:**
    *   **Format:** "CurrentSize / MaxSizeInScope" (e.g., "2.4KB / 5.6KB")
    *   **Color Coding:** Type-specific gradient colors:
        *   String: Teal gradient #00BCD4 -> #00ACC1
        *   Vec: Blue gradient #2196F3 -> #1976D2
        *   Box: Red gradient #F44336 -> #D32F2F
        *   HashMap: Green gradient #4CAF50 -> #388E3C
        *   Custom: Blue-gray gradient #607D8B -> #455A64

#### **9.3 Matrix Header Enhancement:**
*   **Comprehensive Info:** "Scope: [name] | Memory: [total] | Variables: [count] | Lifetime: [duration]ms"

### 10. **JSON EXPORT FOR COMPLETE DATA:**
*   **File:** scope_analysis.json (generated alongside SVG files)
*   **Always Export:** Complete data regardless of SVG display limitations
*   **Overflow Handling:** When >15 scopes or >5 variables per scope

### 11. **LAYOUT COORDINATION & ANTI-OVERLAP:**
*   Calculate total required height for all selected scope matrices
*   Add appropriate spacing between sections (minimum 50px)
*   Ensure no visual overlap between any components

### 12. **ENHANCED VARIABLE RELATIONSHIPS:**
*   **Relationship Detection:** Ownership, borrowing, cloning patterns
*   **Visual Encoding:** Color-coded lines with labels
*   **Legend:** Comprehensive relationship type legend

---

## **FINAL VALIDATION CHECKLIST:**

- [ ] **EXACTLY 2 SVG files generated:** complex_lifecycle_timeline.svg + complex_memory_analysis.svg
- [ ] **NO Chinese text** in any SVG content
- [ ] **Maximum 15 scope matrices** displayed in SVG
- [ ] **Maximum 5 variables per scope** with overflow indicators
- [ ] **JSON export** scope_analysis.json with complete data
- [ ] **No module overlap** or layout conflicts
- [ ] **Enhanced variable cards** with size ratios and gradients
- [ ] **Comprehensive relationship visualization** with proper legends
- [ ] **Dynamic sizing** that adapts to content without breaking layout

---
