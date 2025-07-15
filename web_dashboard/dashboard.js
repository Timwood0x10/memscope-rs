

// Display memory hierarchy tree
function displayMemoryHierarchyTree(memoryHierarchy, container) {
    let hierarchyHTML = "<div class=\"hierarchy-root\">";
    
    for (const [categoryName, categoryData] of Object.entries(memoryHierarchy)) {
        hierarchyHTML += `
            <div class="hierarchy-category">
                <div class="category-header" onclick="toggleCategory('${categoryName}')">
                    <span class="category-icon">[+]</span>
                    <span class="category-name">${categoryName}</span>
                    <span class="category-summary">${categoryData.summary?.total_size_bytes || 0} bytes</span>
                </div>
                <div class="category-content" id="category-${categoryName}" style="display: block;">
        `;
        
        if (categoryData.subcategories) {
            for (const [subName, subData] of Object.entries(categoryData.subcategories)) {
                hierarchyHTML += `
                    <div class="subcategory">
                        <div class="subcategory-header">
                            <span class="subcategory-icon">[-]</span>
                            <span class="subcategory-name">${subName}</span>
                            <span class="subcategory-summary">${subData.summary?.total_size_bytes || 0} bytes</span>
                        </div>
                `;
                
                if (subData.types) {
                    subData.types.forEach(type => {
                        hierarchyHTML += `
                            <div class="type-item">
                                <div class="type-header">
                                    <span class="type-name">${type.type_name || "Unknown"}</span>
                                    <span class="type-size">${type.size_bytes || 0} bytes</span>
                                    <span class="type-count">${type.allocation_count || 0} allocations</span>
                                </div>
                            </div>
                        `;
                    });
                }
                
                hierarchyHTML += "</div>";
            }
        }
        
        hierarchyHTML += "</div></div>";
    }
    
    hierarchyHTML += "</div>";
    container.innerHTML = hierarchyHTML;
}

// Toggle category visibility
function toggleCategory(categoryName) {
    const content = document.getElementById(`category-${categoryName}`);
    if (content) {
        content.style.display = content.style.display === "none" ? "block" : "none";
    }
}

// Update lifecycle visualization
function updateLifecycleVisualization(data) {
    const container = document.getElementById("lifecycleVisualization");
    if (!container) return;
    
    if (!data.lifecycle_stats) {
        container.innerHTML = "<div class=\"no-data\">No lifecycle data available.</div>";
        return;
    }
    
    const lifecycle = data.lifecycle_stats;
    
    let timelineHTML = `
        <div class="lifecycle-summary">
            <div class="lifecycle-stat">
                <span class="stat-label">Average Lifetime:</span>
                <span class="stat-value">${lifecycle.average_lifetime_ms || 0}ms</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Memory Leaks:</span>
                <span class="stat-value">${lifecycle.memory_leaks_detected || 0}</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Short-lived Objects:</span>
                <span class="stat-value">${lifecycle.short_lived_objects || 0}</span>
            </div>
            <div class="lifecycle-stat">
                <span class="stat-label">Long-lived Objects:</span>
                <span class="stat-value">${lifecycle.long_lived_objects || 0}</span>
            </div>
        </div>
    `;
    
    container.innerHTML = timelineHTML;
}
