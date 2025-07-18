# MemScope-RS HTML Templates

This directory contains the HTML, CSS, and JavaScript templates used for generating interactive memory analysis reports.

## Files

### `styles.css`
Contains all the CSS styling for the interactive HTML reports, including:
- Responsive design for different screen sizes
- Modern gradient backgrounds and animations
- Tab navigation styling
- Interactive element hover effects
- Memory visualization card layouts

### `script.js`
Contains the JavaScript logic for interactive features:
- Tab navigation system
- Data filtering and sorting
- Memory statistics calculations
- Interactive allocation explorer
- Performance insights generation

## Usage

These templates are automatically embedded into the generated HTML reports by the `html_export.rs` module. The templates use:

- **CSS**: Modern flexbox and grid layouts with smooth animations
- **JavaScript**: Vanilla JS (no external dependencies) for maximum compatibility
- **Data**: JSON data is embedded directly into the HTML for offline viewing

## Features

### Overview Tab
- Real-time memory statistics
- Type distribution analysis
- Recent allocations list
- Automated performance insights

### Memory Analysis Tab
- Embedded SVG visualization (12-section comprehensive analysis)
- All original functionality preserved

### Lifecycle Timeline Tab
- Embedded SVG timeline visualization
- Scope matrix and progress bars

### Unsafe/FFI Tab
- Embedded unsafe/FFI dashboard SVG when data is available
- Friendly empty state when no unsafe operations detected

### Interactive Explorer Tab
- Filterable allocation list
- Sort by size, timestamp, or type
- Detailed allocation information cards
- Real-time search and filtering

## Design Principles

1. **Offline First**: No external dependencies, works without internet
2. **Responsive**: Adapts to different screen sizes and devices
3. **Accessible**: Proper contrast ratios and keyboard navigation
4. **Performance**: Efficient rendering of large datasets
5. **Modern**: Clean, professional design with smooth animations

## Browser Compatibility

The templates are designed to work in all modern browsers:
- Chrome 60+
- Firefox 55+
- Safari 12+
- Edge 79+

## Customization

To customize the appearance:
1. Modify `styles.css` for visual changes
2. Update `script.js` for behavioral changes
3. Rebuild your application to embed the new templates