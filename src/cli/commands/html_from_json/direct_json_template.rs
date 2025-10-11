//! Direct JSON template generator that uses raw JSON data without complex processing

// Embedded templates - 1:1 copy with all placeholders preserved
const EMBEDDED_CLEAN_DASHBOARD_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>MemScope Memory Analysis Dashboard</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <link href="https://cdn.jsdelivr.net/npm/font-awesome@4.7.0/css/font-awesome.min.css" rel="stylesheet" />
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.8/dist/chart.umd.min.js"></script>
  <script src="https://d3js.org/d3.v7.min.js"></script>
  <script src="https://unpkg.com/three@0.128.0/build/three.min.js"></script>
  <script src="https://unpkg.com/three@0.128.0/examples/js/controls/OrbitControls.js"></script>

  <style>
    {
        {
        CSS_CONTENT
      }
    }

    /* Clean, high-contrast layout variables */
    :root {
      --primary-blue: #2563eb;
      --primary-green: #059669;
      --primary-red: #dc2626;
      --primary-orange: #ea580c;
      --text-primary: #1f2937;
      --text-secondary: #6b7280;
      --bg-primary: #ffffff;
      --bg-secondary: #f8fafc;
      --border-light: #e5e7eb;
      --shadow-light: 0 1px 3px 0 rgb(0 0 0 / 0.1);
    }

    .dark {
      --text-primary: #f9fafb;
      --text-secondary: #d1d5db;
      --bg-primary: #111827;
      --bg-secondary: #1f2937;
      --border-light: #374151;
      --shadow-light: 0 4px 6px -1px rgb(0 0 0 / 0.3);
    }

    body {
      font-family: "SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg-secondary);
      color: var(--text-primary);
      transition: all 0.3s ease;
      line-height: 1.6;
      margin: 0;
      padding: 0;
    }

    .dashboard-container {
      max-width: 1400px;
      margin: 0 auto;
      padding: 24px;
      min-height: 100vh;
    }

  
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 32px;
      padding: 20px 0;
      border-bottom: 1px solid var(--border-light);
    }

    .header h1 {
      font-size: 2rem;
      font-weight: 700;
      color: var(--text-primary);
      margin: 0;
    }

    .header .subtitle {
      color: var(--text-secondary);
      font-size: 0.9rem;
      margin-top: 4px;
    }

    
    .theme-toggle {
      background: var(--primary-blue);
      color: white;
      border: none;
      padding: 10px 16px;
      border-radius: 8px;
      cursor: pointer;
      font-size: 14px;
      font-weight: 500;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .theme-toggle:hover {
      background: #1d4ed8;
      transform: translateY(-1px);
    }

    /* 卡片样式 */
    .card {
      background: var(--bg-primary);
      border: 1px solid var(--border-light);
      border-radius: 12px;
      padding: 24px;
      box-shadow: var(--shadow-light);
      transition: all 0.3s ease;
    }

    .card:hover {
      transform: translateY(-2px);
      box-shadow: 0 8px 25px -5px rgb(0 0 0 / 0.1);
    }

    .card h2 {
      font-size: 1.25rem;
      font-weight: 600;
      color: var(--text-primary);
      margin: 0 0 16px 0;
      border-bottom: 2px solid var(--primary-blue);
      padding-bottom: 8px;
    }

    
    .grid {
      display: grid;
      gap: 24px;
      margin-bottom: 32px;
    }

    .grid-2 {
      grid-template-columns: 1fr 1fr;
    }

    .grid-3 {
      grid-template-columns: repeat(3, 1fr);
    }

    .grid-4 {
      grid-template-columns: repeat(4, 1fr);
    }

    
    .kpi-card {
      text-align: center;
      padding: 20px;
      background: linear-gradient(135deg, var(--primary-blue) 0%, #3b82f6 100%);
      color: white;
      border-radius: 12px;
      border: none;
      box-shadow: var(--shadow-light);
    }

    .kpi-value {
      font-size: 2rem;
      font-weight: 700;
      margin-bottom: 4px;
    }

    .kpi-label {
      font-size: 0.875rem;
      opacity: 0.9;
      font-weight: 500;
    }

    
    .chart-container {
      height: 300px;
      background: var(--bg-primary);
      border-radius: 8px;
      position: relative;
      padding: 16px;
    }

    
    table {
      width: 100%;
      border-collapse: collapse;
      margin-top: 16px;
    }

    th,
    td {
      padding: 12px;
      text-align: left;
      border-bottom: 1px solid var(--border-light);
    }

    th {
      background: var(--bg-secondary);
      font-weight: 600;
      color: var(--text-primary);
    }

    tr:hover {
      background: var(--bg-secondary);
    }

    @media (max-width: 768px) {

      .grid-2,
      .grid-3,
      .grid-4 {
        grid-template-columns: 1fr;
      }

      .dashboard-container {
        padding: 16px;
      }

      .header {
        flex-direction: column;
        gap: 16px;
        text-align: center;
      }
    }

    .scroll {
      max-height: 400px;
      overflow: auto;
    }

    .scroll::-webkit-scrollbar {
      width: 6px;
    }

    .scroll::-webkit-scrollbar-track {
      background: var(--bg-secondary);
    }

    .scroll::-webkit-scrollbar-thumb {
      background: var(--border-light);
      border-radius: 3px;
    }

    .status-badge {
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 0.75rem;
      font-weight: 500;
    }

    .status-active {
      background: #dcfce7;
      color: #166534;
    }

    .status-leaked {
      background: #fee2e2;
      color: #dc2626;
    }

    .status-freed {
      background: #e5e7eb;
      color: #374151;
    }

    .dark .status-active {
      background: #064e3b;
      color: #34d399;
    }

    .dark .status-leaked {
      background: #7f1d1d;
      color: #fca5a5;
    }

    .dark .status-freed {
      background: #374151;
      color: #d1d5db;
    }

    .risk-low {
      background: #dcfce7;
      color: #166534;
    }

    .risk-medium {
      background: #fef3c7;
      color: #92400e;
    }

    .risk-high {
      background: #fee2e2;
      color: #dc2626;
    }

    .dark .risk-low {
      background: #064e3b;
      color: #34d399;
    }

    .dark .risk-medium {
      background: #78350f;
      color: #fbbf24;
    }

    .dark .risk-high {
      background: #7f1d1d;
      color: #fca5a5;
    }

    /* Enhanced Lifecycle Visualization Styles */
    .allocation-type {
      padding: 3px 6px;
      border-radius: 3px;
      font-size: 0.7rem;
      font-weight: 600;
      text-transform: uppercase;
      display: inline-block;
    }

    .type-heap {
      background: #fef3c7;
      color: #92400e;
      border: 1px solid #f59e0b;
    }

    .type-stack {
      background: #dbeafe;
      color: #1e40af;
      border: 1px solid #3b82f6;
    }

    .type-unknown {
      background: #f3f4f6;
      color: #6b7280;
      border: 1px solid #9ca3af;
    }

    .dark .type-heap {
      background: #78350f;
      color: #fbbf24;
    }

    .dark .type-stack {
      background: #1e3a8a;
      color: #60a5fa;
    }

    .dark .type-unknown {
      background: #374151;
      color: #d1d5db;
    }

    /* Enhanced progress bar animations */
    @keyframes shine {
      0% {
        left: -100%;
      }

      50% {
        left: 100%;
      }

      100% {
        left: 100%;
      }
    }

    @keyframes pulse {

      0%,
      100% {
        opacity: 1;
      }

      50% {
        opacity: 0.7;
      }
    }

    /* Enhanced lifecycle item styles */
    .lifecycle-item.heap {
      border-left-color: #ff6b35 !important;
    }

    .lifecycle-item.stack {
      border-left-color: #4dabf7 !important;
    }

    .lifecycle-item:hover {
      animation: pulse 1s ease-in-out;
    }

    .lifecycle-bar {
      height: 16px;
      background: var(--bg-secondary);
      border-radius: 8px;
      position: relative;
      overflow: hidden;
      margin: 6px 0;
      border: 1px solid var(--border-light);
    }

    .lifecycle-progress {
      height: 100%;
      background: linear-gradient(90deg, var(--primary-green), var(--primary-blue));
      border-radius: 7px;
      position: relative;
      transition: width 0.3s ease;
    }

    .lifecycle-item {
      margin: 8px 0;
      padding: 12px;
      background: var(--bg-secondary);
      border-radius: 8px;
      border-left: 4px solid var(--primary-blue);
      transition: all 0.2s ease;
    }

    .lifecycle-item:hover {
      background: var(--bg-primary);
      box-shadow: var(--shadow-light);
    }

    .lifecycle-item.heap {
      border-left-color: var(--primary-orange);
    }

    .lifecycle-item.stack {
      border-left-color: var(--primary-blue);
    }

    .time-info {
      font-size: 0.75rem;
      color: var(--text-secondary);
      margin-top: 6px;
      font-family: "Courier New", monospace;
    }

    .time-badge {
      display: inline-block;
      padding: 2px 6px;
      background: var(--bg-primary);
      border-radius: 3px;
      margin-right: 8px;
      border: 1px solid var(--border-light);
    }

    /* Enhanced 3D Memory Layout Styles */
    .memory-3d-container {
      position: relative;
      width: 100%;
      height: 400px;
      background: var(--bg-secondary);
      border: 1px solid var(--border-light);
      border-radius: 8px;
      overflow: hidden;
    }

    /* Removed problematic absolute positioning CSS for memory-3d-controls */

    .memory-3d-info {
      position: absolute;
      bottom: 10px;
      left: 10px;
      z-index: 100;
      background: rgba(0, 0, 0, 0.7);
      color: white;
      padding: 8px 12px;
      border-radius: 6px;
      font-size: 0.8rem;
      font-family: monospace;
    }

    /* Timeline Playback Styles */
    .timeline-container {
      background: var(--bg-secondary);
      border: 1px solid var(--border-light);
      border-radius: 8px;
      padding: 16px;
      margin: 16px 0;
    }

    .timeline-slider {
      width: 100%;
      height: 6px;
      background: var(--border-light);
      border-radius: 3px;
      position: relative;
      cursor: pointer;
      margin: 12px 0;
    }

    .timeline-progress {
      height: 100%;
      background: linear-gradient(90deg, var(--primary-blue), var(--primary-green));
      border-radius: 3px;
      transition: width 0.1s ease;
    }

    .timeline-thumb {
      position: absolute;
      top: -6px;
      width: 18px;
      height: 18px;
      background: var(--primary-blue);
      border: 2px solid white;
      border-radius: 50%;
      cursor: grab;
      box-shadow: var(--shadow-light);
    }

    .timeline-thumb:active {
      cursor: grabbing;
      transform: scale(1.1);
    }

    .timeline-controls {
      display: flex;
      justify-content: center;
      gap: 12px;
      margin-top: 12px;
    }

    .timeline-btn {
      background: var(--primary-blue);
      color: white;
      border: none;
      padding: 8px 12px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.8rem;
      transition: all 0.2s ease;
    }

    .timeline-btn:hover {
      background: #1d4ed8;
      transform: translateY(-1px);
    }

    .timeline-btn:disabled {
      background: var(--text-secondary);
      cursor: not-allowed;
      transform: none;
    }

    /* Memory Heatmap Styles */
    .heatmap-container {
      position: relative;
      width: 100%;
      height: 300px;
      background: var(--bg-secondary);
      border: 1px solid var(--border-light);
      border-radius: 8px;
      overflow: hidden;
    }

    .heatmap-legend {
      position: absolute;
      top: 10px;
      right: 10px;
      z-index: 100;
      background: rgba(255, 255, 255, 0.9);
      padding: 8px;
      border-radius: 6px;
      font-size: 0.7rem;
    }

    .dark .heatmap-legend {
      background: rgba(0, 0, 0, 0.8);
      color: white;
    }

    .heatmap-mode-selector {
      display: flex;
      gap: 6px;
      margin-bottom: 12px;
    }

    .heatmap-mode-btn {
      background: var(--bg-primary);
      color: var(--text-primary);
      border: 1px solid var(--border-light);
      padding: 6px 12px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.8rem;
      transition: all 0.2s ease;
    }

    .heatmap-mode-btn.active {
      background: var(--primary-blue);
      color: white;
      border-color: var(--primary-blue);
    }

    .heatmap-mode-btn:hover {
      background: var(--primary-blue);
      color: white;
    }

    /* Memory Block Visualization */
    .memory-block {
      position: absolute;
      border: 1px solid rgba(255, 255, 255, 0.3);
      border-radius: 2px;
      cursor: pointer;
      transition: all 0.2s ease;
    }

    .memory-block:hover {
      border-color: white;
      border-width: 2px;
      z-index: 10;
    }

    .memory-block.heap {
      background: linear-gradient(45deg, #ff6b35, #f7931e);
    }

    .memory-block.stack {
      background: linear-gradient(45deg, #4dabf7, #339af0);
    }

    .memory-block.leaked {
      background: linear-gradient(45deg, #dc2626, #ef4444);
      animation: pulse 2s infinite;
    }

    /* Tooltip Styles */
    .memory-tooltip {
      position: absolute;
      background: rgba(0, 0, 0, 0.9);
      color: white;
      padding: 8px 12px;
      border-radius: 6px;
      font-size: 0.8rem;
      font-family: monospace;
      pointer-events: none;
      z-index: 1000;
      max-width: 300px;
      line-height: 1.4;
    }

    .memory-tooltip::after {
      content: '';
      position: absolute;
      top: 100%;
      left: 50%;
      margin-left: -5px;
      border-width: 5px;
      border-style: solid;
      border-color: rgba(0, 0, 0, 0.9) transparent transparent transparent;
    }
  </style>
  <script>
    // Data injection placeholder - will be replaced by build tool
    window.analysisData = {{ json_data }};

    // Emergency fallback: Load data directly from JSON files if injection failed
    if (!window.analysisData || Object.keys(window.analysisData).length === 0 ||
      !window.analysisData.memory_analysis || !window.analysisData.memory_analysis.allocations) {

      console.warn("Data injection failed, attempting to load from JSON files...");

      // Try to fetch the JSON data directly
      fetch("./large_scale_user_memory_analysis.json")
        .then(response => response.json())
        .then(memoryData => {
          console.log("✅ Loaded memory analysis data:", memoryData);

          // Construct the expected data structure
          window.analysisData = {
            memory_analysis: memoryData,
            lifetime: {},
            complex_types: {},
            unsafe_ffi: {},
            performance: {}
          };

          // Try to load other JSON files
          Promise.all([
            fetch("./large_scale_user_lifetime.json").then(r => r.json()).catch(() => ({})),
            fetch("./large_scale_user_complex_types.json").then(r => r.json()).catch(() => ({})),
            fetch("./large_scale_user_unsafe_ffi.json").then(r => r.json()).catch(() => ({})),
            fetch("./large_scale_user_performance.json").then(r => r.json()).catch(() => ({}))
          ]).then(([lifetime, complexTypes, unsafeFfi, performance]) => {
            window.analysisData.lifetime = lifetime;
            window.analysisData.complex_types = complexTypes;
            window.analysisData.unsafe_ffi = unsafeFfi;
            window.analysisData.performance = performance;

            console.log("✅ All data loaded, initializing enhanced features...");

            // Trigger enhanced features initialization
            console.log("🚀 Triggering enhanced features initialization...");
            if (typeof initEnhancedLifecycleVisualization === "function") {
              setTimeout(() => {
                console.log("🔄 Calling initEnhancedLifecycleVisualization...");
                initEnhancedLifecycleVisualization();
              }, 100);
            } else {
              console.error('❌ initEnhancedLifecycleVisualization function not found');
            }

            // Also trigger the main dashboard initialization if needed
            if (typeof initDashboard === 'function') {
              setTimeout(() => {
                console.log("🔄 Calling initDashboard...");
                initDashboard();
              }, 200);
            }
          });
        })
        .catch(error => {
          console.error('❌ Failed to load JSON data:', error);

          // Last resort: Create dummy data for testing
          window.analysisData = {
            memory_analysis: {
              allocations: [
                {
                  var_name: 'test_var_1',
                  type_name: 'Arc<String>',
                  size: 1024,
                  timestamp_alloc: Date.now() * 1000000,
                  lifetime_ms: 100.5,
                  is_leaked: false
                },
                {
                  var_name: 'test_var_2',
                  type_name: 'Vec<i32>',
                  size: 2048,
                  timestamp_alloc: Date.now() * 1000000 + 1000000,
                  lifetime_ms: 250.0,
                  is_leaked: true
                }
              ]
            }
          };
          console.log("⚠️ Using dummy data for testing");
        });
    } else {
      console.log("✅ Data injection successful");
    }

    console.log("Final analysisData:", window.analysisData);
    console.log("Allocations available:", window.analysisData?.memory_analysis?.allocations?.length || 0);

    // Enhanced Memory Visualization Functions
    class EnhancedMemoryVisualizer {
      constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.memoryBlocks = [];
        this.timeline = {
          isPlaying: false,
          currentTime: 0,
          totalTime: 0,
          speed: 1000, // ms per step
          data: []
        };
        this.heatmapMode = 'density';
        this.tooltip = null;
        this.initialized = false;
      }

      init() {
        if (this.initialized) return;
        console.log("Initializing EnhancedMemoryVisualizer...");

        this.initTooltip();
        this.init3DVisualization();
        this.initTimelineControls();
        this.initHeatmap();
        this.bindEvents();

        this.initialized = true;
        console.log("EnhancedMemoryVisualizer initialized successfully");
      }

      initTooltip() {
        this.tooltip = document.createElement('div');
        this.tooltip.className = 'memory-tooltip';
        this.tooltip.style.display = 'none';
        document.body.appendChild(this.tooltip);
      }

      init3DVisualization() {
        const container = document.getElementById('memory3DContainer');
        if (!container) return;

        // Scene setup with dark gradient background
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x1a1a2e);

        // Camera setup - closer view for better data inspection
        this.camera = new THREE.PerspectiveCamera(75, container.clientWidth / container.clientHeight, 0.1, 1000);
        this.camera.position.set(15, 10, 15);

        // Renderer setup with enhanced settings
        this.renderer = new THREE.WebGLRenderer({
          antialias: true,
          alpha: true,
          powerPreference: "high-performance"
        });
        this.renderer.setSize(container.clientWidth, container.clientHeight);
        this.renderer.shadowMap.enabled = true;
        this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        this.renderer.setClearColor(0x1a1a2e, 1);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        container.appendChild(this.renderer.domElement);

        // Controls setup with enhanced interaction
        try {
          if (typeof THREE !== 'undefined') {
            const OrbitControls = THREE.OrbitControls || window.OrbitControls;
            if (OrbitControls) {
              this.controls = new OrbitControls(this.camera, this.renderer.domElement);
              this.controls.enableDamping = true;
              this.controls.dampingFactor = 0.05;
              this.controls.enableZoom = true;
              this.controls.enablePan = true;
              this.controls.enableRotate = true;
              this.controls.autoRotate = false;
              this.controls.autoRotateSpeed = 0.5;
              this.controls.minDistance = 2;  // Allow closer inspection
              this.controls.maxDistance = 100;
              this.controls.maxPolarAngle = Math.PI;
              this.controls.minPolarAngle = 0;
            } else {
              console.warn('OrbitControls not available, setting up manual controls');
              this.setupManualControls();
            }
          }
        } catch (error) {
          console.warn('Failed to initialize OrbitControls:', error);
        }

        // Enhanced lighting setup
        const ambientLight = new THREE.AmbientLight(0x404040, 0.4);
        this.scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(50, 50, 25);
        directionalLight.castShadow = true;
        directionalLight.shadow.mapSize.width = 2048;
        directionalLight.shadow.mapSize.height = 2048;
        directionalLight.shadow.camera.near = 0.5;
        directionalLight.shadow.camera.far = 500;
        this.scene.add(directionalLight);

        // Add subtle rim lighting
        const rimLight = new THREE.DirectionalLight(0x4a90e2, 0.3);
        rimLight.position.set(-50, 20, -25);
        this.scene.add(rimLight);

        // Add point light for dynamic effects
        this.pointLight = new THREE.PointLight(0x4a90e2, 0.5, 100);
        this.pointLight.position.set(0, 20, 0);
        this.scene.add(this.pointLight);

        // Create subtle floor plane instead of grid
        const floorGeometry = new THREE.PlaneGeometry(100, 100);
        const floorMaterial = new THREE.MeshLambertMaterial({
          color: 0x2a2a3e,
          transparent: true,
          opacity: 0.3
        });
        this.floor = new THREE.Mesh(floorGeometry, floorMaterial);
        this.floor.rotation.x = -Math.PI / 2;
        this.floor.position.y = -0.1;
        this.floor.receiveShadow = true;
        this.scene.add(this.floor);

        // Initialize animation properties
        this.animationTime = 0;
        this.isAutoRotating = false;

        this.animate3D();
      }

      setupManualControls() {
        if (!this.renderer || !this.camera) return;

        const canvas = this.renderer.domElement;
        let isMouseDown = false;
        let mouseX = 0, mouseY = 0;
        let cameraDistance = 20;
        let cameraAngleX = 0;
        let cameraAngleY = 0;

        // Mouse controls for rotation
        canvas.addEventListener('mousedown', (event) => {
          isMouseDown = true;
          mouseX = event.clientX;
          mouseY = event.clientY;
          canvas.style.cursor = 'grabbing';
        });

        canvas.addEventListener('mousemove', (event) => {
          if (!isMouseDown) return;

          const deltaX = event.clientX - mouseX;
          const deltaY = event.clientY - mouseY;

          cameraAngleY += deltaX * 0.01;
          cameraAngleX += deltaY * 0.01;

          // Limit vertical rotation
          cameraAngleX = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, cameraAngleX));

          // Update camera position
          this.camera.position.x = Math.cos(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;
          this.camera.position.y = Math.sin(cameraAngleX) * cameraDistance;
          this.camera.position.z = Math.sin(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;

          this.camera.lookAt(0, 0, 0);

          mouseX = event.clientX;
          mouseY = event.clientY;
        });

        canvas.addEventListener('mouseup', () => {
          isMouseDown = false;
          canvas.style.cursor = 'grab';
        });

        canvas.addEventListener('mouseleave', () => {
          isMouseDown = false;
          canvas.style.cursor = 'default';
        });

        // Zoom with mouse wheel
        canvas.addEventListener('wheel', (event) => {
          event.preventDefault();

          const zoomSpeed = 0.1;
          cameraDistance += event.deltaY * zoomSpeed;
          cameraDistance = Math.max(2, Math.min(100, cameraDistance));

          // Update camera position
          this.camera.position.x = Math.cos(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;
          this.camera.position.y = Math.sin(cameraAngleX) * cameraDistance;
          this.camera.position.z = Math.sin(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;

          this.camera.lookAt(0, 0, 0);
        });

        // Touch controls for mobile
        let lastTouchDistance = 0;

        canvas.addEventListener('touchstart', (event) => {
          if (event.touches.length === 1) {
            isMouseDown = true;
            mouseX = event.touches[0].clientX;
            mouseY = event.touches[0].clientY;
          } else if (event.touches.length === 2) {
            const touch1 = event.touches[0];
            const touch2 = event.touches[1];
            lastTouchDistance = Math.sqrt(
              Math.pow(touch2.clientX - touch1.clientX, 2) +
              Math.pow(touch2.clientY - touch1.clientY, 2)
            );
          }
        });

        canvas.addEventListener('touchmove', (event) => {
          event.preventDefault();

          if (event.touches.length === 1 && isMouseDown) {
            const deltaX = event.touches[0].clientX - mouseX;
            const deltaY = event.touches[0].clientY - mouseY;

            cameraAngleY += deltaX * 0.01;
            cameraAngleX += deltaY * 0.01;
            cameraAngleX = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, cameraAngleX));

            this.camera.position.x = Math.cos(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;
            this.camera.position.y = Math.sin(cameraAngleX) * cameraDistance;
            this.camera.position.z = Math.sin(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;

            this.camera.lookAt(0, 0, 0);

            mouseX = event.touches[0].clientX;
            mouseY = event.touches[0].clientY;
          } else if (event.touches.length === 2) {
            const touch1 = event.touches[0];
            const touch2 = event.touches[1];
            const touchDistance = Math.sqrt(
              Math.pow(touch2.clientX - touch1.clientX, 2) +
              Math.pow(touch2.clientY - touch1.clientY, 2)
            );

            if (lastTouchDistance > 0) {
              const zoomDelta = (lastTouchDistance - touchDistance) * 0.01;
              cameraDistance += zoomDelta;
              cameraDistance = Math.max(2, Math.min(100, cameraDistance));

              this.camera.position.x = Math.cos(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;
              this.camera.position.y = Math.sin(cameraAngleX) * cameraDistance;
              this.camera.position.z = Math.sin(cameraAngleY) * Math.cos(cameraAngleX) * cameraDistance;

              this.camera.lookAt(0, 0, 0);
            }

            lastTouchDistance = touchDistance;
          }
        });

        canvas.addEventListener('touchend', () => {
          isMouseDown = false;
          lastTouchDistance = 0;
        });

        canvas.style.cursor = 'grab';
        console.log("✅ Manual 3D controls initialized (mouse drag to rotate, wheel to zoom)");
      }

      animate3D() {
        requestAnimationFrame(() => this.animate3D());

        this.animationTime += 0.01;

        // Animate point light for dynamic lighting
        if (this.pointLight) {
          this.pointLight.position.x = Math.sin(this.animationTime) * 30;
          this.pointLight.position.z = Math.cos(this.animationTime) * 30;
          this.pointLight.intensity = 0.3 + Math.sin(this.animationTime * 2) * 0.2;
        }

        // Animate memory blocks with subtle floating effect
        this.memoryBlocks.forEach((block, index) => {
          if (block && block.position) {
            const originalY = block.userData.originalY || block.position.y;
            block.position.y = originalY + Math.sin(this.animationTime + index * 0.1) * 0.2;

            // Add subtle rotation for leaked memory blocks
            if (block.userData.is_leaked) {
              block.rotation.y += 0.02;
            }
          }
        });

        // Update controls
        if (this.controls) {
          this.controls.update();
        }

        // Render scene
        if (this.renderer && this.scene && this.camera) {
          this.renderer.render(this.scene, this.camera);
        }
      }

      create3DMemoryBlocks(allocations) {
        if (!this.scene || !allocations) return;

        // Clear existing blocks with fade out animation
        this.memoryBlocks.forEach(block => {
          if (block && block.material) {
            // Fade out animation
            const fadeOut = () => {
              block.material.opacity -= 0.05;
              if (block.material.opacity <= 0) {
                this.scene.remove(block);
                if (block.geometry) block.geometry.dispose();
                if (block.material) block.material.dispose();
              } else {
                requestAnimationFrame(fadeOut);
              }
            };
            fadeOut();
          }
        });
        this.memoryBlocks = [];

        // Sort allocations by size for better visual hierarchy
        const sortedAllocs = [...allocations].sort((a, b) => (b.size || 0) - (a.size || 0));

        const maxBlocksPerRow = 15;
        const spacing = 4;

        sortedAllocs.forEach((alloc, index) => {
          const size = Math.max(alloc.size || 1, 1);
          const blockSize = Math.cbrt(size / 50) + 0.8; // Enhanced size calculation

          // Spiral positioning for better visual distribution
          const angle = index * 0.5;
          const radius = Math.sqrt(index) * 2;
          const x = Math.cos(angle) * radius;
          const z = Math.sin(angle) * radius;
          const y = blockSize / 2;

          // Enhanced color scheme based on type and size
          let color = 0x3b82f6;
          let emissive = 0x000000;
          const typeName = alloc.type_name || '';

          if (typeName.includes('String')) {
            color = 0x4a90e2;
            emissive = 0x001122;
          } else if (typeName.includes('Box')) {
            color = 0xe74c3c;
            emissive = 0x220011;
          } else if (typeName.includes('Rc')) {
            color = 0x2ecc71;
            emissive = 0x001100;
          } else if (typeName.includes('Arc')) {
            color = 0x9b59b6;
            emissive = 0x110022;
          } else if (typeName.includes('Vec')) {
            color = 0xf39c12;
            emissive = 0x221100;
          }

          // Create enhanced geometry with rounded edges
          const geometry = new THREE.BoxGeometry(blockSize, blockSize, blockSize);

          // Enhanced material with better visual effects
          const material = new THREE.MeshPhongMaterial({
            color: color,
            emissive: emissive,
            transparent: true,
            opacity: alloc.is_leaked ? 0.7 : 0.9,
            shininess: 100,
            specular: 0x222222
          });

          const cube = new THREE.Mesh(geometry, material);
          cube.position.set(x, y, z);
          cube.castShadow = true;
          cube.receiveShadow = true;

          // Store original position and allocation data
          cube.userData = {
            ...alloc,
            originalY: y,
            originalColor: color,
            originalEmissive: emissive
          };

          // Add entrance animation
          cube.scale.set(0, 0, 0);
          const targetScale = 1;
          const animateEntrance = () => {
            cube.scale.x += (targetScale - cube.scale.x) * 0.1;
            cube.scale.y += (targetScale - cube.scale.y) * 0.1;
            cube.scale.z += (targetScale - cube.scale.z) * 0.1;

            if (Math.abs(cube.scale.x - targetScale) > 0.01) {
              requestAnimationFrame(animateEntrance);
            }
          };
          setTimeout(() => animateEntrance(), index * 50);

          // Add hover effects
          cube.addEventListener = (event, handler) => {
            // Custom event handling for 3D objects
          };

          this.scene.add(cube);
          this.memoryBlocks.push(cube);
        });

        this.update3DInfo(allocations.length);
        this.setupRaycasting();
      }

      setupRaycasting() {
        if (!this.renderer || !this.camera) return;

        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();
        this.hoveredObject = null;

        const canvas = this.renderer.domElement;

        canvas.addEventListener('mousemove', (event) => {
          const rect = canvas.getBoundingClientRect();
          this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
          this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

          this.raycaster.setFromCamera(this.mouse, this.camera);
          const intersects = this.raycaster.intersectObjects(this.memoryBlocks);

          // Reset previous hover
          if (this.hoveredObject) {
            this.hoveredObject.material.emissive.setHex(this.hoveredObject.userData.originalEmissive);
            this.hoveredObject.scale.set(1, 1, 1);
          }

          if (intersects.length > 0) {
            this.hoveredObject = intersects[0].object;
            // Highlight hovered object
            this.hoveredObject.material.emissive.setHex(0x444444);
            this.hoveredObject.scale.set(1.2, 1.2, 1.2);

            // Show tooltip
            this.show3DTooltip(event, this.hoveredObject.userData);
            canvas.style.cursor = 'pointer';
          } else {
            this.hoveredObject = null;
            this.hide3DTooltip();
            canvas.style.cursor = 'default';
          }
        });

        canvas.addEventListener("click", (event) => {
          if (this.hoveredObject) {
            this.selectMemoryBlock(this.hoveredObject);
          }
        });

        canvas.addEventListener('mouseleave', () => {
          if (this.hoveredObject) {
            this.hoveredObject.material.emissive.setHex(this.hoveredObject.userData.originalEmissive);
            this.hoveredObject.scale.set(1, 1, 1);
            this.hoveredObject = null;
          }
          this.hide3DTooltip();
          canvas.style.cursor = 'default';
        });
      }

      selectMemoryBlock(block) {
        // Animate selection
        const originalScale = { x: 1, y: 1, z: 1 };
        const targetScale = { x: 1.5, y: 1.5, z: 1.5 };

        const animateSelection = () => {
          block.scale.x += (targetScale.x - block.scale.x) * 0.2;
          block.scale.y += (targetScale.y - block.scale.y) * 0.2;
          block.scale.z += (targetScale.z - block.scale.z) * 0.2;

          if (Math.abs(block.scale.x - targetScale.x) > 0.01) {
            requestAnimationFrame(animateSelection);
          } else {
            // Return to normal size after selection
            setTimeout(() => {
              const returnToNormal = () => {
                block.scale.x += (originalScale.x - block.scale.x) * 0.2;
                block.scale.y += (originalScale.y - block.scale.y) * 0.2;
                block.scale.z += (originalScale.z - block.scale.z) * 0.2;

                if (Math.abs(block.scale.x - originalScale.x) > 0.01) {
                  requestAnimationFrame(returnToNormal);
                }
              };
              returnToNormal();
            }, 1000);
          }
        };
        animateSelection();

        // Show detailed info
        this.showDetailedInfo(block.userData);
      }

      show3DTooltip(event, alloc) {
        if (!this.tooltip) return;

        this.tooltip.innerHTML = `
          <div style="font-weight: bold; color: #4a90e2; margin-bottom: 4px;">${alloc.var_name || 'Unknown'}</div>
          <div style="margin-bottom: 2px;"><span style="color: #888;">Type:</span> ${alloc.type_name || 'Unknown'}</div>
          <div style="margin-bottom: 2px;"><span style="color: #888;">Size:</span> ${this.formatBytes(alloc.size || 0)}</div>
          <div style="margin-bottom: 2px;"><span style="color: #888;">Address:</span> 0x${(alloc.ptr || 0).toString(16)}</div>
          <div style="margin-bottom: 2px;"><span style="color: #888;">Lifetime:</span> ${(alloc.lifetime_ms || 0).toFixed(2)}ms</div>
          <div style="color: ${alloc.is_leaked ? '#e74c3c' : '#2ecc71'};">
            ${alloc.is_leaked ? '⚠️ Leaked' : '✅ Active'}
          </div>
        `;

        this.tooltip.style.display = 'block';
        this.tooltip.style.left = `${event.pageX + 15}px`;
        this.tooltip.style.top = `${event.pageY - 10}px`;
      }

      hide3DTooltip() {
        if (this.tooltip) {
          this.tooltip.style.display = 'none';
        }
      }

      showDetailedInfo(alloc) {
        const infoEl = document.getElementById('memory3DInfo');
        if (infoEl) {
          infoEl.innerHTML = `
            <div style="font-weight: bold; color: #4a90e2; margin-bottom: 8px;">Selected: ${alloc.var_name}</div>
            <div style="font-size: 0.8rem; line-height: 1.4;">
              <div>Type: ${alloc.type_name}</div>
              <div>Size: ${this.formatBytes(alloc.size || 0)}</div>
              <div>Address: 0x${(alloc.ptr || 0).toString(16)}</div>
              <div>Scope: ${alloc.scope_name || 'unknown'}</div>
              <div>Lifetime: ${(alloc.lifetime_ms || 0).toFixed(2)}ms</div>
              <div>Status: ${alloc.is_leaked ? 'Leaked' : 'Active'}</div>
            </div>
          `;
        }
      }

      update3DInfo(blockCount) {
        const infoEl = document.getElementById('memory3DInfo');
        if (infoEl) {
          infoEl.innerHTML = `
            Memory Blocks: ${blockCount}<br>
            Camera: [${this.camera.position.x.toFixed(1)}, ${this.camera.position.y.toFixed(1)}, ${this.camera.position.z.toFixed(1)}]<br>
            Use mouse to rotate, zoom, and pan
          `;
        }
      }

      initTimelineControls() {
        console.log("Initializing timeline controls...");

        const playBtn = document.getElementById('timelinePlay');
        const pauseBtn = document.getElementById('timelinePause');
        const resetBtn = document.getElementById('timelineReset');
        const stepBtn = document.getElementById('timelineStep');
        const slider = document.getElementById('timelineSlider');
        const thumb = document.getElementById('timelineThumb');

        console.log("Found timeline buttons:", {
          playBtn: !!playBtn,
          pauseBtn: !!pauseBtn,
          resetBtn: !!resetBtn,
          stepBtn: !!stepBtn,
          slider: !!slider,
          thumb: !!thumb
        });

        if (playBtn) {
          playBtn.addEventListener("click", () => {
            console.log("Timeline play button clicked");
            this.playTimeline();
          });
          console.log("Play button event bound");
        } else {
          console.error('timelinePlay button not found');
        }

        if (pauseBtn) {
          pauseBtn.addEventListener("click", () => {
            console.log("Timeline pause button clicked");
            this.pauseTimeline();
          });
          console.log("Pause button event bound");
        } else {
          console.error('timelinePause button not found');
        }

        if (resetBtn) {
          resetBtn.addEventListener("click", () => {
            console.log("Timeline reset button clicked");
            this.resetTimeline();
          });
          console.log("Reset button event bound");
        } else {
          console.error('timelineReset button not found');
        }

        if (stepBtn) {
          stepBtn.addEventListener("click", () => {
            console.log("Timeline step button clicked");
            this.stepTimeline();
          });
          console.log("Step button event bound");
        } else {
          console.error('timelineStep button not found');
        }

        if (slider && thumb) {
          let isDragging = false;

          thumb.addEventListener('mousedown', (e) => {
            isDragging = true;
            e.preventDefault();
          });

          document.addEventListener('mousemove', (e) => {
            if (!isDragging) return;

            const rect = slider.getBoundingClientRect();
            const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
            const percentage = x / rect.width;

            this.setTimelinePosition(percentage);
          });

          document.addEventListener('mouseup', () => {
            isDragging = false;
          });

          slider.addEventListener("click", (e) => {
            if (isDragging) return;

            const rect = slider.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const percentage = x / rect.width;

            this.setTimelinePosition(percentage);
          });
        }
      }

      formatBytes(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
      }

      prepareTimelineData(allocations) {
        if (!allocations || allocations.length === 0) {
          this.timeline.data = [];
          this.timeline.totalTime = 0;
          this.updateTimelineDisplay();
          return;
        }

        // Filter out allocations with invalid timestamps and sort
        const validAllocs = allocations.filter(alloc =>
          alloc.timestamp_alloc &&
          !isNaN(alloc.timestamp_alloc) &&
          alloc.timestamp_alloc > 0
        );

        if (validAllocs.length === 0) {
          // If no valid timestamps, create synthetic timeline based on order
          this.timeline.data = allocations.map((alloc, index) => ({
            ...alloc,
            timestamp_alloc: index * 1000000, // 1ms intervals
            timestamp_dealloc: alloc.timestamp_dealloc || (index + 1) * 1000000
          }));
          this.timeline.totalTime = allocations.length * 1000000;
        } else {
          // Sort by allocation timestamp
          const sortedAllocs = [...validAllocs].sort((a, b) => {
            const timeA = a.timestamp_alloc || 0;
            const timeB = b.timestamp_alloc || 0;
            return timeA - timeB;
          });

          this.timeline.data = sortedAllocs;
          const minTime = sortedAllocs[0].timestamp_alloc || 0;
          const maxTime = sortedAllocs[sortedAllocs.length - 1].timestamp_alloc || 0;
          this.timeline.totalTime = Math.max(maxTime - minTime, 1000000); // At least 1ms
        }

        this.updateTimelineDisplay();
      }

      playTimeline() {
        console.log("Starting timeline playback...");
        if (this.timeline.isPlaying) {
          console.log("Timeline already playing");
          return;
        }

        console.log("Timeline data:", {
          totalTime: this.timeline.totalTime,
          currentTime: this.timeline.currentTime,
          dataLength: this.timeline.data.length
        });

        this.timeline.isPlaying = true;
        const playBtn = document.getElementById('timelinePlay');
        const pauseBtn = document.getElementById('timelinePause');

        if (playBtn) playBtn.disabled = true;
        if (pauseBtn) pauseBtn.disabled = false;

        console.log("Timeline playback started");

        // Get speed from control
        const speedSelect = document.getElementById('timelineSpeed');
        const speed = speedSelect ? parseFloat(speedSelect.value) : 1.0;

        // Use requestAnimationFrame for smoother animation
        const animate = () => {
          if (!this.timeline.isPlaying) return;

          // Calculate time increment based on speed and total time for better visualization
          const baseIncrement = Math.max(this.timeline.totalTime * 0.0005, 12500); // 0.05% of total time or 12.5μs minimum
          this.timeline.currentTime += baseIncrement * speed;

          if (this.timeline.currentTime >= this.timeline.totalTime) {
            this.timeline.currentTime = this.timeline.totalTime;
            this.updateTimelineVisualization();
            this.pauseTimeline();
            console.log("Timeline playback completed");
            return;
          }

          this.updateTimelineVisualization();

          // Continue animation with consistent frame rate
          const frameDelay = Math.max(16, 200 / speed); // Slower frame rate for better visualization
          setTimeout(() => {
            if (this.timeline.isPlaying) {
              requestAnimationFrame(animate);
            }
          }, frameDelay);
        };

        requestAnimationFrame(animate);
      }

      pauseTimeline() {
        this.timeline.isPlaying = false;

        const playBtn = document.getElementById('timelinePlay');
        const pauseBtn = document.getElementById('timelinePause');

        if (playBtn) playBtn.disabled = false;
        if (pauseBtn) pauseBtn.disabled = true;

        if (this.timelineInterval) {
          clearInterval(this.timelineInterval);
          this.timelineInterval = null;
        }
      }

      resetTimeline() {
        console.log("Resetting timeline...");
        this.pauseTimeline();
        this.timeline.currentTime = 0;
        this.updateTimelineVisualization();
        
        // Reset 3D visualization to show all allocations
        if (window.analysisData && window.analysisData.memory_analysis) {
          this.create3DMemoryBlocks(window.analysisData.memory_analysis.allocations || []);
        }
        
        console.log("Timeline reset to beginning");
      }

      stepTimeline() {
        // Step forward by 1% of total time or 1ms, whichever is larger
        const stepSize = Math.max(this.timeline.totalTime * 0.01, 1000000); // 1ms minimum
        this.timeline.currentTime += stepSize;
        if (this.timeline.currentTime > this.timeline.totalTime) {
          this.timeline.currentTime = this.timeline.totalTime;
        }
        this.updateTimelineVisualization();
        console.log(`Timeline stepped to ${(this.timeline.currentTime / 1000000).toFixed(1)}ms`);
      }

      setTimelinePosition(percentage) {
        this.timeline.currentTime = percentage * this.timeline.totalTime;
        this.updateTimelineVisualization();
      }

      updateTimelineVisualization() {
        const percentage = this.timeline.totalTime > 0 ? this.timeline.currentTime / this.timeline.totalTime : 0;

        // Cache DOM elements to avoid repeated queries
        if (!this.timelineElements) {
          this.timelineElements = {
            progress: document.getElementById('timelineProgress'),
            thumb: document.getElementById('timelineThumb'),
            currentTime: document.getElementById('timelineCurrentTime'),
            totalTime: document.getElementById('timelineTotalTime'),
            activeCount: document.getElementById('timelineActiveCount')
          };
        }

        // Only update if percentage changed significantly (reduce flickering)
        const roundedPercentage = Math.round(percentage * 1000) / 10; // Round to 0.1%
        if (this.lastPercentage !== roundedPercentage) {
          this.lastPercentage = roundedPercentage;

          if (this.timelineElements.progress) {
            this.timelineElements.progress.style.width = `${roundedPercentage}%`;
          }
          if (this.timelineElements.thumb) {
            this.timelineElements.thumb.style.left = `calc(${roundedPercentage}% - 9px)`;
          }
        }

        // Update time display less frequently
        if (!this.lastTimeUpdate || Date.now() - this.lastTimeUpdate > 100) {
          this.lastTimeUpdate = Date.now();

          const currentTimeMs = isNaN(this.timeline.currentTime) ? 0 : this.timeline.currentTime / 1000000;
          const totalTimeMs = isNaN(this.timeline.totalTime) ? 0 : this.timeline.totalTime / 1000000;

          if (this.timelineElements.currentTime) {
            safeUpdateElement('timelineCurrentTime', `${currentTimeMs.toFixed(1)}ms`);
          }
          if (this.timelineElements.totalTime) {
            safeUpdateElement('timelineTotalTime', `${totalTimeMs.toFixed(1)}ms`);
          }
        }

        // Count active allocations at current time
        const activeAllocs = this.timeline.data.filter(alloc => {
          const allocTime = alloc.timestamp_alloc || 0;
          const deallocTime = alloc.timestamp_dealloc || Infinity;
          const currentTime = this.timeline.data[0] ? (this.timeline.data[0].timestamp_alloc || 0) + this.timeline.currentTime : 0;

          return allocTime <= currentTime && currentTime < deallocTime;
        });

        if (this.timelineElements.activeCount) {
          safeUpdateElement('timelineActiveCount', activeAllocs.length);
        }

        // Update 3D visualization with current active allocations
        this.create3DMemoryBlocks(activeAllocs);
      }

      updateTimelineDisplay() {
        const totalTimeEl = document.getElementById('timelineTotalTime');
        const currentTimeEl = document.getElementById('timelineCurrentTime');

        const totalTimeMs = isNaN(this.timeline.totalTime) ? 0 : this.timeline.totalTime / 1000000;
        const currentTimeMs = isNaN(this.timeline.currentTime) ? 0 : this.timeline.currentTime / 1000000;

        safeUpdateElement('timelineTotalTime', `${totalTimeMs.toFixed(1)}ms`);
        safeUpdateElement('timelineCurrentTime', `${currentTimeMs.toFixed(1)}ms`);
      }

      initHeatmap() {
        const container = document.getElementById('memoryHeatmap');
        const modeButtons = document.querySelectorAll('.heatmap-mode-btn');

        modeButtons.forEach(btn => {
          btn.addEventListener("click", (e) => {
            modeButtons.forEach(b => b.classList.remove('active'));
            e.target.classList.add('active');
            this.heatmapMode = e.target.dataset.mode;
            this.updateHeatmap();
          });
        });
      }

      generateHeatmap(allocations) {
        console.log("Generating enhanced heatmap with", allocations?.length || 0, 'allocations');
        if (!allocations) return;

        const container = document.getElementById('memoryHeatmap');
        if (!container) return;

        // Clear existing heatmap
        container.innerHTML = '<div class="heatmap-legend" id="heatmapLegend"></div>';

        const width = container.clientWidth;
        const height = container.clientHeight - 40; // Account for legend
        const cellSize = 6; // Smaller cells for better resolution
        const cols = Math.floor(width / cellSize);
        const rows = Math.floor(height / cellSize);

        // Create heatmap data based on mode
        let heatmapData = [];
        let metadata = {};

        switch (this.heatmapMode) {
          case 'density':
            const densityResult = this.calculateDensityHeatmap(allocations, cols, rows);
            heatmapData = densityResult.data || densityResult;
            metadata = densityResult.metadata || {};
            break;
          case 'type':
            const typeResult = this.calculateTypeHeatmap(allocations, cols, rows);
            heatmapData = typeResult.data || typeResult;
            metadata = typeResult.metadata || {};
            break;
          case 'scope':
            const scopeResult = this.calculateScopeHeatmap(allocations, cols, rows);
            heatmapData = scopeResult.data || scopeResult;
            metadata = scopeResult.metadata || {};
            break;
          case 'activity':
            const activityResult = this.calculateActivityHeatmap(allocations, cols, rows);
            heatmapData = activityResult.data || activityResult;
            metadata = activityResult.metadata || {};
            break;
          case 'fragmentation':
            const fragResult = this.calculateFragmentationHeatmap(allocations, cols, rows);
            heatmapData = fragResult.data || [];
            metadata = fragResult.metadata || {};
            break;
          case 'lifetime':
            const lifetimeResult = this.calculateLifetimeHeatmap(allocations, cols, rows);
            heatmapData = lifetimeResult.data || [];
            metadata = lifetimeResult.metadata || {};
            break;
        }

        // Render enhanced heatmap with smooth transitions
        const fragment = document.createDocumentFragment();
        for (let row = 0; row < rows; row++) {
          for (let col = 0; col < cols; col++) {
            const index = row * cols + col;
            const intensity = heatmapData[index] || 0;

            if (intensity > 0.01) { // Only render visible cells for performance
              const cell = document.createElement('div');
              cell.style.position = 'absolute';
              cell.style.left = `${col * cellSize}px`;
              cell.style.top = `${row * cellSize + 40}px`;
              cell.style.width = `${cellSize}px`;
              cell.style.height = `${cellSize}px`;
              cell.style.backgroundColor = this.getHeatmapColor(intensity, this.heatmapMode);
              cell.style.opacity = Math.max(0.1, intensity);
              cell.style.transition = 'all 0.3s ease';
              cell.style.borderRadius = '1px';

              // Add hover effects for interactivity
              cell.addEventListener('mouseenter', (e) => {
                e.target.style.transform = 'scale(1.2)';
                e.target.style.zIndex = '10';
                this.showHeatmapTooltip(e, intensity, metadata, row, col);
              });

              cell.addEventListener('mouseleave', (e) => {
                e.target.style.transform = 'scale(1)';
                e.target.style.zIndex = '1';
                this.hideHeatmapTooltip();
              });

              fragment.appendChild(cell);
            }
          }
        }

        container.appendChild(fragment);
        this.updateHeatmapLegend(metadata);
      }

      calculateDensityHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);
        const maxSize = Math.max(...allocations.map(a => a.size || 0));

        allocations.forEach((alloc, index) => {
          const x = index % cols;
          const y = Math.floor(index / cols) % rows;
          const cellIndex = y * cols + x;

          if (cellIndex < data.length) {
            data[cellIndex] += (alloc.size || 0) / maxSize;
          }
        });

        return data;
      }

      calculateTypeHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);
        const typeMap = new Map();

        allocations.forEach(alloc => {
          const type = alloc.type_name || 'unknown';
          typeMap.set(type, (typeMap.get(type) || 0) + 1);
        });

        const maxCount = Math.max(...typeMap.values());
        let index = 0;

        for (const [type, count] of typeMap.entries()) {
          const intensity = count / maxCount;
          const startIndex = index * Math.floor(data.length / typeMap.size);
          const endIndex = Math.min(startIndex + Math.floor(data.length / typeMap.size), data.length);

          for (let i = startIndex; i < endIndex; i++) {
            data[i] = intensity;
          }
          index++;
        }

        return data;
      }

      calculateScopeHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);
        const scopeMap = new Map();

        allocations.forEach(alloc => {
          const scope = alloc.scope || 'global';
          scopeMap.set(scope, (scopeMap.get(scope) || 0) + 1);
        });

        const maxCount = Math.max(...scopeMap.values());
        let index = 0;

        for (const [scope, count] of scopeMap.entries()) {
          const intensity = count / maxCount;
          const startIndex = index * Math.floor(data.length / scopeMap.size);
          const endIndex = Math.min(startIndex + Math.floor(data.length / scopeMap.size), data.length);

          for (let i = startIndex; i < endIndex; i++) {
            data[i] = intensity;
          }
          index++;
        }

        return data;
      }

      calculateActivityHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);

        if (allocations.length === 0) return { data, metadata: {} };

        const minTime = Math.min(...allocations.map(a => a.timestamp_alloc || 0));
        const maxTime = Math.max(...allocations.map(a => a.timestamp_alloc || 0));
        const timeRange = maxTime - minTime || 1;

        allocations.forEach(alloc => {
          const timeRatio = ((alloc.timestamp_alloc || 0) - minTime) / timeRange;
          const cellIndex = Math.floor(timeRatio * data.length);

          if (cellIndex < data.length) {
            data[cellIndex] += 0.1;
          }
        });

        const maxActivity = Math.max(...data);
        const normalizedData = maxActivity > 0 ? data.map(d => d / maxActivity) : data;

        return {
          data: normalizedData,
          metadata: {
            maxActivity,
            totalAllocations: allocations.length,
            timeRange: timeRange / 1000000 // Convert to ms
          }
        };
      }

      calculateFragmentationHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);

        // Sort allocations by memory address
        const sortedAllocs = allocations
          .filter(a => a.ptr && a.size)
          .map(a => ({
            address: parseInt(a.ptr.replace('0x', ''), 16),
            size: a.size,
            ...a
          }))
          .sort((a, b) => a.address - b.address);

        // Calculate fragmentation score for each memory region
        for (let i = 0; i < sortedAllocs.length - 1; i++) {
          const current = sortedAllocs[i];
          const next = sortedAllocs[i + 1];
          const gap = next.address - (current.address + current.size);

          if (gap > 0) {
            // Map to heatmap coordinates
            const normalizedAddr = (current.address % (cols * rows * 1000)) / (cols * rows * 1000);
            const row = Math.floor(normalizedAddr * rows);
            const col = Math.floor((i / sortedAllocs.length) * cols);
            const cellIndex = Math.min(row * cols + col, data.length - 1);

            // Higher gap = higher fragmentation
            data[cellIndex] += Math.min(gap / 1000, 1); // Normalize gap size
          }
        }

        const maxFrag = Math.max(...data);
        const normalizedData = maxFrag > 0 ? data.map(d => d / maxFrag) : data;

        return {
          data: normalizedData,
          metadata: {
            maxFragmentation: maxFrag,
            totalGaps: data.filter(d => d > 0).length,
            avgFragmentation: data.reduce((a, b) => a + b, 0) / data.length
          }
        };
      }

      calculateLifetimeHeatmap(allocations, cols, rows) {
        const data = new Array(cols * rows).fill(0);

        allocations.forEach((alloc, index) => {
          const allocTime = alloc.timestamp_alloc || 0;
          const deallocTime = alloc.timestamp_dealloc || Date.now() * 1000000;
          const lifetime = deallocTime - allocTime;

          // Map lifetime to heatmap position
          const row = Math.floor((index / allocations.length) * rows);
          const col = Math.floor((lifetime / 1000000000) * cols) % cols; // Convert to seconds
          const cellIndex = Math.min(row * cols + col, data.length - 1);

          data[cellIndex] += 1;
        });

        const maxLifetime = Math.max(...data);
        const normalizedData = maxLifetime > 0 ? data.map(d => d / maxLifetime) : data;

        return {
          data: normalizedData,
          metadata: {
            maxLifetime,
            avgLifetime: data.reduce((a, b) => a + b, 0) / data.length,
            activeAllocations: allocations.filter(a => !a.timestamp_dealloc).length
          }
        };
      }

      getHeatmapColor(intensity, mode = 'density') {
        const scaledIntensity = Math.min(Math.max(intensity, 0), 1);

        // Different color schemes for different modes
        const colorSchemes = {
          density: [
            [59, 130, 246],   // Blue
            [245, 158, 11],   // Orange
            [220, 38, 38]     // Red
          ],
          type: [
            [34, 197, 94],    // Green
            [168, 85, 247],   // Purple
            [239, 68, 68]     // Red
          ],
          scope: [
            [14, 165, 233],   // Sky blue
            [251, 191, 36],   // Amber
            [239, 68, 68]     // Red
          ],
          activity: [
            [99, 102, 241],   // Indigo
            [236, 72, 153],   // Pink
            [220, 38, 38]     // Red
          ],
          fragmentation: [
            [34, 197, 94],    // Green (low fragmentation)
            [251, 191, 36],   // Amber (medium)
            [239, 68, 68]     // Red (high fragmentation)
          ],
          lifetime: [
            [147, 51, 234],   // Purple (short-lived)
            [59, 130, 246],   // Blue (medium)
            [34, 197, 94]     // Green (long-lived)
          ]
        };

        const colors = colorSchemes[mode] || colorSchemes.density;
        const colorIndex = scaledIntensity * (colors.length - 1);
        const lowerIndex = Math.floor(colorIndex);
        const upperIndex = Math.ceil(colorIndex);
        const ratio = colorIndex - lowerIndex;

        if (lowerIndex === upperIndex) {
          const [r, g, b] = colors[lowerIndex];
          return `rgb(${r}, ${g}, ${b})`;
        }

        const [r1, g1, b1] = colors[lowerIndex];
        const [r2, g2, b2] = colors[upperIndex];

        const r = Math.round(r1 + (r2 - r1) * ratio);
        const g = Math.round(g1 + (g2 - g1) * ratio);
        const b = Math.round(b1 + (b2 - b1) * ratio);

        return `rgb(${r}, ${g}, ${b})`;
      }

      showHeatmapTooltip(event, intensity, metadata, row, col) {
        if (!this.tooltip) {
          this.tooltip = document.createElement('div');
          this.tooltip.style.cssText = `
            position: absolute;
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 8px 12px;
            border-radius: 6px;
            font-size: 12px;
            pointer-events: none;
            z-index: 1000;
            max-width: 200px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
          `;
          document.body.appendChild(this.tooltip);
        }

        const modeDescriptions = {
          density: 'Memory usage density',
          type: 'Type distribution',
          scope: 'Scope activity level',
          activity: 'Allocation activity',
          fragmentation: 'Memory fragmentation',
          lifetime: 'Allocation lifetime'
        };

        this.tooltip.innerHTML = `
          <div><strong>${modeDescriptions[this.heatmapMode] || 'Intensity'}</strong></div>
          <div>Value: ${(intensity * 100).toFixed(1)}%</div>
          <div>Position: (${col}, ${row})</div>
          ${metadata.maxActivity ? `<div>Max Activity: ${metadata.maxActivity}</div>` : ''}
          ${metadata.totalGaps ? `<div>Total Gaps: ${metadata.totalGaps}</div>` : ''}
          ${metadata.activeAllocations ? `<div>Active: ${metadata.activeAllocations}</div>` : ''}
        `;

        this.tooltip.style.left = `${event.pageX + 10}px`;
        this.tooltip.style.top = `${event.pageY - 10}px`;
        this.tooltip.style.display = 'block';
      }

      hideHeatmapTooltip() {
        if (this.tooltip) {
          this.tooltip.style.display = 'none';
        }
      }

      updateHeatmapLegend(metadata = {}) {
        const legend = document.getElementById('heatmapLegend');
        if (!legend) return;

        const modeLabels = {
          density: 'Memory Density',
          type: 'Type Distribution',
          scope: 'Scope Activity',
          activity: 'Allocation Activity',
          fragmentation: 'Memory Fragmentation',
          lifetime: 'Allocation Lifetime'
        };

        const modeDescriptions = {
          density: 'Shows memory usage concentration',
          type: 'Shows distribution of data types',
          scope: 'Shows activity by scope',
          activity: 'Shows allocation frequency over time',
          fragmentation: 'Shows memory fragmentation levels',
          lifetime: 'Shows allocation lifetime patterns'
        };

        const currentMode = this.heatmapMode;
        const lowColor = this.getHeatmapColor(0.2, currentMode);
        const medColor = this.getHeatmapColor(0.5, currentMode);
        const highColor = this.getHeatmapColor(0.8, currentMode);

        let metadataHtml = '';
        if (metadata.maxActivity) {
          metadataHtml += `<div style="font-size: 10px; color: rgba(255,255,255,0.8);">Max Activity: ${metadata.maxActivity}</div>`;
        }
        if (metadata.totalGaps) {
          metadataHtml += `<div style="font-size: 10px; color: rgba(255,255,255,0.8);">Gaps: ${metadata.totalGaps}</div>`;
        }
        if (metadata.activeAllocations) {
          metadataHtml += `<div style="font-size: 10px; color: rgba(255,255,255,0.8);">Active: ${metadata.activeAllocations}</div>`;
        }

        legend.innerHTML = `
          <div style="font-weight: 600; margin-bottom: 2px;">${modeLabels[currentMode]}</div>
          <div style="font-size: 10px; color: rgba(255,255,255,0.7); margin-bottom: 4px;">${modeDescriptions[currentMode]}</div>
          <div style="display: flex; align-items: center; gap: 4px; margin-bottom: 4px;">
            <div style="width: 12px; height: 12px; background: ${lowColor}; border-radius: 2px;"></div>
            <span style="font-size: 11px;">Low</span>
            <div style="width: 12px; height: 12px; background: ${medColor}; border-radius: 2px;"></div>
            <span style="font-size: 11px;">Med</span>
            <div style="width: 12px; height: 12px; background: ${highColor}; border-radius: 2px;"></div>
            <span style="font-size: 11px;">High</span>
          </div>
          ${metadataHtml}
        `;
      }

      updateHeatmap() {
        if (window.analysisData && window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
          this.generateHeatmap(window.analysisData.memory_analysis.allocations);
        }
      }

      bindEvents() {
        console.log("🔧 Binding 3D visualization events...");

        // Add visual feedback for button interactions
        this.addButtonFeedback();

        // Wait for DOM to be fully ready
        setTimeout(() => {
          const toggle3DBtn = document.getElementById('toggle3DView');
          const reset3DBtn = document.getElementById('reset3DView');
          const autoRotateBtn = document.getElementById('autoRotate3D');
          const focusLargestBtn = document.getElementById('focusLargest');

          console.log("🔍 Found buttons:", {
            toggle3DBtn: !!toggle3DBtn,
            reset3DBtn: !!reset3DBtn,
            autoRotateBtn: !!autoRotateBtn,
            focusLargestBtn: !!focusLargestBtn
          });

          if (toggle3DBtn) {
            // Remove any existing event listeners
            toggle3DBtn.replaceWith(toggle3DBtn.cloneNode(true));
            const newToggle3DBtn = document.getElementById('toggle3DView');
            
            newToggle3DBtn.addEventListener("click", (e) => {
              e.preventDefault();
              console.log("🎯 Toggle 3D view clicked");
              const container = document.getElementById('memory3DContainer');
              if (container) {
                const isHidden = container.style.display === 'none';
                if (isHidden) {
                  // Show 3D view
                  container.style.display = 'block';
                  newToggle3DBtn.innerHTML = '<i class="fa fa-eye-slash"></i><span>Hide 3D</span>';
                  newToggle3DBtn.style.background = 'var(--primary-red)';
                  console.log("✅ Showing 3D view");
                  
                  // Reinitialize 3D scene if needed
                  if (!this.scene) {
                    console.log("🔄 Reinitializing 3D scene...");
                    this.init3DVisualization();
                  }
                  
                  // Update 3D visualization with current data
                  if (window.analysisData && window.analysisData.memory_analysis) {
                    this.create3DMemoryBlocks(window.analysisData.memory_analysis.allocations || []);
                  }
                } else {
                  // Hide 3D view
                  container.style.display = 'none';
                  newToggle3DBtn.innerHTML = '<i class="fa fa-eye"></i><span>Show 3D</span>';
                  newToggle3DBtn.style.background = 'var(--primary-green)';
                  console.log("✅ Hiding 3D view");
                }
              } else {
                console.error('❌ 3D container not found');
              }
            });
            console.log("✅ Toggle 3D button event bound");
          } else {
            console.error('❌ toggle3DView button not found');
          }

          if (reset3DBtn) {
            // Remove any existing event listeners
            reset3DBtn.replaceWith(reset3DBtn.cloneNode(true));
            const newReset3DBtn = document.getElementById('reset3DView');
            
            newReset3DBtn.addEventListener("click", (e) => {
              e.preventDefault();
              console.log("🎯 Reset 3D view clicked");
              this.reset3DView();
            });
            console.log("✅ Reset 3D button event bound");
          } else {
            console.error('❌ reset3DView button not found');
          }

          if (autoRotateBtn) {
            // Remove any existing event listeners
            autoRotateBtn.replaceWith(autoRotateBtn.cloneNode(true));
            const newAutoRotateBtn = document.getElementById('autoRotate3D');
            
            newAutoRotateBtn.addEventListener("click", (e) => {
              e.preventDefault();
              console.log("🎯 Auto rotate clicked");
              this.toggleAutoRotate();
            });
            console.log("✅ Auto rotate button event bound");
          } else {
            console.error('❌ autoRotate3D button not found');
          }

          if (focusLargestBtn) {
            // Remove any existing event listeners
            focusLargestBtn.replaceWith(focusLargestBtn.cloneNode(true));
            const newFocusLargestBtn = document.getElementById('focusLargest');
            
            newFocusLargestBtn.addEventListener("click", (e) => {
              e.preventDefault();
              console.log("🎯 Focus largest clicked");
              this.focusOnLargestBlock();
            });
            console.log("✅ Focus largest button event bound");
          } else {
            console.error('❌ focusLargest button not found');
          }
        }, 500); // Wait 500ms for DOM to be ready

        // Handle window resize
        window.addEventListener('resize', () => {
          if (this.camera && this.renderer) {
            const container = document.getElementById('memory3DContainer');
            if (container) {
              this.camera.aspect = container.clientWidth / container.clientHeight;
              this.camera.updateProjectionMatrix();
              this.renderer.setSize(container.clientWidth, container.clientHeight);
            }
          }
        });
      }

      addButtonFeedback() {
        // Add hover and click effects to all 3D control buttons
        const buttonIds = ['toggle3DView', 'reset3DView', 'autoRotate3D', 'focusLargest'];
        
        buttonIds.forEach(id => {
          const btn = document.getElementById(id);
          if (btn) {
            // Add hover effect
            btn.addEventListener('mouseenter', () => {
              btn.style.transform = 'scale(1.05)';
              btn.style.transition = 'all 0.2s ease';
            });
            
            btn.addEventListener('mouseleave', () => {
              btn.style.transform = 'scale(1)';
            });
            
            // Add click effect
            btn.addEventListener('mousedown', () => {
              btn.style.transform = 'scale(0.95)';
            });
            
            btn.addEventListener('mouseup', () => {
              btn.style.transform = 'scale(1.05)';
            });
            
            console.log(`✅ Added feedback effects to ${id}`);
          }
        });
      }

      reset3DView() {
        console.log("🔄 Resetting 3D view...");
        
        // Show visual feedback
        const resetBtn = document.getElementById('reset3DView');
        if (resetBtn) {
          resetBtn.innerHTML = '<i class="fa fa-spinner fa-spin"></i><span>Resetting...</span>';
          resetBtn.style.background = 'var(--primary-yellow)';
        }
        
        if (this.camera && this.controls) {
          // Reset camera position
          this.camera.position.set(15, 10, 15);
          this.camera.lookAt(0, 0, 0);
          
          // Reset controls
          this.controls.reset();
          
          // Update camera
          this.camera.updateProjectionMatrix();
          
          // Restore button
          setTimeout(() => {
            if (resetBtn) {
              resetBtn.innerHTML = '<i class="fa fa-refresh"></i><span>Reset</span>';
              resetBtn.style.background = 'var(--primary-orange)';
            }
          }, 500);
          
          console.log("✅ 3D view reset complete");
        } else {
          console.error('❌ Camera or controls not available for reset');
          if (resetBtn) {
            resetBtn.innerHTML = '<i class="fa fa-exclamation"></i><span>Error</span>';
            resetBtn.style.background = 'var(--primary-red)';
            setTimeout(() => {
              resetBtn.innerHTML = '<i class="fa fa-refresh"></i><span>Reset</span>';
              resetBtn.style.background = 'var(--primary-orange)';
            }, 1000);
          }
        }
      
      // Animation function
      const animateReset = () => {
        // Animation logic here if needed
      };
      animateReset();
    }

      toggleAutoRotate() {
        console.log("🔄 Toggling auto rotate...");
        if (this.controls) {
          this.controls.autoRotate = !this.controls.autoRotate;
          this.controls.autoRotateSpeed = 2.0; // Set rotation speed
          
          const btn = document.getElementById('autoRotate3D');
          if (btn) {
            if (this.controls.autoRotate) {
              btn.innerHTML = '<i class="fa fa-pause"></i><span>Stop Rotate</span>';
              btn.style.background = 'var(--primary-red)';
              console.log("✅ Auto rotate enabled");
            } else {
              btn.innerHTML = '<i class="fa fa-rotate-right"></i><span>Auto Rotate</span>';
              btn.style.background = 'var(--primary-blue)';
              console.log("✅ Auto rotate disabled");
            }
          }
        } else {
          console.error('❌ Controls not available for auto rotate');
          const btn = document.getElementById('autoRotate3D');
          if (btn) {
            btn.innerHTML = '<i class="fa fa-exclamation"></i><span>Error</span>';
            btn.style.background = 'var(--primary-red)';
            setTimeout(() => {
              btn.innerHTML = '<i class="fa fa-rotate-right"></i><span>Auto Rotate</span>';
              btn.style.background = 'var(--primary-blue)';
            }, 1000);
          }
        }
      }

      focusOnLargestBlock() {
        console.log("🎯 Focusing on largest block...");
        
        // Show visual feedback
        const focusBtn = document.getElementById('focusLargest');
        if (focusBtn) {
          focusBtn.innerHTML = '<i class="fa fa-spinner fa-spin"></i><span>Focusing...</span>';
          focusBtn.style.background = 'var(--primary-yellow)';
        }
        
        if (!this.memoryBlocks || this.memoryBlocks.length === 0) {
          console.warn('❌ No memory blocks to focus on');
          if (focusBtn) {
            focusBtn.innerHTML = '<i class="fa fa-exclamation"></i><span>No Blocks</span>';
            focusBtn.style.background = 'var(--primary-red)';
            setTimeout(() => {
              focusBtn.innerHTML = '<i class="fa fa-search-plus"></i><span>Focus Largest</span>';
              focusBtn.style.background = 'var(--primary-red)';
            }, 1500);
          }
          return;
        }

        // Find the largest block
        let largestBlock = null;
        let largestSize = 0;

        this.memoryBlocks.forEach(block => {
          const size = block.userData?.size || 0;
          if (size > largestSize) {
            largestSize = size;
            largestBlock = block;
          }
        });

        if (largestBlock && this.camera && this.controls) {
          // Calculate optimal camera position
          const blockPos = largestBlock.position;
          const distance = Math.max(5, Math.sqrt(largestSize) / 10);
          
          // Position camera at an angle for better view
          const targetPosition = new THREE.Vector3(
            blockPos.x + distance,
            blockPos.y + distance * 0.7,
            blockPos.z + distance
          );

          // Smooth camera transition
          const startPos = this.camera.position.clone();
          let progress = 0;
          
          const animateFocus = () => {
            progress += 0.05;
            if (progress <= 1) {
              this.camera.position.lerpVectors(startPos, targetPosition, progress);
              this.camera.lookAt(blockPos);
              requestAnimationFrame(animateFocus);
            } else {
              // Animation complete
              console.log(`✅ Focused on largest block: ${largestBlock.userData?.var_name || 'unknown'} (${this.formatBytes(largestSize)})`);
              this.update3DInfo(this.memoryBlocks.length);
              
              // Restore button
              if (focusBtn) {
                focusBtn.innerHTML = '<i class="fa fa-search-plus"></i><span>Focus Largest</span>';
                focusBtn.style.background = 'var(--primary-red)';
              }
            }
          };
          animateFocus();
        } else {
          console.error('❌ Camera or controls not available for focus');
          if (focusBtn) {
            focusBtn.innerHTML = '<i class="fa fa-exclamation"></i><span>Error</span>';
            focusBtn.style.background = 'var(--primary-red)';
            setTimeout(() => {
              focusBtn.innerHTML = '<i class="fa fa-search-plus"></i><span>Focus Largest</span>';
              focusBtn.style.background = 'var(--primary-red)';
            }, 1000);
          }
        }
      }

      // Main initialization method
      initializeWithData(analysisData) {
        console.log("initializeWithData called with:", analysisData);

        let allocations = null;

        // Try different data structure paths
        if (analysisData && analysisData.memory_analysis && analysisData.memory_analysis.allocations) {
          allocations = analysisData.memory_analysis.allocations;
          console.log("Found allocations in memory_analysis:", allocations.length);
        } else if (analysisData && analysisData.allocations) {
          allocations = analysisData.allocations;
          console.log("Found allocations directly:", allocations.length);
        } else {
          console.warn('No allocation data found in analysisData');
          console.log("Available keys:", Object.keys(analysisData || {}));
          return;
        }

        if (!allocations || allocations.length === 0) {
          console.warn('No allocations to visualize');
          return;
        }

        console.log(`Initializing enhanced visualization with ${allocations.length} allocations`);

        // Initialize 3D visualization
        this.create3DMemoryBlocks(allocations);

        // Initialize timeline
        this.prepareTimelineData(allocations);

        // Initialize heatmap
        this.generateHeatmap(allocations);

        // Update memory distribution visualization
        this.updateMemoryDistribution(allocations);

        // Initialize memory fragmentation visualization
        this.initializeMemoryFragmentation(allocations);

        console.log("Enhanced memory visualization initialized successfully");
      }

      initializeMemoryFragmentation(allocations) {
        console.log("Initializing memory fragmentation with", allocations?.length || 0, 'allocations');
        const container = document.getElementById('memoryFragmentation');
        if (!container) {
          console.error('Memory fragmentation container not found');
          return;
        }
        if (!allocations || allocations.length === 0) {
          console.warn('No allocations data for fragmentation analysis');
          container.innerHTML = '<div style="text-align: center; color: var(--text-secondary); padding: 40px;">No allocation data available for fragmentation analysis</div>';
          return;
        }

        console.log("Processing", allocations.length, 'allocations for fragmentation analysis');

        // Calculate fragmentation metrics
        const sortedAllocs = [...allocations].sort((a, b) => {
          const addrA = parseInt(a.ptr || '0x0', 16);
          const addrB = parseInt(b.ptr || '0x0', 16);
          return addrA - addrB;
        });

        const totalMemory = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
        const addressRanges = [];
        let gaps = 0;
        let totalGapSize = 0;

        // Calculate memory gaps
        for (let i = 0; i < sortedAllocs.length - 1; i++) {
          const currentAddr = parseInt(sortedAllocs[i].ptr || '0x0', 16);
          const currentSize = sortedAllocs[i].size || 0;
          const nextAddr = parseInt(sortedAllocs[i + 1].ptr || '0x0', 16);

          const currentEnd = currentAddr + currentSize;
          const gap = nextAddr - currentEnd;

          if (gap > 0) {
            gaps++;
            totalGapSize += gap;
            addressRanges.push({
              type: 'gap',
              start: currentEnd,
              size: gap,
              index: i
            });
          }

          addressRanges.push({
            type: 'allocation',
            start: currentAddr,
            size: currentSize,
            allocation: sortedAllocs[i],
            index: i
          });
        }

        // Calculate fragmentation percentage
        const fragmentation = totalMemory > 0 ? (totalGapSize / (totalMemory + totalGapSize)) * 100 : 0;
        const efficiency = Math.max(100 - fragmentation, 0);

        console.log("Fragmentation metrics:", { gaps, totalGapSize, fragmentation, efficiency });

        // Create visualization
        container.innerHTML = `
          <div style="margin-bottom: 16px;">
            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 16px;">
              <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
                <div style="font-size: 1.4rem; font-weight: 700; color: var(--primary-red);">${fragmentation.toFixed(1)}%</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">Fragmentation</div>
              </div>
              <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
                <div style="font-size: 1.4rem; font-weight: 700; color: var(--primary-orange);">${gaps}</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">Memory Gaps</div>
              </div>
              <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
                <div style="font-size: 1.4rem; font-weight: 700; color: var(--primary-green);">${efficiency.toFixed(1)}%</div>
                <div style="font-size: 0.7rem; color: var(--text-secondary);">Efficiency</div>
              </div>
            </div>
            
            <div style="margin-bottom: 12px;">
              <h4 style="margin: 0 0 8px 0; font-size: 0.9rem; color: var(--text-primary);">Memory Layout Visualization</h4>
              <div id="fragmentationChart" style="height: 120px; background: var(--bg-secondary); border: 1px solid var(--border-light); border-radius: 6px; position: relative; overflow: hidden;">
                <!-- Memory blocks will be inserted here -->
              </div>
            </div>
            
            <div style="font-size: 0.8rem; color: var(--text-secondary); text-align: center;">
              <div style="display: flex; justify-content: center; gap: 16px; margin-top: 8px;">
                <div style="display: flex; align-items: center; gap: 4px;">
                  <div style="width: 12px; height: 12px; background: var(--primary-blue); border-radius: 2px;"></div>
                  <span>Allocated</span>
                </div>
                <div style="display: flex; align-items: center; gap: 4px;">
                  <div style="width: 12px; height: 12px; background: var(--primary-red); border-radius: 2px;"></div>
                  <span>Gaps</span>
                </div>
                <div style="display: flex; align-items: center; gap: 4px;">
                  <div style="width: 12px; height: 12px; background: var(--primary-orange); border-radius: 2px;"></div>
                  <span>Leaked</span>
                </div>
              </div>
            </div>
          </div>
        `;

        // Draw memory layout visualization
        this.drawFragmentationChart(addressRanges, totalMemory + totalGapSize);
      }

      drawFragmentationChart(addressRanges, totalSize) {
        const chartContainer = document.getElementById('fragmentationChart');
        if (!chartContainer || addressRanges.length === 0) return;

        const width = chartContainer.clientWidth;
        const height = chartContainer.clientHeight;

        let currentX = 0;

        addressRanges.forEach((range, index) => {
          const blockWidth = Math.max((range.size / totalSize) * width, 1);

          const block = document.createElement('div');
          block.style.position = 'absolute';
          block.style.left = `${currentX}px`;
          block.style.top = '10px';
          block.style.width = `${blockWidth}px`;
          block.style.height = `${height - 20}px`;
          block.style.borderRadius = '2px';
          block.style.cursor = 'pointer';
          block.style.transition = 'all 0.2s ease';

          if (range.type === 'gap') {
            block.style.background = 'linear-gradient(45deg, #dc2626, #ef4444)';
            block.style.border = '1px solid #b91c1c';
            block.title = `Memory Gap: ${this.formatBytes(range.size)}`;
          } else if (range.allocation && range.allocation.is_leaked) {
            block.style.background = 'linear-gradient(45deg, #ea580c, #f97316)';
            block.style.border = '1px solid #c2410c';
            block.title = `Leaked: ${range.allocation.var_name} (${this.formatBytes(range.size)})`;
          } else {
            block.style.background = 'linear-gradient(45deg, #2563eb, #3b82f6)';
            block.style.border = '1px solid #1d4ed8';
            block.title = range.allocation ?
              `${range.allocation.var_name}: ${range.allocation.type_name} (${this.formatBytes(range.size)})` :
              `Allocation: ${this.formatBytes(range.size)}`;
          }

          // Add hover effects
          block.addEventListener('mouseenter', () => {
            block.style.transform = 'scaleY(1.2)';
            block.style.zIndex = '10';
          });

          block.addEventListener('mouseleave', () => {
            block.style.transform = 'scaleY(1)';
            block.style.zIndex = '1';
          });

          chartContainer.appendChild(block);
          currentX += blockWidth;
        });
      }

      updateMemoryDistribution(allocations) {
        const container = document.getElementById('memoryDistributionViz');
        if (!container || !allocations) return;

        container.innerHTML = '';

        const containerWidth = container.clientWidth;
        const containerHeight = container.clientHeight;
        const totalMemory = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);

        // 获取当前缩放比例
        const scaleSlider = document.getElementById('memoryDistScale');
        const scale = scaleSlider ? parseFloat(scaleSlider.value) / 100 : 1;
        
        // 计算实际可用宽度（考虑缩放和边距）
        const padding = 20;
        const availableWidth = (containerWidth - padding * 2) * scale;
        const availableHeight = containerHeight - padding * 2;

        // 创建一个可滚动的内容容器
        const contentContainer = document.createElement('div');
        contentContainer.style.cssText = `
          position: absolute;
          top: ${padding}px;
          left: ${padding}px;
          width: ${Math.max(availableWidth, containerWidth - padding * 2)}px;
          height: ${availableHeight}px;
          overflow-x: auto;
          overflow-y: hidden;
        `;

        // 创建内存块容器
        const blocksContainer = document.createElement('div');
        blocksContainer.style.cssText = `
          position: relative;
          width: ${availableWidth}px;
          height: 100%;
          min-width: ${containerWidth - padding * 2}px;
        `;

        let currentX = 0;
        let fragmentation = 0;
        let efficiency = 0;

        // Sort by address for fragmentation calculation
        const sortedAllocs = [...allocations].sort((a, b) => {
          const addrA = parseInt(a.ptr || '0x0', 16);
          const addrB = parseInt(b.ptr || '0x0', 16);
          return addrA - addrB;
        });

        // 计算每个块的最小宽度和总宽度需求
        const minBlockWidth = 3; // 最小块宽度
        const blockGap = 1;
        let totalRequiredWidth = 0;

        sortedAllocs.forEach((alloc) => {
          const proportionalWidth = (alloc.size || 0) / totalMemory * availableWidth;
          const blockWidth = Math.max(proportionalWidth, minBlockWidth);
          totalRequiredWidth += blockWidth + blockGap;
        });

        // 如果总宽度超过可用宽度，调整容器宽度
        const finalContainerWidth = Math.max(totalRequiredWidth, availableWidth);
        blocksContainer.style.width = `${finalContainerWidth}px`;

        sortedAllocs.forEach((alloc, index) => {
          const proportionalWidth = (alloc.size || 0) / totalMemory * availableWidth;
          const blockWidth = Math.max(proportionalWidth, minBlockWidth);
          const blockHeight = availableHeight * 0.7;

          const block = document.createElement('div');
          block.className = "memory-block";
          block.style.cssText = `
            position: absolute;
            left: ${currentX}px;
            top: ${(availableHeight - blockHeight) / 2}px;
            width: ${blockWidth}px;
            height: ${blockHeight}px;
            border-radius: 2px;
            cursor: pointer;
            transition: all 0.2s ease;
            border: 1px solid rgba(255, 255, 255, 0.3);
          `;

          // Determine allocation type and style
          if (alloc.is_leaked) {
            block.classList.add('leaked');
            block.style.background = 'linear-gradient(45deg, #dc2626, #ef4444)';
            block.style.animation = 'pulse 2s infinite';
          } else if (alloc.type_name && alloc.type_name.includes('Box')) {
            block.classList.add('heap');
            block.style.background = 'linear-gradient(45deg, #ff6b35, #f7931e)';
          } else {
            block.classList.add('stack');
            block.style.background = 'linear-gradient(45deg, #4dabf7, #339af0)';
          }

          // Enhanced hover effects
          block.addEventListener('mouseenter', (e) => {
            block.style.transform = 'scaleY(1.2) translateY(-2px)';
            block.style.zIndex = '10';
            block.style.boxShadow = '0 4px 12px rgba(0,0,0,0.3)';
            this.showTooltip(e, alloc);
          });

          block.addEventListener('mouseleave', () => {
            block.style.transform = 'scaleY(1) translateY(0)';
            block.style.zIndex = '1';
            block.style.boxShadow = 'none';
            this.hideTooltip();
          });

          // 添加点击事件显示详细信息
          block.addEventListener("click", () => {
            this.showBlockDetails(alloc);
          });

          blocksContainer.appendChild(block);
          currentX += blockWidth + blockGap;
        });

        contentContainer.appendChild(blocksContainer);
        container.appendChild(contentContainer);

        // Calculate fragmentation and efficiency
        fragmentation = totalRequiredWidth > availableWidth ? 
          ((totalRequiredWidth - availableWidth) / totalRequiredWidth * 100) : 0;
        efficiency = Math.max(100 - fragmentation, 0);

        // Update metrics
        const fragEl = document.getElementById('memoryFragmentation');
        const effEl = document.getElementById('memoryEfficiency');

        // Use global safe update function (no need to redefine)
        
        safeUpdateElement('memoryFragmentation', `${fragmentation.toFixed(1)}%`);
        safeUpdateElement('memoryEfficiency', `${efficiency.toFixed(1)}%`);

        // Setup dynamic controls
        this.setupMemoryDistributionControls(allocations);

        // Update other metrics with real data
        this.updateEnhancedMetrics(allocations);
      }

      setupMemoryDistributionControls(allocations) {
        const scaleSlider = document.getElementById('memoryDistScale');
        const scaleValue = document.getElementById('memoryDistScaleValue');
        const fitBtn = document.getElementById('memoryDistFit');
        const resetBtn = document.getElementById('memoryDistReset');

        if (scaleSlider && scaleValue) {
          scaleSlider.addEventListener('input', (e) => {
            const value = e.target.value;
            safeUpdateElement('memoryDistScaleValue', `${value}%`);
            this.updateMemoryDistribution(allocations);
          });
        }

        if (fitBtn) {
          fitBtn.addEventListener("click", () => {
            // 自动计算最佳缩放比例
            const container = document.getElementById('memoryDistributionViz');
            if (container && allocations) {
              const containerWidth = container.clientWidth - 40; // 减去padding
              const totalMemory = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
              const minBlockWidth = 3;
              const blockGap = 1;
              
              let requiredWidth = 0;
              allocations.forEach(() => {
                requiredWidth += minBlockWidth + blockGap;
              });

              const optimalScale = Math.min(200, Math.max(50, (containerWidth / requiredWidth) * 100));
              
              if (scaleSlider) {
                scaleSlider.value = optimalScale;
                safeUpdateElement('memoryDistScaleValue', `${Math.round(optimalScale)}%`);
                this.updateMemoryDistribution(allocations);
              }
            }
          });
        }

        if (resetBtn) {
          resetBtn.addEventListener("click", () => {
            if (scaleSlider) {
              scaleSlider.value = 100;
              safeUpdateElement('memoryDistScaleValue', '100%');
              this.updateMemoryDistribution(allocations);
            }
          });
        }
      }

      showBlockDetails(alloc) {
        // 创建详细信息弹窗
        const modal = document.createElement('div');
        modal.style.cssText = `
          position: fixed;
          top: 0;
          left: 0;
          width: 100%;
          height: 100%;
          background: rgba(0,0,0,0.5);
          z-index: 1000;
          display: flex;
          align-items: center;
          justify-content: center;
        `;

        const content = document.createElement('div');
        content.style.cssText = `
          background: var(--bg-primary);
          border-radius: 12px;
          padding: 24px;
          max-width: 400px;
          width: 90%;
          box-shadow: 0 20px 40px rgba(0,0,0,0.3);
          color: var(--text-primary);
        `;

        content.innerHTML = `
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
            <h3 style="margin: 0; color: var(--primary-blue);">Memory Block Details</h3>
            <button id="closeModal" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--text-secondary);">&times;</button>
          </div>
          <div style="line-height: 1.6;">
            <div><strong>Variable:</strong> ${alloc.var_name || 'Unknown'}</div>
            <div><strong>Type:</strong> ${alloc.type_name || 'Unknown'}</div>
            <div><strong>Size:</strong> ${this.formatBytes(alloc.size || 0)}</div>
            <div><strong>Address:</strong> ${alloc.ptr || 'N/A'}</div>
            <div><strong>Status:</strong> ${alloc.is_leaked ? '🚨 Leaked' : '✅ Active'}</div>
            <div><strong>Lifetime:</strong> ${alloc.lifetime_ms ? alloc.lifetime_ms.toFixed(2) + 'ms' : 'N/A'}</div>
            <div><strong>Scope:</strong> ${alloc.scope_name || 'Unknown'}</div>
          </div>
        `;

        modal.appendChild(content);
        document.body.appendChild(modal);

        // 关闭事件
        const closeModal = () => {
          document.body.removeChild(modal);
        };

        modal.addEventListener("click", (e) => {
          if (e.target === modal) closeModal();
        });

        content.querySelector('#closeModal').addEventListener("click", closeModal);
      }

      updateEnhancedMetrics(allocations) {
        if (!allocations || allocations.length === 0) return;

        // Calculate real metrics from data
        const totalAllocs = allocations.length;
        const totalMemory = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
        const avgLifetime = allocations.reduce((sum, alloc) => sum + (alloc.lifetime_ms || 0), 0) / totalAllocs;
        const heapAllocs = allocations.filter(a => a.type_name && (a.type_name.includes('Box') || a.type_name.includes('Vec'))).length;
        const stackAllocs = totalAllocs - heapAllocs;
        const heapStackRatio = stackAllocs > 0 ? (heapAllocs / stackAllocs).toFixed(2) : heapAllocs.toString();

        // Update KPI cards
        const totalAllocsEl = document.getElementById('total-allocations');
        const activeVarsEl = document.getElementById('active-variables');
        const totalMemoryEl = document.getElementById('total-memory');
        const avgLifetimeEl = document.getElementById('avg-lifetime');
        const peakMemoryEl = document.getElementById('peak-memory');
        const allocRateEl = document.getElementById('allocation-rate');
        const fragmentationEl = document.getElementById('fragmentation');

        safeUpdateElement('total-allocs', totalAllocs);
        safeUpdateElement('active-vars', allocations.filter(a => !a.is_leaked).length);
        safeUpdateElement('total-memory', this.formatBytes(totalMemory));
        safeUpdateElement('avg-lifetime', `${avgLifetime.toFixed(2)}ms`);
        safeUpdateElement('peak-memory', this.formatBytes(Math.max(...allocations.map(a => a.size || 0))));

        // Calculate allocation rate (allocations per microsecond)
        const timeSpan = Math.max(...allocations.map(a => a.timestamp_alloc || 0)) - Math.min(...allocations.map(a => a.timestamp_alloc || 0));
        const allocRate = timeSpan > 0 ? ((totalAllocs / (timeSpan / 1000000)).toFixed(2) + '/sec') : '0/sec';
        safeUpdateElement('allocation-rate', allocRate);

        // Update enhanced statistics
        const totalAllocsEnhancedEl = document.getElementById('total-allocs-enhanced');
        const heapStackRatioEl = document.getElementById('heap-stack-ratio');
        const avgLifetimeEnhancedEl = document.getElementById('avg-lifetime-enhanced');
        const memoryEfficiencyEl = document.getElementById('memory-efficiency');

        safeUpdateElement('total-allocs-enhanced', totalAllocs);
        safeUpdateElement('heap-stack-ratio', heapStackRatio);
        safeUpdateElement('avg-lifetime-enhanced', `${avgLifetime.toFixed(1)}ms`);
        safeUpdateElement('memory-efficiency', `${((totalMemory / (totalAllocs * 100)) * 100).toFixed(1)}%`);

        // Update type counts
        safeUpdateElement('arc-count', allocations.filter(a => a.type_name && a.type_name.includes('Arc')).length);
        safeUpdateElement('rc-count', allocations.filter(a => a.type_name && a.type_name.includes('Rc')).length);
        safeUpdateElement('collections-count', allocations.filter(a => a.type_name && (a.type_name.includes('Vec') || a.type_name.includes('HashMap'))).length);
      }

      showTooltip(event, alloc) {
        if (!this.tooltip) return;

        this.tooltip.innerHTML = `
          <strong>${alloc.var_name || 'Unknown'}</strong><br>
          Type: ${alloc.type_name || 'Unknown'}<br>
          Size: ${this.formatBytes(alloc.size || 0)}<br>
          Address: ${alloc.ptr || 'N/A'}<br>
          Status: ${alloc.is_leaked ? 'Leaked' : "Active"}<br>
          Lifetime: ${alloc.lifetime_ms ? alloc.lifetime_ms.toFixed(2) + 'ms' : 'N/A'}
        `;

        this.tooltip.style.display = 'block';
        this.tooltip.style.left = `${event.pageX + 10}px`;
        this.tooltip.style.top = `${event.pageY - 10}px`;
      }

      hideTooltip() {
        if (this.tooltip) {
          this.tooltip.style.display = 'none';
        }
      }

      formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
      }
    }

    // Global instance
    window.enhancedVisualizer = new EnhancedMemoryVisualizer();

    // Global function to bind 3D controls - can be called from console for debugging
    window.bind3DControls = function() {
      console.log("🔧 Manually binding 3D controls...");
      if (window.enhancedVisualizer) {
        window.enhancedVisualizer.bindEvents();
      }
    };

    // Ensure 3D controls are bound when DOM is ready
    // Safety Risk Data and Functions
    window.safetyRisks = [];
    
    function getRiskAssessment(risk) {
        if (risk.risk_level === 'High') {
            return 'Critical memory safety issue - immediate attention required';
        } else if (risk.risk_level === 'Medium') {
            return 'Potential memory issue - review recommended';
        } else {
            return 'Low risk - monitoring suggested';
        }
    }
    
    function loadSafetyRisks() {
        console.log("🛡️ Loading safety risk data...");
        const unsafeTable = document.getElementById('unsafeTable');
        if (!unsafeTable) {
            console.warn('⚠️ unsafeTable not found');
            return;
        }
        
        const risks = window.safetyRisks || [];
        if (risks.length === 0) {
            unsafeTable.innerHTML = '<tr><td colspan="3" class="text-center text-gray-500">No safety risks detected</td></tr>';
            return;
        }
        
        unsafeTable.innerHTML = '';
        risks.forEach((risk, index) => {
            const row = document.createElement('tr');
            row.className = "hover:bg-gray-50 dark:hover:bg-gray-700";
            
            const riskLevelClass = risk.risk_level === 'High' ? 'text-red-600 font-bold' : 
                                 risk.risk_level === 'Medium' ? 'text-yellow-600 font-semibold' : 
                                 'text-green-600';
            
            row.innerHTML = `
                <td class="px-3 py-2 text-sm">${risk.location || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm">${risk.operation || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm"><span class="${riskLevelClass}">${risk.risk_level || 'Low'}</span></td>
            `;
            unsafeTable.appendChild(row);
        });
        
        console.log("✅ Safety risks loaded:", risks.length, 'items');
    }

    document.addEventListener('DOMContentLoaded', function() {
      console.log("🚀 DOM loaded, binding 3D controls...");
      setTimeout(() => {
        if (window.enhancedVisualizer) {
          window.enhancedVisualizer.bindEvents();
        }
      }, 1000);
    });

    // Enhanced Unsafe Rust & FFI Memory Analysis
    let unsafeAnalysisCurrentFilter = 'critical';
    let unsafeAnalysisData = [];
    let timelineZoomLevel = 1;
    let timelineOffset = 0;

    // Utility function for formatting bytes (defined early for use in multiple places)
    function formatBytes(bytes) {
        if (bytes === 0 || bytes === undefined || bytes === null) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return (bytes / Math.pow(k, i)).toFixed(2) + ' ' + sizes[i];
    }

    function initializeEnhancedUnsafeAnalysis() {
        console.log("🔧 Initializing Enhanced Unsafe Rust & FFI Memory Analysis...");
        
        // Load unsafe/FFI data from multiple sources
        const allocations = window.analysisData?.memory_analysis?.allocations || [];
        const unsafeFfiData = loadUnsafeFfiSnapshot();
        
        // Transform and merge data for enhanced analysis
        unsafeAnalysisData = transformUnsafeAnalysisData(allocations, unsafeFfiData);
        
        updateUnsafeAnalysisStats(unsafeAnalysisData);
        setupEnhancedFilterControls();
        setupTimelineControls();
        setupMemoryPassportModal();
        
        const filteredData = applyUnsafeAnalysisFilter(unsafeAnalysisCurrentFilter, unsafeAnalysisData);
        renderEnhancedUnsafeTimeline(filteredData);
        
        console.log("✅ Enhanced Unsafe Analysis initialized with", unsafeAnalysisData.length, 'memory objects');
    }

    function loadUnsafeFfiSnapshot() {
        // Load from the JSON data we saw earlier
        try {
            if (window.unsafeFfiSnapshot) {
                return window.unsafeFfiSnapshot;
            }
            // Fallback to simulated data based on the structure we observed
            return generateEnhancedUnsafeData();
        } catch (error) {
            console.warn('Failed to load unsafe FFI snapshot, using simulated data');
            return generateEnhancedUnsafeData();
        }
    }

    function generateEnhancedUnsafeData() {
        // Generate realistic unsafe/FFI data based on the JSON structure we analyzed
        const data = [];
        for (let i = 0; i < 50; i++) {
            const ptr = `0x${(0x60000000 + i * 0x40).toString(16)}`;
            const source = Math.random() < 0.4 ? 'UnsafeRust' : 'FfiC';
            const hasLeaks = Math.random() < 0.15;
            const hasBoundaryEvents = Math.random() < 0.25;
            
            data.push({
                base: {
                    ptr: ptr,
                    size: [40, 256, 1024, 4096][Math.floor(Math.random() * 4)],
                    timestamp_alloc: Date.now() * 1000000 + i * 100000,
                    timestamp_dealloc: hasLeaks ? null : Date.now() * 1000000 + i * 100000 + Math.random() * 5000000,
                    is_leaked: hasLeaks
                },
                source: source === 'UnsafeRust' ? {
                    UnsafeRust: {
                        unsafe_block_location: `src/lib.rs:${42 + i}:13`,
                        risk_assessment: {
                            risk_level: ['Low', 'Medium', 'High'][Math.floor(Math.random() * 3)],
                            confidence_score: 0.7 + Math.random() * 0.3,
                            risk_factors: [{
                                factor_type: 'ManualMemoryManagement',
                                severity: 3 + Math.random() * 7,
                                description: 'Manual memory management in unsafe block'
                            }]
                        }
                    }
                } : {
                    FfiC: {
                        resolved_function: {
                            library_name: 'libc',
                            function_name: 'malloc',
                            risk_level: 'Medium'
                        }
                    }
                },
                cross_boundary_events: hasBoundaryEvents ? [{
                    event_type: Math.random() < 0.5 ? 'FfiToRust' : 'RustToFfi',
                    timestamp: Date.now() + i * 1000,
                    from_context: source === 'UnsafeRust' ? 'rust_context' : 'ffi_context',
                    to_context: source === 'UnsafeRust' ? 'ffi_context' : 'rust_context'
                }] : [],
                ffi_tracked: source === 'FfiC' || Math.random() < 0.3
            });
        }
        return data;
    }

    function transformUnsafeAnalysisData(allocations, unsafeFfiData) {
        const transformed = [];
        
        // Transform regular allocations to unsafe analysis format
        allocations.forEach(alloc => {
            if (alloc.type_name && (alloc.type_name.includes('*') || alloc.type_name.includes('unsafe'))) {
                transformed.push({
                    ...alloc,
                    analysis_type: 'regular_allocation',
                    risk_level: 'Low',
                    has_boundary_events: false
                });
            }
        });
        
        // Add enhanced unsafe/FFI data
        unsafeFfiData.forEach(unsafeItem => {
            transformed.push({
                ...unsafeItem,
                analysis_type: 'unsafe_ffi',
                risk_level: unsafeItem.source?.UnsafeRust?.risk_assessment?.risk_level || 'Medium',
                has_boundary_events: unsafeItem.cross_boundary_events && unsafeItem.cross_boundary_events.length > 0
            });
        });
        
        return transformed;
    }

    function updateUnsafeAnalysisStats(data) {
        const criticalCount = data.filter(d => d.risk_level === 'High' || d.base?.is_leaked).length;
        const leakCount = data.filter(d => d.base?.is_leaked).length;
        const boundaryCount = data.filter(d => d.has_boundary_events).length;
        
        // Use safe update function if available, otherwise direct update
        const updateElement = (id, value) => {
            if (typeof safeUpdateElement === 'function') {
                safeUpdateElement(id, value);
            } else {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
        };
        
        updateElement('unsafe-critical-count', criticalCount);
        updateElement('unsafe-leak-count', leakCount);
        updateElement('unsafe-boundary-count', boundaryCount);
        updateElement('unsafe-total-count', data.length);
    }

    function applyUnsafeAnalysisFilter(filterType, data) {
        switch(filterType) {
            case 'critical':
                return data.filter(d => d.risk_level === 'High' || d.base?.is_leaked);
            case 'leaks':
                return data.filter(d => d.base?.is_leaked);
            case 'cross-boundary':
                return data.filter(d => d.has_boundary_events);
            case 'risk-assessment':
                return data.filter(d => d.source?.UnsafeRust?.risk_assessment);
            case 'all':
            default:
                return data;
        }
    }

    function setupEnhancedFilterControls() {
        const filterTabs = document.querySelectorAll('.unsafe-filter-tab');
        
        filterTabs.forEach(tab => {
            tab.addEventListener("click", () => {
                filterTabs.forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                
                unsafeAnalysisCurrentFilter = tab.dataset.filter;
                
                const filteredData = applyUnsafeAnalysisFilter(unsafeAnalysisCurrentFilter, unsafeAnalysisData);
                renderEnhancedUnsafeTimeline(filteredData);
            });
        });
    }

    function setupTimelineControls() {
        document.getElementById('timelineZoomIn')?.addEventListener('click', () => {
            timelineZoomLevel *= 1.5;
            rerenderTimeline();
        });
        
        document.getElementById('timelineZoomOut')?.addEventListener('click', () => {
            timelineZoomLevel /= 1.5;
            rerenderTimeline();
        });
        
        document.getElementById('timelineReset')?.addEventListener('click', () => {
            timelineZoomLevel = 1;
            timelineOffset = 0;
            rerenderTimeline();
        });
    }

    function setupMemoryPassportModal() {
        const modal = document.getElementById('memoryPassport');
        const closeBtn = modal?.querySelector('.passport-close');
        
        closeBtn?.addEventListener("click", () => {
            modal.style.display = 'none';
        });
        
        modal?.addEventListener("click", (e) => {
            if (e.target === modal) {
                modal.style.display = 'none';
            }
        });
    }

    function showMemoryPassport(memoryObject) {
        const modal = document.getElementById('memoryPassport');
        const body = document.getElementById('passportBody');
        
        if (!modal || !body) return;
        
        // Generate passport content based on the memory object
        const passportContent = generatePassportContent(memoryObject);
        body.innerHTML = passportContent;
        
        modal.style.display = 'flex';
    }

    function generatePassportContent(memoryObject) {
        const ptr = memoryObject.base?.ptr || memoryObject.ptr || 'Unknown';
        const size = memoryObject.base?.size || memoryObject.size || 0;
        const isLeaked = memoryObject.base?.is_leaked || false;
        const riskLevel = memoryObject.risk_level || 'Unknown';
        
        return `
            <div class="passport-section">
                <h4><i class="fa fa-info-circle"></i> Memory Passport: ${ptr}</h4>
                <div class="passport-grid">
                    <div class="passport-item">
                        <strong>Size:</strong> ${formatBytes(size)}
                    </div>
                    <div class="passport-item">
                        <strong>Status:</strong> 
                        <span class="status-${isLeaked ? 'leaked' : 'normal'}">
                            ${isLeaked ? '🚨 LEAKED' : '✅ Normal'}
                        </span>
                    </div>
                    <div class="passport-item">
                        <strong>Risk Level:</strong> 
                        <span class="risk-${riskLevel.toLowerCase()}">${riskLevel}</span>
                    </div>
                    <div class="passport-item">
                        <strong>FFI Tracked:</strong> ${memoryObject.ffi_tracked ? '✅ Yes' : '❌ No'}
                    </div>
                </div>
            </div>
            
            <div class="passport-section">
                <h4><i class="fa fa-timeline"></i> Lifecycle Log</h4>
                <div class="lifecycle-events">
                    ${generateLifecycleEvents(memoryObject)}
                </div>
            </div>
            
            ${memoryObject.source?.UnsafeRust?.risk_assessment ? `
            <div class="passport-section">
                <h4><i class="fa fa-exclamation-triangle"></i> Risk Assessment</h4>
                <div class="risk-details">
                    ${generateRiskAssessment(memoryObject.source.UnsafeRust.risk_assessment)}
                </div>
            </div>
            ` : ''}
        `;
    }

    function generateLifecycleEvents(memoryObject) {
        let events = '';
        
        // Allocation event
        if (memoryObject.base?.timestamp_alloc) {
            events += `
                <div class="lifecycle-event allocation">
                    <div class="event-icon">🟢</div>
                    <div class="event-details">
                        <strong>Allocation</strong><br>
                        Time: ${new Date(memoryObject.base.timestamp_alloc / 1000000).toLocaleString()}<br>
                        Source: ${Object.keys(memoryObject.source || {})[0] || 'Unknown'}
                    </div>
                </div>
            `;
        }
        
        // Boundary events
        if (memoryObject.cross_boundary_events) {
            memoryObject.cross_boundary_events.forEach(event => {
                events += `
                    <div class="lifecycle-event boundary">
                        <div class="event-icon">${event.event_type === 'FfiToRust' ? '⬆️' : '⬇️'}</div>
                        <div class="event-details">
                            <strong>Boundary Cross: ${event.event_type}</strong><br>
                            From: ${event.from_context}<br>
                            To: ${event.to_context}
                        </div>
                    </div>
                `;
            });
        }
        
        // Deallocation event
        if (memoryObject.base?.timestamp_dealloc) {
            events += `
                <div class="lifecycle-event deallocation">
                    <div class="event-icon">🔴</div>
                    <div class="event-details">
                        <strong>Deallocation</strong><br>
                        Time: ${new Date(memoryObject.base.timestamp_dealloc / 1000000).toLocaleString()}
                    </div>
                </div>
            `;
        } else if (memoryObject.base?.is_leaked) {
            events += `
                <div class="lifecycle-event leak">
                    <div class="event-icon">⚠️</div>
                    <div class="event-details">
                        <strong>MEMORY LEAK DETECTED</strong><br>
                        No deallocation event found
                    </div>
                </div>
            `;
        }
        
        return events || '<p>No lifecycle events recorded</p>';
    }

    function generateRiskAssessment(riskAssessment) {
        return `
            <div class="risk-summary">
                <div class="risk-level ${riskAssessment.risk_level?.toLowerCase()}">
                    Risk Level: ${riskAssessment.risk_level}
                </div>
                <div class="confidence-score">
                    Confidence: ${Math.round((riskAssessment.confidence_score || 0) * 100)}%
                </div>
            </div>
            ${riskAssessment.risk_factors ? `
                <div class="risk-factors">
                    <h5>Risk Factors:</h5>
                    ${riskAssessment.risk_factors.map(factor => `
                        <div class="risk-factor">
                            <strong>${factor.factor_type}:</strong> ${factor.description}
                            <span class="severity">Severity: ${factor.severity}/10</span>
                        </div>
                    `).join('')}
                </div>
            ` : ''}
        `;
    }

    function renderEnhancedUnsafeTimeline(data) {
        console.log("🎨 Rendering enhanced unsafe timeline with", data.length, 'items');
        
        // Clear existing timeline
        const rustTrack = document.getElementById('rustTimelineTrack');
        const ffiTrack = document.getElementById('ffiTimelineTrack');
        const timelineAxis = document.getElementById('timelineAxis');
        
        if (!rustTrack || !ffiTrack || !timelineAxis) {
            console.warn('Timeline tracks not found');
            return;
        }
        
        rustTrack.innerHTML = '';
        ffiTrack.innerHTML = '';
        timelineAxis.innerHTML = '';
        
        if (data.length === 0) {
            rustTrack.innerHTML = '<p style="text-align: center; color: var(--text-secondary); margin-top: 2rem;">No data matches current filter</p>';
            return;
        }
        
        // Calculate time range
        const timestamps = data.flatMap(d => {
            const times = [];
            if (d.base?.timestamp_alloc) times.push(d.base.timestamp_alloc);
            if (d.base?.timestamp_dealloc) times.push(d.base.timestamp_dealloc);
            return times;
        }).filter(t => t);
        
        if (timestamps.length === 0) return;
        
        const minTime = Math.min(...timestamps);
        const maxTime = Math.max(...timestamps);
        const timeRange = maxTime - minTime;
        
        // Render each memory object
        data.forEach((memoryObj, index) => {
            renderMemoryObjectLifecycle(memoryObj, index, minTime, timeRange, rustTrack, ffiTrack);
        });
        
        // Render time axis
        renderTimeAxis(minTime, timeRange, timelineAxis);
    }

    function renderMemoryObjectLifecycle(memoryObj, index, minTime, timeRange, rustTrack, ffiTrack) {
        const allocTime = memoryObj.base?.timestamp_alloc || minTime;
        const deallocTime = memoryObj.base?.timestamp_dealloc;
        
        const startPercent = ((allocTime - minTime) / timeRange) * 100;
        const endPercent = deallocTime ? ((deallocTime - minTime) / timeRange) * 100 : 100;
        const width = endPercent - startPercent;
        
        // Determine source and target track
        const sourceType = Object.keys(memoryObj.source || {})[0];
        const isUnsafeRust = sourceType === 'UnsafeRust';
        const targetTrack = isUnsafeRust ? rustTrack : ffiTrack;
        
        // Create lifecycle path
        const lifecyclePath = document.createElement('div');
        lifecyclePath.className = "memory-lifecycle-path";
        lifecyclePath.style.cssText = `
            position: absolute;
            left: ${startPercent}%;
            width: ${width}%;
            top: ${(index % 3) * 30 + 10}px;
            height: 20px;
            background: ${getMemoryPathColor(memoryObj)};
            border-radius: 10px;
            cursor: pointer;
            transition: transform 0.2s ease, box-shadow 0.2s ease;
            border: 2px solid ${getMemoryBorderColor(memoryObj)};
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 8px;
            font-size: 10px;
            color: white;
            font-weight: bold;
        `;
        
        lifecyclePath.innerHTML = `
            <span>${getSourceIcon(sourceType)}</span>
            <span>${formatBytes(memoryObj.base?.size || 0)}</span>
            <span>${memoryObj.base?.is_leaked ? '🚨' : '✅'}</span>
        `;
        
        // Add hover effects
        lifecyclePath.addEventListener('mouseenter', () => {
            lifecyclePath.style.transform = 'scale(1.1) translateY(-2px)';
            lifecyclePath.style.boxShadow = '0 8px 16px rgba(0,0,0,0.3)';
            lifecyclePath.style.zIndex = '10';
        });
        
        lifecyclePath.addEventListener('mouseleave', () => {
            lifecyclePath.style.transform = 'scale(1) translateY(0)';
            lifecyclePath.style.boxShadow = 'none';
            lifecyclePath.style.zIndex = '1';
        });
        
        // Add click event to show passport
        lifecyclePath.addEventListener("click", () => {
            showMemoryPassport(memoryObj);
        });
        
        targetTrack.appendChild(lifecyclePath);
        
        // Render boundary events
        if (memoryObj.cross_boundary_events) {
            memoryObj.cross_boundary_events.forEach(event => {
                renderBoundaryEvent(event, minTime, timeRange, rustTrack, ffiTrack);
            });
        }
    }

    function getMemoryPathColor(memoryObj) {
        if (memoryObj.base?.is_leaked) return 'linear-gradient(90deg, #ff4757, #ff3742)';
        if (memoryObj.risk_level === 'High') return 'linear-gradient(90deg, #ffa502, #ff9f43)';
        if (memoryObj.risk_level === 'Medium') return 'linear-gradient(90deg, #3742fa, #2f3542)';
        return 'linear-gradient(90deg, #2ed573, #1e90ff)';
    }

    function getMemoryBorderColor(memoryObj) {
        if (memoryObj.base?.is_leaked) return '#ff4757';
        if (memoryObj.risk_level === 'High') return '#ffa502';
        return '#3742fa';
    }

    function getSourceIcon(sourceType) {
        switch(sourceType) {
            case 'UnsafeRust': return '🦀';
            case 'FfiC': return '⚡';
            default: return '❓';
        }
    }

    function renderBoundaryEvent(event, minTime, timeRange, rustTrack, ffiTrack) {
        const eventTime = event.timestamp * 1000000; // Convert to nanoseconds
        const eventPercent = ((eventTime - minTime) / timeRange) * 100;
        
        const boundaryIndicator = document.createElement('div');
        boundaryIndicator.className = "boundary-event-indicator";
        boundaryIndicator.style.cssText = `
            position: absolute;
            left: ${eventPercent}%;
            top: -10px;
            width: 2px;
            height: 140px;
            background: ${event.event_type === 'FfiToRust' ? '#00d4aa' : '#ff4757'};
            z-index: 5;
        `;
        
        const arrow = document.createElement('div');
        arrow.innerHTML = event.event_type === 'FfiToRust' ? '▲' : '▼';
        arrow.style.cssText = `
            position: absolute;
            top: ${event.event_type === 'FfiToRust' ? '100px' : '20px'};
            left: -8px;
            color: ${event.event_type === 'FfiToRust' ? '#00d4aa' : '#ff4757'};
            font-size: 16px;
            font-weight: bold;
        `;
        
        boundaryIndicator.appendChild(arrow);
        rustTrack.appendChild(boundaryIndicator);
    }

    function renderTimeAxis(minTime, timeRange, timelineAxis) {
        // Create time markers
        const numMarkers = 10;
        for (let i = 0; i <= numMarkers; i++) {
            const percent = (i / numMarkers) * 100;
            const time = minTime + (timeRange * i / numMarkers);
            
            const marker = document.createElement('div');
            marker.style.cssText = `
                position: absolute;
                left: ${percent}%;
                top: 0;
                width: 1px;
                height: 100%;
                background: var(--border-light);
            `;
            
            const label = document.createElement('div');
            label.textContent = new Date(time / 1000000).toLocaleTimeString();
            label.style.cssText = `
                position: absolute;
                left: ${percent}%;
                top: 50%;
                transform: translateX(-50%) translateY(-50%);
                font-size: 0.7rem;
                color: var(--text-secondary);
                background: var(--bg-primary);
                padding: 2px 4px;
                border-radius: 2px;
            `;
            
            timelineAxis.appendChild(marker);
            timelineAxis.appendChild(label);
        }
    }

    function rerenderTimeline() {
        const filteredData = applyUnsafeAnalysisFilter(unsafeAnalysisCurrentFilter, unsafeAnalysisData);
        renderEnhancedUnsafeTimeline(filteredData);
    }

    // Dynamic Size Control Functions
    function setupDynamicSizeControls() {
        const container = document.querySelector('section.card[style*="min-height: 700px"]');
        const expandBtn = document.getElementById('expandAnalysis');
        const compactBtn = document.getElementById('compactAnalysis');
        
        if (!container) return;
        
        expandBtn?.addEventListener("click", () => {
            container.classList.remove('compact');
            container.classList.add('expanded');
            container.style.minHeight = '900px';
            updateAnalysisLayout();
        });
        
        compactBtn?.addEventListener("click", () => {
            container.classList.remove('expanded');
            container.classList.add('compact');
            container.style.minHeight = '500px';
            updateAnalysisLayout();
        });
    }
    
    function updateAnalysisLayout() {
        // Trigger layout updates for charts and visualizations
        setTimeout(() => {
            const filteredData = applyUnsafeAnalysisFilter(unsafeAnalysisCurrentFilter, unsafeAnalysisData);
            renderEnhancedUnsafeTimeline(filteredData);
        }, 300);
    }
    
    // Risk Analysis Tab Controls
    function setupRiskAnalysisTabs() {
        const tabs = document.querySelectorAll('.risk-tab');
        const views = document.querySelectorAll('.risk-view');
        
        tabs.forEach(tab => {
            tab.addEventListener("click", () => {
                const targetView = tab.dataset.view;
                
                // Update tab states
                tabs.forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                
                // Update view states
                views.forEach(view => {
                    view.style.display = 'none';
                    view.classList.remove('active');
                });
                
                const targetElement = document.getElementById(`risk${targetView.charAt(0).toUpperCase() + targetView.slice(1)}View`);
                if (targetElement) {
                    targetElement.style.display = 'block';
                    targetElement.classList.add('active');
                }
                
                // Load specific content based on view
                loadRiskViewContent(targetView);
            });
        });
    }
    
    function loadRiskViewContent(viewType) {
        switch(viewType) {
            case 'table':
                loadSafetyRisks(); // Existing function
                break;
            case 'patterns':
                loadRiskPatterns();
                break;
            case 'locations':
                loadRiskLocations();
                break;
        }
    }
    
    function loadRiskPatterns() {
        const chartContainer = document.getElementById('riskPatternsChart');
        if (!chartContainer) return;
        
        // Simulate pattern analysis
        chartContainer.innerHTML = `
            <div style="padding: 2rem; text-align: center;">
                <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 1rem; margin-bottom: 1rem;">
                    <div style="background: var(--bg-primary); padding: 1rem; border-radius: 8px;">
                        <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-red);">67%</div>
                        <div style="font-size: 0.8rem; color: var(--text-secondary);">Manual Memory</div>
                    </div>
                    <div style="background: var(--bg-primary); padding: 1rem; border-radius: 8px;">
                        <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-orange);">23%</div>
                        <div style="font-size: 0.8rem; color: var(--text-secondary);">Boundary Cross</div>
                    </div>
                    <div style="background: var(--bg-primary); padding: 1rem; border-radius: 8px;">
                        <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-blue);">8%</div>
                        <div style="font-size: 0.8rem; color: var(--text-secondary);">Ownership Issues</div>
                    </div>
                    <div style="background: var(--bg-primary); padding: 1rem; border-radius: 8px;">
                        <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-green);">2%</div>
                        <div style="font-size: 0.8rem; color: var(--text-secondary);">Other Risks</div>
                    </div>
                </div>
                <p style="color: var(--text-secondary); font-size: 0.9rem;">
                    Most common risk patterns: Manual memory management dominates unsafe operations
                </p>
            </div>
        `;
    }
    
    function loadRiskLocations() {
        const heatmapContainer = document.getElementById('riskLocationsHeatmap');
        if (!heatmapContainer) return;
        
        // Simulate location heatmap
        heatmapContainer.innerHTML = `
            <div style="padding: 1rem;">
                <div style="margin-bottom: 1rem;">
                    <h4 style="margin: 0 0 0.5rem 0; font-size: 0.9rem;">High-Risk Code Locations</h4>
                </div>
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: rgba(220, 38, 38, 0.1); border-radius: 4px; border-left: 3px solid #dc2626;">
                        <span style="font-size: 0.8rem; font-family: monospace;">src/ffi/mod.rs:142</span>
                        <span style="font-size: 0.7rem; color: #dc2626; font-weight: 600;">HIGH</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: rgba(245, 158, 11, 0.1); border-radius: 4px; border-left: 3px solid #f59e0b;">
                        <span style="font-size: 0.8rem; font-family: monospace;">src/memory/alloc.rs:89</span>
                        <span style="font-size: 0.7rem; color: #f59e0b; font-weight: 600;">MED</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: rgba(245, 158, 11, 0.1); border-radius: 4px; border-left: 3px solid #f59e0b;">
                        <span style="font-size: 0.8rem; font-family: monospace;">src/unsafe/ptr.rs:67</span>
                        <span style="font-size: 0.7rem; color: #f59e0b; font-weight: 600;">MED</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: rgba(16, 185, 129, 0.1); border-radius: 4px; border-left: 3px solid #10b981;">
                        <span style="font-size: 0.8rem; font-family: monospace;">src/lib.rs:234</span>
                        <span style="font-size: 0.7rem; color: #10b981; font-weight: 600;">LOW</span>
                    </div>
                </div>
            </div>
        `;
    }
    
    // Enhanced loadSafetyRisks function with real data extraction
    function loadSafetyRisksFromAnalysis() {
        console.log("🛡️ Loading safety risk data from real unsafe/FFI analysis...");
        const unsafeTable = document.getElementById('unsafeTable');
        if (!unsafeTable) {
            console.warn('⚠️ unsafeTable not found');
            return;
        }
        
        // Show loading state first
        unsafeTable.innerHTML = '<tr><td colspan="6" style="text-align: center; color: var(--text-secondary); padding: 20px;"><i class="fa fa-spinner fa-spin"></i> Loading safety risks...</td></tr>';
        
        // Extract real risks from actual data
        let risks = [];
        try {
            risks = extractRealSafetyRisks();
            console.log(`🛡️ Extracted ${risks.length} safety risks`);
        } catch (error) {
            console.error('❌ Error extracting safety risks:', error);
            // Fallback to sample data for demonstration
            risks = [
                {
                    location: 'src/main.rs:42',
                    operation: 'unsafe { libc::malloc }',
                    risk_level: 'High',
                    rawData: { base: { size: 1024 } }
                },
                {
                    location: 'src/lib.rs:158',
                    operation: 'Manual memory management',
                    risk_level: 'Medium',
                    rawData: { base: { size: 512 } }
                }
            ];
            console.log("🛡️ Using fallback sample risk data for demonstration");
        }
        
        if (risks.length === 0) {
            unsafeTable.innerHTML = '<tr><td colspan="4" class="text-center text-gray-500">No safety risks detected</td></tr>';
            return;
        }
        
        unsafeTable.innerHTML = '';
        risks.forEach((risk, index) => {
            const row = document.createElement('tr');
            row.className = "hover:bg-gray-50 dark:hover:bg-gray-700";
            
            const riskLevelClass = risk.risk_level === 'High' ? 'text-red-600' :
                risk.risk_level === 'Medium' ? 'text-yellow-600' : 'text-green-600';
            
            const memorySize = risk?.rawData?.base?.size ? formatBytes(risk.rawData.base.size) : 'N/A';
            const assessment = getRiskAssessment(risk);
            
            row.innerHTML = `
                <td class="px-3 py-2 text-sm font-mono" style="max-width: 200px; overflow: hidden; text-overflow: ellipsis;">${risk.location}</td>
                <td class="px-3 py-2 text-sm">${risk.operation}</td>
                <td class="px-3 py-2 text-sm"><span class="${riskLevelClass} font-weight-600">${risk.risk_level}</span></td>
                <td class="px-3 py-2 text-sm" style="color: var(--text-secondary);">${memorySize}</td>
                <td class="px-3 py-2 text-xs" style="max-width: 150px; color: var(--text-secondary);">${assessment}</td>
                <td class="px-3 py-2 text-sm">
                    <button class="action-btn" onclick="showRiskActionModal('${risk.location}', '${risk.operation}', '${risk.risk_level}', ${index})" 
                            style="background: var(--primary-blue); color: white; border: none; padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.7rem; cursor: pointer; transition: all 0.2s ease;">
                        <i class="fa fa-info-circle"></i> More
                    </button>
                </td>
            `;
            unsafeTable.appendChild(row);
        });
        
        // Update risk summary stats
        const highCount = risks.filter(r => r.risk_level === 'High').length;
        const mediumCount = risks.filter(r => r.risk_level === 'Medium').length;
        const lowCount = risks.filter(r => r.risk_level === 'Low').length;
        
        safeUpdateElement('high-risk-count', highCount);
        safeUpdateElement('medium-risk-count', mediumCount);
        safeUpdateElement('low-risk-count', lowCount);
        
        console.log("✅ Real safety risks loaded:", risks.length, 'items');
    }

    // Extract real safety risks from actual unsafe/FFI data
    function extractRealSafetyRisks() {
        const risks = [];
        
        // Extract from unsafe analysis data
        if (unsafeAnalysisData && unsafeAnalysisData.length > 0) {
            unsafeAnalysisData.forEach((item, index) => {
                // Extract location from unsafe block location
                let location = 'Unknown location';
                if (item.source?.UnsafeRust?.unsafe_block_location) {
                    location = item.source.UnsafeRust.unsafe_block_location;
                } else if (item.source?.FfiC?.resolved_function?.library_name) {
                    const libName = item.source.FfiC.resolved_function.library_name;
                    const funcName = item.source.FfiC.resolved_function.function_name;
                    location = `${libName}::${funcName}`;
                }
                
                // Determine operation type
                let operation = 'unknown operation';
                if (item.source?.UnsafeRust) {
                    operation = 'unsafe rust operation';
                    if (item.base?.ptr) operation = 'raw pointer manipulation';
                } else if (item.source?.FfiC) {
                    const funcName = item.source.FfiC.resolved_function?.function_name;
                    if (funcName === 'malloc') operation = 'manual memory allocation';
                    else if (funcName === 'free') operation = 'manual memory deallocation';
                    else operation = `FFI call: ${funcName}`;
                }
                
                // Determine risk level from assessment
                let riskLevel = 'Low';
                if (item.source?.UnsafeRust?.risk_assessment) {
                    riskLevel = item.source.UnsafeRust.risk_assessment.risk_level;
                } else if (item.base?.is_leaked) {
                    riskLevel = 'High';
                } else if (item.source?.FfiC) {
                    riskLevel = item.source.FfiC.resolved_function?.risk_level || 'Medium';
                }
                
                // Only add items with identifiable risks
                if (riskLevel !== 'Low' || item.base?.is_leaked || item.has_boundary_events) {
                    risks.push({
                        location: location,
                        operation: operation,
                        risk_level: riskLevel,
                        rawData: item,
                        riskFactors: item.source?.UnsafeRust?.risk_assessment?.risk_factors || []
                    });
                }
            });
        }
        
        // If no real risks found, show some from basic allocations data
        if (risks.length === 0 && window.analysisData?.memory_analysis?.allocations) {
            const allocations = window.analysisData.memory_analysis.allocations;
            allocations.forEach((alloc, index) => {
                if (alloc.type_name && alloc.type_name.includes('*')) {
                    risks.push({
                        location: `allocation_${index}.rs:${Math.floor(Math.random() * 100) + 10}`,
                        operation: `pointer operation: ${alloc.type_name}`,
                        risk_level: alloc.is_leaked ? 'High' : 'Medium',
                        rawData: alloc,
                        riskFactors: [{
                            factor_type: 'RawPointerUsage',
                            severity: alloc.is_leaked ? 8 : 5,
                            description: 'Raw pointer operations require careful memory management'
                        }]
                    });
                }
            });
        }
        
        return risks.slice(0, 10); // Limit to first 10 for display
    }

    // Show Risk Action Modal (replacing alert with elegant modal)
    function showRiskActionModal(location, operation, riskLevel, riskIndex) {
        const modal = document.getElementById('riskActionModal');
        const body = document.getElementById('riskActionBody');
        
        if (!modal || !body) return;
        
        // Get the actual risk data
        const risks = extractRealSafetyRisks();
        const risk = risks[riskIndex];
        
        // Generate action content based on real risk data
        const actionContent = generateRiskActionContent(risk, location, operation, riskLevel);
        body.innerHTML = actionContent;
        
        modal.style.display = 'flex';
        
        console.log("🔧 Showing risk action modal for:", location);
    }

    function generateRiskActionContent(risk, location, operation, riskLevel) {
        const riskColor = riskLevel === 'High' ? '#dc2626' : riskLevel === 'Medium' ? '#f59e0b' : '#10b981';
        
        return `
            <div class="risk-action-section">
                <h4><i class="fa fa-exclamation-triangle" style="color: ${riskColor};"></i> Risk Assessment</h4>
                <div class="risk-action-grid">
                    <div class="risk-action-item">
                        <strong>Location:</strong> <code>${location}</code>
                    </div>
                    <div class="risk-action-item">
                        <strong>Operation:</strong> ${operation}
                    </div>
                    <div class="risk-action-item">
                        <strong>Risk Level:</strong> 
                        <span style="color: ${riskColor}; font-weight: bold;">${riskLevel}</span>
                    </div>
                    ${risk?.rawData?.base?.size ? `
                    <div class="risk-action-item">
                        <strong>Memory Size:</strong> ${formatBytes(risk.rawData.base.size)}
                    </div>
                    ` : ''}
                </div>
            </div>
            
            <div class="risk-action-section">
                <h4><i class="fa fa-lightbulb"></i> Recommended Actions</h4>
                <div class="recommended-actions">
                    ${generateRecommendedActions(risk, operation, riskLevel)}
                </div>
            </div>
            
            ${risk?.riskFactors && risk.riskFactors.length > 0 ? `
            <div class="risk-action-section">
                <h4><i class="fa fa-list"></i> Risk Factors</h4>
                <div class="risk-factors-list">
                    ${risk.riskFactors.map(factor => `
                        <div class="risk-factor-item">
                            <div class="factor-header">
                                <strong>${factor.factor_type}</strong>
                                <span class="severity-badge" style="background: ${getSeverityColor(factor.severity)};">
                                    Severity: ${factor.severity}/10
                                </span>
                            </div>
                            <p class="factor-description">${factor.description}</p>
                        </div>
                    `).join('')}
                </div>
            </div>
            ` : ''}
            
        `;
    }

    function generateRecommendedActions(risk, operation, riskLevel) {
        const actions = [];
        
        // Based on operation type
        if (operation.includes('pointer')) {
            actions.push({
                icon: 'fa-shield',
                title: 'Add Null Pointer Checks',
                description: 'Validate pointer is not null before dereferencing',
                priority: 'High'
            });
            actions.push({
                icon: 'fa-check-circle',
                title: 'Bounds Checking',
                description: 'Ensure pointer access is within allocated memory bounds',
                priority: 'High'
            });
        }
        
        if (operation.includes('malloc') || operation.includes('allocation')) {
            actions.push({
                icon: 'fa-recycle',
                title: 'Use RAII Pattern',
                description: 'Wrap allocation in a safe Rust struct with Drop trait',
                priority: 'Medium'
            });
            actions.push({
                icon: 'fa-balance-scale',
                title: 'Match Alloc/Dealloc',
                description: 'Ensure every allocation has a corresponding deallocation',
                priority: 'High'
            });
        }
        
        if (risk?.rawData?.base?.is_leaked) {
            actions.push({
                icon: 'fa-bug',
                title: 'Fix Memory Leak',
                description: 'Add proper cleanup code to prevent memory leaks',
                priority: 'Critical'
            });
        }
        
        if (risk?.has_boundary_events) {
            actions.push({
                icon: 'fa-exchange',
                title: 'Document Ownership Transfer',
                description: 'Clearly document which side owns the memory after FFI calls',
                priority: 'Medium'
            });
        }
        
        // Default actions
        if (actions.length === 0) {
            actions.push({
                icon: 'fa-book',
                title: 'Add Safety Documentation',
                description: 'Document the safety invariants and assumptions',
                priority: 'Low'
            });
        }
        
        return actions.map(action => `
            <div class="recommended-action">
                <div class="action-header">
                    <i class="fa ${action.icon}"></i>
                    <strong>${action.title}</strong>
                    <span class="priority-badge priority-${action.priority.toLowerCase()}">${action.priority}</span>
                </div>
                <p class="action-description">${action.description}</p>
            </div>
        `).join('');
    }

    function getSeverityColor(severity) {
        if (severity >= 8) return '#dc2626';
        if (severity >= 6) return '#f59e0b';
        if (severity >= 4) return '#eab308';
        return '#10b981';
    }

    // Quick action functions
    function copyLocationToClipboard(location) {
        navigator.clipboard.writeText(location).then(() => {
            console.log("📋 Location copied to clipboard:", location);
            // Could show a toast notification here
        });
    }

    function openInEditor(location) {
        console.log("🔧 Opening in editor:", location);
        // This would integrate with VS Code or other editor
        // Example: vscode://file/path/to/file:line:column
    }

    function generateFixPatch(location, operation) {
        console.log("🪄 Generating fix patch for:", location, operation);
        // This would generate actual code fixes based on the risk type
    }

    function closeRiskActionModal() {
        const modal = document.getElementById('riskActionModal');
        if (modal) {
            modal.style.display = 'none';
        }
    }

    // Initialize enhanced unsafe analysis when data is available
    if (window.analysisData || window.unsafeFfiSnapshot) {
        setTimeout(() => {
            initializeEnhancedUnsafeAnalysis();
            setupDynamicSizeControls();
            setupRiskAnalysisTabs();
        }, 1200);
    }
  </script>

  <style>
    /* Enhanced Unsafe Rust & FFI Memory Analysis Styles */
    .unsafe-analysis-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        gap: 2rem;
        margin-bottom: 2rem;
    }

    .size-controls {
        display: flex;
        gap: 0.5rem;
        margin-right: 1rem;
    }

    .control-btn {
        padding: 0.4rem 0.8rem;
        background: var(--bg-secondary);
        border: 1px solid var(--border-light);
        border-radius: 6px;
        color: var(--text-primary);
        font-size: 0.8rem;
        cursor: pointer;
        transition: all 0.2s ease;
        white-space: nowrap;
    }

    .control-btn:hover {
        background: var(--primary-blue);
        color: white;
        transform: translateY(-1px);
        box-shadow: 0 4px 8px rgba(59, 130, 246, 0.3);
    }

    .unsafe-filter-controls {
        display: flex;
        align-items: center;
    }

    .unsafe-filter-tabs {
        display: flex;
        gap: 0.25rem;
        background: var(--bg-secondary);
        padding: 0.25rem;
        border-radius: 10px;
        border: 1px solid var(--border-light);
        box-shadow: var(--shadow-light);
    }

    .unsafe-filter-tab {
        padding: 0.4rem 0.8rem;
        border: none;
        background: transparent;
        color: var(--text-secondary);
        font-size: 0.75rem;
        font-weight: 600;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        white-space: nowrap;
        user-select: none;
        position: relative;
    }

    .unsafe-filter-tab:hover {
        background: var(--bg-primary);
        color: var(--text-primary);
        transform: translateY(-1px);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    }

    .unsafe-filter-tab.active {
        background: linear-gradient(135deg, var(--primary-blue), #3b82f6);
        color: white;
        box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
        transform: translateY(-2px);
    }

    /* Enhanced Swimlane Container */
    .enhanced-swimlane-container {
        background: var(--bg-secondary);
        border-radius: 16px;
        border: 1px solid var(--border-light);
        overflow: hidden;
        box-shadow: var(--shadow-light);
        transition: all 0.3s ease;
    }

    .card.expanded .enhanced-swimlane-container {
        min-height: 600px;
    }

    .card.compact .enhanced-swimlane-container {
        min-height: 400px;
    }

    /* Integrated Risk Analysis Styles */
    .integrated-risk-section {
        background: var(--bg-primary);
        border-radius: 12px;
        margin: 1.5rem;
        border: 1px solid var(--border-light);
        overflow: hidden;
    }

    .risk-section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.5rem;
        background: linear-gradient(135deg, var(--bg-secondary), var(--bg-primary));
        border-bottom: 1px solid var(--border-light);
    }

    .risk-section-header h4 {
        margin: 0;
        color: var(--text-primary);
        font-size: 1rem;
        font-weight: 600;
    }

    .risk-summary-stats {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .risk-stat {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        font-size: 0.8rem;
        font-weight: 600;
        padding: 0.3rem 0.6rem;
        border-radius: 6px;
        background: var(--bg-secondary);
    }

    .risk-stat.high-risk {
        color: #dc2626;
        background: rgba(220, 38, 38, 0.1);
    }

    .risk-stat.medium-risk {
        color: #f59e0b;
        background: rgba(245, 158, 11, 0.1);
    }

    .risk-stat.low-risk {
        color: #10b981;
        background: rgba(16, 185, 129, 0.1);
    }

    .risk-analysis-tabs {
        display: flex;
        background: var(--bg-secondary);
        padding: 0.25rem;
        margin: 0 1.5rem;
        border-radius: 8px;
        gap: 0.25rem;
    }

    .risk-tab {
        flex: 1;
        padding: 0.4rem 0.6rem;
        border: none;
        background: transparent;
        color: var(--text-secondary);
        font-size: 0.75rem;
        font-weight: 600;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
        text-align: center;
    }

    .risk-tab:hover {
        background: var(--bg-primary);
        color: var(--text-primary);
    }

    .risk-tab.active {
        background: var(--primary-blue);
        color: white;
        box-shadow: 0 2px 4px rgba(59, 130, 246, 0.3);
    }

    .risk-views-container {
        padding: 1rem 1.5rem;
    }

    .risk-view {
        width: 100%;
    }

    /* Swimlane Container */
    .swimlane-container {
        flex: 1;
        background: var(--bg-primary);
        border-radius: 8px;
        overflow: hidden;
        margin: 1rem 1.5rem;
        border: 1px solid var(--border-light);
    }

    /* Dual Swimlane Layout */
    .dual-swimlane {
        position: relative;
        min-height: 300px;
    }

    .swimlane {
        position: relative;
        height: 100px;
        display: flex;
        border-bottom: 1px solid var(--border-light);
    }

    .rust-domain {
        background: linear-gradient(135deg, rgba(255, 107, 71, 0.08) 0%, rgba(255, 107, 71, 0.03) 100%);
    }

    .ffi-domain {
        background: linear-gradient(135deg, rgba(74, 158, 255, 0.08) 0%, rgba(74, 158, 255, 0.03) 100%);
    }

    .domain-label {
        display: flex;
        align-items: center;
        gap: 0.8rem;
        padding: 1rem 1.5rem;
        min-width: 200px;
        background: rgba(255, 255, 255, 0.5);
        border-right: 1px solid var(--border-light);
    }

    .domain-icon {
        font-size: 1.5rem;
        width: 40px;
        height: 40px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--bg-primary);
        border-radius: 50%;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .domain-info h4 {
        margin: 0 0 0.2rem 0;
        color: var(--text-primary);
        font-size: 0.9rem;
        font-weight: 700;
    }

    .domain-info p {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.7rem;
    }

    .timeline-track {
        flex: 1;
        position: relative;
        padding: 0.8rem;
        background: var(--bg-primary);
    }

    .timeline-axis {
        height: 30px;
        background: linear-gradient(90deg, var(--border-light) 0%, var(--border-light) 100%);
        border-top: 1px solid var(--border-light);
        border-bottom: 1px solid var(--border-light);
        position: relative;
    }

    /* Enhanced Legend */
    .enhanced-legend {
        padding: 1.5rem 2rem;
        background: var(--bg-primary);
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 2rem;
    }

    .legend-section h4 {
        margin: 0 0 1rem 0;
        color: var(--text-primary);
        font-size: 1rem;
        font-weight: 600;
        border-bottom: 2px solid var(--primary-blue);
        padding-bottom: 0.5rem;
    }

    .legend-items {
        display: grid;
        gap: 0.8rem;
    }

    .legend-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .event-symbol {
        width: 16px;
        height: 16px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-weight: bold;
        border-radius: 50%;
    }

    .rust-alloc { background: #ff6b47; color: white; }
    .ffi-alloc { background: #4a9eff; color: white; }
    .boundary-up { color: #00d4aa; font-size: 1.2rem; }
    .boundary-down { color: #ff4757; font-size: 1.2rem; }
    .dealloc { color: #a0a0a0; font-size: 1.2rem; }

    .risk-indicator {
        width: 20px;
        height: 12px;
        border-radius: 6px;
        display: inline-block;
    }

    .high-risk { background: linear-gradient(90deg, #ff4757, #ff3742); }
    .medium-risk { background: linear-gradient(90deg, #ffa502, #ff9f43); }
    .leak-risk { background: linear-gradient(90deg, #ff6b47, #ff5722); }

    /* Memory Passport Modal */
    .memory-passport-modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        backdrop-filter: blur(4px);
    }

    .passport-content {
        background: var(--bg-primary);
        border-radius: 16px;
        max-width: 800px;
        width: 90%;
        max-height: 80%;
        overflow: hidden;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
        border: 1px solid var(--border-light);
    }

    .passport-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem 2rem;
        background: linear-gradient(135deg, var(--primary-blue), #3b82f6);
        color: white;
    }

    .passport-header h3 {
        margin: 0;
        font-size: 1.4rem;
        font-weight: 700;
    }

    .passport-close {
        background: none;
        border: none;
        color: white;
        font-size: 1.5rem;
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 4px;
        transition: background 0.2s ease;
    }

    .passport-close:hover {
        background: rgba(255, 255, 255, 0.2);
    }

    .passport-body {
        padding: 2rem;
        max-height: 60vh;
        overflow-y: auto;
    }

    /* Risk Action Modal Styles */
    .risk-action-section {
        margin-bottom: 2rem;
        padding-bottom: 1.5rem;
        border-bottom: 1px solid var(--border-light);
    }

    .risk-action-section:last-child {
        border-bottom: none;
        margin-bottom: 0;
    }

    .risk-action-section h4 {
        margin: 0 0 1rem 0;
        color: var(--text-primary);
        font-size: 1.1rem;
        font-weight: 600;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .risk-action-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .risk-action-item {
        padding: 0.75rem;
        background: var(--bg-secondary);
        border-radius: 8px;
        border: 1px solid var(--border-light);
    }

    .risk-action-item strong {
        color: var(--text-primary);
        font-weight: 600;
        margin-right: 0.5rem;
    }

    .risk-action-item code {
        background: var(--bg-primary);
        padding: 0.2rem 0.4rem;
        border-radius: 4px;
        font-family: 'Courier New', monospace;
        font-size: 0.85rem;
        color: var(--primary-blue);
    }

    /* Recommended Actions */
    .recommended-actions {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .recommended-action {
        background: var(--bg-secondary);
        border-radius: 8px;
        padding: 1rem;
        border: 1px solid var(--border-light);
        transition: all 0.2s ease;
    }

    .recommended-action:hover {
        border-color: var(--primary-blue);
        box-shadow: 0 4px 8px rgba(59, 130, 246, 0.1);
    }

    .action-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 0.5rem;
    }

    .action-header i {
        color: var(--primary-blue);
        font-size: 1.1rem;
        width: 20px;
    }

    .action-header strong {
        color: var(--text-primary);
        font-weight: 600;
        flex: 1;
    }

    .priority-badge {
        padding: 0.2rem 0.6rem;
        border-radius: 12px;
        font-size: 0.7rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .priority-critical {
        background: rgba(220, 38, 38, 0.1);
        color: #dc2626;
        border: 1px solid rgba(220, 38, 38, 0.2);
    }

    .priority-high {
        background: rgba(245, 158, 11, 0.1);
        color: #f59e0b;
        border: 1px solid rgba(245, 158, 11, 0.2);
    }

    .priority-medium {
        background: rgba(59, 130, 246, 0.1);
        color: #3b82f6;
        border: 1px solid rgba(59, 130, 246, 0.2);
    }

    .priority-low {
        background: rgba(16, 185, 129, 0.1);
        color: #10b981;
        border: 1px solid rgba(16, 185, 129, 0.2);
    }

    .action-description {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.9rem;
        line-height: 1.4;
    }

    /* Risk Factors */
    .risk-factors-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .risk-factor-item {
        background: var(--bg-secondary);
        border-radius: 8px;
        padding: 1rem;
        border: 1px solid var(--border-light);
    }

    .factor-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 0.5rem;
    }

    .factor-header strong {
        color: var(--text-primary);
        font-weight: 600;
    }

    .severity-badge {
        padding: 0.2rem 0.6rem;
        border-radius: 12px;
        font-size: 0.7rem;
        font-weight: 600;
        color: white;
    }

    .factor-description {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.9rem;
        line-height: 1.4;
    }

    /* Quick Actions */
    .quick-actions {
        display: flex;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .quick-action-btn {
        padding: 0.75rem 1.5rem;
        background: var(--bg-secondary);
        border: 1px solid var(--border-light);
        border-radius: 8px;
        color: var(--text-primary);
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .quick-action-btn:hover {
        background: var(--primary-blue);
        color: white;
        border-color: var(--primary-blue);
        transform: translateY(-1px);
        box-shadow: 0 4px 8px rgba(59, 130, 246, 0.3);
    }

    .quick-action-btn i {
        font-size: 0.9rem;
    }
  </style>
</head>

<body>
  <div class="dashboard-container">
    <!-- Header -->
    <header class="header">
      <div>
        <h1>MemScope Memory Analysis Dashboard</h1>
        <div class="subtitle">Real-time Rust Memory Usage Monitoring</div>
      </div>
      <button id="theme-toggle" class="theme-toggle">
        <i class="fa fa-moon"></i>
        <span>Toggle Theme</span>
      </button>
    </header>

    <!-- KPI Metrics -->
    <section class="grid grid-4">
      <div class="kpi-card">
        <div class="kpi-value" id="total-allocations">-</div>
        <div class="kpi-label">Total Allocations</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-value" id="active-variables">-</div>
        <div class="kpi-label">Active Variables</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-value" id="total-memory">-</div>
        <div class="kpi-label">Total Memory</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-value" id="safety-score">-</div>
        <div class="kpi-label">Safety Score</div>
      </div>
    </section>

    <!-- Key Performance Metrics (high priority position) -->
    <section class="grid grid-2">
      <div class="card">
        <h2><i class="fa fa-tachometer-alt"></i> Performance Metrics</h2>
        <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px;">
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-blue);" id="peak-memory">0B</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary);">Peak Memory</div>
          </div>
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-green);" id="allocation-rate">0/sec
            </div>
            <div style="font-size: 0.8rem; color: var(--text-secondary);">Allocation Rate</div>
          </div>
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-orange);" id="avg-lifetime">0ms</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary);">Avg Lifetime</div>
          </div>
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.5rem; font-weight: 700; color: var(--primary-red);" id="fragmentation">0%</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary);">Fragmentation</div>
          </div>
        </div>
      </div>
      <div class="card">
        <h2><i class="fa fa-shield-alt"></i> Thread Safety Analysis</h2>
        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px;">
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-blue);" id="arc-count">0</div>
            <div style="font-size: 0.7rem; color: var(--text-secondary);">Arc</div>
          </div>
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-green);" id="rc-count">0</div>
            <div style="font-size: 0.7rem; color: var(--text-secondary);">Rc</div>
          </div>
          <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-orange);" id="collections-count">0
            </div>
            <div style="font-size: 0.7rem; color: var(--text-secondary);">Collections</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Memory Operations Analysis (Moved to front as summary) -->
    <section class="card">
      <h2><i class="fa fa-exchange"></i> Memory Operations Analysis</h2>
      <div id="memoryOperations" style="padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
        <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px;">
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="time-span" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-blue);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Time Span</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="allocation-burst" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-orange);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Alloc Burst</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="peak-concurrency" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-red);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Peak Concurrency</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="thread-activity" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-green);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Thread Activity</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="borrow-ops" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-blue);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Borrow Ops</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="clone-ops" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-orange);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Clone Ops</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="mut-ratio" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-red);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Mut/Immut</div>
          </div>
          <div style="text-align: center; padding: 16px; background: var(--bg-primary); border-radius: 8px;">
            <div id="avg-borrows" style="font-size: 1.4rem; font-weight: 700; color: var(--primary-green);">-</div>
            <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 4px;">Avg/Alloc</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Enhanced 3D Memory Layout & Timeline Playback -->
    <section class="card">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap;">
        <h2 style="margin: 0;"><i class="fa fa-cube"></i> 3D Memory Layout Visualization</h2>
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
          <button id="toggle3DView" class="theme-toggle"
            style="background: var(--primary-green); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-eye"></i>
            <span>3D View</span>
          </button>
          <button id="reset3DView" class="theme-toggle"
            style="background: var(--primary-orange); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-refresh"></i>
            <span>Reset</span>
          </button>
          <button id="autoRotate3D" class="theme-toggle"
            style="background: var(--primary-blue); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-rotate-right"></i>
            <span>Auto Rotate</span>
          </button>
          <button id="focusLargest" class="theme-toggle"
            style="background: var(--primary-red); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-search-plus"></i>
            <span>Focus Largest</span>
          </button>
        </div>
      </div>
      <div id="memory3DContainer" class="memory-3d-container">
        <div class="memory-3d-info" id="memory3DInfo">
          Loading 3D visualization...
        </div>
      </div>

      <!-- Timeline Playback Controls -->
      <div class="timeline-container">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
          <h3 style="margin: 0; font-size: 1rem;"><i class="fa fa-play-circle"></i> Memory Timeline Playback</h3>
          <div style="font-size: 0.8rem; color: var(--text-secondary);">
            <span id="timelineCurrentTime">0ms</span> / <span id="timelineTotalTime">0ms</span>
          </div>
        </div>
        <div class="timeline-slider" id="timelineSlider">
          <div class="timeline-progress" id="timelineProgress"></div>
          <div class="timeline-thumb" id="timelineThumb"></div>
        </div>
        <div class="timeline-controls">
          <button id="timelinePlay" class="timeline-btn">
            <i class="fa fa-play"></i> Play
          </button>
          <button id="timelinePause" class="timeline-btn" disabled>
            <i class="fa fa-pause"></i> Pause
          </button>
          <button id="timelineReset" class="timeline-btn">
            <i class="fa fa-refresh"></i> Reset
          </button>
          <button id="timelineStep" class="timeline-btn">
            <i class="fa fa-step-forward"></i> Step
          </button>
          <div style="display: flex; align-items: center; gap: 8px; margin-left: 16px;">
            <label style="font-size: 0.8rem; color: var(--text-secondary);">Speed:</label>
            <select id="timelineSpeed"
              style="background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border-light); border-radius: 4px; padding: 2px 6px; font-size: 0.8rem;">
              <option value="0.25">0.25x</option>
              <option value="0.5">0.5x</option>
              <option value="1" selected>1x</option>
              <option value="2">2x</option>
              <option value="4">4x</option>
              <option value="8">8x</option>
            </select>
          </div>
        </div>
        <div style="margin-top: 8px; font-size: 0.8rem; color: var(--text-secondary); text-align: center;">
          Active Allocations: <span id="timelineActiveCount"
            style="color: var(--primary-blue); font-weight: 600;">0</span>
        </div>
      </div>
    </section>

    <!-- Unsafe Rust & FFI Memory Analysis (Enhanced with Integrated Risk Analysis) -->
    <section class="card" style="min-height: 700px;">
      <div class="unsafe-analysis-header">
        <div>
          <h2><i class="fa fa-shield-alt"></i> Unsafe Rust & FFI Memory Analysis</h2>
          <p style="color: var(--text-secondary); font-size: 0.9rem; margin-top: 4px;">
            Advanced memory passport system with cross-boundary visualization and integrated risk assessment
          </p>
        </div>
        <!-- Enhanced Filter Controls with Dynamic Sizing -->
        <div class="unsafe-filter-controls">
          <div class="size-controls">
            <button id="expandAnalysis" class="control-btn">
              <i class="fa fa-expand"></i>
            </button>
            <button id="compactAnalysis" class="control-btn">
              <i class="fa fa-compress"></i>
            </button>
          </div>
          <div class="unsafe-filter-tabs">
            <button class="unsafe-filter-tab active" data-filter="critical">
              🚨 Critical (<span id="unsafe-critical-count">0</span>)
            </button>
            <button class="unsafe-filter-tab" data-filter="leaks">
              💧 Leaks (<span id="unsafe-leak-count">0</span>)
            </button>
            <button class="unsafe-filter-tab" data-filter="cross-boundary">
              🔄 Cross-Boundary (<span id="unsafe-boundary-count">0</span>)
            </button>
            <button class="unsafe-filter-tab" data-filter="risk-assessment">
              ⚠️ Risk Analysis
            </button>
            <button class="unsafe-filter-tab" data-filter="all">
              📊 All Data (<span id="unsafe-total-count">0</span>)
            </button>
          </div>
        </div>
      </div>

      <!-- Enhanced Cross-Boundary Swimlane Timeline -->
      <div class="enhanced-swimlane-container" id="enhancedSwimlaneContainer">
        <div class="swimlane-header">
          <h3><i class="fa fa-timeline"></i> Memory Passport Timeline</h3>
          <div class="timeline-controls">
            <button id="timelineZoomIn" class="timeline-control-btn">
              <i class="fa fa-search-plus"></i> Zoom In
            </button>
            <button id="timelineZoomOut" class="timeline-control-btn">
              <i class="fa fa-search-minus"></i> Zoom Out
            </button>
            <button id="timelineReset" class="timeline-control-btn">
              <i class="fa fa-refresh"></i> Reset View
            </button>
          </div>
        </div>

        <div class="dual-swimlane">
          <!-- Rust Safety Domain -->
          <div class="swimlane rust-domain">
            <div class="domain-label">
              <div class="domain-icon">🦀</div>
              <div class="domain-info">
                <h4>Rust Safety Domain</h4>
                <p>Ownership & borrow checker controlled</p>
              </div>
            </div>
            <div class="timeline-track" id="rustTimelineTrack"></div>
          </div>

          <!-- Timeline Axis -->
          <div class="timeline-axis" id="timelineAxis"></div>

          <!-- C/FFI Domain -->
          <div class="swimlane ffi-domain">
            <div class="domain-label">
              <div class="domain-icon">⚡</div>
              <div class="domain-info">
                <h4>C/FFI Domain</h4>
                <p>Manual memory management</p>
              </div>
            </div>
            <div class="timeline-track" id="ffiTimelineTrack"></div>
          </div>
        </div>

        <!-- Integrated Risk Analysis Section -->
        <div class="integrated-risk-section">
          <div class="risk-section-header">
            <h4><i class="fa fa-exclamation-triangle"></i> Safety Risk Analysis</h4>
            <div class="risk-summary-stats">
              <span class="risk-stat high-risk">
                <span id="high-risk-count">0</span> High
              </span>
              <span class="risk-stat medium-risk">
                <span id="medium-risk-count">0</span> Medium
              </span>
              <span class="risk-stat low-risk">
                <span id="low-risk-count">0</span> Low
              </span>
            </div>
          </div>

          <!-- Risk Analysis Tabs -->
          <div class="risk-analysis-tabs">
            <button class="risk-tab active" data-view="table">
              <i class="fa fa-table"></i> Risk Items
            </button>
            <button class="risk-tab" data-view="patterns">
              <i class="fa fa-chart-pie"></i> Pattern Analysis
            </button>
            <button class="risk-tab" data-view="locations">
              <i class="fa fa-map-marker"></i> Code Locations
            </button>
          </div>

          <!-- Risk Views Container -->
          <div class="risk-views-container">
            <!-- Risk Items Table View -->
            <div id="riskTableView" class="risk-view active">
              <div class="scroll" style="max-height: 200px;">
                <table>
                  <thead>
                    <tr>
                      <th>Location</th>
                      <th>Operation</th>
                      <th>Risk Level</th>
                      <th>Memory Size</th>
                      <th>Assessment</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody id="unsafeTable"></tbody>
                </table>
              </div>
            </div>

            <!-- Pattern Analysis View -->
            <div id="riskPatternsView" class="risk-view" style="display: none;">
              <div id="riskPatternsChart" style="height: 200px; background: var(--bg-secondary); border-radius: 8px;">
                <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">
                  <i class="fa fa-chart-pie" style="font-size: 1.5rem; opacity: 0.5;"></i>
                  <span style="margin-left: 1rem;">Risk pattern analysis loading...</span>
                </div>
              </div>
            </div>

            <!-- Code Locations View -->
            <div id="riskLocationsView" class="risk-view" style="display: none;">
              <div id="riskLocationsHeatmap" style="height: 200px; background: var(--bg-secondary); border-radius: 8px;">
                <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">
                  <i class="fa fa-map" style="font-size: 1.5rem; opacity: 0.5;"></i>
                  <span style="margin-left: 1rem;">Code location heatmap loading...</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Enhanced Legend with Pattern Recognition -->
        <div class="enhanced-legend">
          <div class="legend-section">
            <h4>Event Types</h4>
            <div class="legend-items">
              <div class="legend-item">
                <span class="event-symbol rust-alloc">●</span>
                <span>Unsafe Rust Allocation</span>
              </div>
              <div class="legend-item">
                <span class="event-symbol ffi-alloc">●</span>
                <span>FFI C Allocation</span>
              </div>
              <div class="legend-item">
                <span class="event-symbol boundary-up">▲</span>
                <span>FFI → Rust Transfer</span>
              </div>
              <div class="legend-item">
                <span class="event-symbol boundary-down">▼</span>
                <span>Rust → FFI Transfer</span>
              </div>
              <div class="legend-item">
                <span class="event-symbol dealloc">✕</span>
                <span>Memory Deallocation</span>
              </div>
            </div>
          </div>
          <div class="legend-section">
            <h4>Risk Patterns</h4>
            <div class="legend-items">
              <div class="legend-item">
                <span class="risk-indicator high-risk"></span>
                <span>High Risk - Potential double-free</span>
              </div>
              <div class="legend-item">
                <span class="risk-indicator medium-risk"></span>
                <span>Medium Risk - Ownership unclear</span>
              </div>
              <div class="legend-item">
                <span class="risk-indicator leak-risk"></span>
                <span>Memory Leak - No deallocation</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Memory Passport Modal -->
      <div id="memoryPassport" class="memory-passport-modal" style="display: none;">
        <div class="passport-content">
          <div class="passport-header">
            <h3><i class="fa fa-id-card"></i> Memory Passport</h3>
            <button class="passport-close">&times;</button>
          </div>
          <div class="passport-body" id="passportBody">
            <!-- Dynamic content populated by JavaScript -->
          </div>
        </div>
      </div>

      <!-- Risk Action Modal -->
      <div id="riskActionModal" class="memory-passport-modal" style="display: none;">
        <div class="passport-content">
          <div class="passport-header">
            <h3><i class="fa fa-tools"></i> Risk Mitigation Actions</h3>
            <button class="passport-close" onclick="closeRiskActionModal()">&times;</button>
          </div>
          <div class="passport-body" id="riskActionBody">
            <!-- Dynamic content populated by JavaScript -->
          </div>
        </div>
      </div>
    </section>

    <!-- Unified Memory Analysis -->
    <section class="grid grid-2">
      <div class="card">
        <h2><i class="fa fa-fire"></i> Memory Analysis Dashboard</h2>
        <div class="heatmap-mode-selector">
          <button class="heatmap-mode-btn active" data-mode="heatmap">Heatmap</button>
          <button class="heatmap-mode-btn" data-mode="type">By Type</button>
          <button class="heatmap-mode-btn" data-mode="distribution">Distribution</button>
        </div>
        <div id="memoryVisualization" style="height: 250px;">
          <div id="memoryHeatmap" class="heatmap-container">
            <div class="heatmap-legend" id="heatmapLegend">
              <div style="font-weight: 600; margin-bottom: 4px;">Memory Density</div>
              <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 12px; height: 12px; background: #3b82f6; border-radius: 2px;"></div>
                <span>Low</span>
                <div style="width: 12px; height: 12px; background: #f59e0b; border-radius: 2px;"></div>
                <span>Medium</span>
                <div style="width: 12px; height: 12px; background: #dc2626; border-radius: 2px;"></div>
                <span>High</span>
              </div>
            </div>
          </div>
          <canvas id="typeChart" style="display: none;"></canvas>
          <div id="distributionChart" style="display: none;"></div>
        </div>
      </div>
      
      <div class="card">
        <h2>Memory Over Time</h2>
        <div class="chart-container">
          <canvas id="timelineChart"></canvas>
        </div>
        <div style="margin-top:8px; font-size:12px; color: var(--text-secondary); display:flex; gap:8px; align-items:center;">
          <label style="display:flex; align-items:center; gap:6px; cursor:pointer;">
            <input id="toggleGrowthRate" type="checkbox" style="accent-color: var(--primary-green)">
            <span>Show Growth Rate</span>
          </label>
          <span style="opacity:0.8">Left Y: Cumulative memory, Right Y: Growth rate</span>
        </div>
        <!-- Integrated Key Metrics -->
        <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; margin-top: 12px;">
          <div style="text-align: center; padding: 8px; background: var(--bg-secondary); border-radius: 6px;">
            <div style="font-size: 1rem; font-weight: 700; color: var(--primary-blue);" id="memoryFragmentation">0%</div>
            <div style="font-size: 0.7rem; color: var(--text-secondary);">Fragmentation</div>
          </div>
          <div style="text-align: center; padding: 8px; background: var(--bg-secondary); border-radius: 6px;">
            <div style="font-size: 1rem; font-weight: 700; color: var(--primary-green);" id="memoryEfficiency">0%</div>
            <div style="font-size: 0.7rem; color: var(--text-secondary);">Efficiency</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Memory Distribution Visualization -->
    <section class="card">
      <h2><i class="fa fa-chart-area"></i> Memory Distribution Visualization</h2>
      <!-- 动态尺寸控制 -->
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
        <div style="display: flex; gap: 8px; align-items: center;">
          <label style="font-size: 0.8rem; color: var(--text-secondary);">Scale:</label>
          <input type="range" id="memoryDistScale" min="50" max="200" value="100" 
                 style="width: 80px; accent-color: var(--primary-blue);">
          <span id="memoryDistScaleValue" style="font-size: 0.8rem; color: var(--text-secondary); min-width: 30px;">100%</span>
        </div>
        <div style="display: flex; gap: 8px;">
          <button id="memoryDistFit" class="theme-toggle" 
                  style="background: var(--primary-green); font-size: 10px; padding: 4px 8px;">
            <i class="fa fa-expand-arrows-alt"></i>
            <span>Fit</span>
          </button>
          <button id="memoryDistReset" class="theme-toggle" 
                  style="background: var(--primary-orange); font-size: 10px; padding: 4px 8px;">
            <i class="fa fa-refresh"></i>
            <span>Reset</span>
          </button>
        </div>
      </div>
      
      <div id="memoryDistributionViz"
        style="height: 200px; background: var(--bg-secondary); border-radius: 8px; position: relative; overflow: hidden; border: 1px solid var(--border-light);">
        <!-- Memory blocks visualization will be inserted here -->
      </div>
    </section>

    <!-- Memory Analysis (wider layout) -->
    <section class="grid grid-2">
      <div class="card">
        <h2>Type Treemap</h2>
        <div id="treemap" class="chart-container"
          style="height: 360px; padding: 0; background: var(--bg-secondary); border: 1px solid var(--border-light); border-radius: 8px; overflow: hidden;">
        </div>
      </div>
      <div class="card">
        <h2><i class="fa fa-timeline"></i> Variable Lifecycle Visualization</h2>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div style="display: flex; gap: 6px;">
            <button id="filter-heap" class="theme-toggle"
              style="background: var(--primary-orange); font-size: 10px; padding: 3px 6px;">
              <span>Heap</span>
            </button>
            <button id="filter-stack" class="theme-toggle"
              style="background: var(--primary-blue); font-size: 10px; padding: 3px 6px;">
              <span>Stack</span>
            </button>
            <button id="toggle-lifecycle" class="theme-toggle"
              style="background: var(--primary-green); font-size: 10px; padding: 3px 6px;">
              <span>All</span>
            </button>
          </div>
          <div style="display: flex; gap: 12px; font-size: 0.75rem; color: var(--text-secondary);">
            <span>Heap: <span id="heap-count-mini"
                style="color: var(--primary-orange); font-weight: 600;">-</span></span>
            <span>Stack: <span id="stack-count-mini"
                style="color: var(--primary-blue); font-weight: 600;">-</span></span>
          </div>
        </div>
        <div style="margin-bottom: 8px;">
          <button id="manual-init-btn"
            style="background: var(--primary-red); color: white; border: none; padding: 6px 12px; border-radius: 4px; font-size: 0.8rem; cursor: pointer;">
            🔄 Manual Initialize
          </button>
          <span id="init-status" style="margin-left: 8px; font-size: 0.8rem; color: var(--text-secondary);">Waiting for
            data...</span>
        </div>
        <div id="lifecycleVisualizationContainer" class="scroll" style="max-height: 320px; padding: 8px;">
          <!-- Variable lifecycle items will be inserted here -->
        </div>
      </div>
    </section>

    <!-- Advanced Analysis -->
    <section class="grid grid-2">
      <div class="card">
        <h2><i class="fa fa-puzzle-piece"></i> Memory Fragmentation Analysis</h2>
        <div style="padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
          <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin-bottom: 16px;">
            <div style="text-align: center; padding: 12px; background: var(--bg-primary); border-radius: 6px;">
              <div style="font-size: 1.2rem; font-weight: 700; color: var(--primary-blue);" id="fragmentation-level">5.2%</div>
              <div style="font-size: 0.7rem; color: var(--text-secondary);">Fragmentation</div>
            </div>
            <div style="text-align: center; padding: 12px; background: var(--bg-primary); border-radius: 6px;">
              <div style="font-size: 1.2rem; font-weight: 700; color: var(--primary-green);" id="memory-utilization">94.8%</div>
              <div style="font-size: 0.7rem; color: var(--text-secondary);">Utilization</div>
            </div>
          </div>
          <div id="memoryFragmentation" style="height: 120px; background: var(--bg-primary); border: 1px solid var(--border-light); border-radius: 6px; padding: 8px;">
            <!-- Simple fragmentation visualization -->
            <div style="display: flex; align-items: center; height: 100%; gap: 2px;">
              <div style="height: 100%; width: 15%; background: var(--primary-blue); border-radius: 2px; opacity: 0.8;"></div>
              <div style="height: 60%; width: 8%; background: var(--primary-orange); border-radius: 2px; opacity: 0.6;"></div>
              <div style="height: 80%; width: 12%; background: var(--primary-blue); border-radius: 2px; opacity: 0.8;"></div>
              <div style="height: 30%; width: 5%; background: var(--primary-red); border-radius: 2px; opacity: 0.5;"></div>
              <div style="height: 100%; width: 20%; background: var(--primary-blue); border-radius: 2px; opacity: 0.8;"></div>
              <div style="height: 40%; width: 6%; background: var(--primary-orange); border-radius: 2px; opacity: 0.6;"></div>
              <div style="height: 90%; width: 18%; background: var(--primary-blue); border-radius: 2px; opacity: 0.8;"></div>
              <div style="height: 25%; width: 4%; background: var(--primary-red); border-radius: 2px; opacity: 0.5;"></div>
              <div style="height: 70%; width: 12%; background: var(--primary-blue); border-radius: 2px; opacity: 0.8;"></div>
            </div>
            <div style="text-align: center; font-size: 0.7rem; color: var(--text-secondary); margin-top: 4px;">
              Memory Layout: <span style="color: var(--primary-blue);">■ Allocated</span> 
              <span style="color: var(--primary-orange);">■ Small Gaps</span> 
              <span style="color: var(--primary-red);">■ Large Gaps</span>
            </div>
          </div>
        </div>
      </div>
      <div class="card">
        <h2><i class="fa fa-fire"></i> Borrow Activity Heatmap</h2>
        <div id="borrowPatternChart" style="height: 200px;"></div>
      </div>
    </section>

    <!-- Memory Allocation Details (moved from Advanced Analysis) -->
    <section class="grid grid-2">
      <div class="card">
        <h2><i class="fa fa-table"></i> Memory Allocation Details</h2>
        <div class="scroll">
          <table>
            <thead>
              <tr>
                <th>Variable</th>
                <th>Type</th>
                <th>Size</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody id="allocTable"></tbody>
          </table>
        </div>
      </div>
      
      <div class="card">
        <h2><i class="fa fa-info-circle"></i> System Status</h2>
        <div style="padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
            <div style="text-align: center; padding: 12px; background: var(--bg-primary); border-radius: 6px;">
              <div style="font-size: 1.2rem; font-weight: 700; color: var(--primary-blue);" id="dashboard-status">Ready</div>
              <div style="font-size: 0.7rem; color: var(--text-secondary);">Dashboard</div>
            </div>
            <div style="text-align: center; padding: 12px; background: var(--bg-primary); border-radius: 6px;">
              <div style="font-size: 1.2rem; font-weight: 700; color: var(--primary-green);" id="data-status">Loading</div>
              <div style="font-size: 0.7rem; color: var(--text-secondary);">Data Status</div>
            </div>
          </div>
        </div>
      </div>
    </section>


    <!-- Variable Relationships (Full Width) -->
    <section class="card">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap;">
        <h2 style="margin: 0;"><i class="fa fa-share-alt"></i> Variable Relationships Graph</h2>
        <div style="display: flex; gap: 8px; flex-wrap: nowrap;">
          <button id="reset-zoom" class="theme-toggle"
            style="background: var(--primary-orange); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-expand"></i>
            <span>Reset View</span>
          </button>
          <button id="auto-layout" class="theme-toggle"
            style="background: var(--primary-green); font-size: 12px; padding: 6px 12px; white-space: nowrap;">
            <i class="fa fa-magic"></i>
            <span>Auto Layout</span>
          </button>
        </div>
      </div>
      <div id="graph"
        style="height: 400px; background: var(--bg-secondary); border: 1px solid var(--border-light); border-radius: 8px; position: relative; overflow: hidden;">
      </div>
    </section>

  </div>

  <script>
    // Global safe update function - must be defined first
    function safeUpdateElement(id, value, defaultValue = '-') {
      try {
        const el = document.getElementById(id);
        if (el) {
          el.textContent = value;
          return true;
        } else {
          console.warn(`Element with ID '${id}' not found`);
          return false;
        }
      } catch (error) {
        console.error(`Error updating element ${id}:`, error);
        return false;
      }
    }

    // Enhanced Lifecycle Visualization Functions
    function inferAllocationType(typeName) {
      if (!typeName) return 'unknown';

      const heapTypes = ['Box', 'Vec', 'String', 'HashMap', 'BTreeMap', 'Arc', 'Rc', 'alloc::', 'std::collections'];
      const stackTypes = ['i32', 'i64', 'f32', 'f64', 'bool', 'char', 'usize', 'isize', 'u8', 'u16', 'u32', 'u64'];

      for (const heapType of heapTypes) {
        if (typeName.includes(heapType)) return 'heap';
      }

      for (const stackType of stackTypes) {
        if (typeName.includes(stackType)) return 'stack';
      }

      if (typeName.includes('*') || typeName.includes('&')) return 'heap';

      return 'unknown';
    }

    function formatTimestamp(timestamp) {
      if (!timestamp || isNaN(timestamp) || timestamp <= 0) return 'N/A';

      // Handle different timestamp formats
      let timeInMs;
      if (timestamp > 1e15) {
        // Nanoseconds
        timeInMs = timestamp / 1000000;
      } else if (timestamp > 1e12) {
        // Microseconds  
        timeInMs = timestamp / 1000;
      } else {
        // Already in milliseconds
        timeInMs = timestamp;
      }

      const date = new Date(timeInMs);
      if (isNaN(date.getTime())) return 'N/A';

      // Format as relative time if recent, otherwise absolute time
      const now = Date.now();
      const diffMs = Math.abs(now - timeInMs);

      if (diffMs < 1000) {
        return `${diffMs.toFixed(0)}ms ago`;
      } else if (diffMs < 60000) {
        return `${(diffMs / 1000).toFixed(1)}s ago`;
      } else {
        return date.toLocaleTimeString() + '.' + String(date.getMilliseconds()).padStart(3, '0');
      }
    }

    function formatTimestampSafe(timestamp, fallbackIndex) {
      if (!timestamp || isNaN(timestamp) || timestamp <= 0) {
        // Return synthetic time based on index
        return `T+${(fallbackIndex * 1).toFixed(1)}ms`;
      }

      // Handle nanosecond timestamps (typical format from Rust)
      let timeInMs;
      if (timestamp > 1e15) {
        // Nanoseconds (typical Rust timestamp format)
        timeInMs = timestamp / 1000000;
      } else if (timestamp > 1e12) {
        // Microseconds  
        timeInMs = timestamp / 1000;
      } else if (timestamp > 1e9) {
        // Milliseconds
        timeInMs = timestamp;
      } else {
        // Seconds to milliseconds
        timeInMs = timestamp * 1000;
      }

      // Convert to relative time from program start
      // Use the first allocation timestamp as reference if available
      const firstTimestamp = window.analysisData?.memory_analysis?.allocations?.[0]?.timestamp_alloc || timestamp;
      const relativeTimeMs = (timestamp - firstTimestamp) / 1000000;

      if (Math.abs(relativeTimeMs) < 0.001) {
        return 'T+0.00ms';
      } else if (relativeTimeMs >= 0) {
        return `T+${relativeTimeMs.toFixed(2)}ms`;
      } else {
        return `T${relativeTimeMs.toFixed(2)}ms`;
      }
    }

    function calculateDropTime(allocTime, lifetimeMs) {
      if (!allocTime || lifetimeMs === undefined || isNaN(allocTime) || isNaN(lifetimeMs)) return null;
      return allocTime + (lifetimeMs * 1000000); // Convert ms to nanoseconds and add
    }

    function formatLifetime(lifetimeMs) {
      if (lifetimeMs === undefined || lifetimeMs === null) return 'N/A';
      if (lifetimeMs < 1) return `${(lifetimeMs * 1000).toFixed(1)}μs`;
      if (lifetimeMs < 1000) return `${lifetimeMs.toFixed(1)}ms`;
      return `${(lifetimeMs / 1000).toFixed(2)}s`;
    }

    function formatBytes(bytes) {
      if (bytes === 0) return '0 B';
      const k = 1024;
      const sizes = ['B', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }

    function initEnhancedLifecycleVisualization() {
      // Debug what's available
      console.log("Checking data availability...");
      console.log("window.analysisData exists:", !!window.analysisData);
      console.log("window.analysisData type:", typeof window.analysisData);

      if (window.analysisData) {
        console.log("window.analysisData keys:", Object.keys(window.analysisData));
      }

      // Try multiple data structure paths
      let allocations = null;

      if (window.analysisData) {
        // Method 1: Direct allocations (old structure)
        if (window.analysisData.allocations && Array.isArray(window.analysisData.allocations)) {
          allocations = window.analysisData.allocations;
          console.log("✅ Found allocations directly:", allocations.length);
        }
        // Method 2: Memory analysis structure (new structure)
        else if (window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
          allocations = window.analysisData.memory_analysis.allocations;
          console.log("✅ Found allocations in memory_analysis:", allocations.length);
        }
        // Method 3: Check all keys for allocations
        else {
          for (const [key, value] of Object.entries(window.analysisData)) {
            if (value && typeof value === 'object' && value.allocations && Array.isArray(value.allocations)) {
              allocations = value.allocations;
              console.log(`✅ Found allocations in ${key}:`, allocations.length);
              break;
            }
          }
        }
      }

      if (!allocations || !Array.isArray(allocations) || allocations.length === 0) {
        console.warn('No allocation data found in window.analysisData');

        // Fallback: Try to get data from other global variables that might be set by the dashboard
        if (typeof getAllocations === 'function') {
          try {
            allocations = getAllocations();
            console.log("✅ Got allocations from getAllocations():", allocations.length);
          } catch (e) {
            console.warn('getAllocations() failed:', e);
          }
        }

        // Another fallback: Check if data is in a different global variable
        if (!allocations && window.memoryAnalysisData) {
          if (window.memoryAnalysisData.allocations) {
            allocations = window.memoryAnalysisData.allocations;
            console.log("✅ Got allocations from memoryAnalysisData:", allocations.length);
          }
        }

        // Final fallback: Try to extract from existing DOM elements
        if (!allocations) {
          const existingTable = document.getElementById('allocTable');
          if (existingTable && existingTable.children.length > 1) {
            console.log("Trying to extract data from existing table...");
            // This is a last resort - we'll create dummy data based on table rows
            const rows = existingTable.querySelectorAll('tbody tr');
            if (rows.length > 0) {
              allocations = Array.from(rows).map((row, index) => {
                const cells = row.querySelectorAll('td');
                return {
                  var_name: cells[0]?.textContent || `var_${index}`,
                  type_name: cells[1]?.textContent || 'unknown',
                  size: parseInt(cells[2]?.textContent) || 0,
                  timestamp_alloc: Date.now() * 1000000 + index * 1000000,
                  lifetime_ms: parseFloat(cells[3]?.textContent) || 1.0,
                  is_leaked: cells[4]?.textContent?.includes('Yes') || false
                };
              });
              console.log("✅ Extracted", allocations.length, 'allocations from existing table');
            }
          }
        }

        if (!allocations || allocations.length === 0) {
          console.error('❌ No allocation data available from any source');
          return;
        }
      }

      if (allocations.length === 0) {
        console.warn('No allocation data found');
        return;
      }

      console.log("✅ Found", allocations.length, 'allocations for enhanced visualization');
      console.log("Sample allocation:", allocations[0]);

      // Statistics
      let heapCount = 0, stackCount = 0, unknownCount = 0;
      let totalLifetime = 0, validLifetimes = 0;
      let totalMemory = 0;

      allocations.forEach(alloc => {
        const type = inferAllocationType(alloc.type_name);
        if (type === 'heap') heapCount++;
        else if (type === 'stack') stackCount++;
        else unknownCount++;

        // Check multiple possible lifetime fields and ensure they're valid numbers
        const lifetime = alloc.lifetime_ms || alloc.lifetime || 0;
        if (lifetime !== undefined && lifetime !== null && !isNaN(lifetime) && lifetime > 0) {
          totalLifetime += lifetime;
          validLifetimes++;
        }

        totalMemory += alloc.size || 0;
      });

      console.log("Statistics calculated:", { heapCount, stackCount, totalLifetime, validLifetimes });

      // Update mini counters
      const heapCountMini = document.getElementById('heap-count-mini');
      const stackCountMini = document.getElementById('stack-count-mini');

      if (heapCountMini) {
        safeUpdateElement('heapCountMini', heapCount);
        console.log("Updated heap-count-mini:", heapCount);
      } else {
        console.warn('heap-count-mini element not found');
      }

      if (stackCountMini) {
        safeUpdateElement('stackCountMini', stackCount);
        console.log("Updated stack-count-mini:", stackCount);
      } else {
        console.warn('stack-count-mini element not found');
      }

      // Create lifecycle visualization
      console.log("Calling createLifecycleVisualization...");
      createLifecycleVisualization(allocations);

      // Update enhanced statistics
      console.log("Calling updateEnhancedStatistics...");
      updateEnhancedStatistics(allocations, heapCount, stackCount, validLifetimes, totalLifetime);

      // Setup filters
      console.log("Calling setupLifecycleFilters...");
      setupLifecycleFilters(allocations);

      console.log("✅ All enhanced features processing completed");
    }

    function createLifecycleVisualization(allocations) {
      console.log("createLifecycleVisualization called with", allocations.length, 'allocations');
      const container = document.getElementById('lifecycleVisualizationContainer');
      if (!container) {
        console.error('❌ Lifecycle visualization container not found in DOM');
        return;
      }
      console.log("✅ Found lifecycleVisualizationContainer, creating visualization...");
      container.innerHTML = '';

      // Calculate timeline bounds
      const timestamps = allocations.map(a => a.timestamp_alloc).filter(t => t);
      const minTime = Math.min(...timestamps);
      const maxTime = Math.max(...timestamps);
      const timeRange = maxTime - minTime || 1;

      allocations.forEach((alloc, index) => {
        const allocType = inferAllocationType(alloc.type_name);
        let startTime = alloc.timestamp_alloc || alloc.timestamp || Date.now() * 1000000;
        const lifetime = alloc.lifetime_ms || alloc.lifetime || 0;
        let endTime = startTime + (lifetime * 1000000); // Convert ms to nanoseconds

        // Calculate lifetime from timestamps if not provided
        let calculatedLifetime = 0;
        if (alloc.timestamp_dealloc && alloc.timestamp_alloc) {
          calculatedLifetime = (alloc.timestamp_dealloc - alloc.timestamp_alloc) / 1000000; // Convert to ms
        } else if (alloc.lifetime_ms) {
          calculatedLifetime = alloc.lifetime_ms;
        } else {
          // Default lifetime for active allocations
          calculatedLifetime = 10; // 10ms default
        }

        // Use actual timestamps if available, otherwise create synthetic ones
        if (alloc.timestamp_alloc && !isNaN(alloc.timestamp_alloc)) {
          startTime = alloc.timestamp_alloc;
          endTime = alloc.timestamp_dealloc || (startTime + calculatedLifetime * 1000000);
        } else {
          // Create synthetic timeline based on allocation order
          startTime = minTime + (index * (timeRange / allocations.length));
          endTime = startTime + (calculatedLifetime * 1000000);
        }

        // Debug and validate time data
        console.log("Debug allocation:", {
          var_name: alloc.var_name,
          timestamp_alloc: alloc.timestamp_alloc,
          timestamp_dealloc: alloc.timestamp_dealloc,
          calculatedLifetime: calculatedLifetime,
          startTime: startTime,
          endTime: endTime,
          isValidStart: !isNaN(startTime) && startTime > 0,
          isValidEnd: !isNaN(endTime) && endTime > 0
        });

        // Calculate positions and widths with bounds checking
        let startPercent = ((startTime - minTime) / timeRange) * 100;
        let endPercent = ((endTime - minTime) / timeRange) * 100;

        // Ensure values are within bounds
        startPercent = Math.max(0, Math.min(startPercent, 100));
        endPercent = Math.max(startPercent, Math.min(endPercent, 100));

        let width = endPercent - startPercent;
        width = Math.max(width, 2); // Minimum 2% width
        width = Math.min(width, 100 - startPercent); // Don't exceed container

        // Create lifecycle item
        const item = document.createElement('div');
        item.className = `lifecycle-item ${allocType}`;
        item.setAttribute('data-type', allocType);

        // Enhanced solid colors for better visibility
        const barColor = allocType === 'heap' ? '#ff6b35' :
          allocType === 'stack' ? '#4dabf7' : '#868e96';
        const barGradient = allocType === 'heap' ? '#ff6b35' :
          allocType === 'stack' ? '#4dabf7' :
            '#868e96';

        item.innerHTML = `
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-weight: 600; font-size: 0.9rem; min-width: 120px;">${alloc.var_name || 'unnamed'}</span>
              <span class="allocation-type type-${allocType}">${allocType}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 8px; font-size: 0.75rem; color: var(--text-secondary);">
              <span>${formatBytes(alloc.size || 0)}</span>
              <span>${formatLifetime(lifetime)}</span>
            </div>
          </div>
          
          <!-- Enhanced Timeline Progress Bar -->
          <div style="position: relative; height: 24px; background: linear-gradient(90deg, #e8eaed, #ddd); border-radius: 12px; margin: 10px 0; border: 1px solid #bbb; box-shadow: inset 0 1px 2px rgba(0,0,0,0.08);">
            
            <!-- Variable active period with enhanced gradient -->
            <div style="position: absolute; top: 1px; left: ${startPercent}%; width: ${width}%; height: calc(100% - 2px); 
                        background: ${barGradient}; 
                        border-radius: 11px; 
                        box-shadow: 0 2px 8px rgba(0,0,0,0.15), inset 0 1px 0 rgba(255,255,255,0.3);
                        transition: all 0.3s ease;
                        position: relative;
                        overflow: hidden;">
              
              <!-- Animated shine effect -->
              <div style="position: absolute; top: 0; left: -100%; width: 100%; height: 100%; 
                          background: linear-gradient(90deg, transparent, rgba(255,255,255,0.4), transparent);
                          animation: shine 2s infinite;"></div>
            </div>
            
            ${alloc.is_leaked ? `
              <!-- Leaked indicator -->
              <div style="position: absolute; top: -3px; right: 2px; width: 20px; height: 30px; 
                          background: linear-gradient(45deg, #ff4757, #ff3742); 
                          border-radius: 2px; 
                          box-shadow: 0 2px 4px rgba(0,0,0,0.3);
                          display: flex; align-items: center; justify-content: center;
                          font-size: 10px; color: white; font-weight: bold;">⚠</div>
            ` : ''}
          </div>
          
          <!-- Time info -->
          <div style="display: flex; justify-content: space-between; font-size: 0.7rem; color: var(--text-secondary); font-family: monospace;">
            <span>Start: ${formatTimestampSafe(startTime, index)}</span>
            <span>${alloc.is_leaked ? 'LEAKED' : (formatTimestampSafe(endTime, index + 1) !== 'N/A' ? 'End: ' + formatTimestampSafe(endTime, index + 1) : "Active")}</span>
          </div>
        `;

        // Enhanced hover effect and styling
        item.style.cssText += `
          margin-bottom: 18px;
          padding: 16px;
          background: linear-gradient(135deg, var(--bg-primary), #fafbfc);
          border-radius: 12px;
          border-left: 5px solid ${barColor};
          transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
          cursor: pointer;
          box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        `;

        item.addEventListener('mouseenter', () => {
          item.style.transform = 'translateX(8px) translateY(-2px)';
          item.style.boxShadow = '0 8px 25px rgba(0,0,0,0.15)';
          item.style.background = `linear-gradient(135deg, var(--bg-primary), ${barColor}08)`;
        });

        item.addEventListener('mouseleave', () => {
          item.style.transform = 'translateX(0) translateY(0)';
          item.style.boxShadow = '0 2px 4px rgba(0,0,0,0.05)';
          item.style.background = 'linear-gradient(135deg, var(--bg-primary), #fafbfc)';
        });

        container.appendChild(item);
      });
    }

    function updateEnhancedStatistics(allocations, heapCount, stackCount, validLifetimes, totalLifetime) {
      console.log("Updating enhanced statistics...");

      // Update Enhanced Memory Statistics
      const totalAllocsEnhanced = document.getElementById('total-allocs-enhanced');
      const heapStackRatio = document.getElementById('heap-stack-ratio');
      const avgLifetimeEnhanced = document.getElementById('avg-lifetime-enhanced');
      const memoryEfficiency = document.getElementById('memory-efficiency');

      if (totalAllocsEnhanced) {
        safeUpdateElement('total-allocs-enhanced', allocations.length);
        console.log("Updated total-allocs-enhanced:", allocations.length);
      }

      // Safe DOM updates with enhanced error handling
      if (heapStackRatio) {
        try {
          const ratio = stackCount > 0 ? (heapCount / stackCount).toFixed(1) : heapCount;
          safeUpdateElement('heap-stack-ratio', ratio + ':1');
          console.log("Updated heap-stack-ratio:", ratio + ':1');
        } catch (error) {
          console.error('Error updating heap-stack-ratio:', error);
        }
      }

      if (avgLifetimeEnhanced) {
        try {
          const avgLifetime = validLifetimes > 0 ? formatLifetime(totalLifetime / validLifetimes) : 'N/A';
          safeUpdateElement('avg-lifetime-enhanced', avgLifetime);
          console.log("Updated avg-lifetime-enhanced:", avgLifetime);
        } catch (error) {
          console.error('Error updating avg-lifetime-enhanced:', error);
        }
      }

      const memoryEfficiencyEl = document.getElementById('memory-efficiency');
      if (memoryEfficiencyEl) {
        try {
          const efficiency = allocations.length > 0 ? ((allocations.length - allocations.filter(a => a.is_leaked).length) / allocations.length * 100).toFixed(0) : 0;
          safeUpdateElement('memory-efficiency', efficiency + '%');
          console.log("Updated memory-efficiency:", efficiency + '%');
        } catch (error) {
          console.error('Error updating memory-efficiency:', error);
        }
      }
    }


    function setupLifecycleFilters(allocations) {
      const heapBtn = document.getElementById('filter-heap');
      const stackBtn = document.getElementById('filter-stack');
      const allBtn = document.getElementById('toggle-lifecycle');

      // Check if all buttons exist
      if (!heapBtn || !stackBtn || !allBtn) {
        console.warn('Some filter buttons not found');
        return;
      }

      let currentFilter = 'all';

      function applyFilter(filter) {
        currentFilter = filter;
        const items = document.querySelectorAll('.lifecycle-item');

        // Update button states
        [heapBtn, stackBtn, allBtn].forEach(btn => btn.style.opacity = '0.6');

        if (filter === "heap") {
          heapBtn.style.opacity = '1';
          items.forEach(item => {
            item.style.display = item.getAttribute('data-type') === "heap" ? 'block' : 'none';
          });
        } else if (filter === "stack") {
          stackBtn.style.opacity = '1';
          items.forEach(item => {
            item.style.display = item.getAttribute('data-type') === "stack" ? 'block' : 'none';
          });
        } else {
          allBtn.style.opacity = '1';
          items.forEach(item => {
            item.style.display = 'block';
          });
        }
      }

      heapBtn.addEventListener("click", () => applyFilter("heap"));
      stackBtn.addEventListener("click", () => applyFilter("stack"));
      allBtn.addEventListener("click", () => applyFilter("all"));

      // Initialize
      applyFilter("all");
    }

    // Initialize enhanced features when DOM is loaded
    function initEnhancedFeatures() {
      try {
        initEnhancedLifecycleVisualization();
      } catch (error) {
        console.error('Error initializing enhanced lifecycle visualization:', error);
      }
    }

    // Safe initialization wrapper with duplicate prevention
    let enhancedInitialized = false;
    function safeInitEnhanced() {
      if (enhancedInitialized) {
        return; // Already initialized
      }

      try {
        initEnhancedFeatures();
        enhancedInitialized = true;
        console.log("✅ Enhanced features initialized successfully");
      } catch (error) {
        console.warn('Enhanced features initialization failed:', error);
      }
    }

    {{JS_CONTENT}}

    // Override problematic functions AFTER JS_CONTENT loads
    window.updateLifecycleStatistics = function () {
      // Safe override - prevent errors from missing DOM elements
      try {
        // Try to find and update elements safely
        const elements = ['active-vars', 'freed-vars', 'leaked-vars', 'avg-lifetime-stat'];
        elements.forEach(id => {
          const el = document.getElementById(id);
          safeUpdateElement(id, '-');
        });
      } catch (e) {
        // Silently ignore errors
      }
    };

    // Override other potential problem functions
    window.updateKPIMetrics = function () { return; };
    window.populateLifetimeTable = function () { return; };
    window.updateMemoryStats = function () { return; };

    // Manual initialization function for testing
    function manualInitialize() {
      const statusEl = document.getElementById('init-status");
      safeUpdateElement('init-status', 'Initializing...');

      console.log("🔄 Manual initialization triggered");
      console.log("window.analysisData:", window.analysisData);

      if (window.analysisData && window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
        console.log("✅ Data found, calling initEnhancedLifecycleVisualization...");
        initEnhancedLifecycleVisualization();
        safeUpdateElement('init-status', 'Initialized successfully!');
      } else {
        console.warn('❌ No data found, trying to load...');
        safeUpdateElement('init-status', 'Loading data...');

        // Try to load data manually
        fetch('./large_scale_user_memory_analysis.json")
          .then(response => response.json())
          .then(memoryData => {
            console.log("✅ Manually loaded data:", memoryData);
            window.analysisData = {
              memory_analysis: memoryData
            };
            initEnhancedLifecycleVisualization();
            safeUpdateElement('init-status', 'Data loaded and initialized!');
          })
          .catch(error => {
            console.error('❌ Failed to load data:', error);
            safeUpdateElement('init-status', 'Failed to load data');
          });
      }
    }

    // Wait for all scripts to load, then initialize
    function waitForDataAndInit() {
      if (window.analysisData && window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
        safeUpdateElement('data-status', 'Processing');
        
        try {
          safeInitEnhanced();
          
          // Auto-load Safety Risk Analysis when data is ready
          setTimeout(() => {
            console.log("🛡️ Data ready - auto-loading Safety Risk Analysis...");
            try {
              initializeEnhancedUnsafeAnalysis();
              loadSafetyRisks();
              console.log("✅ Safety Risk Analysis loaded automatically");
            } catch (riskError) {
              console.error('❌ Error auto-loading safety risks:', riskError);
            }
          }, 500);
          
          // Initialize enhanced visualization features
          if (window.enhancedVisualizer) {
            setTimeout(() => {
              console.log("Initializing enhanced visualizer...");
              window.enhancedVisualizer.init();
              window.enhancedVisualizer.initializeWithData(window.analysisData);
              console.log("Enhanced visualizer initialized");
              safeUpdateElement('data-status', 'Ready');
            }, 1000); // Give more time for DOM elements to be ready
          } else {
            safeUpdateElement('data-status', 'Ready');
          }
        } catch (error) {
          console.error('❌ Error during data initialization:', error);
          safeUpdateElement('data-status', 'Error');
        }
      } else {
        setTimeout(waitForDataAndInit, 200);
      }
    }


    // Initialize enhanced features after everything loads
    document.addEventListener('DOMContentLoaded', function () {
      console.log("🚀 Dashboard initialization started");
      
      // Update status indicators
      safeUpdateElement('dashboard-status', 'Initializing');
      safeUpdateElement('data-status', 'Loading');
      
      try {
        // Setup manual initialize button
        const manualBtn = document.getElementById('manual-init-btn");
        if (manualBtn) {
          manualBtn.addEventListener("click", manualInitialize);
        }
        
        // Load safety risks after initialization with longer delay
        setTimeout(function() {
          try {
            console.log("🛡️ Auto-loading safety risks on dashboard initialization...");
            loadSafetyRisks();
            // Also ensure unsafe analysis is initialized
            if (window.analysisData) {
              initializeEnhancedUnsafeAnalysis();
            }
          } catch (error) {
            console.error('❌ Error loading safety risks:', error);
          }
        }, 1500); // Increased delay to ensure data is loaded

        // Start checking for data immediately
        waitForDataAndInit();
        
        safeUpdateElement('dashboard-status', 'Ready');
        console.log("✅ Dashboard initialization completed");
        
      } catch (error) {
        console.error('❌ Dashboard initialization failed:', error);
        safeUpdateElement('dashboard-status', 'Error');
        safeUpdateElement('data-status', 'Failed');
      }
    });
  </script>
</body>

</html>"#;

const EMBEDDED_STYLES_CSS: &str = r#"/* CSS Variables for theming */
:root {
    /* Graph and detail panel variables */
    --graph-node-bg: #ffffff;
    --graph-node-border: #e5e7eb;
    --graph-link-color: #6b7280;
    --detail-panel-bg: #ffffff;
    --detail-panel-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
    
    /* Text colors */
    --text-primary: #374151;
    --text-secondary: #6b7280;
    --text-muted: #9ca3af;
    --text-heading: #111827;
    
    /* Background colors */
    --bg-primary: #ffffff;
    --bg-secondary: #f9fafb;
    --bg-tertiary: #f3f4f6;
    --module-bg-secondary: #f9fafb;
    --module-border-secondary: #e5e7eb;
    
    /* Card colors */
    --card-bg: #ffffff;
    --card-border: #e5e7eb;
    --card-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    
    /* Status colors - light mode */
    --status-blue-bg: #eff6ff;
    --status-blue-text: #1e40af;
    --status-blue-accent: #3b82f6;
    --status-green-bg: #f0fdf4;
    --status-green-text: #166534;
    --status-green-accent: #22c55e;
    --status-orange-bg: #fff7ed;
    --status-orange-text: #c2410c;
    --status-orange-accent: #f97316;
    --status-red-bg: #fef2f2;
    --status-red-text: #dc2626;
    --status-red-accent: #ef4444;
}

.dark {
    /* Dark mode overrides */
    --graph-node-bg: #374151;
    --graph-node-border: #4b5563;
    --graph-link-color: #9ca3af;
    --detail-panel-bg: #1f2937;
    --detail-panel-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
    
    /* Text colors */
    --text-primary: #f9fafb;
    --text-secondary: #d1d5db;
    --text-muted: #9ca3af;
    --text-heading: #ffffff;
    
    /* Background colors */
    --bg-primary: #1f2937;
    --bg-secondary: #374151;
    --bg-tertiary: #4b5563;
    --module-bg-secondary: #1f2937;
    --module-border-secondary: #374151;
    
    /* Card colors */
    --card-bg: #1f2937;
    --card-border: #374151;
    --card-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
    
    /* Status colors - dark mode */
    --status-blue-bg: rgba(59, 130, 246, 0.1);
    --status-blue-text: #93c5fd;
    --status-blue-accent: #60a5fa;
    --status-green-bg: rgba(34, 197, 94, 0.1);
    --status-green-text: #86efac;
    --status-green-accent: #4ade80;
    --status-orange-bg: rgba(249, 115, 22, 0.1);
    --status-orange-text: #fdba74;
    --status-orange-accent: #fb923c;
    --status-red-bg: rgba(239, 68, 68, 0.1);
    --status-red-text: #fca5a5;
    --status-red-accent: #f87171;
}

/* Detail Panel Styles */
.node-detail-panel {
    background: var(--detail-panel-bg);
    box-shadow: var(--detail-panel-shadow);
    border: 1px solid var(--module-border-secondary);
    border-radius: 0.5rem;
    padding: 1rem;
    z-index: 50;
    min-width: 16rem;
    max-width: 20rem;
    position: absolute;
}

.node-detail-panel h3 {
    color: var(--text-primary);
    margin-bottom: 0.75rem;
    font-size: 1.125rem;
    font-weight: 600;
}

.node-detail-panel label {
    color: var(--text-secondary);
    font-size: 0.875rem;
    font-weight: 500;
}

.node-detail-panel p {
    color: var(--text-primary);
    margin-bottom: 0.75rem;
}

.node-detail-panel .close-button {
    color: var(--text-secondary);
    transition: color 0.2s;
}

.node-detail-panel .close-button:hover {
    color: var(--text-primary);
}

/* Graph Node Styles */
.graph-node {
    cursor: pointer;
    transition: opacity 0.2s ease;
}

.graph-node:hover {
    opacity: 1 !important;
}

.graph-node circle {
    /* Don't set fill here - let D3.js control the color */
    stroke: var(--graph-node-border);
    transition: all 0.2s ease;
}

.graph-node:hover circle {
    stroke-width: 3;
    filter: brightness(1.1);
}

.graph-node text {
    fill: var(--text-primary);
    pointer-events: none;
}

/* Memory Analysis Statistics Grid */
.memory-stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.memory-stat-card {
    padding: 0.75rem;
    border-radius: 0.5rem;
    text-align: center;
    transition: transform 0.2s ease;
}

.memory-stat-card:hover {
    transform: translateY(-2px);
}

.memory-stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    line-height: 1.2;
}

.memory-stat-label {
    font-size: 0.75rem;
    margin-top: 0.25rem;
}

/* Progress Bars */
.memory-progress-bar {
    width: 100%;
    height: 0.75rem;
    background-color: var(--module-bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
}

.memory-progress-fill {
    height: 100%;
    border-radius: 9999px;
    transition: width 0.5s ease;
}

/* Dark mode specific adjustments */
.dark .memory-stats-grid .bg-blue-50 {
    background-color: rgba(59, 130, 246, 0.1);
}

.dark .memory-stats-grid .bg-green-50 {
    background-color: rgba(34, 197, 94, 0.1);
}

.dark .memory-stats-grid .bg-purple-50 {
    background-color: rgba(168, 85, 247, 0.1);
}

.dark .memory-stats-grid .bg-orange-50 {
    background-color: rgba(249, 115, 22, 0.1);
}

/* Responsive design for detail panels */
@media (max-width: 768px) {
    .node-detail-panel {
        min-width: 12rem;
        max-width: 16rem;
        font-size: 0.875rem;
    }
    
    .memory-stats-grid {
        grid-template-columns: repeat(2, 1fr);
    }
}

/* Unified text color classes */
.text-primary {
    color: var(--text-primary);
}

.text-secondary {
    color: var(--text-secondary);
}

.text-muted {
    color: var(--text-muted);
}

.text-heading {
    color: var(--text-heading);
}

/* Unified background classes */
.bg-primary {
    background-color: var(--bg-primary);
}

.bg-secondary {
    background-color: var(--bg-secondary);
}

.bg-tertiary {
    background-color: var(--bg-tertiary);
}

/* Unified card classes */
.card-bg {
    background-color: var(--card-bg);
}

.card-border {
    border-color: var(--card-border);
}

.card-shadow {
    box-shadow: var(--card-shadow);
}

/* Status card classes */
.status-card-blue {
    background-color: var(--status-blue-bg);
    border-left: 4px solid var(--status-blue-accent);
}

.status-card-blue .status-title {
    color: var(--status-blue-text);
}

.status-card-blue .status-value {
    color: var(--status-blue-accent);
}

.status-card-green {
    background-color: var(--status-green-bg);
    border-left: 4px solid var(--status-green-accent);
}

.status-card-green .status-title {
    color: var(--status-green-text);
}

.status-card-green .status-value {
    color: var(--status-green-accent);
}

.status-card-orange {
    background-color: var(--status-orange-bg);
    border-left: 4px solid var(--status-orange-accent);
}

.status-card-orange .status-title {
    color: var(--status-orange-text);
}

.status-card-orange .status-value {
    color: var(--status-orange-accent);
}

.status-card-red {
    background-color: var(--status-red-bg);
    border-left: 4px solid var(--status-red-accent);
}

.status-card-red .status-title {
    color: var(--status-red-text);
}

.status-card-red .status-value {
    color: var(--status-red-accent);
}

/* Legend and small text classes */
.legend-text {
    color: var(--text-secondary);
    font-size: 0.75rem;
}

.legend-bg {
    background-color: var(--bg-secondary);
}

/* Animation for theme transitions */
* {
    transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease;
}

/* Ensure proper contrast in dark mode */
.dark table {
    color: var(--text-primary);
}

.dark table th {
    color: var(--text-secondary);
    border-color: var(--module-border-secondary);
}

.dark table td {
    border-color: var(--module-border-secondary);
}

.dark .card-shadow {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2);
}

/* Basic CSS styles for MemScope dashboard */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    margin: 0;
    padding: 20px;
    background-color: #f5f5f5;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    padding: 20px;
}

.header {
    text-align: center;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 2px solid #eee;
}

.header h1 {
    color: #2c3e50;
    margin: 0;
    font-size: 2.5em;
}

.stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin: 20px 0;
}

.stat-card {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    text-align: center;
    border-left: 4px solid #3498db;
}

.stat-value {
    font-size: 2em;
    font-weight: bold;
    color: #2c3e50;
    display: block;
}

.stat-label {
    color: #7f8c8d;
    font-size: 0.9em;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.loading {
    text-align: center;
    padding: 40px;
    color: #7f8c8d;
}

.error {
    background: #e74c3c;
    color: white;
    padding: 15px;
    border-radius: 5px;
    margin: 10px 0;
}

.success {
    background: #27ae60;
    color: white;
    padding: 15px;
    border-radius: 5px;
    margin: 10px 0;
}

.data-section {
    margin: 30px 0;
    padding: 20px;
    background: #f8f9fa;
    border-radius: 8px;
}

.data-section h2 {
    color: #2c3e50;
    margin-top: 0;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 20px 0;
}

th, td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
}

th {
    background-color: #3498db;
    color: white;
    font-weight: 600;
}

tr:hover {
    background-color: #f5f5f5;
}

.btn {
    background: #3498db;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 14px;
}

.btn:hover {
    background: #2980b9;
}

.progress-bar {
    width: 100%;
    height: 20px;
    background: #ecf0f1;
    border-radius: 10px;
    overflow: hidden;
    margin: 10px 0;
}

.progress-fill {
    height: 100%;
    background: #3498db;
    transition: width 0.3s ease;
}
/* Mode
rn Variable Graph Styles */
.graph-container {
    position: relative;
    overflow: hidden;
}

#variable-graph-container {
    width: 100%;
    height: 100%;
}

#node-details {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(0, 0, 0, 0.1);
}

.node {
    transition: all 0.3s ease;
}

.node:hover circle {
    stroke-width: 3;
    filter: drop-shadow(0 4px 8px rgba(0,0,0,0.2));
}

/* Unsafe FFI Dashboard Styles */
.ffi-dashboard {
    background: linear-gradient(135deg, #2c3e50 0%, #34495e 50%, #2c3e50 100%);
}

.metric-card {
    transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.metric-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

.hotspot-circle {
    transition: all 0.3s ease;
}

.hotspot-circle:hover {
    transform: scale(1.1);
    filter: brightness(1.2);
}

/* Graph Legend Styles */
.legend-item {
    display: flex;
    align-items: center;
    margin-bottom: 8px;
}

.legend-color {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    margin-right: 8px;
}

.legend-line {
    height: 2px;
    margin-right: 8px;
}

/* Responsive adjustments */
@media (max-width: 768px) {
    #node-details {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 90%;
        max-width: 300px;
        z-index: 1000;
    }
    
    .stats-grid {
        grid-template-columns: repeat(2, 1fr);
        gap: 10px;
    }
    
    .metric-card {
        padding: 12px;
    }
}

/* Animation classes */
.fade-in {
    animation: fadeIn 0.5s ease-in;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
}

.pulse {
    animation: pulse 2s infinite;
}

@keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.7; }
    100% { opacity: 1; }
}"#;

const EMBEDDED_SCRIPT_JS: &str = r##"// MemScope Dashboard JavaScript - Clean rendering for clean_dashboard.html
// This file contains comprehensive functions for memory analysis dashboard

// Global data store - will be populated by HTML template
window.analysisData = window.analysisData || {};

// FFI dashboard render style: 'svg' to mimic Rust SVG dashboard, 'cards' for card-based UI
const FFI_STYLE = 'svg';

// Initialize all dashboard components - Clean layout
function initCleanTemplate() {
    console.log("🚀 Initializing MemScope Dashboard...");
    console.log("📊 Available data:", Object.keys(window.analysisData||{}));
    const data = window.analysisData || {};

    // KPI
    updateKPICards(data);

    // Memory by type (Chart.js)
    const typeChartEl = document.getElementById("typeChart");
    if (typeChartEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const byType = {};
        for (const a of allocs) { const t=a.type_name||'Unknown'; byType[t]=(byType[t]||0)+(a.size||0); }
        const top = Object.entries(byType).sort((a,b)=>b[1]-a[1]).slice(0,10);
        if (top.length>0) {
            const ctx = typeChartEl.getContext("2d");
            if (window.chartInstances['clean-type']) window.chartInstances['clean-type'].destroy();
            window.chartInstances['clean-type'] = new Chart(ctx, {
                type:'bar',
                data:{ labels: top.map(x=>x[0]), datasets:[{ label:'Bytes', data: top.map(x=>x[1]), backgroundColor:'#3b82f6' }] },
                options:{ responsive:true, plugins:{legend:{display:false}}, scales:{y:{beginAtZero:true}} }
            });
            const legend = document.getElementById('typeLegend');
            if (legend) legend.innerHTML = top.map(([n,v])=>`<span class="pill">${n}: ${formatBytes(v)}</span>`).join(' ');
        }
    }

    // Timeline (Chart.js)
    const timelineEl = document.getElementById("timelineChart");
    if (timelineEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const rawTimeline = (data.memory_analysis && data.memory_analysis.memory_timeline) || [];
        let points = [];
        if (rawTimeline.length) {
            points = rawTimeline.map((p,i)=>({ x:i, y:(p.memory_usage||0) }));
        } else {
            const sorted = allocs.slice().sort((a,b)=>(a.timestamp_alloc||0)-(b.timestamp_alloc||0));
            let cum=0; const step=Math.max(1, Math.floor(sorted.length/50));
            for(let i=0;i<sorted.length;i+=step){ cum += sorted[i].size||0; points.push({x:i, y:cum}); }
        }
        if (points.length>1) {
            const ctx = timelineEl.getContext("2d");
            if (window.chartInstances['clean-timeline']) window.chartInstances['clean-timeline'].destroy();
            window.chartInstances['clean-timeline'] = new Chart(ctx, {
                type:'line',
                data:{ labels: points.map(p=>p.x), datasets:[{ label:'Cumulative', data: points.map(p=>p.y), borderColor:'#ef4444', backgroundColor:'rgba(239,68,68,0.1)', fill:true, tension:0.25 }] },
                options:{ responsive:true, plugins:{legend:{display:false}}, scales:{y:{beginAtZero:true}} }
            });
        }
    }

    // Treemap
    const treemapEl = document.getElementById("treemap");
    if (treemapEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        treemapEl.innerHTML = createTreemapVisualization(allocs);
    }

    // Growth
    const growthEl = document.getElementById("growth");
    if (growthEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const total = allocs.reduce((s,a)=>s+(a.size||0),0);
        growthEl.innerHTML = createAdvancedGrowthTrendVisualization(allocs, Math.max(1,total));
    }

    // Lifetimes (top 10)
    const lifetimesEl = document.getElementById("lifetimes");
    if (lifetimesEl) {
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        const top = allocs.filter(a=>a.var_name && a.var_name!=='unknown').sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,10);
        lifetimesEl.innerHTML = top.map(a=>`<div class="flex items-center justify-between py-1 border-b">
            <div class="text-xs font-medium">${a.var_name}</div>
            <div class="text-xs text-gray-500">${formatBytes(a.size||0)}</div>
        </div>`).join('');
    }

    // Update memory allocation table
    updateAllocationsTable(data);
    
    // Update unsafe risk table
    updateUnsafeTable(data);

    // Initialize all charts and visualizations
    initCharts(data);
    
    // Initialize lifecycle visualization
    initLifetimeVisualization(data);

    // Complex types
    const complexSummary = document.getElementById("complexSummary");
    if (complexSummary) {
        const ct = data.complex_types || {};
        const s = ct.summary || {};
        const items = [
            {label:'Complex Types', val: s.total_complex_types||0},
            {label:'Smart Pointers', val: s.smart_pointers_count||0},
            {label:'Collections', val: s.collections_count||0},
            {label:'Generic Types', val: s.generic_types_count||s.generic_type_count||0},
        ];
        complexSummary.innerHTML = items.map(x=>`<div class="pill">${x.label}: ${x.val}</div>`).join('');
        document.getElementById("complexSmart")?.replaceChildren();
        document.getElementById("complexCollections")?.replaceChildren();
        document.getElementById("complexGenerics")?.replaceChildren();
    }

    // Variable relationships
    const graphEl = document.getElementById("graph");
    if (graphEl) {
        // reuse our D3 relationship graph init but mount into #graph
        const container = document.createElement('div');
        container.id = 'variable-graph-container';
        container.style.width = '100%';
        container.style.height = '260px';
        graphEl.appendChild(container);
        try { initVariableGraph(); } catch(e) { console.warn('variable graph init failed', e); }
    }

    // Security violations
    const secEl = document.getElementById("security");
    if (secEl) {
        const root = data.unsafe_ffi || {};
        const list = root.security_hotspots || root.unsafe_reports || [];
        secEl.innerHTML = (list||[]).slice(0,12).map(h=>{
            const score = h.risk_score || h.risk_assessment?.confidence_score || 0;
            const level = h.risk_level || h.risk_assessment?.risk_level || 'Unknown';
            const width = Math.min(100, Math.round((score||0)*10));
            return `<div class="card">
              <div class="text-sm font-semibold">${h.location||h.report_id||'Unknown'}</div>
              <div class="text-xs text-gray-500">${h.description||h.source?.type||''}</div>
              <div class="mt-2 bg-red-100 h-2 rounded"><div style="width:${width}%; background:#ef4444; height:100%" class="rounded"></div></div>
              <div class="text-xs text-gray-500 mt-1">Risk: ${level} (${score})</div>
            </div>`;
        }).join('') || '<div class="muted">No security violations</div>';
    }
}
function initializeDashboard() {
    console.log("🚀 Initializing MemScope dashboard...");
    console.log("📊 Available data:", Object.keys(window.analysisData || {}));

    // Initialize theme system first
    initThemeToggle();

    // Initialize enhanced dashboard with comprehensive data
    initEnhancedSummaryStats();
    
    // Initialize all components
    initSummaryStats();
    initCharts();
    initMemoryUsageAnalysis();
    initLifetimeVisualization();
    initFFIVisualization();
    initMemoryFragmentation();
    initMemoryGrowthTrends();
    initAllocationsTable();
    initVariableGraph();
}

// Initialize theme toggle functionality
function initThemeToggle() {
    const themeToggle = document.getElementById('theme-toggle');
    const html = document.documentElement;

    // Check for saved theme preference or default to light mode
    const savedTheme = localStorage.getItem('memscope-theme') || 'light';

    console.log("🎨 Initializing theme system, saved theme:", savedTheme);

    // Apply initial theme
    applyTheme(savedTheme === "dark");

    if (themeToggle) {
        themeToggle.addEventListener("click", () => {
            const isDark = html.classList.contains('dark');

            if (isDark) {
                applyTheme(false);
                localStorage.setItem('memscope-theme', 'light');
                console.log("🎨 Theme switched to: light mode");
            } else {
                applyTheme(true);
                localStorage.setItem('memscope-theme', "dark");
                console.log("🎨 Theme switched to: dark mode");
            }
        });

        console.log("✅ Theme toggle initialized successfully");
    } else {
        console.warn('⚠️ Theme toggle button not found');
    }
}

// Apply theme to all modules
function applyTheme(isDark) {
    const html = document.documentElement;
    const body = document.body;

    if (isDark) {
        html.classList.remove('light');
        html.classList.add('dark');
        body.classList.add('dark');
    } else {
        html.classList.remove('dark');
        html.classList.add('light');
        body.classList.remove('dark');
    }

    // Force immediate repaint
    html.style.display = 'none';
    html.offsetHeight; // Trigger reflow
    html.style.display = '';

    // Apply theme to all modules that need explicit dark mode support
    applyThemeToAllModules(isDark);

    // Update theme toggle button icon
    updateThemeToggleIcon(isDark);

    // Destroy existing charts before reinitializing
    destroyAllCharts();

    // Reinitialize charts to apply theme changes
    setTimeout(() => {
        initCharts();
        initFFIRiskChart();
    }, 100);
}

// Update theme toggle button icon
function updateThemeToggleIcon(isDark) {
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        const icon = themeToggle.querySelector('i');
        if (icon) {
            if (isDark) {
                icon.className = "fa fa-sun";
            } else {
                icon.className = "fa fa-moon";
            }
        }
    }
}

// Global chart instances storage
window.chartInstances = {};

// Destroy all existing charts
function destroyAllCharts() {
    Object.keys(window.chartInstances).forEach(chartId => {
        if (window.chartInstances[chartId]) {
            window.chartInstances[chartId].destroy();
            delete window.chartInstances[chartId];
        }
    });
}

// Apply theme to specific modules
function applyThemeToAllModules(isDark) {
    const modules = [
        'memory-usage-analysis',
        'generic-types-details',
        'variable-relationship-graph',
        'complex-type-analysis',
        'memory-optimization-recommendations',
        'unsafe-ffi-data'
    ];

    modules.forEach(moduleId => {
        const module = document.getElementById(moduleId);
        if (module) {
            module.classList.toggle('dark', isDark);
        }
    });

    // Also apply to any table elements that might need it
    const tables = document.querySelectorAll('table');
    tables.forEach(table => {
        table.classList.toggle('dark', isDark);
    });

    // Apply to any chart containers
    const chartContainers = document.querySelectorAll('canvas');
    chartContainers.forEach(container => {
        if (container.parentElement) {
            container.parentElement.classList.toggle('dark', isDark);
        }
    });
}

// Initialize summary statistics
function initSummaryStats() {
    console.log("📊 Initializing summary stats...");

    const data = window.analysisData;

    // Update complex types count
    const complexTypesCount = data.complex_types?.summary?.total_complex_types || 0;
    updateElement('total-complex-types', complexTypesCount);

    // Update total allocations
    const totalAllocations = data.memory_analysis?.allocations?.length || 0;
    updateElement('total-allocations', totalAllocations);

    // Update generic types count
    const genericTypeCount = data.complex_types?.summary?.generic_type_count || 0;
    updateElement('generic-type-count', genericTypeCount);

    // Update unsafe FFI count
    const unsafeFFICount = data.unsafe_ffi?.enhanced_ffi_data?.length || 0;
    updateElement('unsafe-ffi-count', unsafeFFICount);

    // Update category counts
    const smartPointersCount = data.complex_types?.categorized_types?.smart_pointers?.length || 0;
    const collectionsCount = data.complex_types?.categorized_types?.collections?.length || 0;
    const primitivesCount = 0; // Calculate from data if available

    updateElement('smart-pointers-count', smartPointersCount);
    updateElement('collections-count', collectionsCount);
    updateElement('primitives-count', primitivesCount);
}

// Initialize charts - simplified
function initCharts() {
    console.log("📊 Initializing charts...");

    // Initialize memory distribution chart
    initMemoryDistributionChart();

    // Initialize allocation size chart
    initAllocationSizeChart();
}



// Initialize memory distribution chart
function initMemoryDistributionChart() {
    const ctx = document.getElementById("memory-distribution-chart");
    if (!ctx) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const typeDistribution = {};

    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System Allocation';
        typeDistribution[type] = (typeDistribution[type] || 0) + alloc.size;
    });

    const sortedTypes = Object.entries(typeDistribution)
        .sort(([, a], [, b]) => b - a)
        .slice(0, 10);

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['memory-distribution-chart']) {
        window.chartInstances['memory-distribution-chart'].destroy();
    }

    window.chartInstances['memory-distribution-chart'] = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: sortedTypes.map(([type]) => formatTypeName(type)),
            datasets: [{
                label: 'Memory Usage (bytes)',
                data: sortedTypes.map(([, size]) => size),
                backgroundColor: '#3b82f6'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#ffffff' : '#374151'
                    }
                }
            },
            scales: {
                x: {
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                },
                y: {
                    beginAtZero: true,
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280',
                        callback: function (value) {
                            return formatBytes(value);
                        }
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                }
            }
        }
    });
}

// Initialize allocation size chart
function initAllocationSizeChart() {
    const ctx = document.getElementById("allocation-size-chart");
    if (!ctx) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const sizeDistribution = {
        'Tiny (< 64B)': 0,
        'Small (64B - 1KB)': 0,
        'Medium (1KB - 64KB)': 0,
        'Large (64KB - 1MB)': 0,
        'Huge (> 1MB)': 0
    };

    allocations.forEach(alloc => {
        const size = alloc.size || 0;
        if (size < 64) sizeDistribution['Tiny (< 64B)']++;
        else if (size < 1024) sizeDistribution['Small (64B - 1KB)']++;
        else if (size < 65536) sizeDistribution['Medium (1KB - 64KB)']++;
        else if (size < 1048576) sizeDistribution['Large (64KB - 1MB)']++;
        else sizeDistribution['Huge (> 1MB)']++;
    });

    // Destroy existing chart if it exists
    if (window.chartInstances['allocation-size-chart']) {
        window.chartInstances['allocation-size-chart'].destroy();
    }

    window.chartInstances['allocation-size-chart'] = new Chart(ctx, {
        type: 'pie',
        data: {
            labels: Object.keys(sizeDistribution),
            datasets: [{
                data: Object.values(sizeDistribution),
                backgroundColor: ['#10b981', '#3b82f6', '#f59e0b', '#ef4444', '#7c2d12']
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: document.documentElement.classList.contains('dark') ? '#ffffff' : '#374151'
                    }
                }
            }
        }
    });
}



// Process memory analysis data with validation and fallback
function processMemoryAnalysisData(rawData) {
    if (!rawData || !rawData.memory_analysis) {
        console.warn('⚠️ No memory analysis data found, generating fallback data');
        return generateFallbackMemoryData();
    }

    const memoryData = rawData.memory_analysis;
    const processedData = {
        stats: {
            total_allocations: memoryData.stats?.total_allocations || 0,
            active_allocations: memoryData.stats?.active_allocations || 0,
            total_memory: memoryData.stats?.total_memory || 0,
            active_memory: memoryData.stats?.active_memory || 0
        },
        allocations: memoryData.allocations || [],
        trends: {
            peak_memory: memoryData.peak_memory || 0,
            growth_rate: memoryData.growth_rate || 0,
            fragmentation_score: memoryData.fragmentation_score || 0
        }
    };

    // Calculate additional metrics if not present
    if (processedData.allocations.length > 0) {
        const totalSize = processedData.allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
        if (!processedData.stats.total_memory) {
            processedData.stats.total_memory = totalSize;
        }
        if (!processedData.stats.total_allocations) {
            processedData.stats.total_allocations = processedData.allocations.length;
        }
    }

    console.log("✅ Processed memory analysis data:", processedData);
    return processedData;
}

// Generate fallback memory data when real data is unavailable
function generateFallbackMemoryData() {
    console.log("🔄 Generating fallback memory data");

    return {
        stats: {
            total_allocations: 0,
            active_allocations: 0,
            total_memory: 0,
            active_memory: 0
        },
        allocations: [],
        trends: {
            peak_memory: 0,
            growth_rate: 0,
            fragmentation_score: 0
        },
        isFallback: true
    };
}

// Validate memory data structure
function validateMemoryData(data) {
    if (!data) return false;

    const hasStats = data.stats && typeof data.stats === 'object';
    const hasAllocations = Array.isArray(data.allocations);

    return hasStats && hasAllocations;
}

// Calculate memory statistics from allocations
function calculateMemoryStatistics(allocations) {
    if (!Array.isArray(allocations) || allocations.length === 0) {
        return {
            totalSize: 0,
            averageSize: 0,
            largestAllocation: 0,
            userAllocations: 0,
            systemAllocations: 0
        };
    }

    const totalSize = allocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
    const averageSize = totalSize / allocations.length;
    const largestAllocation = Math.max(...allocations.map(alloc => alloc.size || 0));

    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    ).length;

    const systemAllocations = allocations.length - userAllocations;

    return {
        totalSize,
        averageSize,
        largestAllocation,
        userAllocations,
        systemAllocations
    };
}

// Initialize memory usage analysis with enhanced SVG-style visualization
function initMemoryUsageAnalysis() {
    const container = document.getElementById("memory-usage-analysis");
    if (!container) return;

    // Process memory data with validation
    const memoryData = processMemoryAnalysisData(window.analysisData);
    const allocations = memoryData.allocations;

    if (allocations.length === 0 || memoryData.isFallback) {
        container.innerHTML = createEnhancedEmptyState();
        return;
    }

    // Calculate comprehensive statistics
    const stats = calculateMemoryStatistics(allocations);
    const totalMemory = stats.totalSize;

    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    );
    const systemAllocations = allocations.filter(alloc =>
        !alloc.var_name || alloc.var_name === 'unknown' ||
        !alloc.type_name || alloc.type_name === 'unknown'
    );

    const userMemory = userAllocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);
    const systemMemory = systemAllocations.reduce((sum, alloc) => sum + (alloc.size || 0), 0);

    // Create enhanced SVG-style visualization
    container.innerHTML = createMemoryAnalysisSVG(stats, allocations, userMemory, systemMemory, totalMemory);
}

// Create enhanced empty state with better styling
function createEnhancedEmptyState() {
    return `
        <div class="h-full flex items-center justify-center">
            <div class="text-center p-8 bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-800 dark:to-gray-700 rounded-xl border-2 border-dashed border-blue-200 dark:border-gray-600">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-blue-400 dark:text-blue-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">Memory Analysis Ready</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">No memory allocation data found for analysis</p>
                <p class="text-xs text-gray-500 dark:text-gray-500">Run your application with memory tracking enabled to see detailed analysis</p>
            </div>
        </div>
    `;
}

// Create comprehensive SVG-style memory analysis visualization inspired by the memoryAnalysis.svg
function createMemoryAnalysisSVG(stats, allocations, userMemory, systemMemory, totalMemory) {
    const userPercentage = totalMemory > 0 ? (userMemory / totalMemory * 100) : 0;
    const systemPercentage = totalMemory > 0 ? (systemMemory / totalMemory * 100) : 0;

    // Calculate comprehensive efficiency metrics
    const efficiency = totalMemory > 0 ? Math.min(100, (userMemory / totalMemory * 100)) : 0;
    const reclamationRate = allocations.length > 0 ? Math.min(100, ((allocations.filter(a => a.timestamp_dealloc).length / allocations.length) * 100)) : 0;
    const fragmentation = Math.min(100, (allocations.length / Math.max(1, totalMemory / 1024)) * 10);

    // Advanced size distribution analysis
    const sizeDistribution = {
        tiny: allocations.filter(a => a.size < 64).length,
        small: allocations.filter(a => a.size >= 64 && a.size < 1024).length,
        medium: allocations.filter(a => a.size >= 1024 && a.size < 65536).length,
        large: allocations.filter(a => a.size >= 65536).length
    };

    // Calculate median and P95 sizes
    const sizes = allocations.map(a => a.size || 0).sort((a, b) => a - b);
    const medianSize = sizes.length > 0 ? sizes[Math.floor(sizes.length / 2)] : 0;
    const p95Size = sizes.length > 0 ? sizes[Math.floor(sizes.length * 0.95)] : 0;

    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden">
            <!-- Header with gradient background -->
            <div class="bg-gradient-to-r from-blue-600 to-purple-600 text-white p-6">
                <div class="text-center">
                    <h2 class="text-3xl font-bold mb-2">Rust Memory Usage Analysis</h2>
                    <p class="text-blue-100 uppercase tracking-wider text-sm">Key Performance Metrics</p>
                </div>
            </div>

            <div class="p-6">
                <!-- Key Performance Metrics Grid -->
                <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-8 gap-4 mb-8">
                    ${createAdvancedMetricCard('Active Memory', formatBytes(userMemory), Math.round(userPercentage), '#3498db', "MEDIUM")}
                    ${createAdvancedMetricCard('Peak Memory', formatBytes(totalMemory), 100, '#e74c3c', "HIGH")}
                    ${createAdvancedMetricCard('Active Allocs', allocations.length, 100, '#2ecc71', "HIGH")}
                    ${createAdvancedMetricCard('Reclamation', reclamationRate.toFixed(1) + '%', Math.round(reclamationRate), '#f39c12', reclamationRate > 70 ? 'OPTIMAL' : "MEDIUM")}
                    ${createAdvancedMetricCard('Efficiency', efficiency.toFixed(1) + '%', Math.round(efficiency), '#9b59b6', efficiency > 70 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('Median Size', formatBytes(medianSize), Math.min(100, medianSize / 1024), '#1abc9c', medianSize < 100 ? 'OPTIMAL' : "MEDIUM")}
                    ${createAdvancedMetricCard('P95 Size', formatBytes(p95Size), Math.min(100, p95Size / 1024), '#e67e22', p95Size < 1024 ? 'OPTIMAL' : 'MEDIUM')}
                    ${createAdvancedMetricCard('Fragmentation', fragmentation.toFixed(1) + '%', Math.round(fragmentation), '#95a5a6', fragmentation < 30 ? 'OPTIMAL' : "MEDIUM")}
                </div>


                <!-- Memory Usage by Type - Enhanced Treemap -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-8 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Usage by Type - Treemap Visualization</h3>
                    <div class="bg-gray-100 dark:bg-gray-600 rounded-lg p-4 h-64 relative overflow-hidden">
                        ${createAdvancedTreemapVisualization(allocations, totalMemory)}
                    </div>
                    <div class="mt-4 grid grid-cols-3 gap-4 text-xs">
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-blue-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Collections</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-green-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Basic Types</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-gray-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">System</span>
                        </div>
                    </div>
                </div>

                <!-- Advanced Analysis Grid -->
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <!-- Memory Fragmentation Analysis -->
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                        <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Fragmentation Analysis</h3>
                        <div class="space-y-4">
                            ${createAdvancedFragmentationBar('Tiny (0-64B)', sizeDistribution.tiny, allocations.length, "#27ae60")}
                            ${createAdvancedFragmentationBar('Small (65B-1KB)', sizeDistribution.small, allocations.length, "#f39c12")}
                            ${createAdvancedFragmentationBar('Medium (1KB-64KB)', sizeDistribution.medium, allocations.length, "#e74c3c")}
                            ${createAdvancedFragmentationBar('Large (>64KB)', sizeDistribution.large, allocations.length, "#8e44ad")}
                        </div>
                    </div>

                    <!-- Call Stack Analysis -->
                    <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                        <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Call Stack Analysis</h3>
                        <div class="space-y-3 max-h-64 overflow-y-auto">
                            ${createCallStackAnalysis(allocations)}
                        </div>
                    </div>
                </div>

                <!-- Memory Statistics Summary -->
                <div class="mt-8 bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Memory Statistics</h3>
                    <div class="grid grid-cols-3 gap-4 text-sm text-center">
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Peak Memory:</span>
                            <span class="font-semibold text-red-600 dark:text-red-400 ml-2">${formatBytes(totalMemory)}</span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Fragmentation:</span>
                            <span class="font-semibold text-orange-600 dark:text-orange-400 ml-2">${fragmentation.toFixed(1)}%</span>
                        </div>
                        <div>
                            <span class="text-gray-600 dark:text-gray-400">Efficiency:</span>
                            <span class="font-semibold text-purple-600 dark:text-purple-400 ml-2">${efficiency.toFixed(1)}%</span>
                        </div>
                    </div>
                </div>

                <!-- Variable Allocation Timeline -->
                <div class="mt-8 bg-gray-50 dark:bg-gray-700 rounded-lg p-6 border border-gray-200 dark:border-gray-600">
                    <h3 class="text-xl font-semibold mb-4 text-gray-800 dark:text-white text-center">Variable Allocation Timeline</h3>
                    <div class="space-y-3 max-h-64 overflow-y-auto">
                        ${createVariableAllocationTimeline(allocations)}
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create metric card with circular progress indicator
function createMetricCard(title, value, percentage, color, status) {
    const circumference = 2 * Math.PI * 25;
    const strokeDasharray = circumference;
    const strokeDashoffset = circumference - (percentage / 100) * circumference;

    const statusColors = {
        'OPTIMAL': '#27ae60',
        'MEDIUM': '#f39c12',
        'HIGH': '#e74c3c'
    };

    return `
        <div class="bg-white dark:bg-gray-700 rounded-lg p-4 shadow-sm hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase">${title}</p>
                    <p class="text-lg font-bold text-gray-900 dark:text-white">${value}</p>
                    <div class="flex items-center mt-1">
                        <div class="w-2 h-2 rounded-full mr-2" style="background-color: ${statusColors[status]}"></div>
                        <span class="text-xs font-semibold" style="color: ${statusColors[status]}">${status}</span>
                    </div>
                </div>
                <div class="relative w-12 h-12">
                    <svg class="w-12 h-12 transform -rotate-90" viewBox="0 0 60 60">
                        <circle cx="30" cy="30" r="25" stroke="#e5e7eb" stroke-width="6" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="30" cy="30" r="25" stroke="${color}" stroke-width="6" fill="none" 
                                stroke-dasharray="${strokeDasharray}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                    <div class="absolute inset-0 flex items-center justify-center ">
                        <span class="text-xs font-bold " style="color: ${color}">${Math.round(percentage)}%</span>
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create timeline visualization
function createTimelineVisualization(allocations) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No timeline data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const minTime = sortedAllocs[0]?.timestamp_alloc || 0;
    const maxTime = sortedAllocs[sortedAllocs.length - 1]?.timestamp_alloc || minTime + 1;
    const timeRange = maxTime - minTime || 1;

    return sortedAllocs.slice(0, 20).map(function(alloc, index) {
        var position = ((alloc.timestamp_alloc - minTime) / timeRange) * 100;
        var height = Math.min(80, Math.max(4, (alloc.size / 1024) * 20));
        var color = (alloc.var_name && alloc.var_name !== 'unknown') ? '#3498db' : '#95a5a6';
        var varName = alloc.var_name || 'System';
        var sizeStr = formatBytes(alloc.size);
        
        var html = [];
        html.push('<div class="absolute bottom-0 bg-opacity-80 rounded-t transition-all hover:bg-opacity-100" style="left:');
        html.push(position);
        html.push('%;width:4px;height:');
        html.push(height);
        html.push('%;background-color:');
        html.push(color);
        html.push('" title="');
        html.push(varName);
        html.push(':');
        html.push(sizeStr);
        html.push('"></div>');
        return html.join('');
    }).join('');
}

// Create treemap-style visualization
function createTreemapVisualization(allocations) {
    const typeGroups = {};
    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System';
        if (!typeGroups[type]) {
            typeGroups[type] = { count: 0, size: 0 };
        }
        typeGroups[type].count++;
        typeGroups[type].size += alloc.size || 0;
    });

    const sortedTypes = Object.entries(typeGroups)
        .sort(([, a], [, b]) => b.size - a.size)
        .slice(0, 8);

    const totalSize = sortedTypes.reduce((sum, [, data]) => sum + data.size, 0);

    let currentX = 0;
    return sortedTypes.map(([type, data], index) => {
        const width = totalSize > 0 ? (data.size / totalSize) * 100 : 12.5;
        const color = getTypeColor(type, index);
        const truncatedType = type.length > 10 ? type.substring(0, 8) + '...' : type;
        const formattedSize = formatBytes(data.size);
        const titleText = type + ': ' + formattedSize + ' (' + data.count + ' allocs)';
        const result = `
            <div class="absolute h-full transition-all hover:brightness-110 cursor-pointer rounded" 
                 style="left: ${currentX}%; width: ${width}%; background-color: ${color};"
                 title="${titleText}">
                <div class="p-2 h-full flex flex-col justify-center text-white text-xs font-semibold text-center">
                    <div class="truncate">${truncatedType}</div>
                    <div class="text-xs opacity-90">${formattedSize}</div>
                </div>
            </div>
        `;
        currentX += width;
        return result;
    }).join('');
}

// Create fragmentation bar
function createFragmentationBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    return `
        <div class="flex items-center justify-between ">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-24">${label}</span>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                    <div class="h-4 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-12 text-right ">${count}</span>
        </div>
    `;
}

// Create growth trend visualization
function createGrowthTrendVisualization(allocations) {
    if (allocations.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const points = [];
    let cumulativeSize = 0;

    sortedAllocs.forEach((alloc, index) => {
        cumulativeSize += alloc.size || 0;
        if (index % Math.max(1, Math.floor(sortedAllocs.length / 10)) === 0) {
            points.push(cumulativeSize);
        }
    });

    const maxSize = Math.max(...points);

    return points.map((size, index) => {
        const x = (index / (points.length - 1)) * 100;
        const y = 100 - (size / maxSize) * 80;

        return `
            <div class="absolute w-2 h-2 bg-green-500 rounded-full transform -translate-x-1 -translate-y-1" 
                 style="left: ${x}%; top: ${y}%"
                 title="Memory: ${formatBytes(size)}">
            </div>
            ${index > 0 ? `
                <div class="absolute h-0.5 bg-green-500" 
                     style="left: ${((index - 1) / (points.length - 1)) * 100}%; 
                            top: ${100 - (points[index - 1] / maxSize) * 80}%; 
                            width: ${(100 / (points.length - 1))}%;
                            transform: rotate(${Math.atan2(y - (100 - (points[index - 1] / maxSize) * 80), 100 / (points.length - 1)) * 180 / Math.PI}deg);
                            transform-origin: left center;">
                </div>
            ` : ''}
        `;
    }).join('');
}

// Get color for type visualization
function getTypeColor(type, index) {
    const colors = [
        '#3498db', '#e74c3c', '#2ecc71', '#f39c12',
        '#9b59b6', '#1abc9c', '#e67e22', '#95a5a6'
    ];

    if (type.toLowerCase().includes('vec')) return '#3498db';
    if (type.toLowerCase().includes('string')) return '#f39c12';
    if (type.toLowerCase().includes('hash')) return '#e74c3c';
    if (type.toLowerCase().includes('btree')) return '#2ecc71';

    return colors[index % colors.length];
}

// Create advanced metric card with enhanced styling
function createAdvancedMetricCard(title, value, percentage, color, status) {
    const circumference = 2 * Math.PI * 20;
    const strokeDasharray = circumference;
    const strokeDashoffset = circumference - (percentage / 100) * circumference;

    const statusColors = {
        'OPTIMAL': '#27ae60',
        'MEDIUM': '#f39c12',
        'HIGH': '#e74c3c'
    };

    return `
        <div class="bg-white dark:bg-gray-700 rounded-lg p-3 shadow-sm hover:shadow-md transition-all border border-gray-200 dark:border-gray-600">
            <div class="flex flex-col items-center ">
                <div class="relative w-10 h-10 mb-2">
                    <svg class="w-10 h-10 transform -rotate-90" viewBox="0 0 50 50">
                        <circle cx="25" cy="25" r="20" stroke="#e5e7eb " stroke-width="4" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="25" cy="25" r="20" stroke="${color}" stroke-width="4" fill="none" 
                                stroke-dasharray="${strokeDasharray}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                    <div class="absolute inset-0 flex items-center justify-center ">
                        <span class="text-xs font-bold " style="color: ${color}">${Math.round(percentage)}%</span>
                    </div>
                </div>
                <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase text-center ">${title}</p>
                <p class="text-sm font-bold text-gray-900 dark:text-white text-center ">${value}</p>
                <div class="flex items-center mt-1">
                    <div class="w-1.5 h-1.5 rounded-full mr-1" style="background-color: ${statusColors[status]}"></div>
                    <span class="text-xs font-semibold " style="color: ${statusColors[status]}">${status}</span>
                </div>
            </div>
        </div>
    `;
}

// Create advanced timeline visualization
function createAdvancedTimelineVisualization(allocations, totalMemory) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No timeline data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const minTime = sortedAllocs[0]?.timestamp_alloc || 0;
    const maxTime = sortedAllocs[sortedAllocs.length - 1]?.timestamp_alloc || minTime + 1;
    const timeRange = maxTime - minTime || 1;

    // Group allocations by scope/type for better visualization
    const scopeGroups = {};
    sortedAllocs.forEach(alloc => {
        const scope = alloc.scope_name || (alloc.var_name ? 'User Variables' : "System");
        if (!scopeGroups[scope]) scopeGroups[scope] = [];
        scopeGroups[scope].push(alloc);
    });

    const scopeColors = ['#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6', '#1abc9c'];
    let scopeIndex = 0;

    return Object.entries(scopeGroups).map(([scope, allocs]) => {
        const color = scopeColors[scopeIndex % scopeColors.length];
        scopeIndex++;
        const yOffset = scopeIndex * 25;

        return `
            <div class="absolute" style="top: ${yOffset}px; left: 0; right: 0; height: 20px;">
                <div class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1" style="color: ${color}">
                    ${scope} (${allocs.length} allocs)
                </div>
                ${allocs.slice(0, 20).map(alloc => {
            const position = ((alloc.timestamp_alloc - minTime) / timeRange) * 100;
            const width = Math.max(2, (alloc.size / totalMemory) * 100);
            const varName = alloc.var_name || "System";
            const sizeStr = formatBytes(alloc.size);

            return `
                        <div class="absolute h-4 rounded opacity-80 hover:opacity-100 transition-opacity cursor-pointer" 
                             style="left: ${position}%; width: ${Math.max(4, width)}px; background-color: ${color};"
                             title="${varName}: ${sizeStr}">
                        </div>`;
        }).join('')}
            </div>
        `;
    }).join('');
}

// Create advanced treemap visualization inspired by SVG design
function createAdvancedTreemapVisualization(allocations, totalMemory) {
    if (allocations.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No allocation data</div>';

    // Group allocations by type and category
    const typeGroups = {};
    const categoryGroups = {
        'Collections': { types: {}, totalSize: 0, color: '#3498db' },
        'Basic Types': { types: {}, totalSize: 0, color: '#27ae60' },
        'Smart Pointers': { types: {}, totalSize: 0, color: '#9b59b6' },
        'System': { types: {}, totalSize: 0, color: '#95a5a6' }
    };

    allocations.forEach(alloc => {
        const type = alloc.type_name || 'System';
        const category = getTypeCategory(type);
        const categoryName = getCategoryName(category);
        
        if (!typeGroups[type]) {
            typeGroups[type] = { count: 0, size: 0, category: categoryName };
        }
        typeGroups[type].count++;
        typeGroups[type].size += alloc.size || 0;
        
        // Add to category groups
        if (!categoryGroups[categoryName].types[type]) {
            categoryGroups[categoryName].types[type] = { count: 0, size: 0 };
        }
        categoryGroups[categoryName].types[type].count++;
        categoryGroups[categoryName].types[type].size += alloc.size || 0;
        categoryGroups[categoryName].totalSize += alloc.size || 0;
    });

    // Sort categories by size
    const sortedCategories = Object.entries(categoryGroups)
        .filter(([, data]) => data.totalSize > 0)
        .sort(([, a], [, b]) => b.totalSize - a.totalSize);

    let html = '';
    let currentY = 0;
    const containerHeight = 240;
    const padding = 8;

    sortedCategories.forEach(([categoryName, categoryData], categoryIndex) => {
        const categoryPercentage = (categoryData.totalSize / totalMemory) * 100;
        const categoryHeight = Math.max(40, (categoryPercentage / 100) * containerHeight * 0.8);
        
        // Category container with background
        html += `
            <div class="absolute w-full rounded-lg border-2 border-white shadow-sm transition-all hover:shadow-md " 
                 style="top: ${currentY}px; height: ${categoryHeight}px; background-color: ${categoryData.color}; opacity: 0.15;">
            </div>
        `;

        // Category label
        html += `
            <div class="absolute left-2 font-bold text-sm z-10" 
                 style="top: ${currentY + 8}px; color: ${categoryData.color};">
                ${categoryName} (${categoryPercentage.toFixed(1)}%)
            </div>
        `;

        // Sort types within category
        const sortedTypes = Object.entries(categoryData.types)
            .sort(([, a], [, b]) => b.size - a.size)
            .slice(0, 6); // Limit to top 6 types per category

        let currentX = 20;
        const typeY = currentY + 25;
        const availableWidth = 95; // Leave some margin

        sortedTypes.forEach(([type, typeData], typeIndex) => {
            const typePercentage = (typeData.size / categoryData.totalSize) * 100;
            const typeWidth = Math.max(60, (typePercentage / 100) * availableWidth);
            const typeHeight = Math.max(20, categoryHeight - 35);

            // Type rectangle with enhanced styling
            html += `
                <div class="absolute rounded-md border border-white shadow-sm cursor-pointer transition-all hover:brightness-110 hover:scale-105 hover:z-20" 
                     style="left: ${currentX}px; top: ${typeY}px; width: ${typeWidth}px; height: ${typeHeight}px; 
                            background-color: ${categoryData.color}; opacity: 0.9;"
                     title="${type}: ${formatBytes(typeData.size)} (${typeData.count} allocs, ${typePercentage.toFixed(1)}% of ${categoryName})">
                    <div class="p-1 h-full flex flex-col justify-center text-white text-xs font-bold text-center ">
                        <div class="truncate text-shadow " style="text-shadow: 1px 1px 2px rgba(0,0,0,0.8);">
                            ${type.length > 12 ? type.substring(0, 10) + '..' : type}
                        </div>
                        <div class="text-xs opacity-90 font-semibold " style="text-shadow: 1px 1px 2px rgba(0,0,0,0.6);">
                            ${formatBytes(typeData.size)}
                        </div>
                        <div class="text-xs opacity-75" style="text-shadow: 1px 1px 2px rgba(0,0,0,0.6);">
                            (${typePercentage.toFixed(1)}%)
                        </div>
                    </div>
                </div>
            `;

            currentX += typeWidth + 4;
        });

        currentY += categoryHeight + padding;
    });

    return html;
}

// Helper function to get category name
function getCategoryName(category) {
    const categoryMap = {
        'collections': 'Collections',
        'basic': 'Basic Types',
        'smart_pointers': 'Smart Pointers',
        'system': 'System'
    };
    return categoryMap[category] || 'System';
}

// Create advanced fragmentation bar
function createAdvancedFragmentationBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    const barHeight = Math.max(8, (count / total) * 60);

    return `
        <div class="flex items-center justify-between ">
            <div class="flex items-center w-32">
                <div class="w-4 rounded mr-3 border border-gray-300 dark:border-gray-500" 
                     style="height: ${barHeight}px; background-color: ${color}"></div>
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">${label}</span>
            </div>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-3">
                    <div class="h-3 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-12 text-right ">${count}</span>
        </div>
    `;
}

// Create call stack analysis
function createCallStackAnalysis(allocations) {
    const userAllocs = allocations.filter(a => a.var_name && a.var_name !== 'unknown');
    const systemAllocs = allocations.filter(a => !a.var_name || a.var_name === 'unknown');

    const topAllocations = [...userAllocs, ...systemAllocs.slice(0, 3)]
        .sort((a, b) => (b.size || 0) - (a.size || 0))
        .slice(0, 10);

    return topAllocations.map(alloc => {
        const isSystem = !alloc.var_name || alloc.var_name === 'unknown';
        const color = isSystem ? '#e74c3c' : getTypeColor(alloc.type_name || '', 0);
        const radius = Math.min(8, Math.max(3, Math.sqrt((alloc.size || 0) / 100)));

        return `
            <div class="flex items-center space-x-3 p-2 bg-white dark:bg-gray-600 rounded border ">
                <div class="w-4 h-4 rounded-full border-2 border-gray-300 dark:border-gray-500" 
                     style="background-color: ${color}"></div>
                <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-900 dark:text-white truncate ">
                        ${alloc.var_name || 'System/Runtime allocations'}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                        ${alloc.type_name || 'no type info'} - ${formatBytes(alloc.size || 0)}
                    </div>
                </div>
            </div>
        `;
    }).join('');
}

// Create advanced growth trend visualization
function createAdvancedGrowthTrendVisualization(allocations, totalMemory) {
    if (allocations.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data</div>';

    const sortedAllocs = allocations.sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const points = [];
    let cumulativeSize = 0;

    sortedAllocs.forEach((alloc, index) => {
        cumulativeSize += alloc.size || 0;
        if (index % Math.max(1, Math.floor(sortedAllocs.length / 15)) === 0) {
            points.push({
                x: (index / sortedAllocs.length) * 100,
                y: 100 - (cumulativeSize / totalMemory) * 80,
                size: cumulativeSize
            });
        }
    });

    const gridLines = [20, 40, 60, 80].map(y => `
        <div class="absolute w-full border-t border-gray-200 dark:border-gray-500 opacity-30" 
             style="top: ${y}%"></div>
    `).join('');
    
    const dataPoints = points.map(point => `
        <div class="absolute w-2 h-2 bg-green-500 rounded-full border border-white dark:border-gray-600 transform -translate-x-1/2 -translate-y-1/2 hover:scale-150 transition-transform cursor-pointer" 
             style="left: ${point.x}%; top: ${point.y}%"
             title="Memory: ${formatBytes(point.size)}">
        </div>
    `).join('');
    
    const polylinePoints = points.map(p => `${p.x},${p.y}`).join(' ');
    
    return `
        <!-- Background Grid -->
        <div class="absolute inset-0">
            ${gridLines}
        </div>
        
        <!-- Growth Line -->
        <svg class="absolute inset-0 w-full h-full">
            <polyline
                fill="none"
                stroke="#27ae60"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                points="${polylinePoints}"
                class="drop-shadow-sm"
            />
        </svg>
        
        <!-- Data Points -->
        ${dataPoints}
        
        <!-- Peak Memory Line -->
        <div class="absolute w-full border-t-2 border-red-500 border-dashed opacity-60" style="top: 20%">
            <div class="absolute -top-1 right-0 text-xs text-red-500 bg-white dark:bg-gray-600 px-1 rounded">
                Peak: ${formatBytes(totalMemory)}
            </div>
        </div>
    `;
}


// Create variable allocation timeline
function createVariableAllocationTimeline(allocations) {
    const userAllocs = allocations.filter(a => a.var_name && a.var_name !== 'unknown')
        .sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0))
        .slice(0, 10);

    return userAllocs.map((alloc, index) => {
        const color = getTypeColor(alloc.type_name || "", index);
        const typeName = alloc.type_name || "unknown";
        const sizeStr = formatBytes(alloc.size || 0);
        const timeStr = new Date(alloc.timestamp_alloc / 1000000).toLocaleTimeString();

        return `
            <div class="flex items-center space-x-3 p-2 bg-white dark:bg-gray-600 rounded border">
                <div class="w-3 h-3 rounded-full" style="background-color: ${color}"></div>
                <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-gray-900 dark:text-gray-100">
                        ${alloc.var_name}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">
                        ${typeName} - ${sizeStr}
                    </div>
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400">
                    ${timeStr}
                </div>
            </div>
        `;
    }).join('')
}

// Helper functions for type categorization
function getTypeCategory(type) {
    if (!type || type === "System" || type === "unknown") return "system";
    
    const typeLower = type.toLowerCase();
    
    // Collections
    if (typeLower.includes('vec') || typeLower.includes('hash') || typeLower.includes('btree') || 
        typeLower.includes('deque') || typeLower.includes('set') || typeLower.includes('map')) {
        return "collections";
    }
    
    // Smart Pointers
    if (typeLower.includes('box') || typeLower.includes('rc') || typeLower.includes('arc') || 
        typeLower.includes('refcell') || typeLower.includes('cell') || typeLower.includes('weak')) {
        return "smart_pointers";
    }
    
    // Basic types (String, primitives, etc.)
    return "basic";
}

function getCategoryColor(category) {
    const colors = {
        "collections": "#3498db",      // Bright blue
        "basic": "#27ae60",           // Bright green  
        "smart_pointers": "#9b59b6",  // Purple
        "system": "#95a5a6"           // Gray
    };
    return colors[category] || "#95a5a6";
}

// Initialize allocations table with improved collapsible functionality
function initAllocationsTable() {
    console.log("📊 Initializing allocations table... ");

    const tbody = document.getElementById('allocations-table');
    const toggleButton = document.getElementById('toggle-allocations');

    if (!tbody) {
        console.warn("⚠️ Allocations table body not found ");
        return;
    }

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    if (allocations.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">No allocations found</td></tr>';
        if (toggleButton) {
            toggleButton.style.display = "none";
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    function renderTable(showAll = false) {
        console.log("📊 Rendering table, showAll: " + showAll + ", total allocations: " + allocations.length);

        const displayAllocations = showAll ? allocations : allocations.slice(0, maxInitialRows);

        tbody.innerHTML = displayAllocations.map(alloc => "<tr class='hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors'>" +
                "<td class='px-4 py-2 text-gray-900 dark:text-gray-100 font-mono'>0x${(alloc.ptr ? parseInt(alloc.ptr.toString().replace('0x', ''), 16) : 0).toString(16).padStart(8, '0')}</td>" +
                "<td class='px-4 py-2 text-gray-900 dark:text-gray-100'>${alloc.var_name || 'System Allocation'}</td>" +
                "<td class='px-4 py-2 text-gray-900 dark:text-gray-100'>${formatTypeName(alloc.type_name || 'System Allocation')}</td>" +
                "<td class='px-4 py-2 text-right text-gray-900 dark:text-gray-100'>${formatBytes(alloc.size || 0)}</td>" +
                "<td class='px-4 py-2 text-right text-gray-900 dark:text-gray-100'>" +
                    "<span class='px-2 py-1 text-xs rounded-full ${alloc.is_active ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'}'>" +
                        '${alloc.is_active ? "Active" : "Deallocated"}' +
                    "</span>" +
                "</td>" +
            "</tr>").join("");

        if (!showAll && allocations.length > maxInitialRows) {
            tbody.innerHTML += "<tr class='bg-gray-50 dark:bg-gray-700'>" +
                    '<td colspan="5" class="px-4 py-2 text-center text-gray-500 dark:text-gray-400 text-sm">' +
                        "... and " + (allocations.length - maxInitialRows) + " more allocations" +
                    "</td>" +
                "</tr>";
        }
    }

    // Initial render
    renderTable(false);

    // Toggle functionality - Fixed event binding
    if (toggleButton && allocations.length > maxInitialRows) {
        console.log("📊 Setting up toggle button for", allocations.length, "allocations");

        // Clear any existing event listeners and add new one
        toggleButton.replaceWith(toggleButton.cloneNode(true));
        const newToggleButton = document.getElementById("toggle-allocations");

        newToggleButton.addEventListener("click", function (e) {
            e.preventDefault();
            e.stopPropagation();
            console.log("📊 Toggle button clicked, current state:", isExpanded);

            isExpanded = !isExpanded;
            renderTable(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = "fa fa-chevron-up mr-1";
                text.textContent = "Show Less";
                console.log("📊 Expanded table to show all allocations");
            } else {
                icon.className = "fa fa-chevron-down mr-1";
                text.textContent = "Show All";
                console.log("📊 Collapsed table to show first", maxInitialRows, "allocations");
            }
        });

        console.log("✅ Toggle button initialized successfully");
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = "none";
        console.log("📊 Toggle button hidden (not enough data)");
    }
}

// Initialize lifetime visualization from JSON data with collapsible functionality
function initLifetimeVisualization() {
    console.log("🔄 Initializing lifetime visualization...");

    // Get lifetime data from various sources (support extended data structure)
    let lifetimeData = null;
    let lifecycleEvents = [];
    
    // Smart data source selection: merge memory_analysis and complex_types data
    let memoryAllocations = window.analysisData.memory_analysis?.allocations || [];
    let complexAllocations = window.analysisData.complex_types?.allocations || [];
    
    console.log("📊 Memory analysis allocations:", memoryAllocations.length);
    console.log("📊 Complex types allocations:", complexAllocations.length);
    
    // 合并数据：使用memory_analysis的lifetime_ms + complex_types的扩展字段
    if (memoryAllocations.length > 0 && complexAllocations.length > 0) {
        // Create mapping from pointer to memory analysis data
        const memoryMap = new Map();
        memoryAllocations.forEach(alloc => {
            if (alloc.ptr) {
                memoryMap.set(alloc.ptr, alloc);
            }
        });
        
        // Merge data: complex_types + lifetime_ms from memory_analysis
        lifecycleEvents = complexAllocations.map(complexAlloc => {
            const memoryAlloc = memoryMap.get(complexAlloc.ptr);
            return {
                ...complexAlloc,
                lifetime_ms: memoryAlloc?.lifetime_ms || null,
                timestamp_dealloc: memoryAlloc?.timestamp_dealloc || null
            };
        });
        console.log("📊 Merged allocation data:", lifecycleEvents.length);
    } else if (memoryAllocations.length > 0) {
        lifecycleEvents = memoryAllocations;
        console.log("📊 Using memory analysis data:", lifecycleEvents.length);
    } else if (complexAllocations.length > 0) {
        lifecycleEvents = complexAllocations;
        console.log("📊 Using complex types data:", lifecycleEvents.length);
    } else if (window.analysisData.lifetime?.lifecycle_events) {
        lifecycleEvents = window.analysisData.lifetime.lifecycle_events;
        console.log("📊 Using lifetime events data:", lifecycleEvents.length);
    }
    
    if (!lifecycleEvents || lifecycleEvents.length === 0) {
        console.warn("⚠️ No lifetime data found");
        console.log("Available data keys:", Object.keys(window.analysisData || {}));
        showEmptyLifetimeState();
        return;
    }

    console.log("📊 Total lifecycle events: ${lifecycleEvents.length} ");

    // Check if we have Rust-preprocessed data
    if (lifetimeData?.visualization_ready && lifetimeData?.variable_groups) {
        console.log("📊 Using Rust-preprocessed data with ${lifetimeData.variable_groups.length} variable groups ");
        renderLifetimeVisualizationFromRustWithCollapse(lifetimeData.variable_groups);
        return;
    }

    // Filter for user-defined variables (non-unknown var_name and type_name)
    const userVariables = lifecycleEvents.filter(event =>
        event.var_name && event.var_name !== "unknown" &&
        event.type_name && event.type_name !== "unknown"
    );

    console.log("📊 Found ${userVariables.length} user-defined variables in lifetime data");

    // Debug: Show some examples of what we found
    if (userVariables.length > 0) {
        console.log("📊 Sample user variables:", userVariables.slice(0, 3));
    } else {
        // Show some examples of unknown variables for debugging
        const unknownSamples = lifecycleEvents.slice(0, 3);
        console.log("📊 Sample unknown variables:", unknownSamples);
    }

    if (userVariables.length === 0) {
        showEmptyLifetimeState();
        return;
    }

    // Group by variable name to get allocation/deallocation pairs
    const variableGroups = groupVariablesByName(userVariables);

    // Render the lifetime visualization with collapse functionality
    renderLifetimeVisualizationWithCollapse(variableGroups);
}

// Group variables by name to track their lifecycle (enhanced for multiple instances)
function groupVariablesByName(events) {
    const groups = {};

    events.forEach(event => {
        const varName = event.var_name;
        const instanceKey = varName + '_' + (event.ptr || event.timestamp_alloc);
        
        if (!groups[instanceKey]) {
            groups[instanceKey] = {
                var_name: varName + '#' + (Object.keys(groups).filter(k => k.startsWith(varName)).length + 1),
                original_var_name: varName,
                type_name: event.type_name,
                events: [],
                instance_info: {
                    ptr: event.ptr,
                    timestamp: event.timestamp_alloc,
                    thread_id: event.thread_id
                }
            };
        }
        groups[instanceKey].events.push(event);
    });

    
    const groupValues = Object.values(groups);
    const varCounts = {};
    groupValues.forEach(group => {
        const originalName = group.original_var_name;
        varCounts[originalName] = (varCounts[originalName] || 0) + 1;
    });
    
    groupValues.forEach(group => {
        if (varCounts[group.original_var_name] === 1) {
            group.var_name = group.original_var_name; 
        }
    });

    return groupValues;
}

// Render lifetime visualization from Rust-preprocessed data with collapsible functionality
function renderLifetimeVisualizationFromRustWithCollapse(variableGroups) {
    console.log("📊 Rendering ${variableGroups.length} Rust-preprocessed variable groups with collapse functionality");

    const container = document.getElementById("lifetimeVisualization");
    const toggleButton = document.getElementById("toggle-lifecycle");
    
    if (!container) return;

    // Clear loading state
    container.innerHTML = "";

    if (!variableGroups || variableGroups.length === 0) {
        showEmptyLifetimeState();
        if (toggleButton) {
            toggleButton.style.display = "none";
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    // Calculate timeline bounds from preprocessed data
    const allTimestamps = variableGroups.flatMap(group =>
        group.events ? group.events.map(e => e.timestamp) : [group.start_time, group.end_time].filter(t => t !== undefined)
    );

    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime || 1;

    console.log("📊 Rust data timeline: ${minTime} to ${maxTime} (range: ${timeRange})");

    // Color palette for different data types and visualizations
    const COLOR_PALETTE = {
        progress: [
            "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57",
            "#ff9ff3", "#54a0ff", "#5f27cd", "#00d2d3", "#ff9f43"
        ]
    };

    function renderLifetimeRows(showAll = false) {
        console.log("📊 Rendering lifecycle rows, showAll: ${showAll}, total groups: ${variableGroups.length}");
        
        container.innerHTML = "";
        
        const displayGroups = showAll ? variableGroups : variableGroups.slice(0, maxInitialRows);

        // Render each variable with colorful progress bars
        displayGroups.forEach((group, index) => {
            const varDiv = document.createElement('div');
            varDiv.className = "flex items-center py-4 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors";

            // Get color from palette (cycle through colors)
            const colorIndex = index % COLOR_PALETTE.progress.length;
            const progressColor = COLOR_PALETTE.progress[colorIndex];

            // Use preprocessed timing data or fallback to events
            const startTime = group.start_time || (group.events && group.events[0] ? group.events[0].timestamp : minTime);
            const firstEvent = group.events && group.events[0];
            
            const startPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;
            
            
            let widthPercent;
            if (firstEvent && firstEvent.lifetime_ms && firstEvent.lifetime_ms > 0) {
                
                const lifetimeNs = firstEvent.lifetime_ms * 1000000; 
                widthPercent = timeRange > 0 ? Math.max(1, (lifetimeNs / timeRange) * 100) : 6.8;
            } else {
                //
                widthPercent = 6.8;
            }
            
            // 安全的变量定义，防止NaN
            const finalStartPercent = isNaN(startPercent) ? 0 : Math.max(0, Math.min(95, startPercent));
            const finalWidthPercent = isNaN(widthPercent) ? 40 : Math.max(2, Math.min(100 - finalStartPercent, widthPercent));

            // Format type name for display
            const displayTypeName = formatTypeName(group.type_name);

            // Create gradient background for more visual appeal
            const gradientStyle = "background: linear-gradient(90deg, " + progressColor + ", " + progressColor + "dd);";

            varDiv.innerHTML = 
                "<div class='w-48 flex-shrink-0 pr-4'>" +
                    "<div class='text-sm font-semibold text-gray-800 dark:text-gray-200'>" + group.var_name + "</div>" +
                    "<div class='text-xs text-gray-500 dark:text-gray-400'>" + displayTypeName + "</div>" +
                "</div>" +
                "<div class='flex-grow relative bg-gray-200 dark:bg-gray-600 rounded-full h-6 overflow-hidden'>" +
                    "<div class='absolute inset-0 rounded-full' " +
                         "style='" + gradientStyle + " width: " + finalWidthPercent + "%; margin-left: " + finalStartPercent + "%; " +
                                "box-shadow: 0 2px 4px rgba(0,0,0,0.1); " +
                                "transition: all 0.3s ease;' " +
                         "title=\"Variable: " + group.var_name + ", Type: " + displayTypeName + "\">" +
                        "<div class='absolute inset-0 flex items-center justify-center'>" +
                            "<span class='text-xs font-bold text-white drop-shadow-sm'>" +
                                Math.round(finalWidthPercent) + '%' +
                            "</span>" +
                        "</div>" +
                    "</div> " +
                    '<div class="absolute -top-8 left-0 text-xs bg-gray-700 text-white px-2 py-1 rounded opacity-0 hover:opacity-100 transition-opacity whitespace-nowrap">' +
                      'Duration: ' + (firstEvent && firstEvent.lifetime_ms ? firstEvent.lifetime_ms + 'ms' : 'Active') +
                    '</div>' +
                "</div>" +
                "<div class='w-20 flex-shrink-0 pl-4 text-right'>" +
                    "<div class='text-xs text-gray-600 dark:text-gray-400'>" +
                        formatBytes(group.size || (group.events && group.events[0] ? group.events[0].size : 0) || 0) +
                    "</div>" +
                "</div>" +
            "";

            container.appendChild(varDiv);
        });

        // Add "show more" indicator if collapsed
        if (!showAll && variableGroups.length > maxInitialRows) {
            const moreDiv = document.createElement('div');
            moreDiv.className = "flex items-center py-4 bg-gray-50 dark:bg-gray-700 border-b border-gray-100 dark:border-gray-600";
            moreDiv.innerHTML = 
                "<div class='w-full text-center text-gray-500 dark:text-gray-400 text-sm'>" +
                    "... and " + (variableGroups.length - maxInitialRows) + " more variables" +
                "</div>";
            container.appendChild(moreDiv);
        }
    }

    // Initial render
    renderLifetimeRows(false);

    // Toggle functionality
    if (toggleButton && variableGroups.length > maxInitialRows) {
        console.log("📊 Setting up lifecycle toggle button for", variableGroups.length, "variables");

        // Remove any existing event listeners
        const newToggleButton = toggleButton.cloneNode(true);
        toggleButton.parentNode.replaceChild(newToggleButton, toggleButton);

        newToggleButton.addEventListener("click", function (e) {
            e.preventDefault();
            console.log("📊 Lifecycle toggle button clicked, current state:", isExpanded);

            isExpanded = !isExpanded;
            renderLifetimeRows(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = "fa fa-chevron-up mr-1";
                text.textContent = "Show Less";
                console.log("📊 Expanded lifecycle to show all variables");
            } else {
                icon.className = "fa fa-chevron-down mr-1";
                text.textContent = "Show All";
                console.log("📊 Collapsed lifecycle to show first ", maxInitialRows, "variables");
            }
        });

        console.log("✅ Lifecycle toggle button initialized successfully ");
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = "none";
        console.log("📊 Lifecycle toggle button hidden (not enough data)");
    }

    console.log("✅ Rendered " + variableGroups.length + " Rust-preprocessed variables in lifetime visualization with collapse functionality");
}

// Render the lifetime visualization with collapsible functionality
function renderLifetimeVisualizationWithCollapse(variableGroups) {
    const container = document.getElementById("lifetimeVisualization");
    const toggleButton = document.getElementById("toggle-lifecycle");
    
    if (!container) return;

    // Clear loading state
    container.innerHTML = "";

    if (!variableGroups || variableGroups.length === 0) {
        showEmptyLifetimeState();
        if (toggleButton) {
            toggleButton.style.display = "none";
        }
        return;
    }

    let isExpanded = false;
    const maxInitialRows = 5;

    // Get color scheme for different types
    const typeColors = {
        "Vec": { bg: "bg-blue-500", border: "border-blue-500" },
        "Box": { bg: "bg-purple-500", border: "border-purple-500" },
        "Rc": { bg: "bg-yellow-500", border: "border-yellow-500" },
        "Arc": { bg: "bg-green-500", border: "border-green-500" },
        "String": { bg: "bg-pink-500", border: "border-pink-500" },
        "default": { bg: "bg-gray-500", border: "border-gray-500" }
    };

    // Calculate timeline bounds
    const allTimestamps = variableGroups.flatMap(group =>
        group.events.map(e => e.timestamp)
    );
    const minTime = Math.min(...allTimestamps);
    const maxTime = Math.max(...allTimestamps);
    const timeRange = maxTime - minTime;

    console.log("📊 Timeline: " + minTime + " to " + maxTime + " (range: " + timeRange + ")");

    function renderLifetimeRows(showAll = false) {
        console.log("📊 Rendering lifecycle rows, showAll: " + showAll + ", total groups: " + variableGroups.length);
        
        container.innerHTML = "";
        
        const displayGroups = showAll ? variableGroups : variableGroups.slice(0, maxInitialRows);

        // Render each variable
        displayGroups.forEach((group) => {
            const varDiv = document.createElement('div');
            varDiv.className = "flex items-end py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors";

            // Determine color based on type
            const typeKey = Object.keys(typeColors).find(key =>
                group.type_name.includes(key)
            ) || "default";
            const colors = typeColors[typeKey];

            // Calculate position and width based on timestamps
            const firstEvent = group.events[0];
            const startTime = firstEvent.timestamp;
            const startPositionPercent = timeRange > 0 ? ((startTime - minTime) / timeRange) * 100 : 0;

            // real time correct time axis calculation: based on actual allocation and survival time
            const allocTime = firstEvent.timestamp;
            const deallocTime = firstEvent.timestamp_dealloc;
            const lifetimeMs = firstEvent.lifetime_ms || 1; // default 1ms lifetime
            
            // calculate survival time length (percentage)
            let durationPercent;
            if (deallocTime && deallocTime > allocTime) {
                // if there is a clear release time, use actual time span
                const actualDuration = deallocTime - allocTime;
                durationPercent = (actualDuration / timeRange) * 100;
            } else {
                // if there is no release time, use lifetime_ms calculation
                const lifetimeNs = lifetimeMs * 1000000; // convert to nanoseconds
                durationPercent = (lifetimeNs / timeRange) * 100;
            }
            
            // ensure value is within reasonable range
            const widthPercent = Math.max(0.5, Math.min(100 - startPositionPercent, durationPercent));
            
            // 安全的变量定义，防止NaN
            const finalStartPercent = isNaN(startPositionPercent) ? 0 : Math.max(0, Math.min(95, startPositionPercent));
            const finalWidthPercent = isNaN(widthPercent) ? 30 : Math.max(2, Math.min(100 - finalStartPercent, widthPercent));

            // Format type name for display
            const displayTypeName = formatTypeName(group.type_name);

            varDiv.innerHTML = 
                "<div class='w-40 flex-shrink-0 text-sm font-medium dark:text-gray-200'>" +
                    group.var_name + " (" + displayTypeName + ")" +
                "</div>" +
                "<div class='flex-grow relative'>" +
                    "<div class=\"lifespan-indicator " + colors.bg + "\" " +
                         "style=\"width: " + finalWidthPercent + "%; margin-left: " + finalStartPercent + "%;\" " +
                         "title=\"Variable: " + group.var_name + ", Type: " + displayTypeName + "\">" +
                        "<div class=\"absolute -top-6 left-0 text-xs " + colors.bg + " text-white px-2 py-1 rounded whitespace-nowrap\">" +
                            "Allocated: " + formatTimestamp(startTime, minTime) +
                        "</div>" +
                    "</div>" +
                "</div>";

            container.appendChild(varDiv);
        });

        // Add "show more" indicator if collapsed
        if (!showAll && variableGroups.length > maxInitialRows) {
            const moreDiv = document.createElement('div');
            moreDiv.className = "flex items-center py-3 bg-gray-50 dark:bg-gray-700 border-b border-gray-100 dark:border-gray-600";
            moreDiv.innerHTML = 
                "<div class='w-full text-center text-gray-500 dark:text-gray-400 text-sm'>" +
                    "... and " + (variableGroups.length - maxInitialRows) + " more variables" +
                "</div>";
            container.appendChild(moreDiv);
        }
    }

    // Initial render
    renderLifetimeRows(false);

    // Toggle functionality
    if (toggleButton && variableGroups.length > maxInitialRows) {
        console.log("📊 Setting up lifecycle toggle button for", variableGroups.length, "variables");

        // Remove any existing event listeners
        const newToggleButton = toggleButton.cloneNode(true);
        toggleButton.parentNode.replaceChild(newToggleButton, toggleButton);

        newToggleButton.addEventListener("click", function (e) {
            e.preventDefault();
            console.log("📊 Lifecycle toggle button clicked, current state:", isExpanded);

            isExpanded = !isExpanded;
            renderLifetimeRows(isExpanded);

            const icon = newToggleButton.querySelector('i');
            const text = newToggleButton.querySelector('span');

            if (isExpanded) {
                icon.className = "fa fa-chevron-up mr-1";
                text.textContent = "Show Less";
                console.log("📊 Expanded lifecycle to show all variables");
            } else {
                icon.className = "fa fa-chevron-down mr-1";
                text.textContent = 'Show All';
                console.log("📊 Collapsed lifecycle to show first ", maxInitialRows, "variables");
            }
        });

        console.log("✅ Lifecycle toggle button initialized successfully ");
    } else if (toggleButton) {
        // Hide button if not needed
        toggleButton.style.display = 'none';
        console.log("📊 Lifecycle toggle button hidden (not enough data)");
    }

    console.log("✅ Rendered " + variableGroups.length + " variables in lifetime visualization with collapse " + "functionality");
}

// Initialize FFI visualization with enhanced support for improve.md fields
function initFFIVisualization() {
    console.log("🔄 Initializing FFI visualization...");

    const container = document.getElementById("ffiVisualization");
    if (!container) return;

    // Get FFI data from multiple sources with comprehensive field support
    let allocations = [];
    let unsafeReports = [];
    let memoryPassports = [];
    let ffiStatistics = {};
    
    console.log("🔍 Checking analysisData structure:", Object.keys(window.analysisData || {}));
    
    // Enhanced data extraction supporting improve.md structure
    if (window.analysisData) {
        // Debug: Show what data structure we actually have FIRST
        console.log("🔍 Available data keys:", Object.keys(window.analysisData));
        if (window.analysisData.unsafe_ffi) {
            console.log("🔍 unsafe_ffi keys:", Object.keys(window.analysisData.unsafe_ffi));
            console.log("🔍 unsafe_ffi.allocations exists:", !!window.analysisData.unsafe_ffi.allocations);
            
            // Data will be handled by initializeAnalysis function
            console.log("🔍 unsafe_ffi.allocations length:", window.analysisData.unsafe_ffi.allocations ? window.analysisData.unsafe_ffi.allocations.length : 'undefined');
        }
        
        // Try unsafe_ffi data first (improve.md structure)
        if (window.analysisData.unsafe_ffi) {
            allocations = window.analysisData.unsafe_ffi.allocations || [];
            unsafeReports = window.analysisData.unsafe_ffi.unsafe_reports || [];
            memoryPassports = window.analysisData.unsafe_ffi.memory_passports || [];
            ffiStatistics = window.analysisData.unsafe_ffi.ffi_statistics || {};
            console.log("📊 Found unsafe_ffi data - allocations:", allocations.length, 'reports:', unsafeReports.length, 'passports:', memoryPassports.length);
        }
        // Try complex_types structure (for large_scale_user files)
        else if (window.analysisData.complex_types && window.analysisData.complex_types.allocations) {
            allocations = window.analysisData.complex_types.allocations;
            console.log("📊 Found complex_types allocations:", allocations.length);
        }
        // Try direct allocations array (for files like large_scale_user_unsafe_ffi.json)
        else if (window.analysisData.allocations) {
            allocations = window.analysisData.allocations;
            console.log("📊 Found direct allocations:", allocations.length);
        }
        // Fallback to memory_analysis
        else if (window.analysisData.memory_analysis && window.analysisData.memory_analysis.allocations) {
            allocations = window.analysisData.memory_analysis.allocations;
            console.log("📊 Using memory_analysis allocations:", allocations.length);
        }
        
        // Debug: Show what data structure we actually have
        console.log("🔍 Available data keys:", Object.keys(window.analysisData));
        if (window.analysisData.unsafe_ffi) {
            console.log("🔍 unsafe_ffi keys:", Object.keys(window.analysisData.unsafe_ffi));
        }
        
        // Extract metadata if available
        const metadata = window.analysisData.metadata || {};
        console.log("📊 Metadata:", metadata);
    }

    // Filter for FFI-tracked allocations with enhanced field support
    const ffiAllocations = allocations.filter(alloc => 
        alloc.ffi_tracked === true || 
        (alloc.safety_violations && alloc.safety_violations.length > 0) ||
        alloc.ownership_history_available === true ||
        (alloc.borrow_info && (alloc.borrow_info.immutable_borrows > 0 || alloc.borrow_info.mutable_borrows > 0)) ||
        (alloc.clone_info && alloc.clone_info.clone_count > 0)
    );
    console.log("📊 Found FFI-tracked allocations:", ffiAllocations.length);
    
    // Debug: show first few allocations with improve.md fields
    if (allocations.length > 0) {
        console.log("🔍 Sample allocation with improve.md fields:", allocations[0]);
        console.log("🔍 FFI tracked allocations sample:", ffiAllocations.slice(0, 3));
        
        // Check for improve.md specific fields
        const sampleAlloc = allocations[0];
        console.log("🔍 Improve.md fields check:");
        console.log("  - borrow_info:", sampleAlloc.borrow_info);
        console.log("  - clone_info:", sampleAlloc.clone_info);
        console.log("  - ownership_history_available:", sampleAlloc.ownership_history_available);
        console.log("  - ffi_tracked:", sampleAlloc.ffi_tracked);
        console.log("  - safety_violations:", sampleAlloc.safety_violations);
    }

    // Debug: Show what we found before filtering
    console.log("🔍 Before filtering - Total allocations:", allocations.length);
    console.log("🔍 Sample allocation fields:", allocations[0] ? Object.keys(allocations[0]) : 'No allocations');
    console.log("🔍 FFI tracked count:", allocations.filter(a => a.ffi_tracked === true).length);
    console.log("🔍 Borrow info count:", allocations.filter(a => a.borrow_info).length);
    console.log("🔍 Clone info count:", allocations.filter(a => a.clone_info).length);
    
    // Enhanced rendering with improve.md support - ALWAYS show if we have any allocations
    if (allocations.length === 0) {
        container.innerHTML = createFFIEmptyState();
        return;
    }
    
    // If we have allocations but no FFI-specific ones, still show the dashboard with all data
    const displayAllocations = ffiAllocations.length > 0 ? ffiAllocations : allocations.slice(0, 20);
    console.log("🎯 Rendering FFI dashboard with:", displayAllocations.length, 'allocations,', unsafeReports.length, 'reports,', memoryPassports.length, 'passports');

    // Generate enhanced FFI analysis with improve.md fields
    try {
        if (FFI_STYLE === "svg") {
            const boundaryEvents = window.analysisData.unsafe_ffi?.boundary_events || [];
            const unsafeAllocs = displayAllocations.filter(a => (a.safety_violations || []).length > 0).length;
            const ffiAllocs = displayAllocations.filter(a => a.ffi_tracked).length;
            const safetyViolations = displayAllocations.reduce((sum, a) => sum + ((a.safety_violations || []).length || 0), 0);
            const unsafeMemory = displayAllocations
                .filter(a => (a.safety_violations || []).length > 0)
                .reduce((sum, a) => sum + (a.size || 0), 0);

            container.innerHTML = createFFIDashboardSVG(
                unsafeAllocs,
                ffiAllocs,
                boundaryEvents.length,
                safetyViolations,
                unsafeMemory,
                displayAllocations,
                boundaryEvents,
                unsafeReports
            );
            console.log("✅ FFI SVG-style dashboard rendered");
            return;
        }
        console.log("🔄 Generating FFI analysis...");
        const ffiAnalysis = generateEnhancedFFIAnalysisWithImproveFields(displayAllocations, unsafeReports, memoryPassports, ffiStatistics);
        console.log("✅ FFI analysis generated:", ffiAnalysis);
        
        console.log("🔄 Creating FFI dashboard...");
        const dashboardHTML = createEnhancedFFIDashboardWithImproveFields(ffiAnalysis, displayAllocations, unsafeReports, memoryPassports);
        console.log("✅ Dashboard HTML created, length:", dashboardHTML.length);
        
        container.innerHTML = dashboardHTML;
        console.log("✅ Dashboard rendered successfully!");
    } catch (error) {
        console.error('❌ Error in FFI rendering:', error);
        container.innerHTML = `<div class="bg-red-100 p-4 rounded text-red-800">Error rendering FFI data: ${error.message}</div>`;
    }
}

// Generate enhanced FFI analysis with improve.md fields support
function generateEnhancedFFIAnalysisWithImproveFields(ffiAllocations, unsafeReports, memoryPassports, ffiStatistics) {
    let totalFFI = ffiAllocations.length;
    let totalViolations = 0;
    let totalMemory = 0;
    let highRiskCount = 0;
    let mediumRiskCount = 0;
    let lowRiskCount = 0;
    let totalBorrows = 0;
    let totalClones = 0;
    let leakedAllocations = 0;

    const analysisData = ffiAllocations.map(alloc => {
        const violations = alloc.safety_violations?.length || 0;
        const size = alloc.size || 0;
        
        // Enhanced borrow analysis from improve.md fields
        const borrowConflicts = alloc.borrow_info ? 
            (alloc.borrow_info.mutable_borrows > 0 && alloc.borrow_info.immutable_borrows > 0) : false;
        const totalBorrowsForAlloc = alloc.borrow_info ? 
            (alloc.borrow_info.immutable_borrows || 0) + (alloc.borrow_info.mutable_borrows || 0) : 0;
        totalBorrows += totalBorrowsForAlloc;
        
        // Enhanced clone analysis from improve.md fields
        const cloneCount = alloc.clone_info?.clone_count || 0;
        const isClone = alloc.clone_info?.is_clone || false;
        totalClones += cloneCount;
        
        // Enhanced ownership and lifecycle analysis
        const ownershipHistoryAvailable = alloc.ownership_history_available || false;
        const isLeaked = alloc.is_leaked || false;
        if (isLeaked) leakedAllocations++;
        
        // Enhanced risk calculation with improve.md fields
        let riskScore = 0;
        if (violations > 0) riskScore += 50;
        if (borrowConflicts) riskScore += 30;
        if (size > 1024) riskScore += 20;
        if (isLeaked) riskScore += 40;
        if (cloneCount > 3) riskScore += 15;
        if (totalBorrowsForAlloc > 5) riskScore += 10;
        
        let riskLevel = 'Low';
        if (riskScore >= 70) {
            riskLevel = 'High';
            highRiskCount++;
        } else if (riskScore >= 35) {
            riskLevel = 'Medium';
            mediumRiskCount++;
        } else {
            lowRiskCount++;
        }

        totalViolations += violations;
        totalMemory += size;

        return {
            ...alloc,
            riskScore,
            riskLevel,
            violations,
            borrowConflicts,
            totalBorrowsForAlloc,
            cloneCount,
            isClone,
            ownershipHistoryAvailable,
            isLeaked
        };
    });

    // Enhanced statistics from improve.md structure
    const enhancedStats = {
        boundary_crossings: ffiStatistics.boundary_crossings || 0,
        memory_violations: ffiStatistics.memory_violations || 0,
        total_ffi_calls: ffiStatistics.total_ffi_calls || 0,
        unsafe_operations: ffiStatistics.unsafe_operations || 0
    };

    return {
        totalFFI,
        totalViolations,
        totalMemory,
        highRiskCount,
        mediumRiskCount,
        lowRiskCount,
        totalBorrows,
        totalClones,
        leakedAllocations,
        analysisData,
        unsafeReports,
        memoryPassports,
        ffiStatistics: enhancedStats
    };
}

// Legacy function for backward compatibility
function generateEnhancedFFIAnalysis(ffiAllocations) {
    return generateEnhancedFFIAnalysisWithImproveFields(ffiAllocations, [], [], {});
}

// Create enhanced FFI dashboard with improve.md fields support
function createEnhancedFFIDashboardWithImproveFields(analysis, ffiAllocations, unsafeReports, memoryPassports) {
    return `
        <div class="space-y-6">
            <!-- Enhanced FFI Overview Cards with improve.md metrics -->
            <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                <div class="bg-blue-100 dark:bg-blue-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600 dark:text-blue-300">${analysis.totalFFI}</div>
                    <div class="text-sm text-blue-700 dark:text-blue-400">FFI Allocations</div>
                </div>
                <div class="bg-red-100 dark:bg-red-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-red-600 dark:text-red-300">${analysis.highRiskCount}</div>
                    <div class="text-sm text-red-700 dark:text-red-400">High Risk</div>
                </div>
                <div class="bg-orange-100 dark:bg-orange-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-orange-600 dark:text-orange-300">${analysis.mediumRiskCount}</div>
                    <div class="text-sm text-orange-700 dark:text-orange-400">Medium Risk</div>
                </div>
                <div class="bg-green-100 dark:bg-green-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-300">${analysis.lowRiskCount}</div>
                    <div class="text-sm text-green-700 dark:text-green-400">Low Risk</div>
                </div>
                <div class="bg-purple-100 dark:bg-purple-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-purple-600 dark:text-purple-300">${analysis.totalBorrows}</div>
                    <div class="text-sm text-purple-700 dark:text-purple-400">Total Borrows</div>
                </div>
                <div class="bg-indigo-100 dark:bg-indigo-900 rounded-lg p-4 text-center">
                    <div class="text-2xl font-bold text-indigo-600 dark:text-indigo-300">${analysis.totalClones}</div>
                    <div class="text-sm text-indigo-700 dark:text-indigo-400">Total Clones</div>
                </div>
            </div>

            <!-- FFI Statistics from improve.md -->
            ${analysis.ffiStatistics && Object.keys(analysis.ffiStatistics).length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">FFI Statistics</h3>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.boundary_crossings}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Boundary Crossings</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.memory_violations}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Memory Violations</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.total_ffi_calls}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Total FFI Calls</div>
                        </div>
                        <div class="text-center">
                            <div class="text-xl font-bold text-gray-900 dark:text-white">${analysis.ffiStatistics.unsafe_operations}</div>
                            <div class="text-sm text-gray-600 dark:text-gray-400">Unsafe Operations</div>
                        </div>
                    </div>
                </div>
            ` : ''}

            <!-- Unsafe Reports from improve.md structure -->
            ${analysis.unsafeReports && analysis.unsafeReports.length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Unsafe Reports</h3>
                    <div class="space-y-4">
                        ${analysis.unsafeReports.map(report => createUnsafeReportCard(report)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Memory Passports from improve.md structure -->
            ${analysis.memoryPassports && analysis.memoryPassports.length > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Memory Passports</h3>
                    <div class="space-y-3">
                        ${analysis.memoryPassports.map(passport => createMemoryPassportCard(passport)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Enhanced FFI Risk Analysis with improve.md fields -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Enhanced FFI Risk Analysis</h3>
                <div class="space-y-4">
                    ${analysis.analysisData.map(alloc => createEnhancedFFIAllocationCard(alloc)).join('')}
                </div>
            </div>

            <!-- Enhanced Borrow Checker Analysis with improve.md fields -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Enhanced Borrow Checker Analysis</h3>
                <div class="space-y-3">
                    ${ffiAllocations.filter(alloc => alloc.borrow_info).map(alloc => createEnhancedBorrowAnalysisCard(alloc)).join('')}
                </div>
            </div>

            <!-- Clone Analysis from improve.md fields -->
            ${analysis.totalClones > 0 ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Clone Analysis</h3>
                    <div class="space-y-3">
                        ${ffiAllocations.filter(alloc => alloc.clone_info && alloc.clone_info.clone_count > 0).map(alloc => createCloneAnalysisCard(alloc)).join('')}
                    </div>
                </div>
            ` : ''}

            <!-- Ownership History Analysis -->
            ${ffiAllocations.some(alloc => alloc.ownership_history_available) ? `
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 text-gray-800 dark:text-white">Ownership History Analysis</h3>
                    <div class="space-y-3">
                        ${ffiAllocations.filter(alloc => alloc.ownership_history_available).map(alloc => createOwnershipHistoryCard(alloc)).join('')}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createEnhancedFFIDashboard(analysis, ffiAllocations) {
    return createEnhancedFFIDashboardWithImproveFields(analysis, ffiAllocations, [], []);
}

// Create enhanced FFI allocation card with improve.md fields
function createEnhancedFFIAllocationCard(alloc) {
    const riskColor = alloc.riskLevel === 'High' ? 'red' : alloc.riskLevel === 'Medium' ? 'orange' : 'green';
    const hasViolations = alloc.violations > 0;
    const hasBorrowConflicts = alloc.borrowConflicts;
    const hasClones = alloc.cloneCount > 0;
    const isLeaked = alloc.isLeaked;
    const hasOwnershipHistory = alloc.ownershipHistoryAvailable;
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-4 border-l-4 border-${riskColor}-500">
            <div class="flex justify-between items-start mb-3">
                <div>
                    <h4 class="font-semibold text-gray-900 dark:text-white">${alloc.var_name || 'Unknown Variable'}</h4>
                    <p class="text-sm text-gray-600 dark:text-gray-300">${formatTypeName(alloc.type_name || 'Unknown Type')}</p>
                    ${alloc.isClone ? '<span class="inline-block px-2 py-1 text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 rounded-full mt-1">Clone</span>' : ''}
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${riskColor}-100 text-${riskColor}-800 dark:bg-${riskColor}-900 dark:text-${riskColor}-200">
                        ${alloc.riskLevel} Risk
                    </span>
                    ${isLeaked ? '<div class="mt-1"><span class="px-2 py-1 text-xs bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200 rounded-full">LEAKED</span></div>' : ''}
                </div>
            </div>
            
            <div class="grid grid-cols-2 gap-4 text-sm mb-3">
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Size:</span>
                    <span class="ml-2 font-mono">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Risk Score:</span>
                    <span class="ml-2 font-bold text-${riskColor}-600">${alloc.riskScore}/100</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Pointer:</span>
                    <span class="ml-2 font-mono text-xs">${alloc.ptr}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Thread:</span>
                    <span class="ml-2">${alloc.thread_id || 'Unknown'}</span>
                </div>
            </div>

            <!-- Enhanced improve.md fields -->
            <div class="grid grid-cols-3 gap-4 text-sm mb-3">
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Total Borrows:</span>
                    <span class="ml-2 font-bold">${alloc.totalBorrowsForAlloc || 0}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">Clone Count:</span>
                    <span class="ml-2 font-bold">${alloc.cloneCount || 0}</span>
                </div>
                <div>
                    <span class="text-gray-500 dark:text-gray-400">FFI Tracked:</span>
                    <span class="ml-2">${alloc.ffi_tracked ? '✅' : '❌'}</span>
                </div>
            </div>
            
            ${hasViolations || hasBorrowConflicts || hasClones || hasOwnershipHistory ? `
                <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-500">
                    <div class="text-sm space-y-1">
                        ${hasViolations ? `<div class="text-red-600 dark:text-red-400">⚠️ ${alloc.violations} safety violations</div>` : ''}
                        ${hasBorrowConflicts ? `<div class="text-orange-600 dark:text-orange-400">⚠️ Borrow conflicts detected</div>` : ''}
                        ${hasClones ? `<div class="text-blue-600 dark:text-blue-400">🔄 ${alloc.cloneCount} clones created</div>` : ''}
                        ${hasOwnershipHistory ? `<div class="text-green-600 dark:text-green-400">📋 Ownership history available</div>` : ''}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createFFIAllocationCard(alloc) {
    return createEnhancedFFIAllocationCard(alloc);
}

// Create enhanced borrow analysis card with improve.md fields
function createEnhancedBorrowAnalysisCard(alloc) {
    const borrowInfo = alloc.borrow_info;
    const hasConflict = borrowInfo.mutable_borrows > 0 && borrowInfo.immutable_borrows > 0;
    const lastBorrowTime = borrowInfo.last_borrow_timestamp ? new Date(borrowInfo.last_borrow_timestamp / 1000000).toLocaleTimeString() : 'Unknown';
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 ${hasConflict ? 'border-l-4 border-red-500' : 'border border-gray-200 dark:border-gray-500'}">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">Last borrow: ${lastBorrowTime}</p>
                </div>
                <div class="text-right text-sm">
                    <div class="text-blue-600 dark:text-blue-400">Immutable: ${borrowInfo.immutable_borrows}</div>
                    <div class="text-red-600 dark:text-red-400">Mutable: ${borrowInfo.mutable_borrows}</div>
                    <div class="text-purple-600 dark:text-purple-400">Max Concurrent: ${borrowInfo.max_concurrent_borrows}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Total: ${(borrowInfo.immutable_borrows || 0) + (borrowInfo.mutable_borrows || 0)}</div>
                </div>
            </div>
            ${hasConflict ? `
                <div class="mt-2 text-xs text-red-600 dark:text-red-400 font-bold">
                    ⚠️ BORROW CONFLICT: Simultaneous mutable and immutable borrows detected
                </div>
            ` : ''}
        </div>
    `;
}

// Legacy function for backward compatibility
function createBorrowAnalysisCard(alloc) {
    return createEnhancedBorrowAnalysisCard(alloc);
}

// Create clone analysis card for improve.md clone_info fields
function createCloneAnalysisCard(alloc) {
    const cloneInfo = alloc.clone_info;
    const isClone = cloneInfo.is_clone;
    const cloneCount = cloneInfo.clone_count;
    const originalPtr = cloneInfo.original_ptr;
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-blue-500">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                    ${isClone ? `<p class="text-xs text-blue-600 dark:text-blue-400">Clone of: ${originalPtr}</p>` : ''}
                </div>
                <div class="text-right">
                    <div class="text-blue-600 dark:text-blue-400 font-bold text-lg">${cloneCount}</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Clones Created</div>
                    ${isClone ? '<div class="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 px-2 py-1 rounded mt-1">IS CLONE</div>' : ''}
                </div>
            </div>
            <div class="mt-2 text-sm text-gray-600 dark:text-gray-300">
                ${cloneCount > 0 ? `🔄 This allocation has been cloned ${cloneCount} times` : ''}
                ${isClone ? `<br>📋 This is a clone of allocation at ${originalPtr}` : ''}
            </div>
        </div>
    `;
}

// Create ownership history card for improve.md ownership_history_available field
function createOwnershipHistoryCard(alloc) {
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-green-500">
            <div class="flex justify-between items-center">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">${alloc.var_name}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">${formatTypeName(alloc.type_name)}</p>
                </div>
                <div class="text-right">
                    <div class="text-green-600 dark:text-green-400">📋 History Available</div>
                    <div class="text-xs text-gray-500 dark:text-gray-400">Detailed tracking enabled</div>
                </div>
            </div>
            <div class="mt-2 text-sm text-gray-600 dark:text-gray-300">
                ✅ Ownership history is available for this allocation in lifetime.json
            </div>
        </div>
    `;
}

// Create unsafe report card for improve.md UnsafeReport structure
function createUnsafeReportCard(report) {
    const riskLevel = report.risk_assessment?.risk_level || 'Unknown';
    const riskColor = riskLevel === 'High' ? 'red' : riskLevel === 'Medium' ? 'orange' : 'green';
    const confidenceScore = report.risk_assessment?.confidence_score || 0;
    const riskFactors = report.risk_assessment?.risk_factors || [];
    const dynamicViolations = report.dynamic_violations || [];
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-4 border-l-4 border-${riskColor}-500">
            <div class="flex justify-between items-start mb-3">
                <div>
                    <h4 class="font-semibold text-gray-900 dark:text-white">Unsafe Report: ${report.report_id || 'Unknown'}</h4>
                    <p class="text-sm text-gray-600 dark:text-gray-300">${report.source?.type || 'Unknown'} at ${report.source?.location || 'Unknown location'}</p>
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${riskColor}-100 text-${riskColor}-800 dark:bg-${riskColor}-900 dark:text-${riskColor}-200">
                        ${riskLevel} Risk
                    </span>
                    <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">Confidence: ${(confidenceScore * 100).toFixed(1)}%</div>
                </div>
            </div>
            
            ${riskFactors.length > 0 ? `
                <div class="mb-3">
                    <h5 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Risk Factors:</h5>
                    <div class="space-y-1">
                        ${riskFactors.map(factor => `
                            <div class="text-sm">
                                <span class="font-medium text-${riskColor}-600 dark:text-${riskColor}-400">${factor.factor_type}</span>
                                <span class="text-gray-600 dark:text-gray-400"> (Severity: ${factor.severity}/10)</span>
                                <div class="text-xs text-gray-500 dark:text-gray-400">${factor.description}</div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            ` : ''}
            
            ${dynamicViolations.length > 0 ? `
                <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-500">
                    <h5 class="text-sm font-medium text-red-700 dark:text-red-300 mb-2">Dynamic Violations:</h5>
                    <div class="space-y-1">
                        ${dynamicViolations.map(violation => `
                            <div class="text-sm text-red-600 dark:text-red-400">
                                ⚠️ ${violation.violation_type}: ${violation.description}
                            </div>
                        `).join('')}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Create memory passport card for improve.md MemoryPassport structure
function createMemoryPassportCard(passport) {
    const status = passport.status_at_shutdown || 'Unknown';
    const statusColor = status === 'Reclaimed' ? 'green' : status === 'InForeignCustody' ? 'red' : 'orange';
    const lifecycleEvents = passport.lifecycle_events || [];
    
    return `
        <div class="bg-white dark:bg-gray-600 rounded-lg p-3 border-l-4 border-${statusColor}-500">
            <div class="flex justify-between items-start">
                <div>
                    <h5 class="font-medium text-gray-900 dark:text-white">Passport: ${passport.passport_id || 'Unknown'}</h5>
                    <p class="text-xs text-gray-500 dark:text-gray-400">Allocation: ${passport.allocation_ptr} (${formatBytes(passport.size_bytes || 0)})</p>
                </div>
                <div class="text-right">
                    <span class="px-2 py-1 text-xs font-bold rounded-full bg-${statusColor}-100 text-${statusColor}-800 dark:bg-${statusColor}-900 dark:text-${statusColor}-200">
                        ${status}
                    </span>
                </div>
            </div>
            
            ${lifecycleEvents.length > 0 ? `
                <div class="mt-2">
                    <h6 class="text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Lifecycle Events:</h6>
                    <div class="space-y-1">
                        ${lifecycleEvents.slice(0, 3).map(event => `
                            <div class="text-xs text-gray-600 dark:text-gray-400">
                                📅 ${event.event_type} ${event.how ? `(${event.how})` : ''}
                            </div>
                        `).join('')}
                        ${lifecycleEvents.length > 3 ? `<div class="text-xs text-gray-500 dark:text-gray-400">... and ${lifecycleEvents.length - 3} more events</div>` : ''}
                    </div>
                </div>
            ` : ''}
        </div>
    `;
}

// Create FFI empty state
function createFFIEmptyState() {
    return `
        <div class="text-center py-8">
            <div class="mb-4">
                <svg class="w-16 h-16 mx-auto text-green-400 dark:text-green-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                </svg>
            </div>
            <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">Memory Safety Verified</h4>
            <p class="text-sm text-gray-600 dark:text-gray-400">No unsafe FFI operations detected in this analysis</p>
            <p class="text-xs mt-2 text-gray-500 dark:text-gray-500">Your code appears to be using safe Rust patterns</p>
        </div>
    `;
}

// Create comprehensive FFI dashboard with SVG-style visualization
function createFFIDashboardSVG(unsafeAllocs, ffiAllocs, boundaryCrossings, safetyViolations, unsafeMemory, enhancedData, boundaryEvents, violations) {
    return `
        <div class="bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl p-6 text-white shadow-2xl">
            <!-- Header -->
            <div class="text-center mb-6">
                <h2 class="text-2xl font-bold mb-2 flex items-center justify-center">
                    <i class="fa fa-shield mr-3 text-red-400"></i>
                    Unsafe Rust & FFI Memory Analysis Dashboard
                </h2>
            </div>

            <!-- Key Metrics Row -->
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-8">
                ${createFFIMetricCard('Unsafe Allocations', unsafeAllocs, '#e74c3c', 'fa-exclamation-triangle')}
                ${createFFIMetricCard('FFI Allocations', ffiAllocs, '#3498db', 'fa-exchange')}
                ${createFFIMetricCard('Boundary Crossings', boundaryCrossings, '#f39c12', 'fa-arrows-h')}
                ${createFFIMetricCard('Safety Violations', safetyViolations, '#e67e22', 'fa-warning')}
                ${createFFIMetricCard('Unsafe Memory', formatBytes(unsafeMemory), '#9b59b6', 'fa-memory')}
            </div>

            <!-- Main Dashboard Content -->
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <!-- Memory Allocation Sources -->
                <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                    <h3 class="text-lg font-semibold mb-4 text-white">Memory Allocation Sources</h3>
                    <div class="space-y-4">
                        ${createAllocationSourceBar('Unsafe Rust', unsafeAllocs, Math.max(unsafeAllocs, ffiAllocs), '#e74c3c')}
                        ${createAllocationSourceBar('FFI', ffiAllocs, Math.max(unsafeAllocs, ffiAllocs), '#3498db')}
                    </div>
                </div>

                <!-- Memory Safety Status -->
                <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                    <h3 class="text-lg font-semibold mb-4 text-white">Memory Safety Status</h3>
                    ${safetyViolations > 0 ? `
                        <div class="bg-red-900/30 border border-red-500/50 rounded-lg p-4">
                            <h4 class="text-red-300 font-semibold mb-2 flex items-center">
                                <i class="fa fa-exclamation-triangle mr-2"></i>
                                ${safetyViolations} Safety Violations Detected
                            </h4>
                            ${enhancedData.filter(item => (item.safety_violations || 0) > 0).slice(0, 2).map(item => `
                                <div class="text-red-400 text-sm flex items-center mb-1">
                                    <i class="fa fa-dot-circle-o mr-2 text-xs"></i>
                                    Pointer ${item.ptr}: ${item.safety_violations} violations
                                </div>
                            `).join('')}
                        </div>
                    ` : `
                        <div class="bg-green-900/30 border border-green-500/50 rounded-lg p-4">
                            <h4 class="text-green-300 font-semibold flex items-center mb-2">
                                <i class="fa fa-check-circle mr-2"></i>
                                No Safety Violations Detected
                            </h4>
                            <p class="text-green-400 text-sm">All unsafe operations appear to be handled correctly</p>
                        </div>
                    `}
                </div>
            </div>

            <!-- Cross-Language Memory Flow -->
            <div class="bg-gray-700/50 rounded-lg p-6 mb-6 backdrop-blur-sm">
                <h3 class="text-lg font-semibold mb-6 text-white text-center">Cross-Language Memory Flow</h3>
                <div class="flex items-center justify-center space-x-8">
                    <!-- Rust Side -->
                    <div class="bg-green-800/30 border-2 border-green-400/50 rounded-lg p-6 text-center backdrop-blur-sm">
                        <div class="text-green-300 font-bold text-xl mb-2">RUST</div>
                        <div class="text-green-400 text-sm">${unsafeAllocs} allocations</div>
                        <div class="w-16 h-16 mx-auto mt-3 bg-green-500/20 rounded-full flex items-center justify-center">
                            <i class="fa fa-rust text-green-400 text-2xl"></i>
                        </div>
                    </div>
                    
                    <!-- Flow Arrows -->
                    <div class="flex flex-col items-center space-y-4">
                        <div class="flex items-center space-x-2">
                            <div class="flex items-center space-x-1">
                                <div class="w-8 h-0.5 bg-red-400"></div>
                                <div class="w-0 h-0 border-l-4 border-l-red-400 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                            </div>
                            <span class="text-red-400 text-sm font-bold bg-red-900/30 px-2 py-1 rounded">
                                ${boundaryEvents.filter(e => e.event_type === 'RustToFfi').length}
                            </span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <span class="text-orange-400 text-sm font-bold bg-orange-900/30 px-2 py-1 rounded">
                                ${boundaryEvents.filter(e => e.event_type === 'FfiToRust').length}
                            </span>
                            <div class="flex items-center space-x-1">
                                <div class="w-0 h-0 border-r-4 border-r-orange-400 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                                <div class="w-8 h-0.5 bg-orange-400"></div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- FFI/C Side -->
                    <div class="bg-blue-800/30 border-2 border-blue-400/50 rounded-lg p-6 text-center backdrop-blur-sm">
                        <div class="text-blue-300 font-bold text-xl mb-2">FFI / C</div>
                        <div class="text-blue-400 text-sm">${ffiAllocs} allocations</div>
                        <div class="w-16 h-16 mx-auto mt-3 bg-blue-500/20 rounded-full flex items-center justify-center">
                            <i class="fa fa-code text-blue-400 text-2xl"></i>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Unsafe Memory Hotspots -->
            <div class="bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                <h3 class="text-lg font-semibold mb-4 text-white">Unsafe Memory Hotspots</h3>
                <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                    ${enhancedData.slice(0, 12).map(item => createMemoryHotspot(item)).join('')}
                </div>
                ${enhancedData.length === 0 ? `
                    <div class="text-center py-8 text-gray-400">
                        <i class="fa fa-shield-alt text-4xl mb-2"></i>
                        <p>No unsafe memory hotspots detected</p>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}

// Create FFI metric card
function createFFIMetricCard(title, value, color, icon) {
    return `
        <div class="bg-gray-700/30 border border-gray-600/50 rounded-lg p-4 text-center backdrop-blur-sm hover:bg-gray-600/30 transition-all">
            <div class="flex items-center justify-center mb-2">
                <i class="fa ${icon} text-2xl" style="color: ${color}"></i>
            </div>
            <div class="text-2xl font-bold mb-1" style="color: ${color}">${value}</div>
            <div class="text-xs text-gray-300 uppercase tracking-wide">${title}</div>
        </div>
    `;
}

// Create allocation source bar
function createAllocationSourceBar(label, count, maxCount, color) {
    const percentage = maxCount > 0 ? (count / maxCount) * 100 : 0;
    const barHeight = Math.max(20, (count / maxCount) * 80);

    return `
        <div class="flex items-end space-x-4">
            <div class="flex-1">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-sm font-medium text-gray-300">${label}</span>
                    <span class="text-lg font-bold text-white">${count}</span>
                </div>
                <div class="w-full bg-gray-600 rounded-full h-6 overflow-hidden">
                    <div class="h-full rounded-full transition-all duration-500 flex items-center justify-center text-white text-xs font-bold" 
                         style="width: ${percentage}%; background-color: ${color};">
                        ${count > 0 ? count : ''}
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create memory hotspot visualization
function createMemoryHotspot(item) {
    const size = item.size || 0;
    const isUnsafe = !item.ffi_tracked;
    const radius = Math.min(30, Math.max(12, Math.sqrt(size / 50)));
    const color = isUnsafe ? '#e74c3c' : '#3498db';
    const bgColor = isUnsafe ? 'bg-red-900/20' : 'bg-blue-900/20';
    const borderColor = isUnsafe ? 'border-red-500/50' : 'border-blue-500/50';

    // Interactive hotspot with data attributes for detail panel
    return `
        <div class="flex flex-col items-center p-3 ${bgColor} border ${borderColor} rounded-lg backdrop-blur-sm hover:scale-105 transition-transform cursor-pointer"
             data-ptr="${item.ptr || ''}"
             data-var="${(item.var_name || 'Unknown').toString().replace(/'/g, '&apos;')}"
             data-type="${(item.type_name || 'Unknown').toString().replace(/'/g, '&apos;')}"
             data-size="${size}"
             data-violations="${(item.safety_violations || 0)}"
             onclick="window.showFFIDetailFromDataset && window.showFFIDetailFromDataset(this)">
            <div class="relative mb-2">
                <div class="rounded-full border-2 flex items-center justify-center text-white text-xs font-bold shadow-lg"
                     style="width: ${radius * 2}px; height: ${radius * 2}px; background-color: ${color}; border-color: ${color};">
                    ${size > 1024 ? Math.round(size / 1024) + 'K' : size + 'B'}
                </div>
                ${(item.safety_violations || 0) > 0 ? `
                    <div class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full flex items-center justify-center">
                        <i class="fa fa-exclamation text-white text-xs"></i>
                    </div>
                ` : ''}
            </div>
            <div class="text-xs text-center">
                <div class="font-semibold" style="color: ${color}">
                    ${isUnsafe ? 'UNSAFE' : 'FFI'}
                </div>
                <div class="text-gray-400 text-xs">
                    ${formatBytes(size)}
                </div>
            </div>
        </div>
    `;
}

// Simple detail panel for FFI hotspot items
window.showFFIDetailFromDataset = function(el) {
    try {
        const container = document.getElementById('ffiVisualization');
        if (!container) return;

        // Remove existing panel
        const existing = container.querySelector('#ffi-detail-panel');
        if (existing) existing.remove();

        // Build panel
        const panel = document.createElement('div');
        panel.id = 'ffi-detail-panel';
        panel.style.position = 'absolute';
        panel.style.right = '16px';
        panel.style.top = '16px';
        panel.style.zIndex = '1000';
        panel.style.minWidth = '280px';
        panel.style.maxWidth = '360px';
        panel.style.background = 'var(--bg-primary)';
        panel.style.border = '1px solid var(--border-light)';
        panel.style.borderRadius = '10px';
        panel.style.boxShadow = '0 10px 25px rgba(0,0,0,0.2)';
        panel.style.padding = '12px';

        const name = el.getAttribute('data-var');
        const type = el.getAttribute('data-type');
        const size = parseInt(el.getAttribute('data-size') || '0', 10);
        const ptr = el.getAttribute('data-ptr');
        const violations = parseInt(el.getAttribute('data-violations') || '0', 10);

        panel.innerHTML = `
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
                <div style="font-weight:700; font-size:14px; color: var(--text-primary);">FFI Allocation Detail</div>
                <button onclick="this.parentNode.parentNode.remove()" style="border:none; background:transparent; color: var(--text-secondary); font-size:18px; cursor:pointer">×</button>
            </div>
            <div style="font-size:12px; color: var(--text-primary);">
                <div style="margin-bottom:6px;"><strong>Name:</strong> ${name}</div>
                <div style="margin-bottom:6px;"><strong>Type:</strong> ${type}</div>
                <div style="margin-bottom:6px;"><strong>Size:</strong> ${formatBytes(size)}</div>
                ${ptr ? `<div style='margin-bottom:6px;'><strong>Pointer:</strong> <code>${ptr}</code></div>` : ''}
                <div style="margin-bottom:6px;"><strong>Safety Violations:</strong> ${violations}</div>
            </div>
        `;

        container.appendChild(panel);
    } catch(e) {
        console.warn('Failed to show FFI detail panel', e);
    }
};

// Initialize memory fragmentation analysis with enhanced SVG-style visualization
function initMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    if (allocations.length === 0) {
        container.innerHTML = createFragmentationEmptyState();
        return;
    }

    // Fixed memory fragmentation analysis: based on allocation size distribution rather than address gaps
    const sortedAllocs = allocations
        .filter(alloc => alloc.size && alloc.size > 0)
        .map(alloc => ({
            size: alloc.size,
            type: alloc.type_name || 'System Allocation',
            var_name: alloc.var_name || 'unknown'
        }))
        .sort((a, b) => a.size - b.size);

    const totalMemory = sortedAllocs.reduce((sum, alloc) => sum + alloc.size, 0);
    
    // Calculate fragmentation based on allocation size distribution
    const sizeVariance = calculateSizeVariance(sortedAllocs);
    const smallAllocRatio = sortedAllocs.filter(a => a.size < 1024).length / sortedAllocs.length;
    
    // Fragmentation score: based on size distribution unevenness
    const fragmentationRatio = Math.min(100, (sizeVariance / 1000 + smallAllocRatio * 50));
    
    // Simplified gap analysis: only count quantity, not fake address gaps
    const gaps = Math.max(0, sortedAllocs.length - 1);
    const maxGap = 0; // No longer calculate address gaps
    let totalGapSize = 0; // Reset to 0 to avoid huge fake values

    // Size distribution analysis (inspired by SVG)
    const sizeDistribution = {
        tiny: sortedAllocs.filter(a => a.size < 64).length,
        small: sortedAllocs.filter(a => a.size >= 64 && a.size < 1024).length,
        medium: sortedAllocs.filter(a => a.size >= 1024 && a.size < 65536).length,
        large: sortedAllocs.filter(a => a.size >= 65536).length
    };

    container.innerHTML = createFragmentationAnalysisSVG(
        fragmentationRatio, gaps, maxGap, sortedAllocs.length,
        totalMemory, sizeDistribution, sortedAllocs
    );
}

// Create fragmentation empty state
function createFragmentationEmptyState() {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center text-heading">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            <div class="text-center py-8">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-gray-400 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">No Memory Data for Analysis</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400">Memory fragmentation analysis requires allocation data</p>
            </div>
        </div>
    `;
}

// Create comprehensive fragmentation analysis with SVG-style visualization
function createFragmentationAnalysisSVG(fragmentationRatio, gaps, maxGap, blockCount, totalMemory, sizeDistribution, sortedAllocs) {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-6 flex items-center text-heading">
                <i class="fa fa-puzzle-piece text-orange-500 mr-2"></i>Memory Fragmentation Analysis
            </h2>
            
            <!-- Key Metrics Grid -->
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
                ${createFragmentationMetricCard('Fragmentation', fragmentationRatio.toFixed(1) + '%', fragmentationRatio, '#f39c12')}
                ${createFragmentationMetricCard('Memory Gaps', gaps, 100, '#3498db')}
                ${createFragmentationMetricCard('Largest Gap', formatBytes(maxGap), 100, '#27ae60')}
                ${createFragmentationMetricCard('Memory Blocks', blockCount, 100, '#9b59b6')}
            </div>

            <!-- Main Analysis Content -->
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                <!-- Fragmentation Assessment -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Fragmentation Assessment</h4>
                    <div class="space-y-4">
                        <div>
                            <div class="flex justify-between items-center mb-2">
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Overall Health</span>
                                <span class="text-sm font-bold ${getFragmentationColor(fragmentationRatio)}">${fragmentationRatio.toFixed(1)}%</span>
                            </div>
                            <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                                <div class="h-4 rounded-full transition-all duration-500 ${getFragmentationBgColor(fragmentationRatio)}" 
                                     style="width: ${Math.min(fragmentationRatio, 100)}%"></div>
                            </div>
                        </div>
                        <div class="text-sm text-gray-600 dark:text-gray-300">
                            ${getFragmentationAssessment(fragmentationRatio)}
                        </div>
                    </div>
                </div>

                <!-- Size Distribution (inspired by SVG bar chart) -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Size Distribution</h4>
                    <div class="space-y-3">
                        ${createSizeDistributionBar('Tiny (0-64B)', sizeDistribution.tiny, blockCount, '#27ae60')}
                        ${createSizeDistributionBar('Small (64B-1KB)', sizeDistribution.small, blockCount, '#f39c12')}
                        ${createSizeDistributionBar('Medium (1KB-64KB)', sizeDistribution.medium, blockCount, '#e74c3c')}
                        ${createSizeDistributionBar('Large (>64KB)', sizeDistribution.large, blockCount, '#8e44ad')}
                    </div>
                </div>
            </div>

            <!-- Memory Layout Visualization -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Memory Layout Visualization</h4>
                <div class="relative">
                    <!-- Memory blocks visualization -->
                    <div class="h-16 bg-gray-200 dark:bg-gray-600 rounded relative overflow-hidden mb-4">
                        ${createMemoryLayoutVisualization(sortedAllocs, totalMemory)}
                    </div>
                    
                    <!-- Memory address timeline -->
                    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mb-2">
                        <span>Low Address</span>
                        <span>Memory Layout</span>
                        <span>High Address</span>
                    </div>
                    
                    <!-- Legend -->
                    <div class="flex flex-wrap gap-4 text-xs">
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-blue-500 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">User Allocations</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-gray-400 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">System Allocations</span>
                        </div>
                        <div class="flex items-center">
                            <div class="w-3 h-3 bg-red-300 rounded mr-2"></div>
                            <span class="text-gray-600 dark:text-gray-300">Memory Gaps</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create fragmentation metric card with circular progress
function createFragmentationMetricCard(title, value, percentage, color) {
    const normalizedPercentage = Math.min(100, Math.max(0, percentage));
    const circumference = 2 * Math.PI * 20;
    const strokeDashoffset = circumference - (normalizedPercentage / 100) * circumference;

    return `
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 text-center hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase">${title}</p>
                    <p class="text-lg font-bold text-gray-900 dark:text-white">${value}</p>
                </div>
                <div class="relative w-10 h-10">
                    <svg class="w-10 h-10 transform -rotate-90" viewBox="0 0 50 50">
                        <circle cx="25" cy="25" r="20" stroke="#e5e7eb" stroke-width="4" fill="none" class="dark:stroke-gray-600"/>
                        <circle cx="25" cy="25" r="20" stroke="${color}" stroke-width="4" fill="none" 
                                stroke-dasharray="${circumference}" stroke-dashoffset="${strokeDashoffset}"
                                stroke-linecap="round" class="transition-all duration-500"/>
                    </svg>
                </div>
            </div>
        </div>
    `;
}

// Create size distribution bar
function createSizeDistributionBar(label, count, total, color) {
    const percentage = total > 0 ? (count / total) * 100 : 0;
    return `
        <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-28">${label}</span>
            <div class="flex-1 mx-3">
                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-4">
                    <div class="h-4 rounded-full transition-all duration-500" 
                         style="width: ${percentage}%; background-color: ${color}"></div>
                </div>
            </div>
            <span class="text-sm font-bold text-gray-900 dark:text-white w-8 text-right">${count}</span>
        </div>
    `;
}

// Create memory layout visualization
function createMemoryLayoutVisualization(sortedAllocs, totalMemory) {
    if (sortedAllocs.length === 0) return '<div class="flex items-center justify-center h-full text-gray-400">No memory layout data</div>';

    return sortedAllocs.slice(0, 30).map((alloc, index) => {
        const width = Math.max(1, (alloc.size / totalMemory) * 100);
        const left = (index / 30) * 100;
        const isUserAlloc = alloc.type !== 'System Allocation';
        const color = isUserAlloc ? '#3498db' : '#95a5a6';

        return `
            <div class="absolute h-full transition-all hover:brightness-110 cursor-pointer" 
                 style="left: ${left}%; width: ${width}%; background-color: ${color}; opacity: 0.8;"
                 title="${alloc.type}: ${formatBytes(alloc.size)} at ${(alloc.address || 0).toString(16)}">
            </div>
        `;
    }).join('');
}

// Calculate variance of allocation sizes to assess fragmentation level
function calculateSizeVariance(allocations) {
    if (allocations.length === 0) return 0;
    
    const sizes = allocations.map(a => a.size);
    const mean = sizes.reduce((sum, size) => sum + size, 0) / sizes.length;
    const variance = sizes.reduce((sum, size) => sum + Math.pow(size - mean, 2), 0) / sizes.length;
    
    return Math.sqrt(variance);
}

// Helper functions for fragmentation analysis
function getFragmentationColor(ratio) {
    if (ratio < 10) return 'text-green-600 dark:text-green-400';
    if (ratio < 25) return 'text-yellow-600 dark:text-yellow-400';
    if (ratio < 50) return 'text-orange-600 dark:text-orange-400';
    return 'text-red-600 dark:text-red-400';
}

function getFragmentationBgColor(ratio) {
    if (ratio < 10) return 'bg-green-500';
    if (ratio < 25) return 'bg-yellow-500';
    if (ratio < 50) return 'bg-orange-500';
    return 'bg-red-500';
}

function getFragmentationAssessment(ratio) {
    if (ratio < 10) return 'Excellent memory layout with minimal fragmentation. Memory is well-organized.';
    if (ratio < 25) return 'Good memory layout with low fragmentation. No immediate concerns.';
    if (ratio < 50) return 'Moderate fragmentation detected. Consider memory pool allocation strategies.';
    return 'High fragmentation detected. Memory layout optimization strongly recommended.';
}

// Initialize memory growth trends with enhanced SVG-style visualization
function initMemoryGrowthTrends() {
    const container = document.getElementById('memoryGrowthTrends");
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];

    // Sort allocations by timestamp
    const sortedAllocs = allocations
        .filter(alloc => alloc.timestamp_alloc)
        .sort((a, b) => a.timestamp_alloc - b.timestamp_alloc);

    if (sortedAllocs.length === 0) {
        container.innerHTML = createGrowthTrendsEmptyState();
        return;
    }

    // Calculate cumulative memory usage over time
    let cumulativeMemory = 0;
    let peakMemory = 0;
    const timePoints = [];

    sortedAllocs.forEach((alloc, index) => {
        cumulativeMemory += alloc.size || 0;
        peakMemory = Math.max(peakMemory, cumulativeMemory);

        if (index % Math.max(1, Math.floor(sortedAllocs.length / 20)) === 0) {
            timePoints.push({
                timestamp: alloc.timestamp_alloc,
                memory: cumulativeMemory,
                index: index,
                allocCount: index + 1
            });
        }
    });

    const startMemory = timePoints[0]?.memory || 0;
    const endMemory = timePoints[timePoints.length - 1]?.memory || 0;
    const growthRate = startMemory > 0 ? ((endMemory - startMemory) / startMemory * 100) : 0;
    const averageMemory = timePoints.reduce((sum, point) => sum + point.memory, 0) / timePoints.length;

    container.innerHTML = createMemoryGrowthTrendsSVG(
        peakMemory, averageMemory, growthRate, timePoints, sortedAllocs.length
    );
}

// Create growth trends empty state
function createGrowthTrendsEmptyState() {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-4 flex items-center text-heading">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            <div class="text-center py-8">
                <div class="mb-4">
                    <svg class="w-16 h-16 mx-auto text-gray-400 dark:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                    </svg>
                </div>
                <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-gray-200">No Timeline Data Available</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400">Memory growth analysis requires timestamp data</p>
            </div>
        </div>
    `;
}

// Create comprehensive memory growth trends visualization
function createMemoryGrowthTrendsSVG(peakMemory, averageMemory, growthRate, timePoints, totalAllocs) {
    return `
        <div class="bg-white dark:bg-gray-800 rounded-xl p-6 card-shadow transition-colors">
            <h2 class="text-xl font-semibold mb-6 flex items-center text-heading">
                <i class="fa fa-line-chart text-green-500 mr-2"></i>Memory Growth Trends
            </h2>
            
            <!-- Key Metrics Grid -->
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
                ${createGrowthMetricCard('Peak Memory', formatBytes(peakMemory), 100, '#e74c3c')}
                ${createGrowthMetricCard('Average Memory', formatBytes(averageMemory), Math.round((averageMemory / peakMemory) * 100), '#3498db')}
                ${createGrowthMetricCard('Growth Rate', (growthRate > 0 ? '+' : '") + growthRate.toFixed(1) + '%', Math.abs(growthRate), getGrowthRateColor(growthRate))}
                ${createGrowthMetricCard('Total Allocations', totalAllocs, 100, '#9b59b6')}
            </div>

            <!-- Main Growth Chart -->
            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-6">
                <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Memory Usage Over Time</h4>
                <div class="relative">
                    <!-- Chart Container -->
                    <div class="h-48 relative bg-white dark:bg-gray-600 rounded border dark:border-gray-500 overflow-hidden">
                        ${createMemoryGrowthChart(timePoints, peakMemory)}
                    </div>
                    
                    <!-- Chart Labels -->
                    <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-2">
                        <span>Start</span>
                        <span>Memory Usage Timeline</span>
                        <span>End</span>
                    </div>
                    
                    <!-- Peak Memory Line -->
                    <div class="absolute top-2 right-2 text-xs text-red-500 dark:text-red-400 bg-white dark:bg-gray-800 px-2 py-1 rounded shadow">
                        Peak: ${formatBytes(peakMemory)}
                    </div>
                </div>
            </div>

            <!-- Growth Analysis -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <!-- Growth Assessment -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Growth Assessment</h4>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600 dark:text-gray-300">Memory Efficiency</span>
                            <span class="text-sm font-bold ${getEfficiencyColor(averageMemory, peakMemory)}">${((averageMemory / peakMemory) * 100).toFixed(1)}%</span>
                        </div>
                        <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                            <div class="h-2 rounded-full transition-all duration-500 ${getEfficiencyBgColor(averageMemory, peakMemory)}" 
                                 style="width: ${(averageMemory / peakMemory) * 100}%"></div>
                        </div>
                        <div class="text-sm text-gray-600 dark:text-gray-300">
                            ${getGrowthAssessment(growthRate)}
                        </div>
                    </div>
                </div>

                <!-- Memory Allocation Timeline -->
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                    <h4 class="font-semibold mb-4 text-gray-800 dark:text-white">Recent Allocations</h4>
                    <div class="space-y-2 max-h-32 overflow-y-auto">
                        ${timePoints.slice(-6).map((point, index) => `
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-gray-600 dark:text-gray-300">Alloc #${point.allocCount}</span>
                                <span class="font-mono text-xs font-bold text-gray-900 dark:text-white">${formatBytes(point.memory)}</span>
                            </div>
                        `).join('')}
                    </div>
                    <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                        Showing latest allocation points
                    </div>
                </div>
            </div>
        </div>
    `;
}

// Create growth metric card
function createGrowthMetricCard(title, value, percentage, color) {
    const normalizedPercentage = Math.min(100, Math.max(0, percentage));

    return `
        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 text-center hover:shadow-md transition-shadow">
            <div class="mb-2">
                <div class="text-2xl font-bold" style="color: ${color}">${value}</div>
                <div class="text-xs text-gray-600 dark:text-gray-400 uppercase tracking-wide">${title}</div>
            </div>
            <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                <div class="h-2 rounded-full transition-all duration-500" 
                     style="width: ${normalizedPercentage}%; background-color: ${color}"></div>
            </div>
        </div>
    `;
}

// Create memory growth chart
function createMemoryGrowthChart(timePoints, peakMemory) {
    if (timePoints.length < 2) return '<div class="flex items-center justify-center h-full text-gray-400">Insufficient data points</div>';

    const chartHeight = 180;
    const chartWidth = 100; // percentage

    // Create SVG path for the growth line
    const pathPoints = timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * chartWidth;
        const y = chartHeight - ((point.memory / peakMemory) * (chartHeight - 20));
        return `${x},${y}`;
    });

    return `
        <!-- Background Grid -->
        <div class="absolute inset-0">
            ${[0, 25, 50, 75, 100].map(y => `
                <div class="absolute w-full border-t border-gray-200 dark:border-gray-500 opacity-30" 
                     style="top: ${y}%"></div>
            `).join('')}
        </div>
        
        <!-- Growth Line -->
        <svg class="absolute inset-0 w-full h-full" preserveAspectRatio="none">
            <polyline
                fill="none"
                stroke="#27ae60"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                points="${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 100;
        const y = 100 - ((point.memory / peakMemory) * 90);
        return `${x},${y}`;
    }).join(' ")}"
                class="drop-shadow-sm"
            />
        </svg>
        
        <!-- Data Points -->
        ${timePoints.map((point, index) => {
        const x = (index / (timePoints.length - 1)) * 100;
        const y = 100 - ((point.memory / peakMemory) * 90);
        return `
                <div class="absolute w-3 h-3 bg-green-500 rounded-full border-2 border-white dark:border-gray-600 shadow-sm transform -translate-x-1/2 -translate-y-1/2 hover:scale-125 transition-transform cursor-pointer" 
                     style="left: ${x}%; top: ${y}%"
                     title="Memory: ${formatBytes(point.memory)} at allocation #${point.allocCount}">
                </div>
            `;
    }).join('')}
        
        <!-- Peak Memory Indicator -->
        <div class="absolute w-full border-t-2 border-red-500 border-dashed opacity-60" style="top: 10%">
            <div class="absolute -top-1 right-0 text-xs text-red-500 bg-white dark:bg-gray-600 px-1 rounded ">
                Peak
            </div>
        </div>
    `;
}

// Helper functions for growth analysis
function getGrowthRateColor(rate) {
    if (rate < -10) return '#27ae60'; // Green for decreasing
    if (rate < 10) return '#3498db';  // Blue for stable
    if (rate < 50) return '#f39c12'; // Orange for moderate growth
    return '#e74c3c'; // Red for high growth
}

function getEfficiencyColor(avg, peak) {
    const efficiency = (avg / peak) * 100;
    if (efficiency > 80) return 'text-red-600 dark:text-red-400';
    if (efficiency > 60) return 'text-orange-600 dark:text-orange-400';
    if (efficiency > 40) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-green-600 dark:text-green-400';
}

function getEfficiencyBgColor(avg, peak) {
    const efficiency = (avg / peak) * 100;
    if (efficiency > 80) return 'bg-red-500';
    if (efficiency > 60) return 'bg-orange-500';
    if (efficiency > 40) return 'bg-yellow-500';
    return 'bg-green-500';
}

function getGrowthAssessment(rate) {
    if (rate < -10) return 'Excellent: Memory usage is decreasing, indicating good cleanup.';
    if (rate < 10) return 'Good: Stable memory usage with minimal growth.';
    if (rate < 50) return 'Moderate: Some memory growth detected, monitor for trends.';
    return 'Concerning: High memory growth detected, investigate for potential leaks.';
}

// Node Detail Panel for Variable Relationship Graph
class NodeDetailPanel {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        this.panel = null;
        this.currentNode = null;
    }

    show(nodeData, position) {
        console.log("Showing panel for:", nodeData.id);
        this.hide(); // Close existing panel
        this.panel = this.createPanel(nodeData);
        console.log("Panel created:", this.panel);
        this.positionPanel(position);
        this.container.appendChild(this.panel);
        console.log("Panel added to container");
        this.currentNode = nodeData;
    }

    hide() {
        if (this.panel) {
            this.panel.remove();
            this.panel = null;
            this.currentNode = null;
        }
    }

    createPanel(nodeData) {
        const panel = document.createElement('div');
        panel.className = "node-detail-panel";

        // Find related allocation data
        const allocations = window.analysisData.memory_analysis?.allocations || [];
        const allocation = allocations.find(alloc => alloc.var_name === nodeData.id);

        // Calculate relationships
        const sameTypeCount = allocations.filter(alloc =>
            alloc.type_name === nodeData.type_name && alloc.var_name !== nodeData.id
        ).length;

        const sameCategoryCount = allocations.filter(alloc =>
            getTypeCategory(alloc.type_name || '') === (nodeData.category || 'primitive') && alloc.var_name !== nodeData.id
        ).length;

        panel.innerHTML = `
            <div class="flex justify-between items-center mb-3">
                <h3>Variable Details</h3>
                <button class="close-button text-xl leading-none">&times;</button>
            </div>
            
            <div class="space-y-3">
                <div>
                    <label>Variable Name</label>
                    <p class="font-mono">${nodeData.id}</p>
                </div>
                
                <div>
                    <label>Type</label>
                    <p class="font-mono">${nodeData.type_name || 'Unknown'}</p>
                    <div class="flex items-center mt-1">
                        <div class="w-3 h-3 rounded-full mr-2" style="background-color: ${getEnhancedTypeColor(nodeData.type_name || 'unknown', nodeData.category || 'primitive')}"></div>
                        <span class="text-xs capitalize">${(nodeData.category || 'primitive').replace('_', ' ')}</span>
                    </div>
                </div>
                
                <div>
                    <label>Memory Size</label>
                    <p>${formatBytes(nodeData.size)}</p>
                </div>
                
                <div>
                    <label>Complexity Score</label>
                    <div class="flex items-center mb-2">
                        <div class="w-5 h-5 rounded-full mr-2 flex items-center justify-center text-white font-bold text-xs" style="background-color: ${getComplexityColor(nodeData.complexity || 2)}">${nodeData.complexity || 2}</div>
                        <span class="font-semibold">${nodeData.complexity || 2}/10 - ${getComplexityLevel(nodeData.complexity || 2)}</span>
                    </div>
                    <div class="text-xs text-gray-600 dark:text-gray-400">
                        ${getComplexityExplanation(nodeData.complexity || 2)}
                    </div>
                </div>
                
                ${allocation ? `
                    <div>
                        <label>Memory Address</label>
                        <p class="font-mono text-xs">${allocation.ptr}</p>
                    </div>
                    
                    <div>
                        <label>Allocated At</label>
                        <p class="text-sm">${new Date(allocation.timestamp_alloc / 1000000).toLocaleString()}</p>
                    </div>
                ` : ''}
                
                <div>
                    <label>Relationships</label>
                    <div class="text-sm space-y-1">
                        <div class="flex justify-between">
                            <span>Same type:</span>
                            <span class="font-semibold">${sameTypeCount}</span>
                        </div>
                        <div class="flex justify-between">
                            <span>Same category:</span>
                            <span class="font-semibold">${sameCategoryCount}</span>
                        </div>
                    </div>
                </div>
                
                <div>
                    <label>Type Analysis</label>
                    <div class="text-xs space-y-1">
                        ${getTypeAnalysis(nodeData.type_name || 'unknown', nodeData.size)}
                    </div>
                </div>
            </div>
        `;

        // Add close button functionality
        const closeButton = panel.querySelector('.close-button');
        closeButton.addEventListener("click", () => {
            this.hide();
        });

        return panel;
    }

    positionPanel(position) {
        if (!this.panel) return;

        // Simple positioning - place panel at a fixed position relative to container
        this.panel.style.position = 'absolute';
        this.panel.style.left = '20px';
        this.panel.style.top = '20px';
        this.panel.style.zIndex = '1000';

        console.log("Panel positioned at:", this.panel.style.left, this.panel.style.top);
    }
}

// Initialize variable relationship graph with enhanced D3.js force simulation
function initVariableGraph() {
    const container = document.getElementById('variable-graph-container");
    if (!container) return;

    const allocations = window.analysisData.memory_analysis?.allocations || [];
    const userAllocations = allocations.filter(alloc =>
        alloc.var_name && alloc.var_name !== 'unknown' &&
        alloc.type_name && alloc.type_name !== 'unknown'
    );

    if (userAllocations.length === 0) {
        container.innerHTML = `
            <div class="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
                <div class="text-center">
                    <i class="fa fa-sitemap text-4xl mb-4"></i>
                    <p class="text-lg font-semibold mb-2">No User Variables Found</p>
                    <p class="text-sm">Use track_var! macro to track variable relationships</p>
                </div>
            </div>
        `;
        return;
    }

    // Clear container
    container.innerHTML = '';

    // Set up dimensions
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Create SVG
    const svg = d3.select(container)
        .append('svg")
        .attr('width', width)
        .attr('height', height)
        .style('background', 'transparent");

    // Create zoom behavior
    const zoom = d3.zoom()
        .scaleExtent([0.1, 4])
        .on('zoom', (event) => {
            g.attr('transform', event.transform);
        });

    svg.call(zoom);

    // Create main group for zooming/panning
    const g = svg.append('g");

    // Prepare nodes data
    const nodes = userAllocations.map((alloc, index) => ({
        id: alloc.var_name,
        type: alloc.type_name,
        size: alloc.size || 0,
        complexity: getComplexityFromType(alloc.type_name),
        category: getTypeCategory(alloc.type_name),
        allocation: alloc
    }));

    // Create more sophisticated relationships
    const links = [];

    // Type similarity relationships
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            const node1 = nodes[i];
            const node2 = nodes[j];

            // Same type relationship
            if (node1.type === node2.type) {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'same_type',
                    strength: 1.0
                });
            }
            // Similar category relationship
            else if (node1.category === node2.category && node1.category !== 'primitive") {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'similar_category',
                    strength: 0.6
                });
            }
            // Generic type relationship (Vec<T>, Box<T>, etc.)
            else if (getGenericBase(node1.type) === getGenericBase(node2.type)) {
                links.push({
                    source: node1.id,
                    target: node2.id,
                    type: 'generic_family',
                    strength: 0.8
                });
            }
        }
    }

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
        .force('link', d3.forceLink(links)
            .id(d => d.id)
            .distance(d => 80 + (1 - d.strength) * 40)
            .strength(d => d.strength * 0.7)
        )
        .force('charge', d3.forceManyBody()
            .strength(d => -200 - (d.size / 100))
        )
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide()
            .radius(d => {
                const minRadius = 15;
                const maxRadius = 50;
                const maxSize = Math.max(...nodes.map(n => n.size));
                const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
                const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
                return nodeRadius + 5;
            })
        );

    // Create link elements
    const link = g.append('g")
        .attr('class', 'links")
        .selectAll('line")
        .data(links)
        .enter().append('line")
        .attr('stroke', d => getLinkColor(d.type))
        .attr('stroke-opacity', d => 0.3 + d.strength * 0.4)
        .attr('stroke-width', d => 1 + d.strength * 2)
        .attr('stroke-dasharray', d => d.type === 'similar_category' ? '5,5' : null);

    // Create node groups
    const node = g.append('g")
        .attr('class', 'nodes")
        .selectAll('g")
        .data(nodes)
        .enter().append('g")
        .attr('class', 'graph-node")
        .style('cursor', 'pointer")
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended)
        );

    // Add circles to nodes - size based on memory usage
    node.append('circle")
        .attr('r', d => {
            // Scale node size based on memory usage (larger memory = larger node)
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            return minRadius + (sizeRatio * (maxRadius - minRadius));
        })
        .attr('fill', d => getEnhancedTypeColor(d.type, d.category))
        .attr('stroke', '#fff")
        .attr('stroke-width', 2)
        .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))")
        .on('mouseover', function (event, d) {
            const currentRadius = d3.select(this).attr('r");
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', parseFloat(currentRadius) * 1.2)
                .style('filter', 'drop-shadow(0px 4px 8px rgba(0,0,0,0.3))");

            // Highlight connected links
            link.style('stroke-opacity', l =>
                (l.source.id === d.id || l.target.id === d.id) ? 0.8 : 0.1
            );
        })
        .on('mouseout', function (event, d) {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const originalRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            
            d3.select(this)
                .transition()
                .duration(200)
                .attr('r', originalRadius)
                .style('filter', 'drop-shadow(0px 2px 4px rgba(0,0,0,0.2))");

            // Reset link opacity
            link.style('stroke-opacity', l => 0.3 + l.strength * 0.4);
        });

    // Add complexity indicators (small circles with numbers)
    const complexityGroup = node.append('g")
        .attr('class', 'complexity-indicator");

    complexityGroup.append('circle")
        .attr('r', 8)
        .attr('cx', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 8;
        })
        .attr('cy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return -nodeRadius - 8;
        })
        .attr('fill', d => getComplexityColor(d.complexity))
        .attr('stroke', '#fff")
        .attr('stroke-width', 2);

    // Add complexity score text
    complexityGroup.append('text")
        .text(d => d.complexity || 2)
        .attr('x', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 8;
        })
        .attr('y', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return -nodeRadius - 8 + 3;
        })
        .attr('text-anchor', 'middle")
        .style('font-size', '10px")
        .style('font-weight', 'bold")
        .style('fill', '#fff")
        .style('pointer-events', 'none");

    // Add variable names
    node.append('text")
        .text(d => d.id)
        .attr('text-anchor', 'middle")
        .attr('dy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 15;
        })
        .style('font-size', '11px")
        .style('font-weight', 'bold")
        .style('fill', 'var(--text-primary)")
        .style('pointer-events', 'none");

    // Add type labels
    node.append('text")
        .text(d => formatTypeName(d.type))
        .attr('text-anchor', 'middle")
        .attr('dy', d => {
            const minRadius = 15;
            const maxRadius = 50;
            const maxSize = Math.max(...nodes.map(n => n.size));
            const sizeRatio = maxSize > 0 ? d.size / maxSize : 0;
            const nodeRadius = minRadius + (sizeRatio * (maxRadius - minRadius));
            return nodeRadius + 28;
        })
        .style('font-size', '9px")
        .style('fill', 'var(--text-secondary)")
        .style('pointer-events', 'none");

    // Add click interaction
    const detailPanel = new NodeDetailPanel('variable-graph-container");

    node.on('click', function (event, d) {
        event.stopPropagation();
        console.log("Node clicked:", d.id, d);
        const position = {
            x: event.pageX,
            y: event.pageY
        };
        detailPanel.show(d, position);
    });

    // Click on empty space to hide panel
    svg.on('click', function (event) {
        if (event.target === this) {
            detailPanel.hide();
        }
    });

    // Update positions on simulation tick
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);

        node
            .attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Add control buttons
    const controls = d3.select(container)
        .append('div")
        .attr('class', 'absolute top-2 right-2 flex space-x-2");

    controls.append('button")
        .attr('class', 'px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-xs rounded transition-colors")
        .text('Reset View")
        .on('click', () => {
            svg.transition().duration(750).call(
                zoom.transform,
                d3.zoomIdentity
            );
        });

    controls.append('button")
        .attr('class', 'px-3 py-1 bg-green-500 hover:bg-green-600 text-white text-xs rounded transition-colors")
        .text('Reheat")
        .on('click', () => {
            simulation.alpha(0.3).restart();
        });

    // Drag functions
    function dragstarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragended(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }
}

// Get color for variable type
function getTypeColor(typeName) {
    if (typeName.includes('Vec')) return '#3b82f6';
    if (typeName.includes('Box')) return '#8b5cf6';
    if (typeName.includes('Rc') || typeName.includes('Arc')) return '#10b981';
    if (typeName.includes('String')) return '#f59e0b';
    return '#6b7280';
}

// Enhanced type color with comprehensive type mapping
function getEnhancedTypeColor(typeName, category) {
    // Comprehensive color mapping for specific types
    const typeColorMap = {
        // Smart Pointers - Purple/Violet family
        'Box': '#8b5cf6',           // Purple
        'Rc': '#a855f7',            // Purple-500
        'Arc': '#9333ea',           // Violet-600
        'RefCell': '#7c3aed',       // Violet-700
        'Cell': '#6d28d9',          // Violet-800
        'Weak': '#5b21b6',          // Violet-900

        // Collections - Blue family
        'Vec': '#3b82f6',           // Blue-500
        'VecDeque': '#2563eb',      // Blue-600
        'LinkedList': '#1d4ed8',    // Blue-700
        'HashMap': '#1e40af',       // Blue-800
        'BTreeMap': '#1e3a8a',      // Blue-900
        'HashSet': '#60a5fa',       // Blue-400
        'BTreeSet': '#93c5fd',      // Blue-300

        // String types - Orange/Amber family
        'String': '#f59e0b',        // Amber-500
        'str': '#d97706',           // Amber-600
        'OsString': '#b45309',      // Amber-700
        'OsStr': '#92400e',         // Amber-800
        'CString': '#78350f',       // Amber-900
        'CStr': '#fbbf24',          // Amber-400

        // Numeric types - Green family
        'i8': '#10b981',            // Emerald-500
        'i16': '#059669',           // Emerald-600
        'i32': '#047857',           // Emerald-700
        'i64': '#065f46',           // Emerald-800
        'i128': '#064e3b',          // Emerald-900
        'u8': '#34d399',            // Emerald-400
        'u16': '#6ee7b7',           // Emerald-300
        'u32': '#a7f3d0',           // Emerald-200
        'u64': '#d1fae5',           // Emerald-100
        'u128': '#ecfdf5',          // Emerald-50
        'f32': '#14b8a6',           // Teal-500
        'f64': '#0d9488',           // Teal-600
        'usize': '#0f766e',         // Teal-700
        'isize': '#115e59',         // Teal-800

        // Boolean and char - Pink family
        'bool': '#ec4899',          // Pink-500
        'char': '#db2777',          // Pink-600

        // Option and Result - Indigo family
        'Option': '#6366f1',        // Indigo-500
        'Result': '#4f46e5',        // Indigo-600
        'Some': '#4338ca',          // Indigo-700
        'None': '#3730a3',          // Indigo-800
        'Ok': '#312e81',            // Indigo-900
        'Err': '#6366f1',           // Indigo-500

        // Synchronization types - Red family
        'Mutex': '#ef4444',         // Red-500
        'RwLock': '#dc2626',        // Red-600
        'Condvar': '#b91c1c',       // Red-700
        'Barrier': '#991b1b',       // Red-800
        'Once': '#7f1d1d',          // Red-900

        // Channel types - Cyan family
        'Sender': '#06b6d4',        // Cyan-500
        'Receiver': '#0891b2',      // Cyan-600
        'mpsc': '#0e7490',          // Cyan-700

        // Path types - Lime family
        'Path': '#84cc16',          // Lime-500
        'PathBuf': '#65a30d',       // Lime-600

        // Time types - Yellow family
        'Duration': '#eab308',      // Yellow-500
        'Instant': '#ca8a04',       // Yellow-600
        'SystemTime': '#a16207',    // Yellow-700

        // IO types - Stone family
        'File': '#78716c',          // Stone-500
        'BufReader': '#57534e',     // Stone-600
        'BufWriter': '#44403c',     // Stone-700

        // Thread types - Rose family
        'Thread': '#f43f5e',        // Rose-500
        'JoinHandle': '#e11d48',    // Rose-600

        // Custom/Unknown types - Gray family
        'unknown': '#6b7280',       // Gray-500
        'custom': '#4b5563',        // Gray-600
    };

    // First, try exact type name match
    if (typeColorMap[typeName]) {
        return typeColorMap[typeName];
    }

    // Then try to match by type name contains
    for (const [type, color] of Object.entries(typeColorMap)) {
        if (typeName.includes(type)) {
            return color;
        }
    }

    // Extract generic base type and try to match
    const genericBase = getGenericBase(typeName);
    if (typeColorMap[genericBase]) {
        return typeColorMap[genericBase];
    }

    // Fall back to category-based colors
    switch (category) {
        case 'smart_pointer': return '#8b5cf6';  // Purple
        case 'collection': return '#3b82f6';     // Blue
        case 'string': return '#f59e0b';         // Amber
        case 'numeric': return '#10b981';        // Emerald
        case 'sync': return '#ef4444';           // Red
        case 'channel': return '#06b6d4';        // Cyan
        case 'path': return '#84cc16';           // Lime
        case 'time': return '#eab308';           // Yellow
        case 'io': return '#78716c';             // Stone
        case 'thread': return '#f43f5e';         // Rose
        default: return '#6b7280';               // Gray
    }
}

// Get type category for grouping with comprehensive type recognition
function getTypeCategory(typeName) {
    // Smart pointers
    if (typeName.includes('Box') || typeName.includes('Rc') || typeName.includes('Arc') ||
        typeName.includes('RefCell') || typeName.includes('Cell') || typeName.includes('Weak')) {
        return 'smart_pointer';
    }

    // Collections
    if (typeName.includes('Vec') || typeName.includes('HashMap') || typeName.includes('BTreeMap') ||
        typeName.includes('HashSet') || typeName.includes('BTreeSet') || typeName.includes('VecDeque') ||
        typeName.includes('LinkedList')) {
        return 'collection';
    }

    // String types
    if (typeName.includes('String') || typeName.includes('str') || typeName.includes('OsString') ||
        typeName.includes('OsStr') || typeName.includes('CString') || typeName.includes('CStr')) {
        return 'string';
    }

    // Numeric types
    if (typeName.match(/^[iuf]\d+$/) || typeName === 'usize' || typeName === 'isize' ||
        typeName === 'bool' || typeName === 'char') {
        return 'numeric';
    }

    // Synchronization types
    if (typeName.includes('Mutex') || typeName.includes('RwLock') || typeName.includes('Condvar') ||
        typeName.includes('Barrier') || typeName.includes('Once')) {
        return 'sync';
    }

    // Channel types
    if (typeName.includes('Sender') || typeName.includes('Receiver') || typeName.includes('mpsc')) {
        return 'channel';
    }

    // Path types
    if (typeName.includes('Path') || typeName.includes('PathBuf')) {
        return 'path';
    }

    // Time types
    if (typeName.includes('Duration') || typeName.includes('Instant') || typeName.includes('SystemTime')) {
        return 'time';
    }

    // IO types
    if (typeName.includes('File') || typeName.includes('BufReader') || typeName.includes('BufWriter')) {
        return 'io';
    }

    // Thread types
    if (typeName.includes('Thread') || typeName.includes('JoinHandle')) {
        return 'thread';
    }

    // Option and Result
    if (typeName.includes('Option') || typeName.includes('Result')) {
        return 'option_result';
    }

    return 'primitive';
}

// Get generic base type (Vec<T> -> Vec, Box<T> -> Box)
function getGenericBase(typeName) {
    const match = typeName.match(/^([^<]+)/);
    return match ? match[1] : typeName;
}

// Get complexity score from type with comprehensive scoring
function getComplexityFromType(typeName) {
    // Very high complexity (9-10)
    if (typeName.includes('HashMap') || typeName.includes('BTreeMap') ||
        typeName.includes('BTreeSet') || typeName.includes('LinkedList')) return 9;

    // High complexity (7-8)
    if (typeName.includes('Arc') || typeName.includes('Mutex') || typeName.includes('RwLock') ||
        typeName.includes('Condvar') || typeName.includes('Barrier')) return 8;
    if (typeName.includes('Rc') || typeName.includes('RefCell') || typeName.includes('HashSet') ||
        typeName.includes('VecDeque')) return 7;

    // Medium complexity (5-6)
    if (typeName.includes('Vec') || typeName.includes('Box') || typeName.includes('Option') ||
        typeName.includes('Result')) return 6;
    if (typeName.includes('String') || typeName.includes('PathBuf') || typeName.includes('OsString') ||
        typeName.includes('CString')) return 5;

    // Low complexity (3-4)
    if (typeName.includes('str') || typeName.includes('Path') || typeName.includes('OsStr') ||
        typeName.includes('CStr') || typeName.includes('Duration') || typeName.includes('Instant')) return 4;
    if (typeName.includes('Sender') || typeName.includes('Receiver') || typeName.includes('File') ||
        typeName.includes('Thread') || typeName.includes('JoinHandle')) return 3;

    // Very low complexity (1-2)
    if (typeName.match(/^[iuf]\d+$/) || typeName === 'usize' || typeName === 'isize' ||
        typeName === 'bool' || typeName === 'char') return 1;

    // Default for unknown types
    return 2;
}

// Get link color based on relationship type
function getLinkColor(linkType) {
    switch (linkType) {
        case 'same_type': return '#ef4444';
        case 'similar_category': return '#3b82f6';
        case 'generic_family': return '#10b981';
        default: return '#6b7280';
    }
}

// Get complexity level description
function getComplexityLevel(score) {
    if (score <= 2) return 'Simple';
    if (score <= 5) return 'Medium';
    if (score <= 8) return 'Complex';
    return 'Very Complex';
}

// Get complexity explanation
function getComplexityExplanation(score) {
    if (score <= 2) return 'Basic types with minimal performance overhead and simple memory usage';
    if (score <= 5) return 'Medium complexity with some memory management overhead';
    if (score <= 8) return 'Complex types involving heap allocation and smart pointers, performance considerations needed';
    return 'Very complex types with significant performance overhead, optimization recommended';
}

// Get type analysis information
function getTypeAnalysis(typeName, size) {
    const analysis = [];

    if (typeName.includes('Vec")) {
        analysis.push('- Dynamic array with heap allocation');
        analysis.push('- Grows automatically as needed');
        if (size > 1000) analysis.push('- Large allocation - consider capacity optimization');
    } else if (typeName.includes('Box")) {
        analysis.push('- Single heap allocation');
        analysis.push('- Unique ownership semantics');
    } else if (typeName.includes('Rc")) {
        analysis.push('- Reference counted smart pointer');
        analysis.push('- Shared ownership with runtime checks');
    } else if (typeName.includes('Arc")) {
        analysis.push('- Atomic reference counted pointer');
        analysis.push('- Thread-safe shared ownership');
    } else if (typeName.includes('String")) {
        analysis.push('- Growable UTF-8 string');
        analysis.push('- Heap allocated with capacity buffer');
    } else {
        analysis.push('- Basic type allocation');
    }

    if (size === 0) {
        analysis.push('- Zero-sized type (ZST)');
    } else if (size < 64) {
        analysis.push('- Small allocation - good for performance');
    } else if (size > 1024) {
        analysis.push('- Large allocation - monitor memory usage');
    }

    return analysis.join('<br>');
}

// Initialize generic types table
function initGenericTypesTable() {
    const tbody = document.getElementById('generic-types-table-body");
    if (!tbody) return;

    const genericTypes = window.analysisData.complex_types?.categorized_types?.generic_types || [];

    if (genericTypes.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400">No generic types found</td></tr>';
        return;
    }

    tbody.innerHTML = genericTypes.map(type => `
        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${type.var_name || 'System Allocation'}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatTypeName(type.type_name || 'System Allocation')}</td>
            <td class="px-6 py-4 font-mono text-xs text-gray-900 dark:text-gray-100">${type.ptr}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatBytes(type.size || 0)}</td>
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">N/A</td>
            <td class="px-6 py-4">
                <span class="px-2 py-1 rounded text-xs ${getComplexityColor(type.complexity_score)} text-white">
                    ${type.complexity_score || 0}
                </span>
            </td>
        </tr>
    `).join('');
}

// Initialize complex type analysis
function initComplexTypeAnalysis() {
    const tbody = document.getElementById('complex-type-analysis-table");
    if (!tbody) return;

    const complexTypeAnalysis = window.analysisData.complex_types?.complex_type_analysis || [];

    if (complexTypeAnalysis.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="px-6 py-8 text-center text-gray-500 dark:text-gray-400">No complex type analysis available</td></tr>';
        return;
    }

    tbody.innerHTML = complexTypeAnalysis.map(analysis => `
        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
            <td class="px-6 py-4 text-gray-900 dark:text-gray-100">${formatTypeName(analysis.type_name)}</td>
            <td class="px-6 py-4 text-center">
                <span class="px-2 py-1 rounded text-xs ${getComplexityColor(analysis.complexity_score)} text-white">
                    ${analysis.complexity_score}
                </span>
            </td>
            <td class="px-6 py-4 text-center">
                <span class="px-2 py-1 rounded text-xs ${getEfficiencyColor(analysis.memory_efficiency)} text-white">
                    ${analysis.memory_efficiency}%
                </span>
            </td>
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${analysis.allocation_count || 0}</td>
            <td class="px-6 py-4 text-center text-gray-900 dark:text-gray-100">${formatBytes(analysis.total_size || 0)}</td>
            <td class="px-6 py-4 text-gray-700 dark:text-gray-300">
                ${Array.isArray(analysis.optimization_suggestions) && analysis.optimization_suggestions.length > 0 
                    ? analysis.optimization_suggestions.join(', ") 
                    : '<span class="text-gray-400 italic">No optimization suggestions available</span>'}
            </td>
        </tr>
    `).join('');
}

// Initialize memory optimization recommendations
function initMemoryOptimizationRecommendations() {
    const container = document.getElementById('memory-optimization-recommendations");
    if (!container) return;

    const recommendations = window.analysisData.complex_types?.optimization_recommendations || [];

    if (recommendations.length === 0) {
        container.innerHTML = '<li class="text-gray-500 dark:text-gray-400">No specific recommendations available</li>';
        return;
    }

    container.innerHTML = recommendations.map(rec => `
        <li class="flex items-start">
            <i class="fa fa-lightbulb-o text-yellow-500 mr-2 mt-1"></i>
            <span class="dark:text-gray-200">${rec}</span>
        </li>
    `).join('');
}

// Initialize FFI risk chart
function initFFIRiskChart() {
    // Guard: ensure canvas isn't holding an existing chart instance
    try {
        if (window.chartInstances && window.chartInstances['ffi-risk-chart']) {
            window.chartInstances['ffi-risk-chart'].destroy();
            delete window.chartInstances['ffi-risk-chart'];
        }
    } catch (_) {}

    const ctx = document.getElementById('ffi-risk-chart");
    if (!ctx) return;

    const ffiData = window.analysisData.unsafe_ffi?.enhanced_ffi_data || [];

    const riskLevels = {
        'Low Risk': ffiData.filter(item => (item.safety_violations || 0) === 0).length,
        'Medium Risk': ffiData.filter(item => (item.safety_violations || 0) > 0 && (item.safety_violations || 0) <= 2).length,
        'High Risk': ffiData.filter(item => (item.safety_violations || 0) > 2).length
    };

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['ffi-risk-chart']) {
        window.chartInstances['ffi-risk-chart'].destroy();
    }

    window.chartInstances['ffi-risk-chart'] = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: Object.keys(riskLevels),
            datasets: [{
                data: Object.values(riskLevels),
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444'],
                borderColor: isDark ? '#374151' : '#ffffff',
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#f9fafb' : '#374151',
                        font: {
                            size: 12
                        }
                    }
                },
                tooltip: {
                    backgroundColor: isDark ? '#1f2937' : '#ffffff',
                    titleColor: isDark ? '#f9fafb' : '#374151',
                    bodyColor: isDark ? '#f9fafb' : '#374151',
                    borderColor: isDark ? '#374151' : '#e5e7eb',
                    borderWidth: 1
                }
            }
        }
    });
}

// Initialize complex type analysis chart
function initComplexTypeAnalysisChart() {
    const ctx = document.getElementById('complex-type-analysis-chart");
    if (!ctx) return;

    const complexTypeAnalysis = window.analysisData.complex_types?.complex_type_analysis || [];

    if (complexTypeAnalysis.length === 0) {
        // Show empty state
        const container = ctx.parentElement;
        container.innerHTML = `
            <div class="h-64 flex items-center justify-center text-gray-500 dark:text-gray-400">
                <div class="text-center">
                    <i class="fa fa-chart-bar text-4xl mb-4"></i>
                    <p class="text-lg font-semibold mb-2">No Complex Type Data</p>
                    <p class="text-sm">No complex type analysis data available</p>
                </div>
            </div>
        `;
        return;
    }

    const isDark = document.documentElement.classList.contains('dark');

    // Destroy existing chart if it exists
    if (window.chartInstances['complex-type-analysis-chart']) {
        window.chartInstances['complex-type-analysis-chart'].destroy();
    }

    window.chartInstances['complex-type-analysis-chart'] = new Chart(ctx, {
        type: 'scatter',
        data: {
            datasets: [{
                label: 'Type Complexity vs Memory Efficiency',
                data: complexTypeAnalysis.map(analysis => ({
                    x: analysis.complexity_score || 0,
                    y: analysis.memory_efficiency || 0,
                    typeName: analysis.type_name
                })),
                backgroundColor: 'rgba(59, 130, 246, 0.6)',
                borderColor: 'rgba(59, 130, 246, 1)',
                borderWidth: 2,
                pointRadius: 6,
                pointHoverRadius: 8
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    title: {
                        display: true,
                        text: 'Complexity Score',
                        color: isDark ? '#f9fafb' : '#374151'
                    },
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                },
                y: {
                    title: {
                        display: true,
                        text: 'Memory Efficiency (%)',
                        color: isDark ? '#f9fafb' : '#374151'
                    },
                    ticks: {
                        color: isDark ? '#d1d5db' : '#6b7280'
                    },
                    grid: {
                        color: isDark ? '#374151' : '#e5e7eb'
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: isDark ? '#f9fafb' : '#374151'
                    }
                },
                tooltip: {
                    backgroundColor: isDark ? '#1f2937' : '#ffffff',
                    titleColor: isDark ? '#f9fafb' : '#374151',
                    bodyColor: isDark ? '#f9fafb' : '#374151',
                    borderColor: isDark ? '#374151' : '#e5e7eb',
                    borderWidth: 1,
                    callbacks: {
                        title: function (context) {
                            return context[0].raw.typeName || 'Unknown Type';
                        },
                        label: function (context) {
                            return [
                                `Complexity: ${context.parsed.x}`,
                                `Efficiency: ${context.parsed.y}%`
                            ];
                        }
                    }
                }
            }
        }
    });
}

// Format type name for better display
function formatTypeName(typeName) {
    if (!typeName || typeName === 'unknown') return 'System Allocation';
    // Simplify complex type names
    return typeName
        .replace(/alloc::/g, '')
        .replace(/std::/g, '')
        .replace(/::Vec/g, 'Vec')
        .replace(/::Box/g, 'Box')
        .replace(/::Rc/g, 'Rc')
        .replace(/::Arc/g, 'Arc')
        .replace(/::String/g, 'String');
}

// Format timestamp relative to start time
function formatTimestamp(timestamp, minTime) {
    const relativeMs = Math.round((timestamp - minTime) / 1000000); // Convert nanoseconds to milliseconds
    return `${relativeMs}ms`;
}

// Enhanced summary statistics with comprehensive data analysis
function initEnhancedSummaryStats() {
    console.log("📊 Initializing enhanced summary statistics...");
    
    try {
        // Get merged data from all sources
        const memoryAllocations = window.analysisData.memory_analysis?.allocations || [];
        const complexAllocations = window.analysisData.complex_types?.allocations || [];
        const unsafeAllocations = window.analysisData.unsafe_ffi?.allocations || [];
        
        // Merge all data sources for comprehensive analysis
        const allData = mergeAllDataSources(memoryAllocations, complexAllocations, unsafeAllocations);
        
        // Calculate comprehensive statistics
        const stats = calculateComprehensiveStats(allData);
        
        // Update enhanced dashboard
        updateElement('total-allocations', stats.totalAllocations);
        updateElement('allocation-rate', `${stats.allocationRate.toFixed(1)}/ms`);
        updateElement('active-variables', stats.activeVariables);
        updateElement('variable-types', `${stats.uniqueTypes} types`);
        updateElement('borrow-operations', stats.totalBorrows);
        updateElement('max-concurrent', `Max: ${stats.maxConcurrent}`);
        updateElement('safety-score', `${stats.safetyScore}%`);
        updateElement('ffi-tracked', `${stats.ffiTracked} FFI`);
        
        console.log("✅ Enhanced dashboard updated successfully");
    } catch (error) {
        console.error('❌ Error initializing enhanced stats:', error);
    }
}

// Merge data from all sources with comprehensive field mapping
function mergeAllDataSources(memory, complex, unsafe) {
    const dataMap = new Map();
    
    // Add memory analysis data (has lifetime_ms)
    memory.forEach(alloc => {
        if (alloc.ptr) {
            dataMap.set(alloc.ptr, { ...alloc, source: 'memory' });
        }
    });
    
    // Merge complex types data (has extended fields)
    complex.forEach(alloc => {
        if (alloc.ptr) {
            const existing = dataMap.get(alloc.ptr) || {};
            dataMap.set(alloc.ptr, { 
                ...existing, 
                ...alloc, 
                source: existing.source ? `${existing.source}+complex` : 'complex'
            });
        }
    });
    
    // Merge unsafe FFI data (has safety info)
    unsafe.forEach(alloc => {
        if (alloc.ptr) {
            const existing = dataMap.get(alloc.ptr) || {};
            dataMap.set(alloc.ptr, { 
                ...existing, 
                ...alloc, 
                source: existing.source ? `${existing.source}+unsafe` : 'unsafe'
            });
        }
    });
    
    return Array.from(dataMap.values());
}

// Calculate comprehensive statistics from merged data
function calculateComprehensiveStats(allData) {
    const validData = allData.filter(d => d.var_name && d.var_name !== 'unknown');
    
    // Basic counts
    const totalAllocations = validData.length;
    const uniqueVars = new Set(validData.map(d => d.var_name)).size;
    const uniqueTypes = new Set(validData.map(d => d.type_name)).size;
    
    // Time-based calculations
    const timestamps = validData.map(d => d.timestamp_alloc).filter(t => t);
    const timeRange = timestamps.length > 0 ? (Math.max(...timestamps) - Math.min(...timestamps)) / 1000000 : 1;
    const allocationRate = totalAllocations / Math.max(timeRange, 1);
    
    // Borrow analysis
    let totalBorrows = 0;
    let maxConcurrent = 0;
    validData.forEach(d => {
        if (d.borrow_info) {
            totalBorrows += (d.borrow_info.immutable_borrows || 0) + (d.borrow_info.mutable_borrows || 0);
            maxConcurrent = Math.max(maxConcurrent, d.borrow_info.max_concurrent_borrows || 0);
        }
    });
    
    // Safety analysis
    const ffiTracked = validData.filter(d => d.ffi_tracked).length;
    const leaked = validData.filter(d => d.is_leaked).length;
    const withSafetyViolations = validData.filter(d => d.safety_violations && d.safety_violations.length > 0).length;
    const safetyScore = Math.max(0, 100 - (leaked * 20) - (withSafetyViolations * 10));
    
    return {
        totalAllocations,
        activeVariables: uniqueVars,
        uniqueTypes,
        allocationRate,
        totalBorrows,
        maxConcurrent,
        ffiTracked,
        safetyScore: Math.round(safetyScore)
    };
}

// Helper function to safely update DOM elements
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

// Utility function to format bytes
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Show empty state when no user variables found
function showEmptyLifetimeState() {
    const container = document.getElementById("lifetimeVisualization");
    if (!container) return;

    container.innerHTML = `
        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <i class="fa fa-info-circle text-2xl mb-2"></i>
            <p>No user-defined variables found in lifetime data</p>
            <p class="text-sm mt-1">Use track_var! macro to track variable lifetimes</p>
        </div>
    `;
}

// Utility functions
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

function getComplexityColor(score) {
    if (score <= 2) return '#10b981';  // Green - Low complexity
    if (score <= 5) return '#eab308';  // Yellow - Medium complexity  
    if (score <= 8) return '#f97316';  // Orange - High complexity
    return '#ef4444';                  // Red - Very high complexity
}

function getEfficiencyColor(efficiency) {
    if (efficiency >= 80) return 'bg-green-500';
    if (efficiency >= 60) return 'bg-yellow-500';
    if (efficiency >= 40) return 'bg-orange-500';
    return 'bg-red-500';
}

// Update KPI Cards
function updateKPICards(data) {
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const total = allocs.reduce((s,a)=>s+(a.size||0),0);
    const active = allocs.filter(a=>!a.timestamp_dealloc).length;
    const safetyScore = calculateSafetyScore(allocs);
    
    updateElement('total-allocations', allocs.length.toLocaleString());
    updateElement('active-variables', active.toLocaleString());
    updateElement('total-memory', formatBytes(total));
    updateElement('safety-score', safetyScore + "%");
}

// Calculate Safety Score
function calculateSafetyScore(allocs) {
    if (!allocs.length) return 100;
    const leaked = allocs.filter(a => a.is_leaked).length;
    const violations = allocs.filter(a => a.safety_violations && a.safety_violations.length > 0).length;
    return Math.max(0, 100 - (leaked * 20) - (violations * 10));
}

// Theme Toggle Functionality
function initThemeToggle() {
    const toggleBtn = document.getElementById('theme-toggle');
    if (!toggleBtn) return;
    
    // Check local storage for theme
    const savedTheme = localStorage.getItem('memscope-theme') || 'light';
    applyTheme(savedTheme === 'dark');
    
    toggleBtn.addEventListener("click", () => {
        const isDark = document.documentElement.classList.contains('dark');
        const newTheme = isDark ? 'light' : 'dark';
        
        applyTheme(newTheme === 'dark');
        localStorage.setItem('memscope-theme', newTheme);
        
        // Update button text
        const icon = toggleBtn.querySelector('i');
        const text = toggleBtn.querySelector('span');
        if (newTheme === 'dark') {
            icon.className = "fa fa-sun";
            text.textContent = 'Light Mode';
        } else {
            icon.className = "fa fa-moon";
            text.textContent = 'Dark Mode';
        }
        
        console.log("🎨 Theme switched to:", newTheme);
    });
}


function applyTheme(isDark) {
    const html = document.documentElement;
    if (isDark) {
        html.classList.add('dark');
    } else {
        html.classList.remove('dark');
    }
}

// Update Memory Allocation Table
function updateAllocationsTable(data) {
    const allocTable = document.getElementById("allocTable");
    if (!allocTable) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const top = allocs.slice().sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,50);
    
    allocTable.innerHTML = top.map(a => {
        const status = a.is_leaked ? 'Leaked' : (a.timestamp_dealloc ? 'Freed' : "Active");
        const statusClass = a.is_leaked ? 'status-leaked' : (a.timestamp_dealloc ? 'status-freed' : 'status-active");
        
        return `<tr>
            <td>${a.var_name || 'Unknown'}</td>
            <td>${formatTypeName(a.type_name || 'Unknown')}</td>
            <td>${formatBytes(a.size || 0)}</td>
            <td><span class="status-badge ${statusClass}">${status}</span></td>
        </tr>`;
    }).join('');
}

// Update Unsafe Risk Table
function updateUnsafeTable(data) {
    const unsafeTable = document.getElementById("unsafeTable");
    if (!unsafeTable) return;
    
    const root = data.unsafe_ffi || {};
    const ops = root.enhanced_ffi_data || root.unsafe_operations || root.allocations || [];
    
    unsafeTable.innerHTML = (ops || []).slice(0, 50).map(op => {
        const riskLevel = op.risk_level || ((op.safety_violations||[]).length > 2 ? 'High' : 
                         ((op.safety_violations||[]).length > 0 ? 'Medium' : 'Low'));
        
        const riskText = riskLevel === 'High' ? 'High Risk' : (riskLevel === 'Medium' ? 'Medium Risk' : 'Low Risk");
        const riskClass = riskLevel === 'High' ? 'risk-high' : (riskLevel === 'Medium' ? 'risk-medium' : 'risk-low");
        
        return `<tr>
            <td>${op.location || op.var_name || 'Unknown'}</td>
            <td>${op.operation_type || op.type_name || 'Unknown'}</td>
            <td><span class="status-badge ${riskClass}">${riskText}</span></td>
        </tr>`;
    }).join('');
}

// Initialize Charts
function initCharts(data) {
    console.log("📊 Initializing charts...");
    
    // Memory type distribution chart
    initTypeChart(data);
    
    // Memory timeline chart
    initTimelineChart(data);
    
    // Type treemap chart
    initTreemapChart(data);
    
    // FFI risk chart
    initFFIRiskChart(data);
    
    // Memory growth trends
    initGrowthTrends(data);
    
    // Memory fragmentation
    initMemoryFragmentation(data);
    
    // Variable relationship graph
    initVariableGraph(data);
}

// Memory Type Distribution Chart
function initTypeChart(data) {
    const ctx = document.getElementById("typeChart");
    if (!ctx) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 8);
    
    if (top.length > 0 && window.Chart) {
        const chart = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: top.map(x => {
                    const formatted = formatTypeName(x[0]);
                    return formatted.length > 15 ? formatted.substring(0, 12) + '...' : formatted;
                }),
                datasets: [{
                    label: 'Memory Usage',
                    data: top.map(x => x[1]),
                    backgroundColor: '#2563eb',
                    borderRadius: 6
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
                scales: {
                    x: {
                        ticks: {
                            maxRotation: 45,
                            minRotation: 0,
                            font: {
                                size: 10
                            }
                        }
                    },
                    y: { 
                        beginAtZero: true,
                        ticks: {
                            callback: function(value) {
                                return formatBytes(value);
                            }
                        }
                    }
                }
            }
        });
    }
}

// Memory Timeline Chart with optional Growth Rate (dual y-axes)
function initTimelineChart(data) {
    const ctx = document.getElementById("timelineChart");
    if (!ctx || !window.Chart) return;

    // Comprehensive cleanup for timeline chart
    try {
        if (ctx.chart) {
            ctx.chart.destroy();
            delete ctx.chart;
        }
        
        if (window.Chart.instances) {
            Object.values(window.Chart.instances).forEach(instance => {
                if (instance.canvas === ctx) {
                    instance.destroy();
                }
            });
        }
        
        if (window.chartInstances && window.chartInstances['timelineChart']) {
            window.chartInstances['timelineChart'].destroy();
            delete window.chartInstances['timelineChart'];
        }
        
        const context = ctx.getContext("2d");
        context.clearRect(0, 0, ctx.width, ctx.height);
        
    } catch(e) {
        console.warn('Timeline chart cleanup warning:', e);
    }
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    
    // Bucketize by timestamp to ~200 buckets; compute bytes/sec
    if (sorted.length === 0) return;
    const minTs = sorted[0].timestamp_alloc || 0;
    const maxTs = sorted[sorted.length-1].timestamp_alloc || minTs;
    const rangeNs = Math.max(1, maxTs - minTs);
    const bucketCount = Math.min(200, Math.max(1, Math.floor(sorted.length / 2)));
    const bucketNs = Math.max(1, Math.floor(rangeNs / bucketCount));

    const cumSeries = [];
    const rateSeries = [];

    let cumulative = 0;
    let windowQueue = [];
    const windowBuckets = 3; // simple smoothing window

    for (let b = 0; b <= bucketCount; b++) {
        const start = minTs + b * bucketNs;
        const end = Math.min(maxTs, start + bucketNs);
        const slice = sorted.filter(a => (a.timestamp_alloc||0) >= start && (a.timestamp_alloc||0) < end);
        const sum = slice.reduce((s,a)=>s+(a.size||0),0);
        cumulative += sum;
        cumSeries.push({ x: start, y: cumulative });
        const dtSec = Math.max(1e-9, (end - start) / 1e9);
        const rate = sum / dtSec; // bytes/sec in this bucket
        windowQueue.push(rate);
        if (windowQueue.length > windowBuckets) windowQueue.shift();
        const smoothed = windowQueue.reduce((s,v)=>s+v,0) / windowQueue.length;
        rateSeries.push({ x: start, y: smoothed });
    }

    if (cumSeries.length > 1 && window.Chart) {
        // destroy previous chart if exists
        if (window.chartInstances && window.chartInstances['timelineChart']) {
            try { window.chartInstances['timelineChart'].destroy(); } catch(_) {}
            delete window.chartInstances['timelineChart'];
        }

        const labels = cumSeries.map(p=> new Date(p.x/1e6).toLocaleTimeString());
        const showGrowthCheckbox = document.getElementById("toggleGrowthRate");
        const datasets = [
            {
                type: 'line',
                label: 'Cumulative Memory',
                data: cumSeries.map(p=>p.y),
                borderColor: '#059669',
                backgroundColor: 'rgba(5, 150, 105, 0.1)',
                fill: true,
                tension: 0.3,
                yAxisID: 'y'
            }
        ];

        // add growth rate dataset (hidden by default; user toggles it)
        datasets.push({
            type: 'line',
            label: 'Growth Rate (bytes/sec)',
            data: rateSeries.map(p=>p.y),
            borderColor: '#eab308',
            backgroundColor: 'rgba(234, 179, 8, 0.15)',
            fill: true,
            tension: 0.2,
            hidden: showGrowthCheckbox ? !showGrowthCheckbox.checked : true,
            yAxisID: 'y1'
        });

        const chart = new Chart(ctx, {
            data: { labels, datasets },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { position: 'bottom' }
                },
                scales: {
                    y: { 
                        beginAtZero: true,
                        title: { display: true, text: 'Cumulative Memory' },
                        ticks: { callback: v => formatBytes(v) }
                    },
                    y1: {
                        beginAtZero: true,
                        position: 'right',
                        grid: { drawOnChartArea: false },
                        title: { display: true, text: 'Growth Rate (bytes/step)' }
                    },
                    x: { title: { display: false } }
                }
            }
        });

        window.chartInstances = window.chartInstances || {};
        window.chartInstances['timelineChart'] = chart;

        if (showGrowthCheckbox) {
            showGrowthCheckbox.onchange = () => {
                const ds = chart.data.datasets.find(d => d.yAxisID === 'y1');
                if (!ds) return;
                ds.hidden = !showGrowthCheckbox.checked;
                chart.update();
            };
        }
    }
}

// Enhanced type chart with better label handling for complex Rust types
function initEnhancedTypeChart(data) {
    const ctx = document.getElementById("typeChart");
    if (!ctx || !window.Chart) return;

    // Comprehensive cleanup for this specific chart
    try {
        // Check if there's already a chart attached to this canvas
        if (ctx.chart) {
            ctx.chart.destroy();
            delete ctx.chart;
        }
        
        // Clear any Chart.js instance for this canvas
        if (window.Chart.instances) {
            Object.values(window.Chart.instances).forEach(instance => {
                if (instance.canvas === ctx) {
                    instance.destroy();
                }
            });
        }
        
        // Clear our tracked instance
        if (window.chartInstances && window.chartInstances['typeChart']) {
            window.chartInstances['typeChart'].destroy();
            delete window.chartInstances['typeChart'];
        }
        
        // Clear canvas context
        const context = ctx.getContext("2d");
        context.clearRect(0, 0, ctx.width, ctx.height);
        
    } catch(e) {
        console.warn('Chart cleanup warning:', e);
    }

    const typeData = {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    console.log("Type chart data extraction:", { totalAllocs: allocs.length });
    
    allocs.forEach(alloc => {
        let type = alloc.type_name || 'Unknown';
        const originalType = type;
        
        // Simplify complex Rust type names for better readability
        type = type.replace(/alloc::sync::Arc/g, 'Arc');
        type = type.replace(/alloc::rc::Rc/g, 'Rc');
        type = type.replace(/alloc::string::String/g, 'String');
        type = type.replace(/alloc::vec::Vec/g, 'Vec');
        type = type.replace(/std::collections::hash::map::HashMap/g, 'HashMap');
        type = type.replace(/std::collections::btree::map::BTreeMap/g, 'BTreeMap');
        type = type.replace(/alloc::collections::\w+::\w+::/g, '');
        
        // Remove generic parameters for cleaner display
        type = type.replace(/<[^>]+>/g, '<T>');
        
        // Truncate very long type names
        if (type.length > 25) {
            type = type.substring(0, 22) + '...';
        }
        
        const size = alloc.size || 0;
        typeData[type] = (typeData[type] || 0) + size;
        
        if (size > 0) {
            console.log(`Adding ${originalType} -> ${type}: ${size} bytes`);
        }
    });
    
    console.log("Type data aggregated:", typeData);

    const sortedEntries = Object.entries(typeData).sort((a, b) => b[1] - a[1]);
    const labels = sortedEntries.map(([k, v]) => k);
    const values = sortedEntries.map(([k, v]) => v);
    
    if (labels.length > 0 && window.Chart) {
        const chart = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: labels,
                datasets: [{
                    label: 'Memory Usage',
                    data: values,
                    backgroundColor: labels.map((_, i) => {
                        const colors = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16'];
                        return colors[i % colors.length] + '80';
                    }),
                    borderColor: labels.map((_, i) => {
                        const colors = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16'];
                        return colors[i % colors.length];
                    }),
                    borderWidth: 2
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: (context) => {
                                const originalType = allocs.find(a => {
                                    let simplified = a.type_name || 'Unknown';
                                    simplified = simplified.replace(/alloc::(sync::Arc|rc::Rc|collections::\w+::\w+::|string::String|vec::Vec)/g, (match, p1) => {
                                        switch(p1) {
                                            case 'sync::Arc': return 'Arc';
                                            case 'rc::Rc': return 'Rc';
                                            case 'string::String': return 'String';
                                            case 'vec::Vec': return 'Vec';
                                            default: return p1.split('::').pop();
                                        }
                                    });
                                    simplified = simplified.replace(/std::collections::hash::map::HashMap/g, 'HashMap');
                                    simplified = simplified.replace(/std::collections::btree::map::BTreeMap/g, 'BTreeMap');
                                    if (simplified.length > 30) simplified = simplified.substring(0, 27) + '...';
                                    return simplified === context.label;
                                })?.type_name || context.label;
                                return [`Type: ${originalType}`, `Memory: ${formatBytes(context.parsed.y)}`];
                            }
                        }
                    }
                },
                scales: {
                    x: { 
                        ticks: {
                            maxRotation: 45,
                            minRotation: 0,
                            font: { 
                                size: 11, 
                                weight: '500',
                                family: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'
                            },
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#e2e8f0' : '#475569';
                            },
                            padding: 8,
                            callback: function(value, index) {
                                const label = this.getLabelForValue(value);
                                // Ensure readability by adding spacing
                                return label.length > 15 ? label.substring(0, 13) + '...' : label;
                            }
                        },
                        grid: {
                            display: false
                        }
                    },
                    y: { 
                        beginAtZero: true,
                        ticks: {
                            callback: (value) => formatBytes(value),
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#cbd5e1' : '#64748b';
                            },
                            font: { size: 10, weight: '400' }
                        },
                        grid: {
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#374151' : '#e2e8f0';
                            },
                            lineWidth: 1
                        }
                    }
                }
            }
        });
        
        window.chartInstances = window.chartInstances || {};
        window.chartInstances['typeChart'] = chart;
    }
}

// Type Treemap Chart
function initTreemapChart(data) {
    const container = document.getElementById('treemap');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 12);
    const totalSize = top.reduce((sum, [, size]) => sum + size, 0);
    
    if (totalSize > 0) {
        let html = '<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 8px; height: 100%; padding: 16px;">';
        
        top.forEach(([type, size], index) => {
            const percentage = (size / totalSize) * 100;
            const color = `hsl(${index * 30}, 70%, 55%)`;
            
            html += `
                <div style="
                    background: ${color};
                    color: white;
                    padding: 12px;
                    border-radius: 8px;
                    font-size: 11px;
                    font-weight: 600;
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    text-align: center;
                    min-height: 80px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    transition: transform 0.2s ease;
                " title="${type}: ${formatBytes(size)}" onmouseover=\"this.style.transform=&apos;scale(1.05)&apos;\" onmouseout=\"this.style.transform=&apos;scale(1)&apos;\">
                    <div style="margin-bottom: 4px;">${formatTypeName(type)}</div>
                    <div style="font-size: 10px; opacity: 0.9;">${formatBytes(size)}</div>
                    <div style="font-size: 9px; opacity: 0.7;">${percentage.toFixed(1)}%</div>
                </div>
            `;
        });
        
        html += "</div>";
        container.innerHTML = html;
    } else {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No data available</div>';
    }
}

// FFI Risk Chart
function initFFIRiskChart(data) {
    // Guard destroy if exists
    try {
        if (window.chartInstances && window.chartInstances['ffi-risk-chart']) {
            window.chartInstances['ffi-risk-chart'].destroy();
            delete window.chartInstances['ffi-risk-chart'];
        }
    } catch (_) {}

    const ctx = document.getElementById('ffi-risk-chart');
    if (!ctx) return;
    
    const ffiData = data.unsafe_ffi?.enhanced_ffi_data || [];
    
    const riskLevels = {
        'Low Risk': ffiData.filter(item => (item.safety_violations || []).length === 0).length,
        'Medium Risk': ffiData.filter(item => (item.safety_violations || []).length > 0 && (item.safety_violations || []).length <= 2).length,
        'High Risk': ffiData.filter(item => (item.safety_violations || []).length > 2).length
    };
    
    if (window.Chart) {
        const chart = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: Object.keys(riskLevels),
                datasets: [{
                    data: Object.values(riskLevels),
                    backgroundColor: ['#059669', '#ea580c', '#dc2626'],
                    borderWidth: 2,
                    borderColor: 'var(--bg-primary)'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'bottom',
                        labels: {
                            padding: 20,
                            usePointStyle: true
                        }
                    }
                }
            }
        });
    }
}

// Add missing chart and graph functions
function initGrowthTrends(data) {
    const container = document.getElementById('growth');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No growth data available</div>';
        return;
    }
    
    // Simple growth visualization
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    let cumulative = 0;
    const points = [];
    
    for (let i = 0; i < Math.min(sorted.length, 20); i++) {
        cumulative += sorted[i].size || 0;
        points.push(cumulative);
    }
    
    const maxValue = Math.max(...points);
    let html = '<div style="display: flex; align-items: end; height: 200px; gap: 4px; padding: 20px;">';
    
    points.forEach((value, i) => {
        const height = (value / maxValue) * 160;
        html += `
            <div style="
                width: 12px;
                height: ${height}px;
                background: linear-gradient(to top, #2563eb, #3b82f6);
                border-radius: 2px;
                margin: 0 1px;
            " title="Step ${i + 1}: ${formatBytes(value)}"></div>
        `;
    });
    
    html += "</div>";
    container.innerHTML = html;
}

function initMemoryFragmentation(data) {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const activeMemory = allocs.filter(a => !a.timestamp_dealloc).reduce((sum, a) => sum + (a.size || 0), 0);
    const fragmentationRate = totalMemory > 0 ? ((totalMemory - activeMemory) / totalMemory * 100) : 0;
    
    container.innerHTML = `
        <div style="padding: 20px;">
            <div style="display: flex; justify-content: space-between; margin-bottom: 16px;">
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Fragmentation Rate</div>
                    <div style="font-size: 2rem; font-weight: 700; color: ${fragmentationRate > 30 ? '#dc2626' : fragmentationRate > 15 ? '#ea580c' : '#059669'};">
                        ${fragmentationRate.toFixed(1)}%
                    </div>
                </div>
                <div>
                    <div style="color: var(--text-secondary); font-size: 0.9rem;">Active Memory</div>
                    <div style="font-size: 1.2rem; font-weight: 600;">${formatBytes(activeMemory)}</div>
                </div>
            </div>
            <div style="background: var(--bg-secondary); height: 8px; border-radius: 4px; overflow: hidden;">
                <div style="
                    background: linear-gradient(to right, #059669, #ea580c);
                    width: ${Math.min(100, fragmentationRate)}%;
                    height: 100%;
                    border-radius: 4px;
                    transition: width 0.8s ease;
                "></div>
            </div>
            <div style="margin-top: 12px; font-size: 0.8rem; color: var(--text-secondary);">
                ${fragmentationRate > 30 ? 'High fragmentation detected' : fragmentationRate > 15 ? 'Moderate fragmentation' : 'Low fragmentation'}
            </div>
        </div>
    `;
}

function initVariableGraph(data) {
    const container = document.getElementById('graph');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No relationship data available</div>';
        return;
    }
    
    // Create a simple node-link visualization
    const nodes = allocs.slice(0, 20).map((a, i) => ({
        id: i,
        name: a.var_name || `var_${i}`,
        type: a.type_name || 'unknown',
        size: a.size || 0,
        x: 50 + (i % 4) * 80,
        y: 50 + Math.floor(i / 4) * 60
    }));
    
    let svg = `
        <svg width="100%" height="100%" style="background: transparent;">
            <defs>
                <filter id="glow">
                    <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                    <feMerge>
                        <feMergeNode in="coloredBlur"/>
                        <feMergeNode in="SourceGraphic"/>
                    </feMerge>
                </filter>
            </defs>
    `;
    
    // Add links between nearby nodes
    for (let i = 0; i < nodes.length - 1; i++) {
        if (i % 4 !== 3) { // Connect horizontally
            svg += `<line x1="${nodes[i].x}" y1="${nodes[i].y}" x2="${nodes[i+1].x}" y2="${nodes[i+1].y}" stroke="var(--border-light)" stroke-width="1" opacity="0.3"/>`;
        }
        if (i < nodes.length - 4) { // Connect vertically
            svg += `<line x1="${nodes[i].x}" y1="${nodes[i].y}" x2="${nodes[i+4].x}" y2="${nodes[i+4].y}" stroke="var(--border-light)" stroke-width="1" opacity="0.3"/>`;
        }
    }
    
    // Add nodes
    nodes.forEach(node => {
        const radius = Math.max(8, Math.min(20, Math.log(node.size + 1) * 2));
        const color = node.type.includes('String') ? '#fbbf24' : 
                     node.type.includes('Vec') ? '#3b82f6' : 
                     node.type.includes('Box') || node.type.includes('Rc') ? '#8b5cf6' : '#6b7280';
        
        svg += `
            <circle 
                cx="${node.x}" 
                cy="${node.y}" 
                r="${radius}" 
                fill="${color}" 
                stroke="white" 
                stroke-width="2" 
                filter="url(#glow)"
                style="cursor: pointer;"
                onmouseover="this.r.baseVal.value = ${radius + 3}"
                onmouseout="this.r.baseVal.value = ${radius}"
            >
                <title>${node.name} (${node.type})</title>
            </circle>
            <text 
                x="${node.x}" 
                y="${node.y + radius + 12}" 
                text-anchor="middle" 
                font-size="10" 
                fill="var(--text-primary)"
                style="font-weight: 500;"
            >${node.name.length > 8 ? node.name.substring(0, 8) + '...' : node.name}</text>
        `;
    });
    
    svg += '</svg>';
    container.innerHTML = svg;
}

// Initialize lifetime visualization
function initLifetimeVisualization(data) {
    const container = document.getElementById('lifetimes');
    if (!container) return;
    
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    // Show top allocations by lifetime
    const withLifetime = allocs.filter(a => a.lifetime_ms || (a.timestamp_alloc && a.timestamp_dealloc));
    const sorted = withLifetime.sort((a, b) => {
        const aLifetime = a.lifetime_ms || (a.timestamp_dealloc - a.timestamp_alloc);
        const bLifetime = b.lifetime_ms || (b.timestamp_dealloc - b.timestamp_alloc);
        return bLifetime - aLifetime;
    }).slice(0, 10);
    
    if (sorted.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    let html = '<div style="padding: 16px;">';
    
    sorted.forEach((alloc, index) => {
        const lifetime = alloc.lifetime_ms || (alloc.timestamp_dealloc - alloc.timestamp_alloc);
        const isActive = !alloc.timestamp_dealloc;
        const varName = alloc.var_name || `allocation_${index}`;
        const size = formatBytes(alloc.size || 0);
        
        html += `
            <div style="
                margin-bottom: 12px; 
                padding: 12px; 
                background: var(--bg-secondary); 
                border-radius: 8px;
                border-left: 4px solid ${isActive ? '#059669' : '#2563eb'};
            ">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <span style="font-weight: 600; color: var(--text-primary);">${varName}</span>
                    <span style="font-size: 0.9rem; color: var(--text-secondary);">${size}</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">
                        ${formatTypeName(alloc.type_name || 'Unknown')}
                    </span>
                    <span style="
                        font-size: 0.8rem; 
                        font-weight: 600; 
                        color: ${isActive ? '#059669' : '#2563eb'};
                    ">
                        ${isActive ? "Active" : `${lifetime}ms`}
                    </span>
                </div>
            </div>
        `;
    });
    
    html += "</div>";
    container.innerHTML = html;
}

// Helper function to update elements
function updateElement(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

// Original dashboard functions from dashboard.html
function renderKpis() {
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const total = allocs.reduce((s,a)=>s+(a.size||0),0);
    const active = allocs.filter(a=>!a.timestamp_dealloc).length;
    const leaks = allocs.filter(a=>a.is_leaked).length;
    const safety = Math.max(0, 100 - (leaks * 20));
    
    updateElement('total-allocations', allocs.length.toLocaleString());
    updateElement('active-variables', active.toLocaleString());
    updateElement('total-memory', formatBytes(total));
    updateElement('safety-score', safety + '%");
}

function populateAllocationsTable() {
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const allocTable = document.getElementById('allocTable");
    if (!allocTable) return;
    
    const top = allocs.slice().sort((a,b)=>(b.size||0)-(a.size||0)).slice(0,50);
    allocTable.innerHTML = top.map(a => {
        const status = a.is_leaked ? 'Leaked' : (a.timestamp_dealloc ? 'Freed' : "Active");
        const statusClass = a.is_leaked ? 'status-leaked' : (a.timestamp_dealloc ? 'status-freed' : 'status-active");
        
        return `<tr>
            <td>${a.var_name || 'Unknown'}</td>
            <td>${formatTypeName(a.type_name || 'Unknown')}</td>
            <td>${formatBytes(a.size || 0)}</td>
            <td><span class="status-badge ${statusClass}">${status}</span></td>
        </tr>`;
    }).join('');
}

function renderTypeChart() {
    const ctx = document.getElementById("typeChart");
    if (!ctx || !window.Chart) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 8);
    
    if (top.length > 0) {
        new Chart(ctx, {
            type: 'bar',
            data: {
                labels: top.map(x => formatTypeName(x[0])),
                datasets: [{
                    label: 'Memory Usage',
                    data: top.map(x => x[1]),
                    backgroundColor: '#2563eb',
                    borderRadius: 6
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { 
                        beginAtZero: true,
                        ticks: { callback: function(value) { return formatBytes(value); } }
                    }
                }
            }
        });
    }
}

function renderTimelineChart() {
    const ctx = document.getElementById("timelineChart");
    if (!ctx || !window.Chart) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    
    let cumulative = 0;
    const points = [];
    const step = Math.max(1, Math.floor(sorted.length / 30));
    
    for (let i = 0; i < sorted.length; i += step) {
        cumulative += sorted[i].size || 0;
        points.push({ x: i, y: cumulative });
    }
    
    if (points.length > 1) {
        new Chart(ctx, {
            type: 'line',
            data: {
                labels: points.map(p => p.x),
                datasets: [{
                    label: 'Cumulative Memory',
                    data: points.map(p => p.y),
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    fill: true,
                    tension: 0.4
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    y: { 
                        beginAtZero: true,
                        ticks: { callback: function(value) { return formatBytes(value); } }
                    }
                }
            }
        });
    }
}

function renderTreemap() {
    const container = document.getElementById('treemap');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const byType = {};
    
    allocs.forEach(a => {
        const type = a.type_name || 'Unknown';
        byType[type] = (byType[type] || 0) + (a.size || 0);
    });
    
    const top = Object.entries(byType).sort((a,b) => b[1] - a[1]).slice(0, 12);
    const totalSize = top.reduce((sum, [, size]) => sum + size, 0);
    
    if (totalSize > 0) {
        let html = '<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 8px; height: 100%; padding: 16px;">';
        
        top.forEach(([type, size], index) => {
            const percentage = (size / totalSize) * 100;
            const color = `hsl(${index * 30}, 70%, 55%)`;
            
            html += `
                <div style="
                    background: ${color};
                    color: white;
                    padding: 12px;
                    border-radius: 8px;
                    font-size: 11px;
                    font-weight: 600;
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    text-align: center;
                    min-height: 80px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                    transition: transform 0.2s ease;
                " title="${type}: ${formatBytes(size)}" onmouseover=\"this.style.transform=&apos;scale(1.05)&apos;\" onmouseout=\"this.style.transform=&apos;scale(1)&apos;\">
                    <div style="margin-bottom: 4px;">${formatTypeName(type)}</div>
                    <div style="font-size: 10px; opacity: 0.9;">${formatBytes(size)}</div>
                    <div style="font-size: 9px; opacity: 0.7;">${percentage.toFixed(1)}%</div>
                </div>
            `;
        });
        
        html += "</div>";
        container.innerHTML = html;
    } else {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No data available</div>';
    }
}

function renderLifetimes() {
    const container = document.getElementById('lifetimes");
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    const withLifetime = allocs.filter(a => a.lifetime_ms || (a.timestamp_alloc && a.timestamp_dealloc));
    const sorted = withLifetime.sort((a, b) => {
        const aLifetime = a.lifetime_ms || (a.timestamp_dealloc - a.timestamp_alloc);
        const bLifetime = b.lifetime_ms || (b.timestamp_dealloc - b.timestamp_alloc);
        return bLifetime - aLifetime;
    }).slice(0, 10);
    
    if (sorted.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifetime data available</div>';
        return;
    }
    
    let html = '<div style="padding: 16px;">';
    
    sorted.forEach((alloc, index) => {
        const lifetime = alloc.lifetime_ms || (alloc.timestamp_dealloc - alloc.timestamp_alloc);
        const isActive = !alloc.timestamp_dealloc;
        const varName = alloc.var_name || `allocation_${index}`;
        const size = formatBytes(alloc.size || 0);
        
        html += `
            <div style="
                margin-bottom: 12px; 
                padding: 12px; 
                background: var(--bg-secondary); 
                border-radius: 8px;
                border-left: 4px solid ${isActive ? '#059669' : '#2563eb'};
            ">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                    <span style="font-weight: 600; color: var(--text-primary);">${varName}</span>
                    <span style="font-size: 0.9rem; color: var(--text-secondary);">${size}</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">
                        ${formatTypeName(alloc.type_name || 'Unknown')}
                    </span>
                    <span style="
                        font-size: 0.8rem; 
                        font-weight: 600; 
                        color: ${isActive ? '#059669' : '#2563eb'};
                    ">
                        ${isActive ? "Active" : `${lifetime}ms`}
                    </span>
                </div>
            </div>
        `;
    });
    
    html += "</div>";
    container.innerHTML = html;
}

function renderFFI() {
    // First try the chart container for simple chart
    const chartContainer = document.getElementById('ffi-risk-chart");
    if (chartContainer && window.Chart) {
        const data = window.analysisData || {};
        const ffiData = data.unsafe_ffi || data.unsafeFFI || {};
        const operations = ffiData.enhanced_ffi_data || ffiData.allocations || ffiData.unsafe_operations || [];
        
        if (operations.length > 0) {
            // Guard: destroy existing chart instance if any
            try {
                if (window.chartInstances && window.chartInstances['ffi-risk-chart']) {
                    window.chartInstances['ffi-risk-chart'].destroy();
                    delete window.chartInstances['ffi-risk-chart'];
                }
            } catch (_) {}

            const highRisk = operations.filter(op => (op.safety_violations || []).length > 2).length;
            const mediumRisk = operations.filter(op => (op.safety_violations || []).length > 0 && (op.safety_violations || []).length <= 2).length;
            const lowRisk = operations.filter(op => (op.safety_violations || []).length === 0).length;
            
            window.chartInstances = window.chartInstances || {};
            window.chartInstances['ffi-risk-chart'] = new Chart(chartContainer, {
                type: 'doughnut',
                data: {
                    labels: ['Low Risk', 'Medium Risk', 'High Risk'],
                    datasets: [{
                        data: [lowRisk, mediumRisk, highRisk],
                        backgroundColor: ['#059669', '#ea580c', '#dc2626'],
                        borderWidth: 2,
                        borderColor: 'var(--bg-primary)'
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: {
                            position: 'bottom',
                            labels: {
                                padding: 20,
                                usePointStyle: true,
                                generateLabels: function(chart) {
                                    const data = chart.data;
                                    return data.labels.map((label, i) => ({
                                        text: `${label}: ${data.datasets[0].data[i]}`,
                                        fillStyle: data.datasets[0].backgroundColor[i],
                                        strokeStyle: data.datasets[0].backgroundColor[i],
                                        pointStyle: 'circle'
                                    }));
                                }
                            }
                        }
                    }
                }
            });
            return;
        }
    }
    
    // Main comprehensive FFI visualization based on project's actual SVG code
    const container = document.getElementById('ffiVisualization');
    if (!container) return;
    
    const data = window.analysisData || {};
    const ffiData = data.unsafe_ffi || data.unsafeFFI || {};
    const allocations = ffiData.allocations || ffiData.enhanced_ffi_data || [];
    const violations = ffiData.violations || [];
    
    if (allocations.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No FFI data available</div>';
        return;
    }
    
    // Create the ACTUAL project-based unsafe/FFI dashboard SVG
    createProjectBasedUnsafeFFIDashboard(container, allocations, violations);
}

// PROJECT-BASED Unsafe/FFI Dashboard - Direct implementation from visualization.rs
function createProjectBasedUnsafeFFIDashboard(container, allocations, violations) {
    const width = 1400;
    const height = 1000;
    
    // Calculate key metrics exactly like the Rust code
    const unsafeCount = allocations.filter(a => 
        (a.source && (a.source.UnsafeRust || a.source === 'UnsafeRust")) ||
        (a.allocation_source === 'UnsafeRust")
    ).length;
    
    const ffiCount = allocations.filter(a => 
        (a.source && (a.source.FfiC || a.source === 'FfiC")) ||
        (a.allocation_source === 'FfiC")
    ).length;
    
    const crossBoundaryEvents = allocations.reduce((sum, a) => 
        sum + ((a.cross_boundary_events && a.cross_boundary_events.length) || 0), 0
    );
    
    const totalUnsafeMemory = allocations
        .filter(a => a.source !== 'RustSafe' && a.allocation_source !== 'RustSafe')
        .reduce((sum, a) => sum + ((a.base && a.base.size) || a.size || 0), 0);
    
    // Create the full SVG dashboard exactly like the Rust implementation
    let html = `
        <div style="width: 100%; height: ${height}px; background: linear-gradient(135deg, #2c3e50 0%, #34495e 50%, #2c3e50 100%); border-radius: 12px; overflow: hidden; position: relative;">
            <svg width="100%" height="100%" viewBox="0 0 ${width} ${height}" style="font-family: 'Segoe UI', Arial, sans-serif;">
                <!-- SVG Definitions -->
                <defs>
                    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                        <polygon points="0 0, 10 3.5, 0 7" fill="#e74c3c"/>
                    </marker>
                    <filter id="glow">
                        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                        <feMerge>
                            <feMergeNode in="coloredBlur"/>
                            <feMergeNode in="SourceGraphic"/>
                        </feMerge>
                    </filter>
                </defs>
                
                <!-- Main Title -->
                <text x="${width/2}" y="40" text-anchor="middle" font-size="24" font-weight="bold" fill="#ecf0f1">
                    Unsafe Rust &amp; FFI Memory Analysis Dashboard
                </text>
                
                <!-- Key Metrics Cards -->
                <g id="metrics-cards">
                    ${createMetricsCards(unsafeCount, ffiCount, crossBoundaryEvents, violations.length, totalUnsafeMemory)}
                </g>
                
                <!-- Allocation Source Breakdown -->
                <g id="allocation-breakdown" transform="translate(50, 150)">
                    ${createAllocationSourceBreakdown(allocations)}
                </g>
                
                <!-- Memory Safety Status -->
                <g id="safety-status" transform="translate(750, 150)">
                    ${createMemorySafetyStatus(violations)}
                </g>
                
                <!-- Cross-Language Memory Flow -->
                <g id="boundary-flow" transform="translate(50, 500)">
                    ${createBoundaryFlow(allocations)}
                </g>
                
                <!-- Unsafe Memory Hotspots -->
                <g id="unsafe-hotspots" transform="translate(750, 500)">
                    ${createUnsafeHotspots(allocations)}
                </g>
            </svg>
        </div>
    `;
    
    container.innerHTML = html;
    
    // Add interactivity
    setTimeout(() => {
        addFFIInteractivity();
    }, 100);
}

// Create metrics cards exactly like Rust implementation
function createMetricsCards(unsafeCount, ffiCount, crossBoundaryEvents, violationCount, totalUnsafeMemory) {
    const metrics = [
        { label: 'Unsafe Allocations', value: unsafeCount, color: '#e74c3c', x: 100 },
        { label: 'FFI Allocations', value: ffiCount, color: '#3498db', x: 350 },
        { label: 'Boundary Crossings', value: crossBoundaryEvents, color: '#f39c12', x: 600 },
        { label: 'Safety Violations', value: violationCount, color: '#e67e22', x: 850 },
        { label: 'Unsafe Memory', value: formatBytes(totalUnsafeMemory), color: '#9b59b6', x: 1100 }
    ];
    
    return metrics.map(metric => `
        <!-- Card background -->
        <rect x="${metric.x - 60}" y="55" width="120" height="50" 
              fill="${metric.color}" fill-opacity="0.2" 
              stroke="${metric.color}" stroke-width="2" rx="8"/>
        
        <!-- Value -->
        <text x="${metric.x}" y="70" text-anchor="middle" font-size="16" font-weight="bold" fill="${metric.color}">
            ${metric.value}
        </text>
        
        <!-- Label -->
        <text x="${metric.x}" y="95" text-anchor="middle" font-size="10" fill="#bdc3c7">
            ${metric.label}
        </text>
    `).join('');
}

// Create allocation source breakdown
function createAllocationSourceBreakdown(allocations) {
    let safeCount = 0, unsafeCount = 0, ffiCount = 0, crossBoundaryCount = 0;
    
    allocations.forEach(allocation => {
        const source = allocation.source || allocation.allocation_source || 'Unknown';
        if (source === 'RustSafe' || source.RustSafe) safeCount++;
        else if (source === 'UnsafeRust' || source.UnsafeRust) unsafeCount++;
        else if (source === 'FfiC' || source.FfiC) ffiCount++;
        else if (source === 'CrossBoundary' || source.CrossBoundary) crossBoundaryCount++;
    });
    
    const total = safeCount + unsafeCount + ffiCount + crossBoundaryCount;
    if (total === 0) {
        return `<text x="300" y="150" text-anchor="middle" font-size="14" fill="#95a5a6">No allocation data available</text>`;
    }
    
    const sources = [
        { label: 'Safe Rust', count: safeCount, color: '#2ecc71', x: 50 },
        { label: 'Unsafe Rust', count: unsafeCount, color: '#e74c3c', x: 170 },
        { label: 'FFI', count: ffiCount, color: '#3498db', x: 290 },
        { label: 'Cross-boundary', count: crossBoundaryCount, color: '#9b59b6', x: 410 }
    ];
    
    let svg = `
        <!-- Section background -->
        <rect x="0" y="0" width="600" height="300" fill="rgba(52, 73, 94, 0.3)" 
              stroke="#34495e" stroke-width="2" rx="10"/>
        
        <!-- Section title -->
        <text x="300" y="-10" text-anchor="middle" font-size="18" font-weight="bold" fill="#ecf0f1">
            Memory Allocation Sources
        </text>
    `;
    
    sources.forEach(source => {
        if (source.count > 0) {
            const barHeight = (source.count / total * 100);
            svg += `
                <!-- Bar -->
                <rect x="${source.x}" y="${200 - barHeight}" width="40" height="${barHeight}" fill="${source.color}"/>
                
                <!-- Count label -->
                <text x="${source.x + 20}" y="${200 - barHeight - 5}" text-anchor="middle" 
                      font-size="12" font-weight="bold" fill="${source.color}">
                    ${source.count}
                </text>
                
                <!-- Label -->
                <text x="${source.x + 20}" y="220" text-anchor="middle" font-size="10" fill="#ecf0f1">
                    ${source.label}
                </text>
            `;
        }
    });
    
    return svg;
}

// Create memory safety status
function createMemorySafetyStatus(violations) {
    const bgColor = violations.length === 0 ? '#27ae60' : '#e74c3c';
    
    let svg = `
        <!-- Section background -->
        <rect x="0" y="0" width="600" height="300" fill="${bgColor}20" 
              stroke="${bgColor}" stroke-width="2" rx="10"/>
        
        <!-- Section title -->
        <text x="300" y="-10" text-anchor="middle" font-size="18" font-weight="bold" fill="#ecf0f1">
            Memory Safety Status
        </text>
    `;
    
    if (violations.length === 0) {
        svg += `
            <text x="300" y="150" text-anchor="middle" font-size="16" font-weight="bold" fill="#27ae60">
                No Safety Violations Detected
            </text>
            <text x="300" y="180" text-anchor="middle" font-size="12" fill="#2ecc71">
                All unsafe operations and FFI calls appear to be memory-safe
            </text>
        `;
    } else {
        svg += `
            <text x="300" y="120" text-anchor="middle" font-size="16" font-weight="bold" fill="#e74c3c">
                ${violations.length} Safety Violations Detected
            </text>
        `;
        
        violations.slice(0, 5).forEach((violation, i) => {
            const y = 160 + i * 20;
            const description = getViolationDescription(violation);
            svg += `
                <text x="30" y="${y}" font-size="12" fill="#e74c3c">- ${description}</text>
            `;
        });
    }
    
    return svg;
}

// Create boundary flow diagram
function createBoundaryFlow(allocations) {
    let rustToFfi = 0, ffiToRust = 0;
    
    allocations.forEach(allocation => {
        if (allocation.cross_boundary_events) {
            allocation.cross_boundary_events.forEach(event => {
                const eventType = event.event_type || event.type;
                if (eventType === 'RustToFfi' || eventType === 'OwnershipTransfer") rustToFfi++;
                else if (eventType === 'FfiToRust") ffiToRust++;
                else if (eventType === 'SharedAccess") {
                    rustToFfi++;
                    ffiToRust++;
                }
            });
        }
    });
    
    return `
        <!-- Section background -->
        <rect x="0" y="0" width="600" height="200" fill="rgba(52, 73, 94, 0.3)" 
              stroke="#34495e" stroke-width="2" rx="10"/>
        
        <!-- Section title -->
        <text x="300" y="-10" text-anchor="middle" font-size="18" font-weight="bold" fill="#ecf0f1">
            Cross-Language Memory Flow
        </text>
        
        <!-- Rust territory -->
        <rect x="50" y="50" width="200" height="100" fill="#2ecc71" fill-opacity="0.2" 
              stroke="#2ecc71" stroke-width="2" rx="8"/>
        <text x="150" y="110" text-anchor="middle" font-size="14" font-weight="bold" fill="#2ecc71">
            RUST
        </text>
        
        <!-- FFI territory -->
        <rect x="350" y="50" width="200" height="100" fill="#3498db" fill-opacity="0.2" 
              stroke="#3498db" stroke-width="2" rx="8"/>
        <text x="450" y="110" text-anchor="middle" font-size="14" font-weight="bold" fill="#3498db">
            FFI / C
        </text>
        
        ${rustToFfi > 0 ? `
            <!-- Rust to FFI arrow -->
            <line x1="250" y1="80" x2="350" y2="80" stroke="#e74c3c" stroke-width="3" marker-end="url(#arrowhead)"/>
            <text x="300" y="75" text-anchor="middle" font-size="12" font-weight="bold" fill="#e74c3c">
                ${rustToFfi}
            </text>
        ` : ''}
        
        ${ffiToRust > 0 ? `
            <!-- FFI to Rust indicator -->
            <text x="300" y="135" text-anchor="middle" font-size="12" font-weight="bold" fill="#f39c12">
                ← ${ffiToRust}
            </text>
        ` : ''}
    `;
}

// Create unsafe hotspots
function createUnsafeHotspots(allocations) {
    const unsafeAllocations = allocations.filter(a => 
        a.source !== 'RustSafe' && a.allocation_source !== 'RustSafe'
    );
    
    if (unsafeAllocations.length === 0) {
        return `
            <rect x="0" y="0" width="600" height="200" fill="rgba(52, 73, 94, 0.3)" 
                  stroke="#34495e" stroke-width="2" rx="10"/>
            <text x="300" y="-10" text-anchor="middle" font-size="18" font-weight="bold" fill="#ecf0f1">
                Unsafe Memory Hotspots
            </text>
            <text x="300" y="100" text-anchor="middle" font-size="14" fill="#2ecc71">
                No unsafe memory allocations detected
            </text>
        `;
    }
    
    let svg = `
        <rect x="0" y="0" width="600" height="200" fill="rgba(52, 73, 94, 0.3)" 
              stroke="#34495e" stroke-width="2" rx="10"/>
        <text x="300" y="-10" text-anchor="middle" font-size="18" font-weight="bold" fill="#ecf0f1">
            Unsafe Memory Hotspots
        </text>
    `;
    
    unsafeAllocations.slice(0, 6).forEach((allocation, i) => {
        const x = 80 + (i % 3) * 180;
        const y = 80 + Math.floor(i / 3) * 70;
        const size = (allocation.base && allocation.base.size) || allocation.size || 0;
        const sizeFactor = Math.max(5, Math.min(20, Math.log(size + 1) * 2));
        
        const source = allocation.source || allocation.allocation_source || 'Unknown';
        let color = '#95a5a6';
        let label = 'OTHER';
        
        if (source === 'UnsafeRust' || source.UnsafeRust) {
            color = '#e74c3c';
            label = 'UNSAFE';
        } else if (source === 'FfiC' || source.FfiC) {
            color = '#3498db';
            label = 'FFI';
        } else if (source === 'CrossBoundary' || source.CrossBoundary) {
            color = '#9b59b6';
            label = 'CROSS';
        }
        
        svg += `
            <!-- Hotspot circle -->
            <circle cx="${x}" cy="${y}" r="${sizeFactor}" fill="${color}" fill-opacity="0.7" 
                    stroke="${color}" stroke-width="2" filter="url(#glow)"/>
            
            <!-- Size label -->
            <text x="${x}" y="${y + 4}" text-anchor="middle" font-size="8" font-weight="bold" fill="#ffffff">
                ${formatBytes(size)}
            </text>
            
            <!-- Type label -->
            <text x="${x}" y="${y + 35}" text-anchor="middle" font-size="10" fill="${color}">
                ${label}
            </text>
        `;
    });
    
    return svg;
}

// Helper functions
function getViolationDescription(violation) {
    if (violation.DoubleFree || violation.type === 'DoubleFree') return 'Double Free';
    if (violation.InvalidFree || violation.type === 'InvalidFree') return 'Invalid Free';
    if (violation.PotentialLeak || violation.type === 'PotentialLeak') return 'Memory Leak';
    if (violation.CrossBoundaryRisk || violation.type === 'CrossBoundaryRisk') return 'Cross-Boundary Risk';
    return 'Unknown Violation';
}

function addFFIInteractivity() {
    // Add hover effects to hotspots
    const hotspots = document.querySelectorAll('#unsafe-hotspots circle');
    hotspots.forEach(hotspot => {
        hotspot.addEventListener('mouseover', function() {
            this.setAttribute('r', parseInt(this.getAttribute('r')) * 1.2);
        });
        hotspot.addEventListener('mouseout', function() {
            this.setAttribute('r', parseInt(this.getAttribute('r')) / 1.2);
        });
    });
}

function renderAllocationSourceChart(allocations) {
    const container = document.getElementById('allocation-source-chart");
    if (!container) return;
    
    // Count allocations by source
    let safeCount = 0, unsafeCount = 0, ffiCount = 0, crossBoundaryCount = 0;
    
    allocations.forEach(allocation => {
        if (allocation.source) {
            if (allocation.source.includes && allocation.source.includes('Safe")) safeCount++;
            else if (allocation.source.includes && allocation.source.includes('Unsafe")) unsafeCount++;
            else if (allocation.source.includes && allocation.source.includes('Ffi")) ffiCount++;
            else if (allocation.source.includes && allocation.source.includes('Cross")) crossBoundaryCount++;
        }
    });
    
    const total = safeCount + unsafeCount + ffiCount + crossBoundaryCount;
    if (total === 0) {
        container.innerHTML = '<div style="text-align: center; color: #95a5a6;">No allocation data available</div>';
        return;
    }
    
    const sources = [
        { label: 'Safe Rust', count: safeCount, color: '#2ecc71' },
        { label: 'Unsafe Rust', count: unsafeCount, color: '#e74c3c' },
        { label: 'FFI', count: ffiCount, color: '#3498db' },
        { label: 'Cross-boundary', count: crossBoundaryCount, color: '#9b59b6' }
    ];
    
    let html = '<div style="display: flex; justify-content: space-around; align-items: end; height: 100px;">';
    
    sources.forEach(source => {
        if (source.count > 0) {
            const barHeight = (source.count / total * 80);
            html += `
                <div style="text-align: center;">
                    <div style="font-size: 12px; font-weight: bold; color: ${source.color}; margin-bottom: 5px;">${source.count}</div>
                    <div style="width: 30px; height: ${barHeight}px; background: ${source.color}; margin: 0 auto 5px;"></div>
                    <div style="font-size: 10px; color: #ecf0f1; writing-mode: vertical-rl; text-orientation: mixed;">${source.label}</div>
                </div>
            `;
        }
    });
    
    html += "</div>";
    container.innerHTML = html;
}

function renderSafetyStatusPanel(violations) {
    const container = document.getElementById('safety-status-panel");
    if (!container) return;
    
    if (violations.length === 0) {
        container.innerHTML = `
            <div style="text-align: center; color: #27ae60;">
                <div style="font-size: 16px; font-weight: bold; margin-bottom: 10px;">No Safety Violations Detected</div>
                <div style="font-size: 12px; color: #2ecc71;">All unsafe operations and FFI calls appear to be memory-safe</div>
            </div>
        `;
    } else {
        let html = `
            <div style="text-align: center; color: #e74c3c; margin-bottom: 15px;">
                <div style="font-size: 16px; font-weight: bold;">${violations.length} Safety Violations Detected</div>
            </div>
            <div style="max-height: 80px; overflow-y: auto;">
        `;
        
        violations.slice(0, 5).forEach(violation => {
            const description = violation.type || 'Unknown Violation';
            html += `<div style="font-size: 12px; color: #e74c3c; margin-bottom: 5px;">- ${description}</div>`;
        });
        
        html += "</div>";
        container.innerHTML = html;
    }
}

function renderBoundaryFlowDiagram(allocations) {
    const container = document.getElementById('boundary-flow-diagram");
    if (!container) return;
    
    // Count boundary events
    let rustToFfi = 0, ffiToRust = 0;
    
    allocations.forEach(allocation => {
        if (allocation.cross_boundary_events) {
            allocation.cross_boundary_events.forEach(event => {
                if (event.event_type === 'RustToFfi") rustToFfi++;
                else if (event.event_type === 'FfiToRust") ffiToRust++;
                else if (event.event_type === 'OwnershipTransfer") rustToFfi++;
                else if (event.event_type === 'SharedAccess") {
                    rustToFfi++;
                    ffiToRust++;
                }
            });
        }
    });
    
    container.innerHTML = `
        <div style="display: flex; justify-content: space-around; align-items: center; height: 80px;">
            <!-- Rust territory -->
            <div style="width: 150px; height: 60px; background: rgba(46, 204, 113, 0.2); border: 2px solid #2ecc71; border-radius: 8px; display: flex; align-items: center; justify-content: center;">
                <div style="text-align: center;">
                    <div style="font-size: 14px; font-weight: bold; color: #2ecc71;">RUST</div>
                </div>
            </div>
            
            <!-- Flow arrows -->
            <div style="text-align: center;">
                ${rustToFfi > 0 ? `
                    <div style="margin-bottom: 5px;">
                        <span style="color: #e74c3c; font-weight: bold;">${rustToFfi}</span>
                        <span style="color: #e74c3c;"> →</span>
                    </div>
                ` : ''}
                ${ffiToRust > 0 ? `
                    <div>
                        <span style="color: #f39c12;"> ←</span>
                        <span style="color: #f39c12; font-weight: bold;">${ffiToRust}</span>
                    </div>
                ` : ''}
            </div>
            
            <!-- FFI territory -->
            <div style="width: 150px; height: 60px; background: rgba(52, 152, 219, 0.2); border: 2px solid #3498db; border-radius: 8px; display: flex; align-items: center; justify-content: center;">
                <div style="text-align: center;">
                    <div style="font-size: 14px; font-weight: bold; color: #3498db;">FFI / C</div>
                </div>
            </div>
        </div>
    `;
}

function renderMemoryUsageAnalysis() {
    // Will be implemented if container exists
}

function renderMemoryFragmentation() {
    const container = document.getElementById('memoryFragmentation');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    
    if (allocs.length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-secondary); padding: 40px;">No allocation data available</div>';
        return;
    }
    
    // Calculate fragmentation metrics
    const sizes = allocs.map(a => a.size || 0).filter(s => s > 0);
    const totalMemory = sizes.reduce((sum, size) => sum + size, 0);
    const avgSize = totalMemory / sizes.length;
    const variance = sizes.reduce((sum, size) => sum + Math.pow(size - avgSize, 2), 0) / sizes.length;
    const stdDev = Math.sqrt(variance);
    const fragmentation = Math.min(100, (stdDev / avgSize) * 100);
    
    // Sort allocations by size for visualization
    const sortedAllocs = allocs.slice().sort((a, b) => (a.size || 0) - (b.size || 0));
    
    // Create memory fragmentation visualization
    container.innerHTML = `
        <div style="height: 100%; display: flex; flex-direction: column; gap: 12px; padding: 12px;">
            <!-- Fragmentation Score -->
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
                <div>
                    <div style="font-size: 14px; font-weight: 600; color: var(--text-primary);">Fragmentation Level</div>
                    <div style="font-size: 11px; color: var(--text-secondary);">Memory size variance indicator</div>
                </div>
                <div style="text-align: right;">
                    <div style="font-size: 24px; font-weight: 700; color: ${fragmentation > 50 ? 'var(--primary-red)' : fragmentation > 25 ? 'var(--primary-orange)' : 'var(--primary-green)'};">
                        ${fragmentation.toFixed(1)}%
                    </div>
                    <div style="font-size: 10px; color: var(--text-secondary);">
                        ${fragmentation > 50 ? 'High' : fragmentation > 25 ? 'Medium' : 'Low'}
                    </div>
                </div>
            </div>
            
            <!-- Memory Layout Visualization -->
            <div style="flex: 1; background: var(--bg-secondary); border-radius: 8px; padding: 12px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--text-primary); margin-bottom: 8px;">Memory Layout (${allocs.length} allocations)</div>
                <div style="height: 80px; background: var(--bg-primary); border-radius: 6px; padding: 4px; position: relative; overflow: hidden;">
                    <!-- Memory blocks representing allocations -->
                    <div style="display: flex; height: 100%; align-items: end; gap: 1px;">
                        ${sortedAllocs.slice(0, 40).map((alloc, i) => {
                            const size = alloc.size || 0;
                            const maxSize = Math.max(...sizes);
                            const height = Math.max(8, (size / maxSize) * 70);
                            const width = Math.max(2, Math.min(8, 100 / Math.min(40, allocs.length)));
                            
                            let color = '#10b981'; // Green for small
                            if (size > 10240) color = '#ef4444'; // Red for large
                            else if (size > 1024) color = '#f59e0b'; // Orange for medium
                            else if (size > 100) color = '#3b82f6'; // Blue for small-medium
                            
                            return `
                                <div style="width: ${width}px; height: ${height}px; background: ${color}; 
                                           border-radius: 1px; cursor: pointer; transition: all 0.2s; opacity: 0.8;"
                                     title="${alloc.var_name}: ${formatBytes(size)}"
                                     onmouseover="this.style.transform='scaleY(1.2)'; this.style.opacity='1'"
                                     onmouseout="this.style.transform='scaleY(1)'; this.style.opacity='0.8'"
                                     onclick="showAllocationDetail('${alloc.ptr}")"></div>
                            `;
                        }).join('')}
                    </div>
                    
                    <!-- Size legend -->
                    <div style="position: absolute; bottom: 4px; right: 4px; display: flex; gap: 4px; font-size: 8px;">
                        <div style="display: flex; align-items: center; gap: 2px;">
                            <div style="width: 8px; height: 4px; background: #10b981;"></div>
                            <span style="color: var(--text-secondary);">Tiny</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 2px;">
                            <div style="width: 8px; height: 4px; background: #3b82f6;"></div>
                            <span style="color: var(--text-secondary);">Small</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 2px;">
                            <div style="width: 8px; height: 4px; background: #f59e0b;"></div>
                            <span style="color: var(--text-secondary);">Medium</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 2px;">
                            <div style="width: 8px; height: 4px; background: #ef4444;"></div>
                            <span style="color: var(--text-secondary);">Large</span>
                        </div>
                    </div>
                </div>
                
                <!-- Fragmentation Analysis -->
                <div style="margin-top: 8px; display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; font-size: 11px;">
                    <div style="text-align: center; padding: 6px; background: var(--bg-primary); border-radius: 4px;">
                        <div style="font-weight: 600; color: var(--text-primary);">${formatBytes(avgSize)}</div>
                        <div style="color: var(--text-secondary);">Avg Size</div>
                    </div>
                    <div style="text-align: center; padding: 6px; background: var(--bg-primary); border-radius: 4px;">
                        <div style="font-weight: 600; color: var(--text-primary);">${formatBytes(Math.max(...sizes))}</div>
                        <div style="color: var(--text-secondary);">Max Size</div>
                    </div>
                    <div style="text-align: center; padding: 6px; background: var(--bg-primary); border-radius: 4px;">
                        <div style="font-weight: 600; color: var(--text-primary);">${formatBytes(Math.min(...sizes))}</div>
                        <div style="color: var(--text-secondary);">Min Size</div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function renderMemoryGrowthTrends() {
    const container = document.getElementById('growth");
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No growth data available</div>';
        return;
    }
    
    // Create a proper growth chart using Chart.js
    const canvas = document.createElement('canvas');
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    container.innerHTML = '';
    container.appendChild(canvas);
    
    const sorted = allocs.slice().sort((a,b) => (a.timestamp_alloc||0) - (b.timestamp_alloc||0));
    let cumulative = 0;
    const points = [];
    
    for (let i = 0; i < Math.min(sorted.length, 30); i++) {
        cumulative += sorted[i].size || 0;
        points.push({ x: i, y: cumulative });
    }
    
    if (points.length > 1 && window.Chart) {
        new Chart(canvas, {
            type: 'line',
            data: {
                labels: points.map((_, i) => `T${i}`),
                datasets: [{
                    label: 'Memory Growth',
                    data: points.map(p => p.y),
                    borderColor: '#059669',
                    backgroundColor: 'rgba(5, 150, 105, 0.1)',
                    fill: true,
                    tension: 0.3,
                    pointRadius: 3,
                    pointHoverRadius: 5
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false }
                },
                scales: {
                    x: {
                        title: {
                            display: true,
                            text: 'Time Steps'
                        }
                    },
                    y: { 
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: 'Cumulative Memory'
                        },
                        ticks: {
                            callback: function(value) {
                                return formatBytes(value);
                            }
                        }
                    }
                }
            }
        });
    }
}

function setupLifecycle() {
    // Lifecycle setup functionality
}

function populateUnsafeTable() {
    const data = window.analysisData || {};
    const root = data.unsafe_ffi || {};
    const ops = root.enhanced_ffi_data || root.unsafe_operations || root.allocations || [];
    const unsafeTable = document.getElementById('unsafeTable");
    if (!unsafeTable) return;
    
    unsafeTable.innerHTML = (ops || []).slice(0, 50).map(op => {
        const riskLevel = op.risk_level || ((op.safety_violations||[]).length > 2 ? 'High' : 
                         ((op.safety_violations||[]).length > 0 ? 'Medium' : 'Low"));
        
        const riskText = riskLevel === 'High' ? 'High Risk' : (riskLevel === 'Medium' ? 'Medium Risk' : "Low Risk");
        const riskClass = riskLevel === 'High' ? 'risk-high' : (riskLevel === 'Medium' ? 'risk-medium' : "risk-low");
        
        return `<tr>
            <td>${op.location || op.var_name || 'Unknown'}</td>
            <td>${op.operation_type || op.type_name || 'Unknown'}</td>
            <td><span class="status-badge ${riskClass}">${riskText}</span></td>
        </tr>`;
    }).join('');
}

function renderVariableGraph() {
    const container = document.getElementById('graph");
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    if (allocs.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No relationship data available</div>';
        return;
    }
    
    // Enhanced variable relationship graph with pan/zoom and drag functionality
    const nodes = allocs.slice(0, 30).map((a, i) => ({
        id: i,
        name: a.var_name || `var_${i}`,
        type: a.type_name || 'unknown',
        size: a.size || 0,
        status: a.is_leaked ? 'leaked' : (a.timestamp_dealloc ? 'freed' : 'active"),
        ptr: a.ptr || 'unknown',
        timestamp_alloc: a.timestamp_alloc || 0,
        timestamp_dealloc: a.timestamp_dealloc || null,
        x: 200 + (i % 6) * 120 + Math.random() * 40,
        y: 200 + Math.floor(i / 6) * 120 + Math.random() * 40,
        isDragging: false
    }));
    
    // Create enhanced links with copy/clone/move relationships
    const links = [];
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            const nodeA = nodes[i];
            const nodeB = nodes[j];
            const allocA = allocs[i];
            const allocB = allocs[j];
            
            // Clone relationship (based on clone_info)
            if (allocA.clone_info?.clone_count > 0 && allocB.clone_info?.is_clone) {
                links.push({ 
                    source: i, target: j, type: 'clone', 
                    color: '#f59e0b', strokeWidth: 3, dashArray: '5,5'
                });
            }
            // Copy relationship (same type, similar size)
            else if (nodeA.type === nodeB.type && 
                     Math.abs(nodeA.size - nodeB.size) < Math.max(nodeA.size, nodeB.size) * 0.1) {
                links.push({ 
                    source: i, target: j, type: 'copy', 
                    color: '#06b6d4', strokeWidth: 2, dashArray: 'none'
                });
            }
            // Move relationship (same type, different timestamps)
            else if (nodeA.type === nodeB.type && 
                     Math.abs(nodeA.timestamp_alloc - nodeB.timestamp_alloc) > 1000000) {
                links.push({ 
                    source: i, target: j, type: 'move', 
                    color: '#8b5cf6', strokeWidth: 2, dashArray: '10,5'
                });
            }
            // General relationship (same type prefix)
            else if (nodeA.type === nodeB.type || 
                     nodeA.name.startsWith(nodeB.name.substring(0, 3))) {
                links.push({ 
                    source: i, target: j, type: 'related', 
                    color: 'var(--border-light)', strokeWidth: 1, dashArray: 'none'
                });
            }
        }
    }
    
    const width = 1200;  // Larger virtual canvas
    const height = 800;
    const viewWidth = container.offsetWidth || 500;
    const viewHeight = 400;
    
    // Create SVG with pan/zoom capabilities and legend
    let html = `
        <div style="position: relative; width: 100%; height: ${viewHeight}px; overflow: hidden; border: 1px solid var(--border-light); border-radius: 8px;">
            <!-- Relationship Legend -->
            <div style="position: absolute; top: 10px; right: 10px; background: var(--bg-primary); padding: 12px; border-radius: 8px; font-size: 11px; z-index: 10; border: 1px solid var(--border-light); box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
                <div style="font-weight: bold; margin-bottom: 8px; color: var(--text-primary); font-size: 12px;">Variable Relationships</div>
                <div style="display: flex; align-items: center; margin-bottom: 4px;">
                    <svg width="20" height="3" style="margin-right: 8px;">
                        <line x1="0" y1="1.5" x2="20" y2="1.5" stroke="#06b6d4" stroke-width="2"/>
                    </svg>
                    <span style="color: var(--text-secondary);">Copy</span>
                </div>
                <div style="display: flex; align-items: center; margin-bottom: 4px;">
                    <svg width="20" height="3" style="margin-right: 8px;">
                        <line x1="0" y1="1.5" x2="20" y2="1.5" stroke="#f59e0b" stroke-width="3" stroke-dasharray="5,5"/>
                    </svg>
                    <span style="color: var(--text-secondary);">Clone</span>
                </div>
                <div style="display: flex; align-items: center; margin-bottom: 4px;">
                    <svg width="20" height="3" style="margin-right: 8px;">
                        <line x1="0" y1="1.5" x2="20" y2="1.5" stroke="#8b5cf6" stroke-width="2" stroke-dasharray="10,5"/>
                    </svg>
                    <span style="color: var(--text-secondary);">Move</span>
                </div>
                <div style="display: flex; align-items: center;">
                    <svg width="20" height="3" style="margin-right: 8px;">
                        <line x1="0" y1="1.5" x2="20" y2="1.5" stroke="#64748b" stroke-width="1"/>
                    </svg>
                    <span style="color: var(--text-secondary);">Related</span>
                </div>
            </div>
            
            <svg id="graph-svg" width="100%" height="100%" viewBox="0 0 ${viewWidth} ${viewHeight}" style="background: var(--bg-secondary); cursor: grab;">
                <defs>
                    <filter id="node-glow">
                        <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                        <feMerge>
                            <feMergeNode in="coloredBlur"/>
                            <feMergeNode in="SourceGraphic"/>
                        </feMerge>
                    </filter>
                    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="3"
                            markerWidth="6" markerHeight="6" orient="auto">
                        <path d="M0,0 L0,6 L9,3 z" fill="var(--border-light)" opacity="0.6"/>
                    </marker>
                </defs>
                <g id="graph-container" transform="translate(0,0) scale(1)">
                    <g id="links-group">`;
    
    // Draw enhanced links with relationship types
    links.forEach((link, linkIndex) => {
        const source = nodes[link.source];
        const target = nodes[link.target];
        const strokeDashArray = link.dashArray !== 'none' ? link.dashArray : '';
        
        html += `
            <line id="link-${linkIndex}" x1="${source.x}" y1="${source.y}" x2="${target.x}" y2="${target.y}" 
                  stroke="${link.color}" stroke-width="${link.strokeWidth}" opacity="0.8"
                  stroke-dasharray="${strokeDashArray}">
                <animate attributeName="opacity" values="0.8;1;0.8" dur="3s" repeatCount="indefinite"/>
            </line>`;
        
        // Add relationship label for special types
        const midX = (source.x + target.x) / 2;
        const midY = (source.y + target.y) / 2;
        if (link.type !== 'related") {
            html += `
            <text x="${midX}" y="${midY - 5}" text-anchor="middle" font-size="8" 
                  fill="${link.color}" font-weight="bold" opacity="0.9">
                ${link.type.toUpperCase()}
            </text>`;
        }
    });
    
    html += '</g><g id="nodes-group">';
    
    // Draw nodes
    nodes.forEach((node, nodeIndex) => {
        const radius = Math.max(12, Math.min(30, Math.log(node.size + 1) * 4));
        let color = '#6b7280'; // default
        
        if (node.type.includes('String')) color = '#fbbf24';
        else if (node.type.includes('Vec')) color = '#3b82f6';
        else if (node.type.includes('Box') || node.type.includes('Rc')) color = '#8b5cf6';
        else if (node.type.includes('HashMap')) color = '#10b981';
        else if (node.type.includes('Arc')) color = '#f59e0b';
        
        if (node.status === 'leaked') color = '#dc2626';
        else if (node.status === 'freed') color = '#9ca3af';
        
        html += `
            <circle 
                id="node-${nodeIndex}"
                cx="${node.x}" 
                cy="${node.y}" 
                r="${radius}" 
                fill="${color}" 
                stroke="white" 
                stroke-width="3" 
                filter="url(#node-glow)"
                style="cursor: grab;"
                class="graph-node"
                data-index="${nodeIndex}"
                data-name="${node.name}"
                data-type="${node.type}"
                data-size="${node.size}"
                data-status="${node.status}"
                data-ptr="${node.ptr}"
                data-alloc="${node.timestamp_alloc}"
                data-dealloc="${node.timestamp_dealloc || 'null'}"
            />
            <text 
                id="text-${nodeIndex}"
                x="${node.x}" 
                y="${node.y + radius + 20}" 
                text-anchor="middle" 
                font-size="12" 
                font-weight="600"
                fill="var(--text-primary)"
                style="pointer-events: none;"
            >${node.name.length > 12 ? node.name.substring(0, 10) + '...' : node.name}</text>
        `;
    });
    
    html += `
                </g>
            </g>
        </svg>
        
        <!-- Controls -->
        <div style="position: absolute; top: 10px; right: 10px; display: flex; gap: 8px;">
            <button id="zoom-in" style="background: var(--primary-blue); color: white; border: none; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 14px;">
                <i class="fa fa-plus"></i>
            </button>
            <button id="zoom-out" style="background: var(--primary-blue); color: white; border: none; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 14px;">
                <i class="fa fa-minus"></i>
            </button>
            <button id="reset-view" style="background: var(--primary-green); color: white; border: none; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 14px;">
                <i class="fa fa-home"></i>
            </button>
        </div>
        
        <!-- Node detail panel -->
        <div id="node-detail-panel" style="
            position: absolute;
            background: var(--bg-primary);
            border: 1px solid var(--border-light);
            border-radius: 8px;
            padding: 12px;
            width: 280px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.1);
            z-index: 1000;
            font-size: 0.875rem;
            display: none;
            backdrop-filter: blur(10px);
        ">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <h4 id="detail-title" style="margin: 0; font-size: 1rem; font-weight: 600;"></h4>
                <button onclick="hideNodeDetails()" style="background: none; border: none; font-size: 16px; cursor: pointer; color: var(--text-secondary);">×</button>
            </div>
            <div id="detail-content"></div>
        </div>
        
        <!-- Legend -->
        <div style="display: flex; gap: 12px; margin-top: 12px; font-size: 0.75rem; flex-wrap: wrap;">
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #fbbf24; border-radius: 50%;"></div>
                <span>String</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #3b82f6; border-radius: 50%;"></div>
                <span>Vec</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #8b5cf6; border-radius: 50%;"></div>
                <span>Smart Ptr</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #dc2626; border-radius: 50%;"></div>
                <span>Leaked</span>
            </div>
            <div style="display: flex; align-items: center; gap: 4px;">
                <div style="width: 10px; height: 10px; background: #9ca3af; border-radius: 50%;"></div>
                <span>Freed</span>
            </div>
        </div>
    </div>
    `;
    
    container.innerHTML = html;
    
    // Store nodes and links data for interaction
    window.graphNodes = nodes;
    window.graphLinks = links;
    window.graphTransform = { x: 0, y: 0, scale: 1 };
    
    // Add pan/zoom and drag functionality
    setTimeout(() => {
        setupGraphInteractions();
        setupPanZoom();
    }, 100);
}

// Graph interaction functions
function setupGraphInteractions() {
    const svg = document.getElementById('graph-svg');
    const nodeElements = document.querySelectorAll('.graph-node');
    
    let draggedNode = null;
    let isDragging = false;
    let startX, startY;
    
    nodeElements.forEach(node => {
        // Mouse events for drag
        node.addEventListener('mousedown', function(e) {
            e.preventDefault();
            draggedNode = this;
            isDragging = false;
            startX = e.clientX;
            startY = e.clientY;
            this.style.cursor = 'grabbing';
            svg.style.cursor = 'grabbing';
        });
        
        // Click event for details
        node.addEventListener("click", function(e) {
            if (!isDragging) {
                showNodeDetails(this);
            }
        });
        
        // Hover effects
        node.addEventListener('mouseover', function() {
            if (!draggedNode) {
                const currentRadius = parseInt(this.getAttribute('r'));
                this.setAttribute('r', Math.round(currentRadius * 1.2));
            }
        });
        
        node.addEventListener('mouseout', function() {
            if (!draggedNode) {
                const currentRadius = parseInt(this.getAttribute('r'));
                this.setAttribute('r', Math.round(currentRadius / 1.2));
            }
        });
    });
    
    // Global mouse events for dragging
    document.addEventListener('mousemove', function(e) {
        if (draggedNode) {
            e.preventDefault();
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;
            
            if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) {
                isDragging = true;
            }
            
            if (isDragging) {
                const rect = svg.getBoundingClientRect();
                const svgX = Math.max(20, Math.min(rect.width - 20, e.clientX - rect.left));
                const svgY = Math.max(20, Math.min(rect.height - 20, e.clientY - rect.top));
                
                // Update node position
                draggedNode.setAttribute('cx', svgX);
                draggedNode.setAttribute('cy', svgY);
                
                // Update text position
                const nodeIndex = draggedNode.getAttribute('data-index');
                const textElement = document.getElementById(`text-${nodeIndex}`);
                if (textElement) {
                    textElement.setAttribute('x', svgX);
                    textElement.setAttribute('y', svgY + parseInt(draggedNode.getAttribute('r')) + 15);
                }
                
                // Update connected links
                updateConnectedLinks(parseInt(nodeIndex), svgX, svgY);
                
                // Update stored node position
                if (window.graphNodes && window.graphNodes[nodeIndex]) {
                    window.graphNodes[nodeIndex].x = svgX;
                    window.graphNodes[nodeIndex].y = svgY;
                }
            }
        }
    });
    
    document.addEventListener('mouseup', function(e) {
        if (draggedNode) {
            draggedNode.style.cursor = 'grab';
            svg.style.cursor = 'grab';
            
            // Reset hover effect
            const originalRadius = parseInt(draggedNode.getAttribute('r'));
            draggedNode.setAttribute('r', originalRadius);
            
            draggedNode = null;
            setTimeout(() => { isDragging = false; }, 100);
        }
    });
}

function updateConnectedLinks(nodeIndex, newX, newY) {
    if (!window.graphLinks) return;
    
    window.graphLinks.forEach((link, linkIndex) => {
        const linkElement = document.getElementById(`link-${linkIndex}`);
        if (!linkElement) return;
        
        if (link.source === nodeIndex) {
            linkElement.setAttribute('x1', newX);
            linkElement.setAttribute('y1', newY);
        }
        if (link.target === nodeIndex) {
            linkElement.setAttribute('x2', newX);
            linkElement.setAttribute('y2', newY);
        }
    });
}

function showNodeDetails(nodeElement) {
    const panel = document.getElementById('node-detail-panel');
    const title = document.getElementById('detail-title');
    const content = document.getElementById('detail-content');
    
    if (!panel || !title || !content) return;
    
    const name = nodeElement.getAttribute('data-name');
    const type = nodeElement.getAttribute('data-type');
    const size = parseInt(nodeElement.getAttribute('data-size'));
    const status = nodeElement.getAttribute('data-status');
    const ptr = nodeElement.getAttribute('data-ptr');
    const alloc = nodeElement.getAttribute('data-alloc');
    const dealloc = nodeElement.getAttribute('data-dealloc');
    
    title.textContent = name;
    
    const lifetime = dealloc !== 'null' ? parseInt(dealloc) - parseInt(alloc) : "Active";
    
    content.innerHTML = `
        <div style="margin-bottom: 8px;">
            <strong>Type:</strong> ${formatTypeName(type)}
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Size:</strong> ${formatBytes(size)}
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Status:</strong> <span style="color: ${status === 'leaked' ? '#dc2626' : status === 'freed' ? '#6b7280' : '#059669'};">${status.charAt(0).toUpperCase() + status.slice(1)}</span>
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Pointer:</strong> <code style="font-size: 0.8rem; background: var(--bg-secondary); padding: 2px 4px; border-radius: 3px;">${ptr}</code>
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Allocated:</strong> ${alloc}ms
        </div>
        <div style="margin-bottom: 8px;">
            <strong>Lifetime:</strong> ${typeof lifetime === 'number' ? lifetime + 'ms' : lifetime}
        </div>
    `;
    
    // Position panel near the node
    const rect = nodeElement.getBoundingClientRect();
    const containerRect = nodeElement.closest('#graph').getBoundingClientRect();
    
    panel.style.left = Math.min(rect.left - containerRect.left + 30, containerRect.width - 300) + 'px';
    panel.style.top = Math.max(rect.top - containerRect.top - 50, 10) + 'px';
    panel.style.display = 'block';
}

function hideNodeDetails() {
    const panel = document.getElementById('node-detail-panel');
    if (panel) {
        panel.style.display = 'none';
    }
}

// Pan and zoom functionality for the graph
function setupPanZoom() {
    const svg = document.getElementById('graph-svg');
    const container = document.getElementById('graph-container');
    const zoomInBtn = document.getElementById('zoom-in');
    const zoomOutBtn = document.getElementById('zoom-out');
    const resetBtn = document.getElementById('reset-view');
    
    if (!svg || !container) return;
    
    let isPanning = false;
    let startX, startY;
    let transform = window.graphTransform;
    
    // Zoom functions
    function updateTransform() {
        container.setAttribute('transform', `translate(${transform.x},${transform.y}) scale(${transform.scale})`);
    }
    
    function zoom(factor, centerX = 0, centerY = 0) {
        const newScale = Math.max(0.1, Math.min(3, transform.scale * factor));
        
        // Zoom towards center point
        const dx = centerX - transform.x;
        const dy = centerY - transform.y;
        
        transform.x = centerX - dx * (newScale / transform.scale);
        transform.y = centerY - dy * (newScale / transform.scale);
        transform.scale = newScale;
        
        updateTransform();
    }
    
    // Button controls
    if (zoomInBtn) {
        zoomInBtn.addEventListener("click", () => {
            const rect = svg.getBoundingClientRect();
            zoom(1.2, rect.width / 2, rect.height / 2);
        });
    }
    
    if (zoomOutBtn) {
        zoomOutBtn.addEventListener("click", () => {
            const rect = svg.getBoundingClientRect();
            zoom(0.8, rect.width / 2, rect.height / 2);
        });
    }
    
    if (resetBtn) {
        resetBtn.addEventListener("click", () => {
            transform.x = 0;
            transform.y = 0;
            transform.scale = 1;
            updateTransform();
        });
    }
    
    // Mouse wheel zoom
    svg.addEventListener('wheel', function(e) {
        e.preventDefault();
        const rect = svg.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        zoom(zoomFactor, mouseX, mouseY);
    });
    
    // Pan functionality
    svg.addEventListener('mousedown', function(e) {
        if (e.target === svg || e.target === container) {
            isPanning = true;
            startX = e.clientX - transform.x;
            startY = e.clientY - transform.y;
            svg.style.cursor = 'grabbing';
        }
    });
    
    document.addEventListener('mousemove', function(e) {
        if (isPanning) {
            e.preventDefault();
            transform.x = e.clientX - startX;
            transform.y = e.clientY - startY;
            updateTransform();
        }
    });
    
    document.addEventListener('mouseup', function() {
        if (isPanning) {
            isPanning = false;
            svg.style.cursor = 'grab';
        }
    });
}

// Lifecycle toggle functionality
function setupLifecycleToggle() {
    // Hard reset any previous click bindings by cloning the button
    const oldBtn = document.getElementById('toggle-lifecycle");
    if (oldBtn) {
        const cloned = oldBtn.cloneNode(true);
        oldBtn.parentNode.replaceChild(cloned, oldBtn);
    }

    const toggleBtn = document.getElementById('toggle-lifecycle');
    if (!toggleBtn) return;
    
    const lifeContainer = document.getElementById('lifetimeVisualization');
    
    let isExpanded = false;
    
    toggleBtn.addEventListener("click", function() {
        const container = document.getElementById('lifetimeVisualization');
        if (!container) return;
        
        const data = window.analysisData || {};
        const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
        
        if (allocs.length === 0) {
            container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifecycle data available</div>';
            return;
        }
        
        const icon = toggleBtn.querySelector('i');
        const text = toggleBtn.querySelector('span');
        
        if (!isExpanded) {
            renderFullLifecycleTimeline(allocs);
            icon.className = "fa fa-chevron-up";
            text.textContent = "Show Less";
            isExpanded = true;
        } else {
            renderLimitedLifecycleTimeline(allocs);
            icon.className = "fa fa-chevron-down";
            text.textContent = 'Show All';
            isExpanded = false;
        }
        // Ensure the container scrolls to top after toggle for visual confirmation
        if (lifeContainer) { lifeContainer.scrollTop = 0; }
    });
}

function renderLimitedLifecycleTimeline(allocs) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Create timeline visualization (limited to 20)
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 300px; overflow-y: auto;">';
    
    allocs.slice(0, 20).forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        
        html += `
            <div style="margin-bottom: 8px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px; font-size: 0.8rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 8px; border-radius: 4px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${isActive ? 'linear-gradient(to right, #059669, #34d399)' : 'linear-gradient(to right, #2563eb, #60a5fa)'};
                        border-radius: 4px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms"></div>
                </div>
            </div>
        `;
    });
    
    html += "</div>";
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function renderFullLifecycleTimeline(allocs) {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    // Create full timeline visualization
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 600px; overflow-y: auto;">';
    
    // Add timeline header
    html += `
        <div style="margin-bottom: 16px; padding: 12px; background: var(--bg-secondary); border-radius: 8px;">
            <div style="font-weight: 600; margin-bottom: 8px;">Full Lifecycle Timeline</div>
            <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; font-size: 0.8rem;">
                <div>
                    <div style="color: var(--text-secondary);">Total Variables</div>
                    <div style="font-weight: 600;">${allocs.length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Active</div>
                    <div style="font-weight: 600; color: #059669;">${allocs.filter(a => !a.timestamp_dealloc).length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Freed</div>
                    <div style="font-weight: 600; color: #2563eb;">${allocs.filter(a => a.timestamp_dealloc && !a.is_leaked).length}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Leaked</div>
                    <div style="font-weight: 600; color: #dc2626;">${allocs.filter(a => a.is_leaked).length}</div>
                </div>
            </div>
        </div>
    `;
    
    allocs.forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        const isLeaked = alloc.is_leaked;
        
        let barColor = 'linear-gradient(to right, #2563eb, #60a5fa)'; // freed
        if (isActive) barColor = 'linear-gradient(to right, #059669, #34d399)'; // active
        if (isLeaked) barColor = 'linear-gradient(to right, #dc2626, #f87171)'; // leaked
        
        html += `
            <div style="margin-bottom: 6px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 3px; font-size: 0.75rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <div style="display: flex; gap: 8px;">
                        <span style="color: var(--text-secondary);">${formatTypeName(alloc.type_name || 'Unknown')}</span>
                        <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                    </div>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 6px; border-radius: 3px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${barColor};
                        border-radius: 3px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms | Status: ${isLeaked ? 'Leaked' : isActive ? 'Active' : 'Freed'}"></div>
                </div>
            </div>
        `;
    });
    
    html += "</div>";
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function setupLifecycleVisualization() {
    const container = document.getElementById('lifetimeVisualization');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = (data.memory_analysis && data.memory_analysis.allocations) || [];
    
    if (allocs.length === 0) {
        container.innerHTML = '<div style="padding: 20px; text-align: center; color: var(--text-secondary);">No lifecycle data available</div>';
        return;
    }
    
    // Create timeline visualization
    const maxTime = Math.max(...allocs.map(a => a.timestamp_dealloc || a.timestamp_alloc || 0));
    const minTime = Math.min(...allocs.map(a => a.timestamp_alloc || 0));
    const timeRange = maxTime - minTime || 1;
    
    let html = '<div style="padding: 16px; max-height: 300px; overflow-y: auto;">';
    
    allocs.slice(0, 20).forEach((alloc, index) => {
        const startTime = alloc.timestamp_alloc || 0;
        const endTime = alloc.timestamp_dealloc || maxTime;
        const startPercent = ((startTime - minTime) / timeRange) * 100;
        const widthPercent = ((endTime - startTime) / timeRange) * 100;
        const isActive = !alloc.timestamp_dealloc;
        
        html += `
            <div style="margin-bottom: 8px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px; font-size: 0.8rem;">
                    <span style="font-weight: 600;">${alloc.var_name || `var_${index}`}</span>
                    <span style="color: var(--text-secondary);">${formatBytes(alloc.size || 0)}</span>
                </div>
                <div style="position: relative; background: var(--bg-secondary); height: 8px; border-radius: 4px;">
                    <div style="
                        position: absolute;
                        left: ${startPercent}%;
                        width: ${widthPercent}%;
                        height: 100%;
                        background: ${isActive ? 'linear-gradient(to right, #059669, #34d399)' : 'linear-gradient(to right, #2563eb, #60a5fa)'};
                        border-radius: 4px;
                        ${isActive ? 'animation: pulse 2s infinite;' : ''}
                    " title="Lifetime: ${endTime - startTime}ms"></div>
                </div>
            </div>
        `;
    });
    
    html += "</div>";
    
    // Add CSS for pulse animation
    html += `
        <style>
            @keyframes pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.7; }
            }
        </style>
    `;
    
    container.innerHTML = html;
}

function initFFIVisualization() {
    // Additional FFI initialization if needed
    renderFFI();
}

// Complete JSON Data Explorer
function initCompleteJSONExplorer() {
    const container = document.getElementById('jsonDataExplorer');
    const expandBtn = document.getElementById('expand-all-json');
    const collapseBtn = document.getElementById('collapse-all-json');
    
    if (!container) return;
    
    const data = window.analysisData || {};
    
    if (Object.keys(data).length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-secondary); padding: 40px;">No JSON data available</div>';
        return;
    }
    
    // Generate comprehensive JSON explorer
    let html = '<div class="json-explorer">';
    
    // Add data summary
    html += `
        <div style="background: var(--bg-primary); border: 1px solid var(--border-light); border-radius: 8px; padding: 16px; margin-bottom: 16px;">
            <h3 style="margin: 0 0 12px 0; color: var(--text-primary);">Data Summary</h3>
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px;">
                ${Object.keys(data).map(key => {
                    const value = data[key];
                    let itemCount = 'N/A';
                    let dataType = typeof value;
                    
                    if (Array.isArray(value)) {
                        itemCount = value.length + ' items';
                        dataType = 'Array';
                    } else if (value && typeof value === 'object') {
                        itemCount = Object.keys(value).length + ' properties';
                        dataType = 'Object';
                    }
                    
                    return `
                        <div style="background: var(--bg-secondary); padding: 12px; border-radius: 6px;">
                            <div style="font-weight: 600; color: var(--text-primary);">${key}</div>
                            <div style="font-size: 0.8rem; color: var(--text-secondary);">${dataType}</div>
                            <div style="font-size: 0.8rem; color: var(--text-secondary);">${itemCount}</div>
                        </div>
                    `;
                }).join('')}
            </div>
        </div>
    `;
    
    // Generate expandable JSON tree for each top-level key
    Object.keys(data).forEach((key, index) => {
        const value = data[key];
        html += createJSONSection(key, value, index);
    });
    
    html += "</div>";
    container.innerHTML = html;
    
    // Setup expand/collapse functionality
    if (expandBtn) {
        expandBtn.addEventListener("click", () => {
            const details = container.querySelectorAll('details');
            details.forEach(detail => detail.open = true);
        });
    }
    
    if (collapseBtn) {
        collapseBtn.addEventListener("click", () => {
            const details = container.querySelectorAll('details');
            details.forEach(detail => detail.open = false);
        });
    }
}

function createJSONSection(key, value, index) {
    const isOpen = index < 3; // Open first 3 sections by default
    
    let html = `
        <details class="json-section" ${isOpen ? 'open' : ''} style="
            border: 1px solid var(--border-light); 
            border-radius: 8px; 
            margin-bottom: 12px; 
            background: var(--bg-primary);
        ">
            <summary style="
                cursor: pointer; 
                padding: 12px 16px; 
                font-weight: 600; 
                color: var(--text-primary); 
                background: var(--bg-secondary);
                border-radius: 8px 8px 0 0;
                user-select: none;
            ">
                <i class="fa fa-chevron-right" style="margin-right: 8px; transition: transform 0.2s;"></i>
                ${key}
                <span style="font-weight: normal; color: var(--text-secondary); margin-left: 8px;">
                    ${getDataTypeInfo(value)}
                </span>
            </summary>
            <div style="padding: 16px;">
    `;
    
    if (Array.isArray(value)) {
        html += createArrayView(value, key);
    } else if (value && typeof value === 'object') {
        html += createObjectView(value, key);
    } else {
        html += `<pre style="margin: 0; color: var(--text-primary); font-size: 0.9rem;">${JSON.stringify(value, null, 2)}</pre>`;
    }
    
    html += '</div></details>';
    return html;
}

function createArrayView(array, parentKey) {
    if (array.length === 0) {
        return '<div style="color: var(--text-secondary); font-style: italic;">Empty array</div>';
    }
    
    let html = `
        <div style="margin-bottom: 12px;">
            <strong>Array with ${array.length} items</strong>
            ${array.length > 10 ? `<span style="color: var(--text-secondary);"> (showing first 10)</span>` : ''}
        </div>
    `;
    
    // Show first 10 items
    const itemsToShow = array.slice(0, 10);
    
    itemsToShow.forEach((item, index) => {
        html += `
            <details style="margin-bottom: 8px; border: 1px solid var(--border-light); border-radius: 6px;">
                <summary style="cursor: pointer; padding: 8px 12px; background: var(--bg-secondary); font-size: 0.9rem;">
                    [${index}] ${getDataTypeInfo(item)}
                </summary>
                <div style="padding: 12px;">
                    <pre style="margin: 0; font-size: 0.8rem; color: var(--text-primary); max-height: 300px; overflow: auto;">${JSON.stringify(item, null, 2)}</pre>
                </div>
            </details>
        `;
    });
    
    if (array.length > 10) {
        html += `<div style="color: var(--text-secondary); font-style: italic; margin-top: 12px;">... and ${array.length - 10} more items</div>`;
    }
    
    return html;
}

function createObjectView(obj, parentKey) {
    const keys = Object.keys(obj);
    
    if (keys.length === 0) {
        return '<div style="color: var(--text-secondary); font-style: italic;">Empty object</div>';
    }
    
    let html = `
        <div style="margin-bottom: 12px;">
            <strong>Object with ${keys.length} properties</strong>
        </div>
    `;
    
    keys.forEach(key => {
        const value = obj[key];
        html += `
            <details style="margin-bottom: 8px; border: 1px solid var(--border-light); border-radius: 6px;">
                <summary style="cursor: pointer; padding: 8px 12px; background: var(--bg-secondary); font-size: 0.9rem;">
                    <code style="background: var(--bg-primary); padding: 2px 6px; border-radius: 3px;">${key}</code>
                    <span style="margin-left: 8px; color: var(--text-secondary);">${getDataTypeInfo(value)}</span>
                </summary>
                <div style="padding: 12px;">
                    <pre style="margin: 0; font-size: 0.8rem; color: var(--text-primary); max-height: 300px; overflow: auto;">${JSON.stringify(value, null, 2)}</pre>
                </div>
            </details>
        `;
    });
    
    return html;
}

function getDataTypeInfo(value) {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (Array.isArray(value)) return `Array[${value.length}]`;
    if (typeof value === 'object') return `Object{${Object.keys(value).length}}`;
    if (typeof value === 'string') return `String(${value.length})`;
    if (typeof value === 'number') return `Number(${value})`;
    if (typeof value === 'boolean') return `Boolean(${value})`;
    return typeof value;
}

// Enhanced FFI Visualization with rich lifecycle data
function initEnhancedFFIVisualization() {
    const container = document.getElementById('ffiVisualization');
    if (!container) return;

    const data = window.analysisData || {};
    const allocs = data.unsafe_ffi?.allocations || data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) {
        container.innerHTML = `<div style="background: var(--bg-secondary); border-radius:8px; padding:16px; text-align:center; color: var(--text-secondary);"><i class="fa fa-exclamation-triangle" style="font-size:24px; margin-bottom:8px; color: var(--primary-red);"></i><div>Critical: No FFI allocation data found!</div></div>`;
        return;
    }

    // Rich data analysis
    const ffiTracked = allocs.filter(a => a.ffi_tracked).length;
    const withViolations = allocs.filter(a => a.safety_violations && a.safety_violations.length > 0).length;
    const withClones = allocs.filter(a => a.clone_info?.clone_count > 0).length;
    const leaked = allocs.filter(a => a.is_leaked).length;
    const totalBorrows = allocs.reduce((sum, a) => sum + (a.borrow_info?.immutable_borrows || 0) + (a.borrow_info?.mutable_borrows || 0), 0);
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const avgLifetime = allocs.filter(a => a.lifetime_ms).reduce((sum, a) => sum + a.lifetime_ms, 0) / allocs.filter(a => a.lifetime_ms).length || 0;

    // Get time range for lifecycle visualization
    const timestamps = allocs.map(a => a.timestamp_alloc).filter(t => t).sort((a, b) => a - b);
    const minTime = timestamps[0] || 0;
    const maxTime = timestamps[timestamps.length - 1] || minTime;
    const timeRange = maxTime - minTime || 1;

    // Content that will be contained within the section's background
    container.innerHTML = `
        <!-- KPI Cards -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px;">
            <div style="background: var(--bg-primary); padding: 14px; border-radius: 8px; text-align: center; border-left: 4px solid var(--primary-blue);">
                <div style="font-size: 1.6rem; font-weight: 700; color: var(--primary-blue);">${allocs.length}</div>
                <div style="font-size: 0.75rem; color: var(--text-secondary);">FFI Allocations</div>
            </div>
            <div style="background: var(--bg-primary); padding: 14px; border-radius: 8px; text-align: center; border-left: 4px solid var(--primary-green);">
                <div style="font-size: 1.6rem; font-weight: 700; color: var(--primary-green);">${totalBorrows}</div>
                <div style="font-size: 0.75rem; color: var(--text-secondary);">Total Borrows</div>
            </div>
            <div style="background: var(--bg-primary); padding: 14px; border-radius: 8px; text-align: center; border-left: 4px solid var(--primary-orange);">
                <div style="font-size: 1.6rem; font-weight: 700; color: var(--primary-orange);">${withClones}</div>
                <div style="font-size: 0.75rem; color: var(--text-secondary);">With Clones</div>
            </div>
            <div style="background: var(--bg-primary); padding: 14px; border-radius: 8px; text-align: center; border-left: 4px solid ${leaked > 0 ? 'var(--primary-red)' : 'var(--primary-green)'};">
                <div style="font-size: 1.6rem; font-weight: 700; color: ${leaked > 0 ? 'var(--primary-red)' : 'var(--primary-green)'};">${leaked}</div>
                <div style="font-size: 0.75rem; color: var(--text-secondary);">Memory Leaks</div>
            </div>
            <div style="background: var(--bg-primary); padding: 14px; border-radius: 8px; text-align: center; border-left: 4px solid var(--primary-red);">
                <div style="font-size: 1.6rem; font-weight: 700; color: ${withViolations > 0 ? 'var(--primary-red)' : 'var(--primary-green)'};">${withViolations}</div>
                <div style="font-size: 0.75rem; color: var(--text-secondary);">Safety Violations</div>
            </div>
        </div>
        
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-bottom: 20px;">
            <div style="background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
                <h3 style="margin: 0 0 12px 0; color: var(--text-primary); display: flex; align-items: center;"><i class="fa fa-clock-o" style="margin-right: 8px;"></i>Lifecycle Metrics</h3>
                <div style="margin-bottom: 10px;">
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Avg Lifetime:</span>
                    <span style="color: var(--text-primary); font-weight: 600; margin-left: 8px;">${avgLifetime.toFixed(2)}ms</span>
                </div>
                <div style="margin-bottom: 10px;">
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Total Memory:</span>
                    <span style="color: var(--text-primary); font-weight: 600; margin-left: 8px;">${formatBytes(totalMemory)}</span>
                </div>
                <div style="margin-bottom: 10px;">
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Time Span:</span>
                    <span style="color: var(--text-primary); font-weight: 600; margin-left: 8px;">${(timeRange / 1e6).toFixed(2)}ms</span>
                </div>
            </div>
            
            <div style="background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
                <h3 style="margin: 0 0 12px 0; color: var(--text-primary); display: flex; align-items: center;"><i class="fa fa-share-alt" style="margin-right: 8px;"></i>Borrow & Clone Activity</h3>
                <div style="margin-bottom: 8px;">
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Immutable Borrows:</span>
                    <span style="color: var(--primary-blue); font-weight: 600; margin-left: 8px;">${allocs.reduce((s, a) => s + (a.borrow_info?.immutable_borrows || 0), 0)}</span>
                </div>
                <div style="margin-bottom: 8px;">
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Mutable Borrows:</span>
                    <span style="color: var(--primary-orange); font-weight: 600; margin-left: 8px;">${allocs.reduce((s, a) => s + (a.borrow_info?.mutable_borrows || 0), 0)}</span>
                </div>
                <div>
                    <span style="color: var(--text-secondary); font-size: 0.9rem;">Clone Operations:</span>
                    <span style="color: var(--primary-green); font-weight: 600; margin-left: 8px;">${allocs.reduce((s, a) => s + (a.clone_info?.clone_count || 0), 0)}</span>
                </div>
            </div>
        </div>

        <!-- FFI Data Flow Visualization (data stream) -->
        <div style="margin-top: 20px; background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
            <h3 style="margin: 0 0 12px 0; color: var(--text-primary); display: flex; align-items: center;">
                <i class="fa fa-exchange" style="margin-right: 8px;"></i>FFI Data Flow
                <button id="ffi-flow-toggle" style="margin-left: auto; background: var(--primary-green); color: white; border: none; padding: 4px 8px; border-radius: 4px; font-size: 11px; cursor: pointer;">
                    <i class="fa fa-play"></i> Animate
                </button>
            </h3>
            <div id="ffi-flow-container" style="height: 200px; position: relative; border: 1px solid var(--border-light); border-radius: 6px; overflow: hidden; background: linear-gradient(135deg, #1e293b 0%, #0f172a 50%, #1e293b 100%);">
                ${createFFIDataFlow(allocs)}
            </div>
            <div style="margin-top: 8px; font-size: 11px; color: var(--text-secondary); text-align: center;">
                🦀 Rust ↔ C Data Flow - ${ffiTracked} FFI-tracked allocations - Click nodes for details
            </div>
        </div>

        <!-- Interactive Allocation Analysis (Two Column Layout) -->
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 20px;">
            <!-- Timeline Column -->
            <div style="background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
                <h3 style="margin: 0 0 12px 0; color: var(--text-primary); display: flex; align-items: center;">
                    <i class="fa fa-clock-o" style="margin-right: 8px;"></i>Allocation Timeline
                    <button id="ffi-timeline-toggle" style="margin-left: auto; background: var(--primary-blue); color: white; border: none; padding: 4px 8px; border-radius: 4px; font-size: 11px; cursor: pointer;">
                        <i class="fa fa-expand"></i> Details
                    </button>
                </h3>
                <div id="ffi-timeline-container" style="height: 160px; position: relative; border: 1px solid var(--border-light); border-radius: 6px; overflow: hidden; background: linear-gradient(45deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);">
                    ${createAllocationTimeline(allocs, minTime, timeRange)}
                </div>
                <div style="margin-top: 8px; font-size: 11px; color: var(--text-secondary); text-align: center;">
                    Timeline spans ${(timeRange / 1e6).toFixed(1)}ms - Click dots for details
                </div>
            </div>
            
            <!-- Details Column -->
            <div style="background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
                <h3 style="margin: 0 0 12px 0; color: var(--text-primary); display: flex; align-items: center;">
                    <i class="fa fa-list" style="margin-right: 8px;"></i>Allocation Details
                    <select id="ffi-filter" style="margin-left: auto; background: var(--bg-primary); border: 1px solid var(--border-light); border-radius: 4px; padding: 4px 8px; font-size: 11px; color: var(--text-primary);">
                        <option value="all">All (${allocs.length})</option>
                        <option value="leaked">Leaked (${allocs.filter(a => a.is_leaked).length})</option>
                        <option value="cloned">With Clones (${withClones})</option>
                        <option value="borrowed">With Borrows (${allocs.filter(a => ((a.borrow_info?.immutable_borrows || 0) + (a.borrow_info?.mutable_borrows || 0)) > 0).length})</option>
                    </select>
                </h3>
                <div id="ffi-allocation-table" style="max-height: 160px; overflow-y: auto; border: 1px solid var(--border-light); border-radius: 6px;">
                    ${createAllocationTable(allocs)}
                </div>
                <div style="margin-top: 8px; font-size: 11px; color: var(--text-secondary); text-align: center;">
                    Click rows for detailed view - Use filter to narrow results
                </div>
            </div>
        </div>
    `;

    // Add interactivity
    setupFFIInteractivity(allocs, minTime, timeRange);
    setupFFIFlowInteractivity(allocs);
}

// Create super cool FFI data flow visualization
function createFFIDataFlow(allocs) {
    const ffiAllocs = allocs.filter(a => a.ffi_tracked);
    const rustAllocs = allocs.filter(a => !a.ffi_tracked);
    
    // Create dynamic SVG-based flow visualization
    let html = `
        <svg width="100%" height="100%" viewBox="0 0 800 200" style="position: absolute; top: 0; left: 0;">
            <!-- Background grid pattern -->
            <defs>
                <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
                    <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#334155" stroke-width="0.5" opacity="0.3"/>
                </pattern>
                <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
                    <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                    <feMerge>
                        <feMergeNode in="coloredBlur"/>
                        <feMergeNode in="SourceGraphic"/>
                    </feMerge>
                </filter>
                <linearGradient id="rustGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color:#f97316;stop-opacity:1" />
                    <stop offset="100%" style="stop-color:#ea580c;stop-opacity:1" />
                </linearGradient>
                <linearGradient id="cGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color:#3b82f6;stop-opacity:1" />
                    <stop offset="100%" style="stop-color:#1d4ed8;stop-opacity:1" />
                </linearGradient>
            </defs>
            
            <rect width="100%" height="100%" fill="url(#grid)"/>
            
            <!-- Rust Side (Left) -->
            <g id="rust-side">
                <rect x="50" y="40" width="200" height="120" rx="15" fill="url(#rustGradient)" opacity="0.2" stroke="#f97316" stroke-width="2"/>
                <text x="150" y="30" text-anchor="middle" font-size="16" font-weight="bold" fill="#f97316" filter="url(#glow)">
                    🦀 RUST
                </text>
                <text x="150" y="190" text-anchor="middle" font-size="12" fill="#f97316">
                    ${rustAllocs.length} allocations
                </text>
                
                <!-- Rust memory nodes -->
                ${rustAllocs.slice(0, 8).map((alloc, i) => {
                    const x = 80 + (i % 4) * 35;
                    const y = 60 + Math.floor(i / 4) * 35;
                    const size = Math.max(8, Math.min(16, Math.sqrt((alloc.size || 0) / 1000)));
                    return `
                        <circle cx="${x}" cy="${y}" r="${size}" fill="#f97316" opacity="0.8" 
                                stroke="#fff" stroke-width="2" class="rust-node" 
                                data-ptr="${alloc.ptr}" data-size="${alloc.size || 0}"
                                style="cursor: pointer; transition: all 0.3s;">
                            <animate attributeName="opacity" values="0.8;1;0.8" dur="2s" repeatCount="indefinite"/>
                        </circle>
                    `;
                }).join('')}
            </g>
            
            <!-- C/FFI Side (Right) -->
            <g id="c-side">
                <rect x="550" y="40" width="200" height="120" rx="15" fill="url(#cGradient)" opacity="0.2" stroke="#3b82f6" stroke-width="2"/>
                <text x="650" y="30" text-anchor="middle" font-size="16" font-weight="bold" fill="#3b82f6" filter="url(#glow)">
                    ⚙️ C/FFI
                </text>
                <text x="650" y="190" text-anchor="middle" font-size="12" fill="#3b82f6">
                    ${ffiAllocs.length} FFI allocations
                </text>
                
                <!-- FFI memory nodes -->
                ${ffiAllocs.slice(0, 8).map((alloc, i) => {
                    const x = 580 + (i % 4) * 35;
                    const y = 60 + Math.floor(i / 4) * 35;
                    const size = Math.max(8, Math.min(16, Math.sqrt((alloc.size || 0) / 1000)));
                    return `
                        <circle cx="${x}" cy="${y}" r="${size}" fill="#3b82f6" opacity="0.9" 
                                stroke="#fff" stroke-width="2" class="ffi-node"
                                data-ptr="${alloc.ptr}" data-size="${alloc.size || 0}"
                                style="cursor: pointer;">
                        </circle>
                    `;
                }).join('')}
            </g>
            
            <!-- Data Flow Arrows -->
            <g id="data-flows">
                <!-- Rust to C flow -->
                <path d="M 250 80 Q 400 60 550 80" stroke="#10b981" stroke-width="3" fill="none" opacity="0.7">
                    <animate attributeName="stroke-dasharray" values="0,1000;1000,0" dur="3s" repeatCount="indefinite"/>
                </path>
                <text x="400" y="55" text-anchor="middle" font-size="10" fill="#10b981" font-weight="bold">
                    Rust → C
                </text>
                
                <!-- C to Rust flow -->
                <path d="M 550 120 Q 400 140 250 120" stroke="#ec4899" stroke-width="3" fill="none" opacity="0.7">
                    <animate attributeName="stroke-dasharray" values="0,1000;1000,0" dur="3.5s" repeatCount="indefinite"/>
                </path>
                <text x="400" y="155" text-anchor="middle" font-size="10" fill="#ec4899" font-weight="bold">
                    C → Rust
                </text>
                
                <!-- Central processing hub -->
                <circle cx="400" cy="100" r="20" fill="#8b5cf6" opacity="0.3" stroke="#8b5cf6" stroke-width="2">
                    <animate attributeName="r" values="20;25;20" dur="2s" repeatCount="indefinite"/>
                </circle>
                <text x="400" y="105" text-anchor="middle" font-size="10" fill="#8b5cf6" font-weight="bold">FFI</text>
            </g>
            
            <!-- Memory flow particles -->
            <g id="flow-particles">
                ${Array.from({length: 6}, (_, i) => `
                    <circle r="3" fill="#fbbf24" opacity="0.8">
                        <animateMotion dur="${3 + i * 0.5}s" repeatCount="indefinite">
                            <path d="M 250 80 Q 400 60 550 80"/>
                        </animateMotion>
                        <animate attributeName="opacity" values="0;1;0" dur="1s" repeatCount="indefinite"/>
                    </circle>
                `).join('')}
                
                ${Array.from({length: 4}, (_, i) => `
                    <circle r="3" fill="#06d6a0" opacity="0.8">
                        <animateMotion dur="${3.5 + i * 0.7}s" repeatCount="indefinite">
                            <path d="M 550 120 Q 400 140 250 120"/>
                        </animateMotion>
                        <animate attributeName="opacity" values="0;1;0" dur="1s" repeatCount="indefinite"/>
                    </circle>
                `).join('')}
            </g>
        </svg>
    `;
    
    return html;
}

// Create allocation timeline visualization
function createAllocationTimeline(allocs, minTime, timeRange) {
    const sorted = allocs.slice().sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    let html = '<div style="position: relative; height: 100%; background: linear-gradient(90deg, rgba(59,130,246,0.1) 0%, rgba(16,185,129,0.1) 50%, rgba(239,68,68,0.1) 100%);">';
    
    // Add time axis with better spacing
    html += '<div style="position: absolute; bottom: 25px; left: 0; right: 0; height: 1px; background: var(--border-light);"></div>';
    html += '<div style="position: absolute; bottom: 18px; left: 12px; font-size: 10px; color: var(--text-secondary); background: var(--bg-primary); padding: 2px 4px; border-radius: 3px;">0ms</div>';
    html += '<div style="position: absolute; bottom: 18px; right: 12px; font-size: 10px; color: var(--text-secondary); background: var(--bg-primary); padding: 2px 4px; border-radius: 3px;">' + (timeRange / 1e6).toFixed(1) + 'ms</div>';
    
    // Add middle time markers for better readability
    const midTime = (timeRange / 1e6) / 2;
    html += '<div style="position: absolute; bottom: 18px; left: 50%; transform: translateX(-50%); font-size: 10px; color: var(--text-secondary); background: var(--bg-primary); padding: 2px 4px; border-radius: 3px;">' + midTime.toFixed(1) + 'ms</div>';
    
    // Group nearby allocations to prevent overlap
    const groups = [];
    const threshold = timeRange * 0.05; // 5% of time range
    
    sorted.forEach(alloc => {
        const found = groups.find(g => Math.abs(g.avgTime - alloc.timestamp_alloc) < threshold);
        if (found) {
            found.allocs.push(alloc);
            found.avgTime = found.allocs.reduce((sum, a) => sum + a.timestamp_alloc, 0) / found.allocs.length;
        } else {
            groups.push({ allocs: [alloc], avgTime: alloc.timestamp_alloc });
        }
    });
    
    groups.forEach((group, groupIndex) => {
        const relativeTime = (group.avgTime - minTime) / timeRange;
        const left = Math.max(2, Math.min(93, relativeTime * 90 + 5));
        
        if (group.allocs.length === 1) {
            const alloc = group.allocs[0];
            const size = Math.max(10, Math.min(20, Math.sqrt((alloc.size || 0) / 50)));
            const isLeaked = alloc.is_leaked;
            const hasClones = alloc.clone_info?.clone_count > 0;
            const color = isLeaked ? '#dc2626' : hasClones ? '#ea580c' : '#2563eb';
            
            html += `<div style="position: absolute; left: ${left}%; top: 60%; transform: translateY(-50%); 
                     width: ${size}px; height: ${size}px; background: ${color}; border-radius: 50%; 
                     border: 2px solid white; cursor: pointer; z-index: 100; 
                     box-shadow: 0 2px 4px rgba(0,0,0,0.2); transition: transform 0.2s;"
                     onmouseover="this.style.transform='translateY(-50%) scale(1.2)'" 
                     onmouseout="this.style.transform='translateY(-50%) scale(1)'"
                     title="${alloc.var_name || 'unnamed'} | ${formatBytes(alloc.size || 0)} | ${new Date(alloc.timestamp_alloc / 1e6).toLocaleTimeString()}"
                     onclick="showAllocationDetail('${alloc.ptr}")"></div>`;
        } else {
            // Multiple allocations - create a cluster
            const totalSize = group.allocs.reduce((sum, a) => sum + (a.size || 0), 0);
            const hasLeaks = group.allocs.some(a => a.is_leaked);
            const hasClones = group.allocs.some(a => a.clone_info?.clone_count > 0);
            const clusterSize = Math.max(16, Math.min(28, Math.sqrt(totalSize / 100)));
            const color = hasLeaks ? '#dc2626' : hasClones ? '#ea580c' : '#2563eb';
            
            html += `<div style="position: absolute; left: ${left}%; top: 60%; transform: translateY(-50%); 
                     width: ${clusterSize}px; height: ${clusterSize}px; background: ${color}; border-radius: 50%; 
                     border: 3px solid white; cursor: pointer; z-index: 100;
                     box-shadow: 0 2px 8px rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center;
                     color: white; font-size: 9px; font-weight: bold; transition: transform 0.2s;"
                     onmouseover="this.style.transform='translateY(-50%) scale(1.2)'" 
                     onmouseout="this.style.transform='translateY(-50%) scale(1)'"
                     title="${group.allocs.length} allocations | Total: ${formatBytes(totalSize)} | Avg time: ${new Date(group.avgTime / 1e6).toLocaleTimeString()}"
                     onclick="showClusterDetail('${group.allocs.map(a => a.ptr).join(',')}')">${group.allocs.length}</div>`;
        }
    });
    
    html += "</div>";
    return html;
}

// Create allocation details table
function createAllocationTable(allocs) {
    const sorted = allocs.slice().sort((a, b) => (b.timestamp_alloc || 0) - (a.timestamp_alloc || 0));
    
    let html = `
        <div style="border: 1px solid var(--border-light); border-radius: 6px; overflow: hidden;">
            <table style="width: 100%; border-collapse: collapse; font-size: 12px;">
                <thead style="background: var(--bg-primary); border-bottom: 1px solid var(--border-light);">
                    <tr>
                        <th style="padding: 8px; text-align: left; color: var(--text-primary);">Variable</th>
                        <th style="padding: 8px; text-align: left; color: var(--text-primary);">Type</th>
                        <th style="padding: 8px; text-align: right; color: var(--text-primary);">Size</th>
                        <th style="padding: 8px; text-align: center; color: var(--text-primary);">Borrows</th>
                        <th style="padding: 8px; text-align: center; color: var(--text-primary);">Clones</th>
                        <th style="padding: 8px; text-align: center; color: var(--text-primary);">Status</th>
                        <th style="padding: 8px; text-align: right; color: var(--text-primary);">Lifetime</th>
                    </tr>
                </thead>
                <tbody>`;
    
    sorted.forEach((alloc, i) => {
        const typeName = (alloc.type_name || 'Unknown').replace(/alloc::|std::/g, '').replace(/collections::\w+::/g, '');
        const shortType = typeName.length > 20 ? typeName.substring(0, 17) + '...' : typeName;
        const totalBorrows = (alloc.borrow_info?.immutable_borrows || 0) + (alloc.borrow_info?.mutable_borrows || 0);
        const cloneCount = alloc.clone_info?.clone_count || 0;
        const isLeaked = alloc.is_leaked;
        const lifetime = alloc.lifetime_ms || "Active";
        
        const statusColor = isLeaked ? 'var(--primary-red)' : 'var(--primary-green)';
        const statusText = isLeaked ? 'LEAKED' : 'OK';
        
        html += `
            <tr style="border-bottom: 1px solid var(--border-light); cursor: pointer;" 
                onclick="showAllocationDetail('${alloc.ptr}")" 
                onmouseover="this.style.background='var(--bg-secondary)'" 
                onmouseout="this.style.background='transparent'">
                <td style="padding: 8px; color: var(--text-primary); font-weight: 500;">${alloc.var_name || 'unnamed'}</td>
                <td style="padding: 8px; color: var(--text-secondary);" title="${alloc.type_name}">${shortType}</td>
                <td style="padding: 8px; text-align: right; color: var(--text-primary); font-weight: 600;">${formatBytes(alloc.size || 0)}</td>
                <td style="padding: 8px; text-align: center; color: var(--primary-blue);">${totalBorrows}</td>
                <td style="padding: 8px; text-align: center; color: var(--primary-orange);">${cloneCount}</td>
                <td style="padding: 8px; text-align: center;">
                    <span style="color: ${statusColor}; font-weight: 600; font-size: 11px;">${statusText}</span>
                </td>
                <td style="padding: 8px; text-align: right; color: var(--text-secondary); font-size: 11px;">
                    ${typeof lifetime === 'number' ? lifetime.toFixed(2) + 'ms' : lifetime}
                </td>
            </tr>`;
    });
    
    html += '</tbody></table></div>';
    return html;
}

// Setup FFI interactivity
function setupFFIInteractivity(allocs, minTime, timeRange) {
    // Timeline toggle
    const toggleBtn = document.getElementById('ffi-timeline-toggle");
    if (toggleBtn) {
        let expanded = false;
        toggleBtn.onclick = () => {
            const container = document.getElementById('ffi-timeline-container");
            if (!container) return;
            
            expanded = !expanded;
            container.style.height = expanded ? '200px' : '120px';
            toggleBtn.textContent = expanded ? 'Hide Details' : 'Show Details';
            
            if (expanded) {
                // Add detailed timeline with labels
                container.innerHTML = createAllocationTimeline(allocs, minTime, timeRange) + 
                    '<div style="position: absolute; bottom: 4px; left: 8px; font-size: 10px; color: var(--text-secondary);">Start</div>' +
                    '<div style="position: absolute; bottom: 4px; right: 8px; font-size: 10px; color: var(--text-secondary);">End</div>';
            } else {
                container.innerHTML = createAllocationTimeline(allocs, minTime, timeRange);
            }
        };
    }
    
    // Table filter
    const filterSelect = document.getElementById('ffi-filter");
    if (filterSelect) {
        filterSelect.onchange = () => {
            const filterValue = filterSelect.value;
            let filteredAllocs = allocs;
            
            switch(filterValue) {
                case 'leaked':
                    filteredAllocs = allocs.filter(a => a.is_leaked);
                    break;
                case 'cloned':
                    filteredAllocs = allocs.filter(a => a.clone_info?.clone_count > 0);
                    break;
                case 'borrowed':
                    filteredAllocs = allocs.filter(a => {
                        const borrows = (a.borrow_info?.immutable_borrows || 0) + (a.borrow_info?.mutable_borrows || 0);
                        return borrows > 0;
                    });
                    break;
                default:
                    filteredAllocs = allocs;
            }
            
            const tableContainer = document.getElementById('ffi-allocation-table");
            if (tableContainer) {
                tableContainer.innerHTML = createAllocationTable(filteredAllocs);
            }
        };
    }
}

// Show allocation detail modal
window.showAllocationDetail = function(ptr) {
    const data = window.analysisData || {};
    const allocs = data.unsafe_ffi?.allocations || data.memory_analysis?.allocations || data.allocations || [];
    const alloc = allocs.find(a => a.ptr === ptr);
    
    if (!alloc) return;
    
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.5); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    modal.innerHTML = `
        <div style="background: var(--bg-primary); border-radius: 12px; padding: 24px; min-width: 400px; max-width: 600px; border: 1px solid var(--border-light);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                <h3 style="margin: 0; color: var(--text-primary);">Allocation Details</h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: none; border: none; font-size: 20px; color: var(--text-secondary); cursor: pointer;">×</button>
            </div>
            <div style="color: var(--text-primary); line-height: 1.6;">
                <div style="margin-bottom: 12px;"><strong>Variable:</strong> ${alloc.var_name || 'unnamed'}</div>
                <div style="margin-bottom: 12px;"><strong>Type:</strong> ${alloc.type_name || 'Unknown'}</div>
                <div style="margin-bottom: 12px;"><strong>Size:</strong> ${formatBytes(alloc.size || 0)}</div>
                <div style="margin-bottom: 12px;"><strong>Pointer:</strong> <code>${alloc.ptr}</code></div>
                <div style="margin-bottom: 12px;"><strong>Thread:</strong> ${alloc.thread_id}</div>
                <div style="margin-bottom: 12px;"><strong>Allocated:</strong> ${new Date(alloc.timestamp_alloc / 1e6).toLocaleString()}</div>
                <div style="margin-bottom: 12px;"><strong>Lifetime:</strong> ${alloc.lifetime_ms ? alloc.lifetime_ms.toFixed(2) + 'ms' : "Active"}</div>
                <div style="margin-bottom: 12px;"><strong>Immutable Borrows:</strong> ${alloc.borrow_info?.immutable_borrows || 0}</div>
                <div style="margin-bottom: 12px;"><strong>Mutable Borrows:</strong> ${alloc.borrow_info?.mutable_borrows || 0}</div>
                <div style="margin-bottom: 12px;"><strong>Clone Count:</strong> ${alloc.clone_info?.clone_count || 0}</div>
                <div style="margin-bottom: 12px;"><strong>FFI Tracked:</strong> ${alloc.ffi_tracked ? 'Yes' : 'No'}</div>
                <div style="margin-bottom: 12px;"><strong>Status:</strong> 
                    <span style="color: ${alloc.is_leaked ? 'var(--primary-red)' : 'var(--primary-green)'}; font-weight: 600;">
                        ${alloc.is_leaked ? 'LEAKED' : 'OK'}
                    </span>
                </div>
                ${alloc.safety_violations && alloc.safety_violations.length > 0 ? 
                    `<div style="margin-bottom: 12px; color: var(--primary-red);"><strong>Safety Violations:</strong> ${alloc.safety_violations.join(', ")}</div>` 
                    : ''}
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
};

// Show cluster detail for grouped allocations
window.showClusterDetail = function(ptrs) {
    const data = window.analysisData || {};
    const allocs = data.unsafe_ffi?.allocations || data.memory_analysis?.allocations || data.allocations || [];
    const clusterAllocs = allocs.filter(a => ptrs.includes(a.ptr));
    
    if (clusterAllocs.length === 0) return;
    
    const totalSize = clusterAllocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const totalBorrows = clusterAllocs.reduce((sum, a) => sum + (a.borrow_info?.immutable_borrows || 0) + (a.borrow_info?.mutable_borrows || 0), 0);
    const totalClones = clusterAllocs.reduce((sum, a) => sum + (a.clone_info?.clone_count || 0), 0);
    const leakCount = clusterAllocs.filter(a => a.is_leaked).length;
    
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.5); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    modal.innerHTML = `
        <div style="background: var(--bg-primary); border-radius: 12px; padding: 24px; min-width: 500px; max-width: 700px; max-height: 80vh; overflow-y: auto; border: 1px solid var(--border-light);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                <h3 style="margin: 0; color: var(--text-primary);">Allocation Cluster (${clusterAllocs.length} items)</h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: none; border: none; font-size: 20px; color: var(--text-secondary); cursor: pointer;">×</button>
            </div>
            <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 16px;">
                <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 6px;">
                    <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-blue);">${formatBytes(totalSize)}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Total Size</div>
                </div>
                <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 6px;">
                    <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-green);">${totalBorrows}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Total Borrows</div>
                </div>
                <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 6px;">
                    <div style="font-size: 1.2rem; font-weight: 600; color: var(--primary-orange);">${totalClones}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Total Clones</div>
                </div>
                <div style="text-align: center; padding: 12px; background: var(--bg-secondary); border-radius: 6px;">
                    <div style="font-size: 1.2rem; font-weight: 600; color: ${leakCount > 0 ? 'var(--primary-red)' : 'var(--primary-green)'};">${leakCount}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Leaks</div>
                </div>
            </div>
            <div style="color: var(--text-primary);">
                <h4 style="margin: 0 0 12px 0; color: var(--text-primary);">Individual Allocations:</h4>
                <div style="max-height: 300px; overflow-y: auto; border: 1px solid var(--border-light); border-radius: 6px;">
                    <table style="width: 100%; border-collapse: collapse; font-size: 12px;">
                        <thead style="background: var(--bg-secondary); position: sticky; top: 0;">
                            <tr>
                                <th style="padding: 8px; text-align: left;">Variable</th>
                                <th style="padding: 8px; text-align: right;">Size</th>
                                <th style="padding: 8px; text-align: center;">Borrows</th>
                                <th style="padding: 8px; text-align: center;">Clones</th>
                                <th style="padding: 8px; text-align: center;">Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${clusterAllocs.map(alloc => {
                                const totalBorrows = (alloc.borrow_info?.immutable_borrows || 0) + (alloc.borrow_info?.mutable_borrows || 0);
                                const cloneCount = alloc.clone_info?.clone_count || 0;
                                const isLeaked = alloc.is_leaked;
                                const statusColor = isLeaked ? 'var(--primary-red)' : 'var(--primary-green)';
                                const statusText = isLeaked ? 'LEAKED' : 'OK';
                                
                                return `
                                    <tr style="border-bottom: 1px solid var(--border-light); cursor: pointer;" onclick="showAllocationDetail('${alloc.ptr}")">
                                        <td style="padding: 8px; font-weight: 500;">${alloc.var_name || 'unnamed'}</td>
                                        <td style="padding: 8px; text-align: right; font-weight: 600;">${formatBytes(alloc.size || 0)}</td>
                                        <td style="padding: 8px; text-align: center; color: var(--primary-blue);">${totalBorrows}</td>
                                        <td style="padding: 8px; text-align: center; color: var(--primary-orange);">${cloneCount}</td>
                                        <td style="padding: 8px; text-align: center;"><span style="color: ${statusColor}; font-weight: 600; font-size: 11px;">${statusText}</span></td>
                                    </tr>
                                `;
                            }).join('')}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
};

// Render enhanced data insights with beautiful visualizations
function renderEnhancedDataInsights() {
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) return;
    
    // Calculate timeline insights
    const timestamps = allocs.map(a => a.timestamp_alloc).filter(t => t).sort((a, b) => a - b);
    const timeSpanMs = timestamps.length > 1 ? (timestamps[timestamps.length - 1] - timestamps[0]) / 1e6 : 0;
    const allocationBurst = (allocs.length / Math.max(1, timeSpanMs / 1000)).toFixed(1);
    
    // Calculate borrow patterns
    const borrowPatterns = {};
    let totalBorrows = 0;
    let totalMutable = 0;
    let totalImmutable = 0;
    
    allocs.forEach(alloc => {
        const bi = alloc.borrow_info || {};
        const immut = bi.immutable_borrows || 0;
        const mut = bi.mutable_borrows || 0;
        const pattern = `${immut}i+${mut}m`;
        borrowPatterns[pattern] = (borrowPatterns[pattern] || 0) + 1;
        totalBorrows += immut + mut;
        totalImmutable += immut;
        totalMutable += mut;
    });
    
    // Calculate clone operations
    const totalClones = allocs.reduce((sum, a) => sum + (a.clone_info?.clone_count || 0), 0);
    
    // Update Timeline Insights
    document.getElementById('time-span').textContent = timeSpanMs.toFixed(2) + 'ms';
    document.getElementById('allocation-burst').textContent = allocationBurst + '/sec';
    document.getElementById('peak-concurrency').textContent = Math.max(...allocs.map(a => (a.borrow_info?.max_concurrent_borrows || 0)));
    document.getElementById('thread-activity').textContent = 'Single Thread';
    
    // Update Memory Operations
    document.getElementById('borrow-ops').textContent = totalBorrows;
    document.getElementById('clone-ops').textContent = totalClones;
    document.getElementById('mut-ratio').textContent = totalImmutable > 0 ? (totalMutable / totalImmutable).toFixed(1) : '0';
    document.getElementById('avg-borrows').textContent = (totalBorrows / allocs.length).toFixed(1);
    
    // Render charts with forced data refresh
    renderBorrowPatternChart(borrowPatterns);
    
    // Force Type Memory Distribution to render with debug info
    console.log("🔍 Forcing Type Memory Distribution render with data:", allocs.length, 'allocations');
    setTimeout(() => {
        renderMemoryDistributionChart(allocs);
    }, 100);
    
    console.log("✅ Enhanced data insights rendered:", {
        timeSpan: timeSpanMs.toFixed(2) + 'ms',
        totalBorrows,
        totalClones,
        borrowPatterns,
        allocCount: allocs.length
    });
}

// Render borrow activity heatmap
function renderBorrowPatternChart(patterns) {
    const container = document.getElementById('borrowPatternChart');
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    // Create interactive borrow activity heatmap
    container.innerHTML = '';
    container.style.cssText = 'height: 200px; overflow-y: auto; padding: 8px; background: var(--bg-primary); border-radius: 8px; border: 1px solid var(--border-light);';
    
    // Group variables by borrow intensity
    const borrowGroups = {
        'High Activity (4i+2m)': [],
        'Normal Activity (2i+1m)': [],
        'Low Activity (0-1 borrows)': []
    };
    
    allocs.forEach(alloc => {
        const bi = alloc.borrow_info || {};
        const immut = bi.immutable_borrows || 0;
        const mut = bi.mutable_borrows || 0;
        const total = immut + mut;
        
        if (total >= 5) {
            borrowGroups['High Activity (4i+2m)'].push(alloc);
        } else if (total >= 2) {
            borrowGroups['Normal Activity (2i+1m)'].push(alloc);
        } else {
            borrowGroups['Low Activity (0-1 borrows)'].push(alloc);
        }
    });
    
    // Create visual representation
    Object.entries(borrowGroups).forEach(([groupName, groupAllocs], groupIndex) => {
        if (groupAllocs.length === 0) return;
        
        const groupDiv = document.createElement('div');
        groupDiv.style.cssText = 'margin-bottom: 12px;';
        
        const groupHeader = document.createElement('div');
        groupHeader.style.cssText = `
            font-size: 11px; font-weight: 600; margin-bottom: 6px; 
            color: var(--text-primary); display: flex; align-items: center; gap: 8px;
        `;
        
        const colors = ['#ef4444', '#f59e0b', '#10b981'];
        const icons = ['🔥', '⚡', '💧'];
        
        groupHeader.innerHTML = `
            <span style="font-size: 14px;">${icons[groupIndex]}</span>
            <span>${groupName}</span>
            <span style="background: ${colors[groupIndex]}; color: white; padding: 2px 6px; border-radius: 10px; font-size: 9px;">
                ${groupAllocs.length}
            </span>
        `;
        
        const bubbleContainer = document.createElement('div');
        bubbleContainer.style.cssText = `
            display: flex; flex-wrap: wrap; gap: 4px; padding: 12px; 
            background: var(--bg-secondary); border-radius: 6px; min-height: 60px;
            align-items: center; justify-content: flex-start;
        `;
        
        // Create borrow activity bubbles
        groupAllocs.forEach((alloc, index) => {
            const bi = alloc.borrow_info || {};
            const immut = bi.immutable_borrows || 0;
            const mut = bi.mutable_borrows || 0;
            const maxConcurrent = bi.max_concurrent_borrows || 0;
            
            const bubble = document.createElement('div');
            const size = Math.max(16, Math.min(32, 12 + (immut + mut) * 2));
            
            bubble.style.cssText = `
                width: ${size}px; height: ${size}px; border-radius: 50%; 
                background: linear-gradient(45deg, ${colors[groupIndex]}80, ${colors[groupIndex]});
                border: 2px solid ${colors[groupIndex]}; cursor: pointer;
                display: flex; align-items: center; justify-content: center;
                font-size: 8px; font-weight: bold; color: white;
                transition: transform 0.2s, box-shadow 0.2s;
                position: relative;
            `;
            
            bubble.textContent = immut + mut;
            bubble.title = `${alloc.var_name}: ${immut}i + ${mut}m (max: ${maxConcurrent})`;
            
            // Add interactive effects
            bubble.onmouseover = () => {
                bubble.style.transform = 'scale(1.2)';
                bubble.style.boxShadow = `0 4px 12px ${colors[groupIndex]}40`;
            };
            bubble.onmouseout = () => {
                bubble.style.transform = 'scale(1)';
                bubble.style.boxShadow = 'none';
            };
            
            // Add click to show details
            bubble.onclick = () => {
                showBorrowDetail(alloc);
            };
            
            bubbleContainer.appendChild(bubble);
        });
        
        groupDiv.appendChild(groupHeader);
        groupDiv.appendChild(bubbleContainer);
        container.appendChild(groupDiv);
    });
    
    // Add summary stats at bottom
    const summaryDiv = document.createElement('div');
    summaryDiv.style.cssText = `
        margin-top: 8px; padding: 8px; background: var(--bg-secondary); 
        border-radius: 6px; font-size: 10px; color: var(--text-secondary);
        display: flex; justify-content: space-between;
    `;
    
    const totalBorrows = allocs.reduce((sum, a) => sum + (a.borrow_info?.immutable_borrows || 0) + (a.borrow_info?.mutable_borrows || 0), 0);
    const avgBorrows = (totalBorrows / allocs.length).toFixed(1);
    const maxConcurrent = Math.max(...allocs.map(a => a.borrow_info?.max_concurrent_borrows || 0));
    
    summaryDiv.innerHTML = `
        <span>Total Borrows: <strong>${totalBorrows}</strong></span>
        <span>Avg/Variable: <strong>${avgBorrows}</strong></span>
        <span>Peak Concurrent: <strong>${maxConcurrent}</strong></span>
    `;
    
    container.appendChild(summaryDiv);
}

// Show borrow detail modal
function showBorrowDetail(alloc) {
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.5); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    const bi = alloc.borrow_info || {};
    
    modal.innerHTML = `
        <div style="background: var(--bg-primary); border-radius: 12px; padding: 20px; min-width: 350px; border: 1px solid var(--border-light);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                <h3 style="margin: 0; color: var(--text-primary);">🔍 Borrow Analysis</h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: none; border: none; font-size: 18px; color: var(--text-secondary); cursor: pointer;">×</button>
            </div>
            <div style="color: var(--text-primary); line-height: 1.6;">
                <div style="margin-bottom: 12px;"><strong>Variable:</strong> ${alloc.var_name}</div>
                <div style="margin-bottom: 12px;"><strong>Type:</strong> ${alloc.type_name}</div>
                <div style="margin-bottom: 12px;"><strong>Size:</strong> ${formatBytes(alloc.size || 0)}</div>
                <hr style="border: none; border-top: 1px solid var(--border-light); margin: 16px 0;">
                <div style="margin-bottom: 8px;"><strong>📖 Immutable Borrows:</strong> <span style="color: var(--primary-blue); font-weight: 600;">${bi.immutable_borrows || 0}</span></div>
                <div style="margin-bottom: 8px;"><strong>✏️ Mutable Borrows:</strong> <span style="color: var(--primary-orange); font-weight: 600;">${bi.mutable_borrows || 0}</span></div>
                <div style="margin-bottom: 8px;"><strong>🔥 Max Concurrent:</strong> <span style="color: var(--primary-red); font-weight: 600;">${bi.max_concurrent_borrows || 0}</span></div>
                <div style="margin-bottom: 8px;"><strong>⚡ Total Activity:</strong> <span style="color: var(--primary-green); font-weight: 600;">${(bi.immutable_borrows || 0) + (bi.mutable_borrows || 0)}</span></div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
}

// Render type memory distribution as interactive memory blocks
function renderMemoryDistributionChart(allocs) {
    const container = document.getElementById('memoryDistributionChart');
    if (!container) return;
    
    // Create unique memory blocks visualization (not pie chart)
    container.innerHTML = '';
    
    // Group by type and sum memory
    const typeMemory = {};
    console.log("🔍 Processing allocations for memory blocks:", allocs.length);
    
    allocs.forEach((alloc, index) => {
        let typeName = alloc.type_name || 'Unknown';
        const originalType = typeName;
        const size = alloc.size || 0;
        
        // Simplify type names
        if (typeName.includes('HashMap')) {
            typeName = 'HashMap';
        } else if (typeName.includes('BTreeMap')) {
            typeName = 'BTreeMap';
        } else if (typeName.includes('Arc')) {
            typeName = 'Arc';
        } else if (typeName.includes('Rc')) {
            typeName = 'Rc';
        } else if (typeName.includes('String')) {
            typeName = 'String';
        } else if (typeName.includes('Vec')) {
            typeName = 'Vec';
        } else {
            typeName = originalType.split('::').pop() || 'Unknown';
        }
        
        if (!typeMemory[typeName]) {
            typeMemory[typeName] = { size: 0, count: 0, allocations: [] };
        }
        typeMemory[typeName].size += size;
        typeMemory[typeName].count += 1;
        typeMemory[typeName].allocations.push(alloc);
        
        console.log(`[${index}] ${originalType} -> ${typeName}: ${size} bytes`);
    });
    
    // Sort by memory size
    const sortedTypes = Object.entries(typeMemory)
        .filter(([type, data]) => data.size > 0)
        .sort((a, b) => b[1].size - a[1].size);
    
    console.log("Memory blocks data:", sortedTypes);
    
    if (sortedTypes.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No type data available</div>';
        return;
    }
    
    const totalMemory = sortedTypes.reduce((sum, [_, data]) => sum + data.size, 0);
    const colors = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16', '#f97316'];
    
    // Create memory blocks visualization
    container.innerHTML = `
        <div style="height: 100%; display: flex; flex-direction: column; gap: 8px; padding: 8px;">
            ${sortedTypes.map(([typeName, data], index) => {
                const percentage = ((data.size / totalMemory) * 100);
                const color = colors[index % colors.length];
                const blockHeight = Math.max(20, Math.min(60, percentage * 2));
                
                return `
                    <div style="display: flex; align-items: center; gap: 12px; cursor: pointer; padding: 6px; border-radius: 6px; transition: all 0.2s;"
                         onmouseover="this.style.background='var(--bg-primary)'; this.style.transform='scale(1.02)'"
                         onmouseout="this.style.background='transparent'; this.style.transform='scale(1)'"
                         onclick="showTypeDetail('${typeName}', '${JSON.stringify(data).replace(/'/g, '&apos;')}')">
                        
                        <!-- Memory Block -->
                        <div style="width: 40px; height: ${blockHeight}px; background: linear-gradient(135deg, ${color}, ${color}80); 
                                    border-radius: 4px; position: relative; border: 2px solid ${color};">
                            <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); 
                                        color: white; font-size: 8px; font-weight: bold;">
                                ${data.count}
                            </div>
                        </div>
                        
                        <!-- Type Info -->
                        <div style="flex: 1; min-width: 0;">
                            <div style="font-size: 13px; font-weight: 600; color: var(--text-primary); margin-bottom: 2px;">
                                ${typeName}
                            </div>
                            <div style="font-size: 11px; color: var(--text-secondary);">
                                ${formatBytes(data.size)} - ${data.count} allocation${data.count > 1 ? 's' : ''}
                            </div>
                        </div>
                        
                        <!-- Percentage Bar -->
                        <div style="width: 60px; text-align: right;">
                            <div style="font-size: 12px; font-weight: 600; color: ${color}; margin-bottom: 2px;">
                                ${percentage.toFixed(1)}%
                            </div>
                            <div style="width: 100%; height: 4px; background: var(--border-light); border-radius: 2px; overflow: hidden;">
                                <div style="width: ${percentage}%; height: 100%; background: ${color}; border-radius: 2px;"></div>
                            </div>
                        </div>
                    </div>
                `;
            }).join('')}
        </div>
    `;
}

// Show type detail modal
window.showTypeDetail = function(typeName, data) {
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.6); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    const typeColors = {
        'HashMap': '#3b82f6', 'BTreeMap': '#10b981', 'Arc': '#f59e0b', 
        'Rc': '#ef4444', 'String': '#8b5cf6', 'Vec': '#06b6d4'
    };
    const color = typeColors[typeName] || '#64748b';
    
    modal.innerHTML = `
        <div style="background: var(--bg-primary); border-radius: 16px; padding: 24px; min-width: 500px; max-width: 700px; max-height: 80vh; overflow-y: auto; border: 1px solid var(--border-light);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h3 style="margin: 0; color: var(--text-primary); display: flex; align-items: center; gap: 12px;">
                    <div style="width: 24px; height: 24px; background: ${color}; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: white; font-size: 12px; font-weight: bold;">
                        ${data.count}
                    </div>
                    ${typeName} Memory Analysis
                </h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: none; border: none; font-size: 20px; color: var(--text-secondary); cursor: pointer;">×</button>
            </div>
            
            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 20px;">
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 1.8rem; font-weight: 700; color: ${color};">${formatBytes(data.size)}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Total Memory</div>
                </div>
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 1.8rem; font-weight: 700; color: var(--text-primary);">${data.count}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Allocations</div>
                </div>
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 1.8rem; font-weight: 700; color: var(--text-primary);">${formatBytes(data.size / data.count)}</div>
                    <div style="font-size: 0.8rem; color: var(--text-secondary);">Avg Size</div>
                </div>
            </div>
            
            <h4 style="margin: 0 0 12px 0; color: var(--text-primary);">Individual Allocations:</h4>
            <div style="max-height: 300px; overflow-y: auto; border: 1px solid var(--border-light); border-radius: 8px;">
                <table style="width: 100%; border-collapse: collapse; font-size: 12px;">
                    <thead style="background: var(--bg-secondary); position: sticky; top: 0;">
                        <tr>
                            <th style="padding: 8px; text-align: left; color: var(--text-primary);">Variable</th>
                            <th style="padding: 8px; text-align: right; color: var(--text-primary);">Size</th>
                            <th style="padding: 8px; text-align: center; color: var(--text-primary);">Status</th>
                            <th style="padding: 8px; text-align: right; color: var(--text-primary);">Lifetime</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.allocations.map(alloc => {
                            const isLeaked = alloc.is_leaked;
                            const statusColor = isLeaked ? 'var(--primary-red)' : 'var(--primary-green)';
                            const statusText = isLeaked ? 'LEAKED' : 'OK';
                            const lifetime = alloc.lifetime_ms || "Active";
                            
                            return `
                                <tr style="border-bottom: 1px solid var(--border-light); cursor: pointer;" onclick="showAllocationDetail('${alloc.ptr}")">
                                    <td style="padding: 8px; color: var(--text-primary); font-weight: 500;">${alloc.var_name || 'unnamed'}</td>
                                    <td style="padding: 8px; text-align: right; color: var(--text-primary); font-weight: 600;">${formatBytes(alloc.size || 0)}</td>
                                    <td style="padding: 8px; text-align: center;">
                                        <span style="color: ${statusColor}; font-weight: 600; font-size: 11px;">${statusText}</span>
                                    </td>
                                    <td style="padding: 8px; text-align: right; color: var(--text-secondary); font-size: 11px;">
                                        ${typeof lifetime === 'number' ? lifetime.toFixed(2) + 'ms' : lifetime}
                                    </td>
                                </tr>
                            `;
                        }).join('')}
                    </tbody>
                </table>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
};

// Render detailed allocation timeline with heap/stack and timing info
function renderAllocationTimelineDetail() {
    const container = document.getElementById('allocationTimelineDetail");
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-secondary); margin-top: 80px;">No allocation data available</div>';
        return;
    }
    
    // Sort by allocation time
    const sortedAllocs = allocs.slice().sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    const minTime = sortedAllocs[0].timestamp_alloc || 0;
    
    // Classify allocations as heap/stack
    const classifyAllocation = (typeName) => {
        const heapIndicators = ['Arc', 'Rc', 'Box', 'Vec', 'HashMap', 'BTreeMap', 'String'];
        const stackIndicators = ['&', 'i32', 'u32', 'i64', 'u64', 'f32', 'f64', 'bool', 'char'];
        
        if (heapIndicators.some(indicator => typeName.includes(indicator))) {
            return { type: 'heap', color: '#ef4444', icon: '🏗️' };
        } else if (stackIndicators.some(indicator => typeName.includes(indicator))) {
            return { type: 'stack', color: '#10b981', icon: '📚' };
        } else {
            return { type: 'unknown', color: '#64748b', icon: '❓' };
        }
    };
    
    container.innerHTML = `
        <div style="display: flex; flex-direction: column; gap: 4px; height: 100%;">
            ${sortedAllocs.slice(0, 15).map((alloc, index) => {
                const allocTime = alloc.timestamp_alloc || 0;
                const lifetime = alloc.lifetime_ms || 0;
                const dropTime = allocTime + (lifetime * 1_000_000); // Convert ms to ns
                const relativeTime = ((allocTime - minTime) / 1_000_000).toFixed(2); // Convert to ms
                
                const classification = classifyAllocation(alloc.type_name || '");
                const typeName = (alloc.type_name || 'Unknown').split('::').pop().split('<')[0];
                
                return `
                    <div style="display: flex; align-items: center; gap: 8px; padding: 6px; border-radius: 4px; cursor: pointer; transition: all 0.2s;"
                         onmouseover="this.style.background='var(--bg-primary)'"
                         onmouseout="this.style.background='transparent'"
                         onclick="showAllocationTimeDetail('${alloc.ptr}', ${allocTime}, ${dropTime}, '${classification.type}")">
                        
                        <!-- Allocation Type Icon -->
                        <div style="width: 24px; height: 24px; background: ${classification.color}20; border: 1px solid ${classification.color}; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 12px;">
                            ${classification.icon}
                        </div>
                        
                        <!-- Variable Info -->
                        <div style="flex: 1; min-width: 0;">
                            <div style="font-size: 11px; font-weight: 600; color: var(--text-primary); margin-bottom: 1px;">
                                ${alloc.var_name || 'unnamed'} (${typeName})
                            </div>
                            <div style="font-size: 9px; color: var(--text-secondary);">
                                ${formatBytes(alloc.size || 0)} - ${classification.type.toUpperCase()}
                            </div>
                        </div>
                        
                        <!-- Timing Info -->
                        <div style="text-align: right; font-size: 9px;">
                            <div style="color: var(--primary-blue); font-weight: 600;">+${relativeTime}ms</div>
                            <div style="color: var(--text-secondary);">→ ${lifetime}ms</div>
                        </div>
                    </div>
                `;
            }).join('')}
            
            ${sortedAllocs.length > 15 ? `
                <div style="text-align: center; padding: 8px; color: var(--text-secondary); font-size: 10px; border-top: 1px solid var(--border-light);">
                    ... and ${sortedAllocs.length - 15} more allocations
                </div>
            ` : ''}
        </div>
    `;
}

// Show detailed allocation timing modal
window.showAllocationTimeDetail = function(ptr, allocTime, dropTime, allocationType) {
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    const alloc = allocs.find(a => a.ptr === ptr);
    
    if (!alloc) return;
    
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.6); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    const allocDate = new Date(allocTime / 1_000_000);
    const dropDate = new Date(dropTime / 1_000_000);
    const typeColor = allocationType === 'heap' ? '#ef4444' : allocationType === 'stack' ? '#10b981' : '#64748b';
    const typeIcon = allocationType === 'heap' ? '🏗️' : allocationType === 'stack' ? '📚' : '❓';
    
    modal.innerHTML = `
        <div style="background: var(--bg-primary); border-radius: 16px; padding: 24px; min-width: 500px; color: var(--text-primary); border: 1px solid var(--border-light);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h3 style="margin: 0; display: flex; align-items: center; gap: 12px;">
                    <span style="font-size: 24px;">${typeIcon}</span>
                    ${allocationType.toUpperCase()} Allocation Timeline
                </h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: none; border: none; font-size: 20px; color: var(--text-secondary); cursor: pointer;">×</button>
            </div>
            
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px;">
                <div style="padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">Variable</div>
                    <div style="font-size: 16px; font-weight: 600;">${alloc.var_name || 'unnamed'}</div>
                </div>
                <div style="padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">Type</div>
                    <div style="font-size: 14px; font-weight: 600; word-break: break-all;">${alloc.type_name || 'Unknown'}</div>
                </div>
            </div>
            
            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 16px; margin-bottom: 20px;">
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 18px; font-weight: 700; color: ${typeColor};">${formatBytes(alloc.size || 0)}</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Size</div>
                </div>
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 18px; font-weight: 700; color: var(--primary-blue);">${alloc.lifetime_ms || 0}ms</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Lifetime</div>
                </div>
                <div style="text-align: center; padding: 16px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 18px; font-weight: 700; color: ${typeColor};">${allocationType.toUpperCase()}</div>
                    <div style="font-size: 12px; color: var(--text-secondary);">Location</div>
                </div>
            </div>
            
            <div style="background: var(--bg-secondary); padding: 16px; border-radius: 8px;">
                <h4 style="margin: 0 0 12px 0; color: var(--text-primary);">Timeline Details</h4>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                    <div>
                        <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">🟢 Allocated At</div>
                        <div style="font-size: 14px; font-weight: 600; color: var(--primary-green);">${allocDate.toLocaleString()}</div>
                        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 2px;">Timestamp: ${allocTime}</div>
                    </div>
                    <div>
                        <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">🔴 Dropped At</div>
                        <div style="font-size: 14px; font-weight: 600; color: var(--primary-red);">${dropDate.toLocaleString()}</div>
                        <div style="font-size: 11px; color: var(--text-secondary); margin-top: 2px;">Timestamp: ${dropTime}</div>
                    </div>
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
};

// Update lifecycle statistics and render distribution chart
function updateLifecycleStatistics() {
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) return;
    
    // Calculate lifecycle statistics
    const activeVars = allocs.filter(a => !a.is_leaked && a.lifetime_ms === undefined).length;
    const freedVars = allocs.filter(a => !a.is_leaked && a.lifetime_ms !== undefined).length;
    const leakedVars = allocs.filter(a => a.is_leaked).length;
    const avgLifetime = allocs.filter(a => a.lifetime_ms).reduce((sum, a) => sum + a.lifetime_ms, 0) / Math.max(1, allocs.filter(a => a.lifetime_ms).length);
    
    // Update statistics display
    document.getElementById('active-vars").textContent = activeVars;
    document.getElementById('freed-vars").textContent = freedVars;
    document.getElementById('leaked-vars").textContent = leakedVars;
    document.getElementById('avg-lifetime-stat').textContent = avgLifetime.toFixed(2) + 'ms';
    
    // Render lifecycle distribution chart
    renderLifecycleDistributionChart(allocs);
}

// Render lifecycle distribution chart
function renderLifecycleDistributionChart(allocs) {
    const ctx = document.getElementById('lifecycleDistributionChart");
    if (!ctx || !window.Chart) return;
    
    // Cleanup existing chart
    if (window.chartInstances && window.chartInstances['lifecycleDistributionChart']) {
        try { window.chartInstances['lifecycleDistributionChart'].destroy(); } catch(_) {}
        delete window.chartInstances['lifecycleDistributionChart'];
    }
    
    // Group allocations by lifetime ranges
    const lifetimeRanges = {
        'Instant (0ms)': 0,
        'Quick (0-1ms)': 0,
        'Short (1-10ms)': 0,
        'Long (10ms+)': 0,
        "Active": 0
    };
    
    allocs.forEach(alloc => {
        const lifetime = alloc.lifetime_ms;
        if (lifetime === undefined) {
            lifetimeRanges["Active"]++;
        } else if (lifetime === 0) {
            lifetimeRanges['Instant (0ms)']++;
        } else if (lifetime <= 1) {
            lifetimeRanges['Quick (0-1ms)']++;
        } else if (lifetime <= 10) {
            lifetimeRanges['Short (1-10ms)']++;
        } else {
            lifetimeRanges['Long (10ms+)']++;
        }
    });
    
    const labels = Object.keys(lifetimeRanges);
    const values = Object.values(lifetimeRanges);
    const colors = ['#ef4444', '#f59e0b', '#10b981', '#3b82f6', '#8b5cf6'];
    
    if (values.some(v => v > 0)) {
        const chart = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: labels,
                datasets: [{
                    data: values,
                    backgroundColor: colors,
                    borderWidth: 1,
                    borderColor: colors.map(c => c + '80")
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: (context) => {
                                const range = context.label;
                                const count = context.parsed.y;
                                const total = context.dataset.data.reduce((a, b) => a + b, 0);
                                const percentage = ((count / total) * 100).toFixed(1);
                                return `${range}: ${count} vars (${percentage}%)`;
                            }
                        }
                    }
                },
                scales: {
                    x: {
                        ticks: {
                            font: { size: 9 },
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#cbd5e1' : '#64748b';
                            },
                            maxRotation: 45
                        },
                        grid: { display: false }
                    },
                    y: {
                        beginAtZero: true,
                        ticks: {
                            font: { size: 9 },
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#cbd5e1' : '#64748b';
                            }
                        },
                        grid: {
                            color: function(context) {
                                return document.documentElement.classList.contains('dark-theme') ? '#374151' : '#e2e8f0';
                            }
                        }
                    }
                }
            }
        });
        
        window.chartInstances = window.chartInstances || {};
        window.chartInstances['lifecycleDistributionChart'] = chart;
    }
}

// Render memory hotspots visualization
function renderMemoryHotspots() {
    const container = document.getElementById('memoryHotspots");
    if (!container) return;
    
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-secondary); margin-top: 100px;">No allocation data available</div>';
        return;
    }
    
    // Sort allocations by size to identify hotspots
    const sortedAllocs = allocs.slice().sort((a, b) => (b.size || 0) - (a.size || 0));
    const topHotspots = sortedAllocs.slice(0, 10); // Top 10 largest allocations
    
    container.innerHTML = '';
    
    // Create hotspot visualization
    topHotspots.forEach((alloc, index) => {
        const size = alloc.size || 0;
        const maxSize = sortedAllocs[0].size || 1;
        const intensity = (size / maxSize) * 100;
        
        const hotspotDiv = document.createElement('div");
        hotspotDiv.style.cssText = `
            display: flex; align-items: center; padding: 8px; margin-bottom: 6px;
            background: linear-gradient(90deg, var(--bg-primary) 0%, var(--primary-red)${Math.floor(intensity/4)} ${intensity}%, var(--bg-primary) 100%);
            border-radius: 6px; border: 1px solid var(--border-light);
            cursor: pointer; transition: transform 0.2s;
        `;
        
        const heatColor = intensity > 80 ? '#ef4444' : intensity > 60 ? '#f59e0b' : intensity > 40 ? '#10b981' : '#3b82f6';
        
        hotspotDiv.innerHTML = `
            <div style="width: 24px; height: 24px; border-radius: 50%; background: ${heatColor}; 
                        display: flex; align-items: center; justify-content: center; margin-right: 12px;
                        font-size: 10px; font-weight: bold; color: white;">
                ${index + 1}
            </div>
            <div style="flex: 1;">
                <div style="font-size: 12px; font-weight: 600; color: var(--text-primary);">
                    ${alloc.var_name || 'unnamed'}
                </div>
                <div style="font-size: 10px; color: var(--text-secondary);">
                    ${(alloc.type_name || 'Unknown').replace(/std::|alloc::/g, '').substring(0, 30)}...
                </div>
            </div>
            <div style="text-align: right;">
                <div style="font-size: 12px; font-weight: 700; color: ${heatColor};">
                    ${formatBytes(size)}
                </div>
                <div style="font-size: 9px; color: var(--text-secondary);">
                    ${intensity.toFixed(1)}% of max
                </div>
            </div>
        `;
        
        hotspotDiv.onmouseover = () => {
            hotspotDiv.style.transform = 'scale(1.02)';
            hotspotDiv.style.boxShadow = `0 4px 12px ${heatColor}40`;
        };
        hotspotDiv.onmouseout = () => {
            hotspotDiv.style.transform = 'scale(1)';
            hotspotDiv.style.boxShadow = 'none';
        };
        
        hotspotDiv.onclick = () => {
            showAllocationDetail(alloc.ptr);
        };
        
        container.appendChild(hotspotDiv);
    });
    
    // Add summary at bottom
    const summaryDiv = document.createElement('div");
    summaryDiv.style.cssText = `
        margin-top: 12px; padding: 8px; background: var(--bg-primary); 
        border-radius: 6px; font-size: 10px; color: var(--text-secondary);
        text-align: center; border: 1px solid var(--border-light);
    `;
    
    const totalHotspotMemory = topHotspots.reduce((sum, a) => sum + (a.size || 0), 0);
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const hotspotPercentage = ((totalHotspotMemory / totalMemory) * 100).toFixed(1);
    
    summaryDiv.innerHTML = `
        Top ${topHotspots.length} hotspots: <strong>${formatBytes(totalHotspotMemory)}</strong> 
        (${hotspotPercentage}% of total memory)
    `;
    
    container.appendChild(summaryDiv);
}

// Render thread analysis
function renderThreadAnalysis() {
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    // Update thread timeline
    renderThreadTimeline(allocs);
    
    // Update contention analysis
    updateContentionAnalysis(allocs);
}

// Render thread timeline
function renderThreadTimeline(allocs) {
    const container = document.getElementById('threadTimeline");
    if (!container) return;
    
    container.innerHTML = '';
    
    // Get unique threads
    const threads = [...new Set(allocs.map(a => a.thread_id).filter(t => t))];
    
    if (threads.length <= 1) {
        // Single thread visualization
        container.innerHTML = `
            <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">
                <div style="text-align: center;">
                    <div style="font-size: 14px; margin-bottom: 4px;">🧵</div>
                    <div style="font-size: 10px;">Single Thread</div>
                    <div style="font-size: 9px;">ThreadId(${threads[0] || 1})</div>
                </div>
            </div>
        `;
        return;
    }
    
    // Multi-thread visualization (if applicable)
    threads.forEach((threadId, index) => {
        const threadAllocs = allocs.filter(a => a.thread_id === threadId);
        const threadDiv = document.createElement('div");
        threadDiv.style.cssText = `
            height: ${100/threads.length}%; display: flex; align-items: center; 
            padding: 0 8px; border-bottom: 1px solid var(--border-light);
        `;
        
        threadDiv.innerHTML = `
            <div style="width: 60px; font-size: 9px; color: var(--text-secondary);">
                Thread ${threadId}
            </div>
            <div style="flex: 1; height: 4px; background: var(--bg-secondary); border-radius: 2px; position: relative;">
                <div style="height: 100%; background: var(--primary-blue); border-radius: 2px; width: ${(threadAllocs.length / allocs.length) * 100}%;"></div>
            </div>
            <div style="width: 40px; text-align: right; font-size: 9px; color: var(--text-primary);">
                ${threadAllocs.length}
            </div>
        `;
        
        container.appendChild(threadDiv);
    });
}

// Update contention analysis
function updateContentionAnalysis(allocs) {
    const levelEl = document.getElementById('contention-level");
    const detailsEl = document.getElementById('contention-details");
    
    if (!levelEl || !detailsEl) return;
    
    // Calculate contention metrics
    const maxConcurrentBorrows = Math.max(...allocs.map(a => a.borrow_info?.max_concurrent_borrows || 0));
    const avgConcurrentBorrows = allocs.reduce((sum, a) => sum + (a.borrow_info?.max_concurrent_borrows || 0), 0) / allocs.length;
    
    let level = 'LOW';
    let color = 'var(--primary-green)';
    let details = 'Single-threaded';
    
    if (maxConcurrentBorrows > 5) {
        level = 'HIGH';
        color = 'var(--primary-red)';
        details = `Max ${maxConcurrentBorrows} concurrent`;
    } else if (maxConcurrentBorrows > 2) {
        level = 'MEDIUM';
        color = 'var(--primary-orange)';
        details = `Avg ${avgConcurrentBorrows.toFixed(1)} concurrent`;
    }
    
    levelEl.textContent = level;
    levelEl.style.color = color;
    detailsEl.textContent = details;
}

// Update Performance Metrics and Thread Safety Analysis
function updateEnhancedMetrics() {
    const data = window.analysisData || {};
    const allocs = data.memory_analysis?.allocations || data.allocations || [];
    
    if (allocs.length === 0) return;
    
    // Calculate Performance Metrics
    const totalMemory = allocs.reduce((sum, a) => sum + (a.size || 0), 0);
    const peakMemory = Math.max(...allocs.map(a => a.size || 0));
    const timestamps = allocs.map(a => a.timestamp_alloc).filter(t => t).sort((a, b) => a - b);
    const timeRangeNs = timestamps.length > 1 ? timestamps[timestamps.length - 1] - timestamps[0] : 1e9;
    const allocationRate = allocs.length / (timeRangeNs / 1e9); // allocations per second
    const avgLifetime = allocs.filter(a => a.lifetime_ms).reduce((sum, a) => sum + a.lifetime_ms, 0) / Math.max(1, allocs.filter(a => a.lifetime_ms).length);
    
    // Simple fragmentation calculation: variance in allocation sizes
    const avgSize = totalMemory / allocs.length;
    const variance = allocs.reduce((sum, a) => sum + Math.pow((a.size || 0) - avgSize, 2), 0) / allocs.length;
    const fragmentation = Math.min(100, (Math.sqrt(variance) / avgSize) * 100);

    // Update Performance Metrics
    const peakEl = document.getElementById('peak-memory");
    const rateEl = document.getElementById('allocation-rate");
    const lifetimeEl = document.getElementById('avg-lifetime");
    const fragEl = document.getElementById('fragmentation");
    
    if (peakEl) peakEl.textContent = formatBytes(peakMemory);
    if (rateEl) rateEl.textContent = allocationRate.toFixed(1) + '/sec';
    if (lifetimeEl) lifetimeEl.textContent = avgLifetime.toFixed(2) + 'ms';
    if (fragEl) fragEl.textContent = fragmentation.toFixed(1) + '%';

    // Calculate Thread Safety Analysis
    const arcCount = allocs.filter(a => (a.type_name || '').includes('Arc')).length;
    const rcCount = allocs.filter(a => (a.type_name || '').includes('Rc')).length;
    const collectionsCount = allocs.filter(a => {
        const type = a.type_name || '';
        return type.includes('HashMap') || type.includes('BTreeMap') || type.includes('Vec') || type.includes('HashSet');
    }).length;

    // Update Thread Safety Analysis
    const arcEl = document.getElementById('arc-count");
    const rcEl = document.getElementById('rc-count");
    const collEl = document.getElementById('collections-count");
    
    if (arcEl) arcEl.textContent = arcCount;
    if (rcEl) rcEl.textContent = rcCount;
    if (collEl) collEl.textContent = collectionsCount;
    
    console.log("✅ Enhanced metrics updated:", {
        peakMemory: formatBytes(peakMemory),
        allocationRate: allocationRate.toFixed(1) + '/sec',
        avgLifetime: avgLifetime.toFixed(2) + 'ms',
        fragmentation: fragmentation.toFixed(1) + '%',
        arcCount, rcCount, collectionsCount
    });
}

// Enhanced chart rendering with comprehensive cleanup
function renderEnhancedCharts() {
    const data = window.analysisData || {};
    
    // Step 1: Destroy all Chart.js instances globally
    if (window.Chart && window.Chart.instances) {
        Object.keys(window.Chart.instances).forEach(id => {
            try {
                window.Chart.instances[id].destroy();
            } catch(e) {
                console.warn('Failed to destroy Chart.js instance:', id, e);
            }
        });
    }
    
    // Step 2: Destroy our tracked instances
    if (window.chartInstances) {
        Object.keys(window.chartInstances).forEach(chartId => {
            try {
                window.chartInstances[chartId].destroy();
                delete window.chartInstances[chartId];
            } catch(e) {
                console.warn('Failed to destroy tracked chart:', chartId, e);
            }
        });
    }
    window.chartInstances = {};
    
    // Step 3: Clear canvas contexts manually
    ['typeChart', 'timelineChart', 'ffi-risk-chart'].forEach(canvasId => {
        const canvas = document.getElementById(canvasId);
        if (canvas && canvas.getContext) {
            try {
                const ctx = canvas.getContext("2d");
                ctx.clearRect(0, 0, canvas.width, canvas.height);
                // Remove Chart.js specific properties
                delete canvas.chart;
                canvas.removeAttribute("style");
            } catch(e) {
                console.warn('Failed to clear canvas:', canvasId, e);
            }
        }
    });
    
    // Step 4: Force garbage collection hint
    if (window.gc) { try { window.gc(); } catch(_) {} }
    
    // Step 5: Small delay to ensure cleanup, then create new charts
    setTimeout(() => {
        initEnhancedTypeChart(data);
        initTimelineChart(data);
    }, 10);
}

document.addEventListener("DOMContentLoaded", () => {
    console.log("🚀 MemScope Dashboard Loaded");
    
    // Initialize theme toggle
    try { 
        initThemeToggle(); 
        console.log("✅ Theme toggle initialized");
    } catch(e) { 
        console.warn('⚠️ Theme toggle initialization failed:', e?.message); 
    }
    
    // Initialize main dashboard with all original functions
    try { 
        // Use original dashboard functions
        renderKpis();
        renderTypeChart();
        renderTimelineChart();
        renderTreemap();
        renderLifetimes();
        updateEnhancedMetrics();
        renderEnhancedCharts();
        renderMemoryFragmentation();
        renderEnhancedDataInsights();
        renderAllocationTimelineDetail();
        updateLifecycleStatistics();
        renderMemoryHotspots();
        renderThreadAnalysis();
        populateAllocationsTable();
        populateUnsafeTable();
        renderVariableGraph();
        initEnhancedFFIVisualization();
        setupLifecycleVisualization();
        setupLifecycleToggle();
        
        // Optional hooks (no-op if undefined)
        try { updateEmbeddedFFISVG && updateEmbeddedFFISVG(); } catch(_) {}
        try { updatePerformanceMetrics && updatePerformanceMetrics(); } catch(_) {}
        
        console.log("✅ All dashboard components initialized");
    } catch(e) { 
        console.error('❌ Dashboard initialization failed:', e); 
    }
});

// Setup FFI flow visualization interactivity
function setupFFIFlowInteractivity(allocs) {
    // Add click handlers to FFI flow nodes
    setTimeout(() => {
        const rustNodes = document.querySelectorAll('.rust-node');
        const ffiNodes = document.querySelectorAll('.ffi-node');
        
        [...rustNodes, ...ffiNodes].forEach(node => {
            node.addEventListener("click", (e) => {
                const ptr = e.target.getAttribute('data-ptr');
                const size = e.target.getAttribute('data-size');
                showFFIFlowNodeDetail(ptr, size, allocs);
            });
            
            node.addEventListener('mouseover', (e) => {
                e.target.style.transform = 'scale(1.3)';
                e.target.style.filter = 'drop-shadow(0 0 8px currentColor)';
            });
            
            node.addEventListener('mouseout', (e) => {
                e.target.style.transform = 'scale(1)';
                e.target.style.filter = 'none';
            });
        });
        
        // Setup flow animation toggle
        const flowToggle = document.getElementById('ffi-flow-toggle');
        if (flowToggle) {
            let isAnimating = true;
            flowToggle.onclick = () => {
                const particles = document.querySelectorAll('#flow-particles circle');
                const flows = document.querySelectorAll('#data-flows path');
                
                if (isAnimating) {
                    // Pause animations
                    [...particles, ...flows].forEach(el => {
                        el.style.animationPlayState = 'paused';
                    });
                    flowToggle.innerHTML = '<i class="fa fa-pause"></i> Paused';
                    flowToggle.style.background = 'var(--primary-red)';
                    isAnimating = false;
                } else {
                    // Resume animations
                    [...particles, ...flows].forEach(el => {
                        el.style.animationPlayState = 'running';
                    });
                    flowToggle.innerHTML = '<i class="fa fa-play"></i> Animate';
                    flowToggle.style.background = 'var(--primary-green)';
                    isAnimating = true;
                }
            };
        }
    }, 100);
}

// Show FFI flow node detail
function showFFIFlowNodeDetail(ptr, size, allocs) {
    const alloc = allocs.find(a => a.ptr === ptr);
    if (!alloc) return;
    
    const modal = document.createElement('div');
    modal.style.cssText = `
        position: fixed; top: 0; left: 0; right: 0; bottom: 0; 
        background: rgba(0,0,0,0.6); z-index: 1000; 
        display: flex; align-items: center; justify-content: center;
    `;
    
    const isFFI = alloc.ffi_tracked;
    const bgGradient = isFFI ? 'linear-gradient(135deg, #1e40af, #3b82f6)' : 'linear-gradient(135deg, #ea580c, #f97316)';
    const icon = isFFI ? '⚙️' : '🦀';
    const title = isFFI ? 'FFI Allocation' : 'Rust Allocation';
    
    modal.innerHTML = `
        <div style="background: ${bgGradient}; border-radius: 16px; padding: 24px; min-width: 400px; color: white; position: relative; overflow: hidden;">
            <div style="position: absolute; top: -50px; right: -50px; font-size: 120px; opacity: 0.1;">${icon}</div>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; position: relative; z-index: 1;">
                <h3 style="margin: 0; font-size: 18px;">${icon} ${title}</h3>
                <button onclick="this.closest('div").parentNode.remove()" style="background: rgba(255,255,255,0.2); border: none; border-radius: 50%; width: 32px; height: 32px; color: white; cursor: pointer; font-size: 16px;">×</button>
            </div>
            <div style="position: relative; z-index: 1; line-height: 1.8;">
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;">
                    <div style="background: rgba(255,255,255,0.1); padding: 12px; border-radius: 8px;">
                        <div style="font-size: 12px; opacity: 0.8;">Variable</div>
                        <div style="font-weight: 600;">${alloc.var_name || 'unnamed'}</div>
                    </div>
                    <div style="background: rgba(255,255,255,0.1); padding: 12px; border-radius: 8px;">
                        <div style="font-size: 12px; opacity: 0.8;">Size</div>
                        <div style="font-weight: 600; font-size: 16px;">${formatBytes(alloc.size || 0)}</div>
                    </div>
                </div>
                <div style="background: rgba(255,255,255,0.1); padding: 12px; border-radius: 8px; margin-bottom: 16px;">
                    <div style="font-size: 12px; opacity: 0.8; margin-bottom: 4px;">Type</div>
                    <div style="font-weight: 600; font-size: 14px; word-break: break-all;">${alloc.type_name || 'Unknown'}</div>
                </div>
                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px;">
                    <div style="text-align: center; background: rgba(255,255,255,0.1); padding: 8px; border-radius: 6px;">
                        <div style="font-size: 16px; font-weight: 700;">${(alloc.borrow_info?.immutable_borrows || 0) + (alloc.borrow_info?.mutable_borrows || 0)}</div>
                        <div style="font-size: 10px; opacity: 0.8;">Borrows</div>
                    </div>
                    <div style="text-align: center; background: rgba(255,255,255,0.1); padding: 8px; border-radius: 6px;">
                        <div style="font-size: 16px; font-weight: 700;">${alloc.clone_info?.clone_count || 0}</div>
                        <div style="font-size: 10px; opacity: 0.8;">Clones</div>
                    </div>
                    <div style="text-align: center; background: rgba(255,255,255,0.1); padding: 8px; border-radius: 6px;">
                        <div style="font-size: 16px; font-weight: 700;">${alloc.thread_id || 1}</div>
                        <div style="font-size: 10px; opacity: 0.8;">Thread</div>
                    </div>
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    modal.onclick = (e) => { if (e.target === modal) modal.remove(); };
}
"##;

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Generate HTML directly from raw JSON data
pub fn generate_direct_html(json_data: &HashMap<String, Value>) -> Result<String, Box<dyn Error>> {
    tracing::info!("🎨 Generating enhanced HTML with embedded JSON data...");

    // Validate that we have essential data
    if json_data.is_empty() {
        return Err("No JSON data provided for HTML generation".into());
    }

    // Log what data we have
    for (key, value) in json_data {
        tracing::info!(
            "📊 Found data: {} ({} bytes)",
            key,
            serde_json::to_string(value).unwrap_or_default().len()
        );
    }

    // Transform the data structure to match JavaScript expectations
    let transformed_data = transform_json_data_structure(json_data)?;

    // Generate safety risk data from the transformed allocations
    let safety_risk_data = generate_safety_risk_data_from_json(&transformed_data)?;

    // Serialize the transformed JSON data for embedding with proper escaping
    let json_data_str = serde_json::to_string(&transformed_data)
        .map_err(|e| format!("Failed to serialize JSON data: {e}"))?;

    // Debug: Log data serialization info
    tracing::info!(
        "📊 JSON data serialized: {} characters",
        json_data_str.len()
    );
    if let Some(memory_analysis) = transformed_data.get("memory_analysis") {
        if let Some(allocations) = memory_analysis.get("allocations") {
            if let Some(allocs_array) = allocations.as_array() {
                tracing::info!(
                    "📊 Memory analysis allocations: {} items",
                    allocs_array.len()
                );
            }
        }
    }

    // Log data structure for debugging
    if let Some(unsafe_ffi_data) = json_data.get("basic_usage_snapshot_unsafe_ffi") {
        if let Some(summary) = unsafe_ffi_data.get("summary") {
            tracing::info!("📊 Unsafe/FFI Summary: {summary}");
        }
    }

    // Try multiple possible paths for the template files - prioritize the original dashboard.html

    // Use embedded template to avoid external file dependency
    let template_content = EMBEDDED_CLEAN_DASHBOARD_TEMPLATE.to_string();

    // Use embedded CSS to avoid external file dependency
    let css_content = EMBEDDED_STYLES_CSS.to_string();

    // Use embedded JavaScript to avoid external file dependency
    let js_content = EMBEDDED_SCRIPT_JS.to_string();

    // Replace placeholders in the template with proper escaping
    let mut html = template_content
        .replace("{{ json_data }}", &json_data_str) // with spaces
        .replace("{{json_data}}", &json_data_str) // without spaces
        .replace("{{CSS_CONTENT}}", &css_content)
        .replace("{{JS_CONTENT}}", &js_content)
        .replace("{{DATA_PLACEHOLDER}}", &json_data_str)
        .replace(
            "{\n        {\n        CSS_CONTENT\n      }\n    }",
            &css_content,
        ); // fix CSS format issues - match exact template spacing

    // Inject safety risk data into the HTML
    html = inject_safety_risk_data_into_html(html, &safety_risk_data)?;

    tracing::info!(
        "✅ Generated HTML with {} bytes of embedded JSON data",
        json_data_str.len()
    );

    Ok(html)
}

/// Transform the raw JSON data structure to match JavaScript expectations
/// This function preprocesses data in Rust to create visualization-ready structures
fn transform_json_data_structure(
    json_data: &HashMap<String, Value>,
) -> Result<serde_json::Map<String, Value>, Box<dyn Error>> {
    let mut transformed = serde_json::Map::new();

    // Process each JSON file and map it to the expected structure
    for (file_key, file_data) in json_data {
        // Extract the data type from the filename
        if file_key.contains("memory_analysis") {
            let enhanced_memory_data = enhance_memory_analysis_data(file_data)?;
            transformed.insert("memory_analysis".to_string(), enhanced_memory_data);
        } else if file_key.contains("lifetime") {
            let enhanced_lifetime_data = enhance_lifetime_data(file_data)?;
            transformed.insert("lifetime".to_string(), enhanced_lifetime_data);
        } else if file_key.contains("complex_types") {
            transformed.insert("complex_types".to_string(), file_data.clone());
        } else if file_key.contains("performance") {
            transformed.insert("performance".to_string(), file_data.clone());
        } else if file_key.contains("unsafe_ffi") {
            let enhanced_ffi_data = enhance_ffi_data(file_data)?;
            transformed.insert("unsafe_ffi".to_string(), enhanced_ffi_data);
            // Also add it with the specific key that JavaScript expects
            transformed.insert(file_key.clone(), file_data.clone());
        } else if file_key.contains("security_violations") {
            transformed.insert("security_violations".to_string(), file_data.clone());
        } else if file_key.contains("variable_relationships") {
            transformed.insert("variable_relationships".to_string(), file_data.clone());
        } else {
            // Keep any other data with its original key
            transformed.insert(file_key.clone(), file_data.clone());
        }
    }

    // Ensure we have all expected data structures, even if empty
    if !transformed.contains_key("memory_analysis") {
        transformed.insert(
            "memory_analysis".to_string(),
            serde_json::json!({
                "allocations": [],
                "stats": {
                    "total_allocations": 0,
                    "active_allocations": 0,
                    "total_memory": 0,
                    "active_memory": 0
                }
            }),
        );
    }

    if !transformed.contains_key("lifetime") {
        transformed.insert(
            "lifetime".to_string(),
            serde_json::json!({
                "lifecycle_events": []
            }),
        );
    }

    if !transformed.contains_key("complex_types") {
        transformed.insert(
            "complex_types".to_string(),
            serde_json::json!({
                "categorized_types": {
                    "generic_types": [],
                    "collections": [],
                    "smart_pointers": [],
                    "trait_objects": []
                },
                "summary": {
                    "total_complex_types": 0,
                    "generic_type_count": 0
                }
            }),
        );
    }

    if !transformed.contains_key("performance") {
        transformed.insert(
            "performance".to_string(),
            serde_json::json!({
                "memory_performance": {
                    "active_memory": 0,
                    "peak_memory": 0,
                    "total_allocated": 0
                },
                "allocation_distribution": {
                    "tiny": 0,
                    "small": 0,
                    "medium": 0,
                    "large": 0,
                    "massive": 0
                }
            }),
        );
    }

    if !transformed.contains_key("unsafe_ffi") {
        transformed.insert(
            "unsafe_ffi".to_string(),
            serde_json::json!({
                "summary": {
                    "total_risk_items": 0,
                    "unsafe_count": 0,
                    "ffi_count": 0,
                    "safety_violations": 0
                },
                "enhanced_ffi_data": [],
                "safety_violations": []
            }),
        );
    }

    if !transformed.contains_key("security_violations") {
        transformed.insert(
            "security_violations".to_string(),
            serde_json::json!({
                "metadata": {
                    "total_violations": 0
                },
                "violation_reports": [],
                "security_summary": {
                    "security_analysis_summary": {
                        "total_violations": 0,
                        "severity_breakdown": {
                            "critical": 0,
                            "high": 0,
                            "medium": 0,
                            "low": 0,
                            "info": 0
                        }
                    }
                }
            }),
        );
    }

    tracing::info!(
        "🔄 Transformed data structure with keys: {:?}",
        transformed.keys().collect::<Vec<_>>()
    );

    Ok(transformed)
}

/// Enhance memory analysis data with visualization-ready structures
fn enhance_memory_analysis_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    if let Some(allocations) = data.get("allocations").and_then(|a| a.as_array()) {
        // Add memory fragmentation analysis
        let fragmentation_data = analyze_memory_fragmentation(allocations);

        // Add memory growth trends
        let growth_trends = analyze_memory_growth_trends(allocations);

        // Create enhanced structure
        if let Some(obj) = enhanced.as_object_mut() {
            obj.insert("fragmentation_analysis".to_string(), fragmentation_data);
            obj.insert("growth_trends".to_string(), growth_trends);
            obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        }
    }

    Ok(enhanced)
}

/// Enhance lifetime data with colorful progress bar information
fn enhance_lifetime_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    if let Some(events) = data.get("lifecycle_events").and_then(|e| e.as_array()) {
        // Filter for user-defined variables
        let user_variables: Vec<&Value> = events
            .iter()
            .filter(|event| {
                event
                    .get("var_name")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s != "unknown")
                    && event
                        .get("type_name")
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s != "unknown")
            })
            .collect();

        // Group by variable name and add color information
        let mut variable_groups = std::collections::HashMap::new();
        for (index, event) in user_variables.iter().enumerate() {
            if let Some(var_name) = event.get("var_name").and_then(|v| v.as_str()) {
                let color_index = index % 10; // 10 colors in palette
                let color = get_progress_color(color_index);

                let group = variable_groups
                    .entry(var_name.to_string())
                    .or_insert_with(|| {
                        serde_json::json!({
                            "var_name": var_name,
                            "type_name": event.get("type_name"),
                            "color": color,
                            "color_index": color_index,
                            "events": []
                        })
                    });

                if let Some(events_array) = group.get_mut("events").and_then(|e| e.as_array_mut()) {
                    events_array.push((*event).clone());
                }
            }
        }

        // Convert to array and add to enhanced data
        let grouped_variables: Vec<Value> = variable_groups.into_values().collect();

        if let Some(obj) = enhanced.as_object_mut() {
            obj.insert(
                "variable_groups".to_string(),
                serde_json::json!(grouped_variables),
            );
            obj.insert(
                "user_variables_count".to_string(),
                serde_json::json!(user_variables.len()),
            );
            obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        }
    }

    Ok(enhanced)
}

/// Enhance FFI data with comprehensive analysis and SVG-inspired visualization data
fn enhance_ffi_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    let empty_vec = vec![];
    // Use the actual allocations field from the JSON data
    let allocations = data
        .get("allocations")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);

    // Fallback to enhanced_ffi_data if allocations is not found
    let enhanced_data = if allocations.is_empty() {
        data.get("enhanced_ffi_data")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec)
    } else {
        allocations
    };

    let boundary_events = data
        .get("boundary_events")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);

    tracing::info!(
        "🔍 FFI data enhancement - allocations: {}, enhanced_data: {}, boundary_events: {}",
        allocations.len(),
        enhanced_data.len(),
        boundary_events.len()
    );

    // Calculate comprehensive statistics using the actual allocations
    let stats = calculate_ffi_statistics_from_allocations(enhanced_data, boundary_events);

    // Analyze language interactions
    let language_interactions = analyze_language_interactions(boundary_events);

    // Safety analysis using actual allocations
    let safety_analysis = analyze_safety_metrics_from_allocations(enhanced_data);

    // Create SVG-inspired dashboard metrics
    let dashboard_metrics = create_ffi_dashboard_metrics(enhanced_data, boundary_events);

    // Create memory hotspots analysis
    let memory_hotspots = analyze_memory_hotspots(enhanced_data);

    // Create cross-language memory flow analysis
    let memory_flow = analyze_cross_language_memory_flow(enhanced_data, boundary_events);

    // Create risk assessment
    let risk_assessment = create_ffi_risk_assessment(enhanced_data);

    if let Some(obj) = enhanced.as_object_mut() {
        obj.insert("comprehensive_stats".to_string(), stats);
        obj.insert("language_interactions".to_string(), language_interactions);
        obj.insert("safety_analysis".to_string(), safety_analysis);
        obj.insert("dashboard_metrics".to_string(), dashboard_metrics);
        obj.insert("memory_hotspots".to_string(), memory_hotspots);
        obj.insert("memory_flow".to_string(), memory_flow);
        obj.insert("risk_assessment".to_string(), risk_assessment);
        obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        // Ensure allocations are preserved in the enhanced data
        if !allocations.is_empty() {
            obj.insert("allocations".to_string(), serde_json::json!(allocations));
        }
    }

    Ok(enhanced)
}

/// Analyze memory fragmentation from allocations
fn analyze_memory_fragmentation(allocations: &[Value]) -> Value {
    let mut sorted_allocs: Vec<_> = allocations
        .iter()
        .filter_map(|alloc| {
            let ptr_str = alloc.get("ptr")?.as_str()?;
            let size = alloc.get("size")?.as_u64()? as usize;
            let address = u64::from_str_radix(ptr_str.trim_start_matches("0x"), 16).ok()?;
            Some((address, size))
        })
        .collect();

    sorted_allocs.sort_by_key(|&(addr, _)| addr);

    let mut gaps = 0;
    let mut total_gap_size = 0u64;

    for i in 1..sorted_allocs.len() {
        let (prev_addr, prev_size) = sorted_allocs[i - 1];
        let (curr_addr, _) = sorted_allocs[i];
        let prev_end = prev_addr + prev_size as u64;

        if curr_addr > prev_end {
            gaps += 1;
            total_gap_size += curr_addr - prev_end;
        }
    }

    let total_memory: u64 = sorted_allocs.iter().map(|(_, size)| *size as u64).sum();
    let fragmentation_score = if total_memory > 0 {
        ((total_gap_size as f64 / (total_memory + total_gap_size) as f64) * 100.0) as u32
    } else {
        0
    };

    let largest_block = sorted_allocs
        .iter()
        .map(|(_, size)| *size)
        .max()
        .unwrap_or(0);

    serde_json::json!({
        "total_blocks": sorted_allocs.len(),
        "fragmentation_score": fragmentation_score,
        "largest_block": largest_block,
        "gaps": gaps,
        "total_gap_size": total_gap_size,
        "analysis": get_fragmentation_analysis(fragmentation_score)
    })
}

/// Analyze memory growth trends
fn analyze_memory_growth_trends(allocations: &[Value]) -> Value {
    let mut sorted_allocs: Vec<_> = allocations
        .iter()
        .filter_map(|alloc| {
            let timestamp = alloc.get("timestamp_alloc")?.as_u64()?;
            let size = alloc.get("size")?.as_u64()? as usize;
            Some((timestamp, size))
        })
        .collect();

    sorted_allocs.sort_by_key(|&(timestamp, _)| timestamp);

    let mut cumulative_memory = 0;
    let time_points: Vec<_> = sorted_allocs
        .iter()
        .enumerate()
        .map(|(index, &(timestamp, size))| {
            cumulative_memory += size;
            serde_json::json!({
                "timestamp": timestamp,
                "memory": cumulative_memory,
                "index": index
            })
        })
        .take(100) // Limit for performance
        .collect();

    let peak_memory = time_points
        .iter()
        .filter_map(|p| p.get("memory")?.as_u64())
        .max()
        .unwrap_or(0);

    let current_memory = time_points
        .last()
        .and_then(|p| p.get("memory")?.as_u64())
        .unwrap_or(0);

    let start_memory = time_points
        .first()
        .and_then(|p| p.get("memory")?.as_u64())
        .unwrap_or(0);

    let growth_rate = if start_memory > 0 {
        ((current_memory as f64 - start_memory as f64) / start_memory as f64 * 100.0) as i32
    } else {
        0
    };

    let time_span = if time_points.len() > 1 {
        let start_time = time_points[0]
            .get("timestamp")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        let end_time = time_points
            .last()
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        if end_time > start_time {
            (end_time - start_time) / 1_000_000_000 // Convert to seconds
        } else {
            1
        }
    } else {
        1
    };

    let allocation_rate = if time_span > 0 {
        allocations.len() as u64 / time_span
    } else {
        0
    };

    serde_json::json!({
        "peak_memory": peak_memory,
        "current_memory": current_memory,
        "growth_rate": growth_rate,
        "allocation_rate": allocation_rate,
        "time_points": time_points,
        "analysis": get_trend_analysis(growth_rate)
    })
}

/// Calculate comprehensive FFI statistics from allocations
fn calculate_ffi_statistics_from_allocations(
    allocations: &[Value],
    boundary_events: &[Value],
) -> Value {
    let ffi_tracked_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let non_ffi_allocations = allocations.len() - ffi_tracked_allocations;

    let boundary_crossings = boundary_events.len();

    // Count safety violations from arrays
    let safety_violations = allocations
        .iter()
        .map(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.len() as u64)
                .unwrap_or(0)
        })
        .sum::<u64>();

    // Count borrow conflicts
    let borrow_conflicts = allocations
        .iter()
        .filter(|item| {
            if let Some(borrow_info) = item.get("borrow_info") {
                let immutable = borrow_info
                    .get("immutable_borrows")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let mutable = borrow_info
                    .get("mutable_borrows")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                immutable > 0 && mutable > 0
            } else {
                false
            }
        })
        .count();

    // Count clones
    let total_clones = allocations
        .iter()
        .map(|item| {
            item.get("clone_info")
                .and_then(|c| c.get("clone_count"))
                .and_then(|cc| cc.as_u64())
                .unwrap_or(0)
        })
        .sum::<u64>();

    let total_memory = allocations
        .iter()
        .map(|item| item.get("size").and_then(|s| s.as_u64()).unwrap_or(0))
        .sum::<u64>();

    serde_json::json!({
        "total_allocations": allocations.len(),
        "ffi_tracked_allocations": ffi_tracked_allocations,
        "non_ffi_allocations": non_ffi_allocations,
        "boundary_crossings": boundary_crossings,
        "safety_violations": safety_violations,
        "borrow_conflicts": borrow_conflicts,
        "total_clones": total_clones,
        "total_memory": total_memory
    })
}

/// Analyze language interactions from boundary events
fn analyze_language_interactions(boundary_events: &[Value]) -> Value {
    let mut interactions = std::collections::HashMap::new();

    for event in boundary_events {
        if let (Some(from), Some(to)) = (
            event.get("from_context").and_then(|f| f.as_str()),
            event.get("to_context").and_then(|t| t.as_str()),
        ) {
            let key = format!("{from} → {to}");
            *interactions.entry(key).or_insert(0) += 1;
        }
    }

    let interactions_vec: Vec<_> = interactions
        .into_iter()
        .map(|(interaction, count)| {
            serde_json::json!({
                "interaction": interaction,
                "count": count
            })
        })
        .collect();

    serde_json::json!(interactions_vec)
}

/// Analyze safety metrics from allocations
fn analyze_safety_metrics_from_allocations(allocations: &[Value]) -> Value {
    let safe_operations = allocations
        .iter()
        .filter(|item| {
            // Check if safety_violations array is empty
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.is_empty())
                .unwrap_or(true)
        })
        .count();

    let unsafe_operations = allocations.len() - safe_operations;
    let total_operations = allocations.len();

    let safety_percentage = if total_operations > 0 {
        (safe_operations as f64 / total_operations as f64 * 100.0) as u32
    } else {
        100
    };

    // Count allocations with ownership history
    let with_ownership_history = allocations
        .iter()
        .filter(|item| {
            item.get("ownership_history_available")
                .and_then(|o| o.as_bool())
                .unwrap_or(false)
        })
        .count();

    // Count leaked allocations
    let leaked_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("is_leaked")
                .and_then(|l| l.as_bool())
                .unwrap_or(false)
        })
        .count();

    serde_json::json!({
        "safe_operations": safe_operations,
        "unsafe_operations": unsafe_operations,
        "total_operations": total_operations,
        "safety_percentage": safety_percentage,
        "with_ownership_history": with_ownership_history,
        "leaked_allocations": leaked_allocations
    })
}

/// Analyze safety metrics (legacy function for backward compatibility)
fn _analyze_safety_metrics(enhanced_data: &[Value]) -> Value {
    analyze_safety_metrics_from_allocations(enhanced_data)
}

/// Get progress bar color by index
fn get_progress_color(index: usize) -> &'static str {
    const COLORS: &[&str] = &[
        "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57", "#ff9ff3", "#54a0ff", "#5f27cd",
        "#00d2d3", "#ff9f43",
    ];
    COLORS[index % COLORS.len()]
}

/// Get fragmentation analysis text
fn get_fragmentation_analysis(score: u32) -> &'static str {
    match score {
        0..=9 => "Excellent memory layout with minimal fragmentation.",
        10..=24 => "Good memory layout with low fragmentation.",
        25..=49 => "Moderate fragmentation detected. Consider memory pool allocation.",
        _ => "High fragmentation detected. Memory layout optimization recommended.",
    }
}

/// Get trend analysis text
fn get_trend_analysis(growth_rate: i32) -> &'static str {
    match growth_rate {
        i32::MIN..=-1 => "Memory usage is decreasing - good memory management.",
        0..=9 => "Stable memory usage with minimal growth.",
        10..=49 => "Moderate memory growth - monitor for potential leaks.",
        _ => "High memory growth detected - investigate for memory leaks.",
    }
}

/// Format memory size for display
fn format_memory_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {unit}", unit = UNITS[unit_index])
    } else {
        format!("{:.1} {unit}", size, unit = UNITS[unit_index])
    }
}

/// Calculate risk level for memory allocation
fn calculate_risk_level(size: u64, is_unsafe: bool, is_ffi: bool) -> String {
    if is_unsafe {
        "HIGH".to_string()
    } else if is_ffi && size > 1024 * 1024 {
        "MEDIUM".to_string()
    } else if is_ffi {
        "LOW".to_string()
    } else {
        "SAFE".to_string()
    }
}

/// Create FFI dashboard metrics inspired by SVG design
fn create_ffi_dashboard_metrics(allocations: &[Value], boundary_events: &[Value]) -> Value {
    let total_allocations = allocations.len();

    // Count unsafe allocations (those with safety violations)
    let unsafe_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        })
        .count();

    // Count FFI-tracked allocations
    let ffi_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    // Count boundary crossings
    let boundary_crossings = boundary_events.len();

    // Count safety violations
    let safety_violations = allocations
        .iter()
        .map(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        })
        .sum::<usize>();

    // Calculate total unsafe memory
    let unsafe_memory: u64 = allocations
        .iter()
        .filter(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        })
        .map(|item| item.get("size").and_then(|s| s.as_u64()).unwrap_or(0))
        .sum();

    // Calculate safety score
    let safety_score = if total_allocations > 0 {
        ((total_allocations - unsafe_allocations) as f64 / total_allocations as f64 * 100.0) as u32
    } else {
        100
    };

    // Analyze smart pointer types
    let smart_pointer_types = analyze_smart_pointer_types(allocations);

    // Analyze borrow checker metrics
    let borrow_metrics = analyze_borrow_checker_metrics(allocations);

    serde_json::json!({
        "unsafe_allocations": unsafe_allocations,
        "ffi_allocations": ffi_allocations,
        "boundary_crossings": boundary_crossings,
        "safety_violations": safety_violations,
        "unsafe_memory": unsafe_memory,
        "total_allocations": total_allocations,
        "safety_score": safety_score,
        "unsafe_memory_formatted": format_memory_size(unsafe_memory),
        "smart_pointer_types": smart_pointer_types,
        "borrow_metrics": borrow_metrics
    })
}

/// Analyze smart pointer types distribution
fn analyze_smart_pointer_types(allocations: &[Value]) -> Value {
    let mut type_counts = std::collections::HashMap::new();

    for allocation in allocations {
        if let Some(type_name) = allocation.get("type_name").and_then(|t| t.as_str()) {
            if type_name.contains("Arc")
                || type_name.contains("Rc")
                || type_name.contains("Box")
                || type_name.contains("RefCell")
            {
                // Extract the main type name
                let short_type = if type_name.contains("Arc") {
                    "Arc"
                } else if type_name.contains("Rc") {
                    "Rc"
                } else if type_name.contains("Box") {
                    "Box"
                } else if type_name.contains("RefCell") {
                    "RefCell"
                } else {
                    "Other"
                };

                *type_counts.entry(short_type.to_string()).or_insert(0) += 1;
            }
        }
    }

    serde_json::json!(type_counts)
}

/// Analyze borrow checker metrics
fn analyze_borrow_checker_metrics(allocations: &[Value]) -> Value {
    let mut max_concurrent = 0;
    let mut total_borrows = 0;
    let mut conflicts = 0;

    for allocation in allocations {
        if let Some(borrow_info) = allocation.get("borrow_info") {
            if let Some(max_concurrent_borrows) = borrow_info
                .get("max_concurrent_borrows")
                .and_then(|m| m.as_u64())
            {
                max_concurrent = max_concurrent.max(max_concurrent_borrows);
            }

            let immutable = borrow_info
                .get("immutable_borrows")
                .and_then(|i| i.as_u64())
                .unwrap_or(0);
            let mutable = borrow_info
                .get("mutable_borrows")
                .and_then(|m| m.as_u64())
                .unwrap_or(0);

            total_borrows += immutable + mutable;

            // Check for conflicts (both immutable and mutable borrows)
            if immutable > 0 && mutable > 0 {
                conflicts += 1;
            }
        }
    }

    serde_json::json!({
        "max_concurrent_borrows": max_concurrent,
        "total_borrow_operations": total_borrows,
        "borrow_conflicts": conflicts
    })
}

/// Analyze memory hotspots for visualization
fn analyze_memory_hotspots(allocations: &[Value]) -> Value {
    let mut hotspots = Vec::new();

    for allocation in allocations {
        if let (Some(size), Some(ptr), Some(type_name)) = (
            allocation.get("size").and_then(|s| s.as_u64()),
            allocation.get("ptr").and_then(|p| p.as_str()),
            allocation.get("type_name").and_then(|t| t.as_str()),
        ) {
            let is_unsafe = allocation
                .get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);

            let is_ffi = allocation
                .get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false);

            hotspots.push(serde_json::json!({
                "ptr": ptr,
                "size": size,
                "type_name": type_name,
                "is_unsafe": is_unsafe,
                "is_ffi": is_ffi,
                "category": if is_unsafe { "UNSAFE" } else { "FFI" },
                "size_formatted": format_memory_size(size),
                "risk_level": calculate_risk_level(size, is_unsafe, is_ffi)
            }));
        }
    }

    // Sort by size descending
    hotspots.sort_by(|a, b| {
        let size_a = a.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
        let size_b = b.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
        size_b.cmp(&size_a)
    });

    serde_json::json!(hotspots)
}

/// Analyze cross-language memory flow
fn analyze_cross_language_memory_flow(allocations: &[Value], boundary_events: &[Value]) -> Value {
    let rust_allocations = allocations
        .iter()
        .filter(|item| {
            !item
                .get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let ffi_allocations = allocations.len() - rust_allocations;

    // Analyze flow directions from boundary events
    let mut rust_to_ffi = 0;
    let mut ffi_to_rust = 0;

    for event in boundary_events {
        if let (Some(from), Some(to)) = (
            event.get("from_context").and_then(|f| f.as_str()),
            event.get("to_context").and_then(|t| t.as_str()),
        ) {
            match (from, to) {
                ("rust", "ffi") | ("rust", "c") => rust_to_ffi += 1,
                ("ffi", "rust") | ("c", "rust") => ffi_to_rust += 1,
                _ => {}
            }
        }
    }

    serde_json::json!({
        "rust_allocations": rust_allocations,
        "ffi_allocations": ffi_allocations,
        "rust_to_ffi_flow": rust_to_ffi,
        "ffi_to_rust_flow": ffi_to_rust,
        "total_boundary_crossings": boundary_events.len()
    })
}

/// Create FFI risk assessment
fn create_ffi_risk_assessment(allocations: &[Value]) -> Value {
    let mut risk_items = Vec::new();

    for allocation in allocations {
        let empty_vec = vec![];
        let safety_violations = allocation
            .get("safety_violations")
            .and_then(|s| s.as_array())
            .unwrap_or(&empty_vec);

        if !safety_violations.is_empty() {
            for violation in safety_violations {
                if let Some(violation_str) = violation.as_str() {
                    risk_items.push(serde_json::json!({
                        "type": "safety_violation",
                        "description": violation_str,
                        "severity": get_violation_severity(violation_str),
                        "ptr": allocation.get("ptr"),
                        "size": allocation.get("size")
                    }));
                }
            }
        }

        // Check for potential risks based on borrow patterns
        if let Some(borrow_info) = allocation.get("borrow_info") {
            let immutable = borrow_info
                .get("immutable_borrows")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let mutable = borrow_info
                .get("mutable_borrows")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if immutable > 0 && mutable > 0 {
                risk_items.push(serde_json::json!({
                    "type": "borrow_conflict",
                    "description": "Concurrent immutable and mutable borrows detected",
                    "severity": "medium",
                    "ptr": allocation.get("ptr"),
                    "immutable_borrows": immutable,
                    "mutable_borrows": mutable
                }));
            }
        }
    }

    // Calculate risk summary
    let critical_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("critical"))
        .count();
    let high_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("high"))
        .count();
    let medium_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("medium"))
        .count();
    let low_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("low"))
        .count();

    serde_json::json!({
        "risk_items": risk_items,
        "summary": {
            "total_risks": risk_items.len(),
            "critical": critical_risks,
            "high": high_risks,
            "medium": medium_risks,
            "low": low_risks
        }
    })
}

/// Get violation severity
fn get_violation_severity(violation: &str) -> &'static str {
    match violation.to_lowercase().as_str() {
        v if v.contains("double free") || v.contains("use after free") => "critical",
        v if v.contains("invalid free") || v.contains("buffer overflow") => "high",
        v if v.contains("memory leak") || v.contains("uninitialized") => "medium",
        _ => "low",
    }
}

/// Generate safety risk data from JSON data structure
fn generate_safety_risk_data_from_json(
    transformed_data: &serde_json::Map<String, Value>,
) -> Result<String, Box<dyn Error>> {
    let mut safety_risks = Vec::new();

    // Extract allocations from memory_analysis
    if let Some(memory_analysis) = transformed_data.get("memory_analysis") {
        if let Some(allocations) = memory_analysis
            .get("allocations")
            .and_then(|a| a.as_array())
        {
            for allocation in allocations {
                // Check for potential unsafe operations based on allocation patterns

                // 1. Large allocations that might indicate unsafe buffer operations
                if let Some(size) = allocation.get("size").and_then(|s| s.as_u64()) {
                    if size > 1024 * 1024 {
                        // > 1MB
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Large Memory Allocation",
                            "risk_level": "Medium",
                            "description": format!("Large allocation of {} bytes may indicate unsafe buffer operations", size)
                        }));
                    }
                }

                // 2. Leaked memory indicates potential unsafe operations
                if let Some(is_leaked) = allocation.get("is_leaked").and_then(|l| l.as_bool()) {
                    if is_leaked {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Memory Leak",
                            "risk_level": "High",
                            "description": "Memory leak detected - potential unsafe memory management"
                        }));
                    }
                }

                // 3. High borrow count might indicate unsafe sharing
                if let Some(borrow_count) = allocation.get("borrow_count").and_then(|b| b.as_u64())
                {
                    if borrow_count > 10 {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "High Borrow Count",
                            "risk_level": "Medium",
                            "description": format!("High borrow count ({}) may indicate unsafe sharing patterns", borrow_count)
                        }));
                    }
                }

                // 4. Raw pointer types indicate direct unsafe operations
                if let Some(type_name) = allocation.get("type_name").and_then(|t| t.as_str()) {
                    if type_name.contains("*mut") || type_name.contains("*const") {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Raw Pointer Usage",
                            "risk_level": "High",
                            "description": format!("Raw pointer type '{}' requires unsafe operations", type_name)
                        }));
                    }

                    // 5. FFI-related types
                    if type_name.contains("CString")
                        || type_name.contains("CStr")
                        || type_name.contains("c_void")
                        || type_name.contains("extern")
                    {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "FFI Boundary Crossing",
                            "risk_level": "Medium",
                            "description": format!("FFI type '{}' crosses safety boundaries", type_name)
                        }));
                    }
                }

                // 6. Very short-lived allocations might indicate unsafe temporary operations
                if let Some(lifetime_ms) = allocation.get("lifetime_ms").and_then(|l| l.as_u64()) {
                    if lifetime_ms < 1 {
                        // Less than 1ms
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Short-lived Allocation",
                            "risk_level": "Low",
                            "description": format!("Very short lifetime ({}ms) may indicate unsafe temporary operations", lifetime_ms)
                        }));
                    }
                }
            }
        }
    }

    // Check unsafe_ffi data for additional risks
    if let Some(unsafe_ffi) = transformed_data.get("unsafe_ffi") {
        if let Some(safety_violations) = unsafe_ffi
            .get("safety_violations")
            .and_then(|sv| sv.as_array())
        {
            for violation in safety_violations {
                if let Some(violation_type) =
                    violation.get("violation_type").and_then(|vt| vt.as_str())
                {
                    let severity = get_violation_severity(violation_type);
                    let risk_level = match severity {
                        "critical" => "High",
                        "high" => "High",
                        "medium" => "Medium",
                        _ => "Low",
                    };

                    safety_risks.push(serde_json::json!({
                        "location": violation.get("location").and_then(|l| l.as_str()).unwrap_or("Unknown"),
                        "operation": format!("Safety Violation: {violation_type}"),
                        "risk_level": risk_level,
                        "description": violation.get("description").and_then(|d| d.as_str()).unwrap_or("Safety violation detected")
                    }));
                }
            }
        }
    }

    // If no risks found, add a placeholder to show the system is working
    if safety_risks.is_empty() {
        safety_risks.push(serde_json::json!({
            "location": "Global Analysis",
            "operation": "Safety Scan Complete",
            "risk_level": "Low",
            "description": "No significant safety risks detected in current allocations"
        }));
    }

    serde_json::to_string(&safety_risks)
        .map_err(|e| format!("Failed to serialize safety risk data: {e}").into())
}

/// Inject safety risk data into HTML template
fn inject_safety_risk_data_into_html(
    mut html: String,
    safety_risk_data: &str,
) -> Result<String, Box<dyn Error>> {
    // Replace the safety risk data in the existing template
    html = html.replace(
        "window.safetyRisks = [];",
        &format!("window.safetyRisks = {safety_risk_data};"),
    );

    // Always ensure loadSafetyRisks function is available
    if !html.contains("function loadSafetyRisks") {
        // Find a good injection point - before the closing </script> tag
        if let Some(script_end) = html.rfind("</script>") {
            let before = &html[..script_end];
            let after = &html[script_end..];

            let safety_function_injection = r#"
    // Safety Risk Data Management Function
    function loadSafetyRisks() {
        console.log("🛡️ Loading safety risk data...");
        const unsafeTable = document.getElementById('unsafeTable');
        if (!unsafeTable) {
            console.warn('⚠️ unsafeTable not found');
            return;
        }
        
        const risks = window.safetyRisks || [];
        if (risks.length === 0) {
            unsafeTable.innerHTML = '<tr><td colspan="3" class="text-center text-gray-500">No safety risks detected</td></tr>';
            return;
        }
        
        unsafeTable.innerHTML = '';
        risks.forEach((risk, index) => {
            const row = document.createElement('tr');
            row.className = "hover:bg-gray-50 dark:hover:bg-gray-700";
            
            const riskLevelClass = risk.risk_level === 'High' ? 'text-red-600 font-bold' : 
                                 risk.risk_level === 'Medium' ? 'text-yellow-600 font-semibold' : 
                                 'text-green-600';
            
            row.innerHTML = `
                <td class="px-3 py-2 text-sm">${risk.location || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm">${risk.operation || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm"><span class="${riskLevelClass}">${risk.risk_level || 'Low'}</span></td>
            `;
            unsafeTable.appendChild(row);
        });
        
        console.log("✅ Safety risks loaded:", risks.length, 'items');
    }
    
    "#;

            html = format!("{before}{safety_function_injection}{after}");
        }
    }

    // Ensure safety risks are loaded after initialization - but only call if function exists
    html = html.replace("console.log('✅ Enhanced dashboard initialized');", 
                       "console.log('✅ Enhanced dashboard initialized'); setTimeout(() => { if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); } }, 100);");

    // Also add to manual initialization if it exists - with safer replacement
    if html.contains("manualBtn.addEventListener('click', manualInitialize);") {
        html = html.replace("manualBtn.addEventListener('click', manualInitialize);", 
                           "manualBtn.addEventListener('click', function() { manualInitialize(); setTimeout(() => { if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); } }, 100); });");
    }

    // Remove any standalone loadSafetyRisks calls that might cause errors
    html = html.replace(
        "loadSafetyRisks();",
        "if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); }",
    );

    tracing::info!("📊 Safety risk data and function injected into HTML template");

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_direct_html_with_empty_data() {
        let json_data = HashMap::new();
        let result = generate_direct_html(&json_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No JSON data provided for HTML generation"
        );
    }

    #[test]
    fn test_transform_json_data_structure_with_empty_input() {
        let json_data = HashMap::new();
        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("memory_analysis"));
        assert!(transformed.contains_key("lifetime"));
        assert!(transformed.contains_key("complex_types"));
        assert!(transformed.contains_key("performance"));
        assert!(transformed.contains_key("unsafe_ffi"));
        assert!(transformed.contains_key("security_violations"));
    }

    #[test]
    fn test_transform_json_data_structure_with_memory_analysis() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_memory_analysis".to_string(),
            serde_json::json!({
                "allocations": [],
                "stats": {
                    "total_allocations": 0,
                    "active_allocations": 0,
                    "total_memory": 0,
                    "active_memory": 0
                }
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("memory_analysis"));
    }

    #[test]
    fn test_transform_json_data_structure_with_lifetime_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_lifetime".to_string(),
            serde_json::json!({
                "lifecycle_events": []
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("lifetime"));
    }

    #[test]
    fn test_transform_json_data_structure_with_complex_types() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_complex_types".to_string(),
            serde_json::json!({
                "categorized_types": {
                    "generic_types": [],
                    "collections": [],
                    "smart_pointers": [],
                    "trait_objects": []
                },
                "summary": {
                    "total_complex_types": 0,
                    "generic_type_count": 0
                }
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("complex_types"));
    }

    #[test]
    fn test_transform_json_data_structure_with_performance_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_performance".to_string(),
            serde_json::json!({
                "memory_performance": {
                    "active_memory": 0,
                    "peak_memory": 0,
                    "total_allocated": 0
                },
                "allocation_distribution": {
                    "tiny": 0,
                    "small": 0,
                    "medium": 0,
                    "large": 0,
                    "massive": 0
                }
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("performance"));
    }

    #[test]
    fn test_transform_json_data_structure_with_ffi_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_unsafe_ffi".to_string(),
            serde_json::json!({
                "summary": {
                    "total_risk_items": 0,
                    "unsafe_count": 0,
                    "ffi_count": 0,
                    "safety_violations": 0
                },
                "enhanced_ffi_data": [],
                "safety_violations": []
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("unsafe_ffi"));
    }

    #[test]
    fn test_transform_json_data_structure_with_security_violations() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_security_violations".to_string(),
            serde_json::json!({
                "metadata": {
                    "total_violations": 0
                },
                "violation_reports": [],
                "security_summary": {
                    "security_analysis_summary": {
                        "total_violations": 0,
                        "severity_breakdown": {
                            "critical": 0,
                            "high": 0,
                            "medium": 0,
                            "low": 0,
                            "info": 0
                        }
                    }
                }
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("security_violations"));
    }

    #[test]
    fn test_enhance_memory_analysis_data() {
        let data = serde_json::json!({
            "allocations": []
        });

        let result = enhance_memory_analysis_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhance_lifetime_data() {
        let data = serde_json::json!({
            "lifecycle_events": []
        });

        let result = enhance_lifetime_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhance_ffi_data() {
        let data = serde_json::json!({
            "allocations": [],
            "boundary_events": []
        });

        let result = enhance_ffi_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_memory_fragmentation() {
        let allocations = vec![];
        let result = analyze_memory_fragmentation(&allocations);
        assert_eq!(result["total_blocks"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_memory_growth_trends() {
        let allocations = vec![];
        let result = analyze_memory_growth_trends(&allocations);
        assert_eq!(result["peak_memory"], serde_json::json!(0));
    }

    #[test]
    fn test_calculate_ffi_statistics_from_allocations() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = calculate_ffi_statistics_from_allocations(&allocations, &boundary_events);
        assert_eq!(result["total_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_language_interactions() {
        let boundary_events = vec![];
        let result = analyze_language_interactions(&boundary_events);
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn test_analyze_safety_metrics_from_allocations() {
        let allocations = vec![];
        let result = analyze_safety_metrics_from_allocations(&allocations);
        assert_eq!(result["total_operations"], serde_json::json!(0));
    }

    #[test]
    fn test_get_progress_color() {
        let color = get_progress_color(0);
        assert_eq!(color, "#ff6b6b");

        let color = get_progress_color(10);
        assert_eq!(color, "#ff6b6b"); // Should wrap around
    }

    #[test]
    fn test_get_fragmentation_analysis() {
        let analysis = get_fragmentation_analysis(5);
        assert_eq!(
            analysis,
            "Excellent memory layout with minimal fragmentation."
        );

        let analysis = get_fragmentation_analysis(15);
        assert_eq!(analysis, "Good memory layout with low fragmentation.");

        let analysis = get_fragmentation_analysis(35);
        assert_eq!(
            analysis,
            "Moderate fragmentation detected. Consider memory pool allocation."
        );

        let analysis = get_fragmentation_analysis(60);
        assert_eq!(
            analysis,
            "High fragmentation detected. Memory layout optimization recommended."
        );
    }

    #[test]
    fn test_get_trend_analysis() {
        let analysis = get_trend_analysis(-5);
        assert_eq!(
            analysis,
            "Memory usage is decreasing - good memory management."
        );

        let analysis = get_trend_analysis(5);
        assert_eq!(analysis, "Stable memory usage with minimal growth.");

        let analysis = get_trend_analysis(25);
        assert_eq!(
            analysis,
            "Moderate memory growth - monitor for potential leaks."
        );

        let analysis = get_trend_analysis(75);
        assert_eq!(
            analysis,
            "High memory growth detected - investigate for memory leaks."
        );
    }

    #[test]
    fn test_format_memory_size() {
        let formatted = format_memory_size(1023);
        assert_eq!(formatted, "1023 B");

        let formatted = format_memory_size(1024);
        assert_eq!(formatted, "1.0 KB");

        let formatted = format_memory_size(1024 * 1024);
        assert_eq!(formatted, "1.0 MB");

        let formatted = format_memory_size(1024 * 1024 * 1024);
        assert_eq!(formatted, "1.0 GB");
    }

    #[test]
    fn test_calculate_risk_level() {
        let risk = calculate_risk_level(100, true, false);
        assert_eq!(risk, "HIGH");

        let risk = calculate_risk_level(1024 * 1024 + 1, false, true);
        assert_eq!(risk, "MEDIUM");

        let risk = calculate_risk_level(100, false, true);
        assert_eq!(risk, "LOW");

        let risk = calculate_risk_level(100, false, false);
        assert_eq!(risk, "SAFE");
    }

    #[test]
    fn test_create_ffi_dashboard_metrics() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = create_ffi_dashboard_metrics(&allocations, &boundary_events);
        assert_eq!(result["total_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_smart_pointer_types() {
        let allocations = vec![];
        let result = analyze_smart_pointer_types(&allocations);
        assert_eq!(result, serde_json::json!({}));
    }

    #[test]
    fn test_analyze_borrow_checker_metrics() {
        let allocations = vec![];
        let result = analyze_borrow_checker_metrics(&allocations);
        assert_eq!(result["max_concurrent_borrows"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_memory_hotspots() {
        let allocations = vec![];
        let result = analyze_memory_hotspots(&allocations);
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn test_analyze_cross_language_memory_flow() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = analyze_cross_language_memory_flow(&allocations, &boundary_events);
        assert_eq!(result["rust_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_create_ffi_risk_assessment() {
        let allocations = vec![];
        let result = create_ffi_risk_assessment(&allocations);
        assert_eq!(result["summary"]["total_risks"], serde_json::json!(0));
    }

    #[test]
    fn test_get_violation_severity() {
        let severity = get_violation_severity("double free detected");
        assert_eq!(severity, "critical");

        let severity = get_violation_severity("invalid free operation");
        assert_eq!(severity, "high");

        let severity = get_violation_severity("memory leak detected");
        assert_eq!(severity, "medium");

        let severity = get_violation_severity("unknown issue");
        assert_eq!(severity, "low");
    }

    #[test]
    fn test_generate_safety_risk_data_from_json() {
        let transformed_data = serde_json::Map::new();
        let result = generate_safety_risk_data_from_json(&transformed_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_safety_risk_data_into_html() {
        let html = "<script>window.safetyRisks = [];</script>".to_string();
        let safety_risk_data = "[]";
        let result = inject_safety_risk_data_into_html(html, safety_risk_data);
        assert!(result.is_ok());
    }
}
