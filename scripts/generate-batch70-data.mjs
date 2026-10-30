import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Edge Visibility - Dev (Eugene, Oregon - Dual EdgeVisibility.glb Models)
function generateEdgeVisibilityDev() {
  const model1Box = {
    id: 'edge-vis-model1-box',
    name: 'Primary Model Surface and Edge Volume',
    show: true,
    positionDegrees: [-123.07446, 44.05037, 10.0],
    shape: {
      type: 'box',
      dimensions: [20.0, 20.0, 20.0],
    },
    color: [1.0, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      edgeDisplayMode: 'SURFACES_AND_EDGES',
      modelIndex: 1,
    },
  };

  const model2Cylinder = {
    id: 'edge-vis-model2-cylinder',
    name: 'Secondary Offset Model Geometry Pillar',
    show: true,
    positionDegrees: [-123.07406, 44.05037, 10.0],
    shape: {
      type: 'cylinder',
      length: 20.0,
      topRadius: 10.0,
      bottomRadius: 10.0,
    },
    color: [0.1, 0.85, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      edgeDisplayMode: 'SURFACES_AND_EDGES',
      modelIndex: 2,
    },
  };

  const midpointProbe = {
    id: 'edge-vis-midpoint-probe',
    name: 'Dual Model Center Camera LookAt Benchmark',
    show: true,
    positionDegrees: [-123.07426, 44.05037, 10.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0],
    },
    color: [0.9, 0.8, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      cameraLookAtCenter: true,
    },
  };

  const arenaRect = {
    id: 'edge-vis-arena-rect',
    name: 'Edge Visibility Testing Arena Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.076, 44.048, -123.072, 44.053],
    },
    color: [0.2, 0.3, 0.4, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      extension: 'EXT_mesh_primitive_edge_visibility',
    },
  };

  const entities = [model1Box, model2Cylinder, midpointProbe, arenaRect];

  const variants = [
    {
      label: 'Baseline: Dual Models with Surfaces and Edges (Red & Cyan Wireframes)',
      entities: [
        model1Box,
        model2Cylinder,
        midpointProbe,
        arenaRect,
      ],
    },
    {
      label: 'Surfaces Only Mode (Solid Opaque Volumes)',
      entities: [
        { ...model1Box, color: [0.95, 0.15, 0.3, 0.95], outline: false },
        { ...model2Cylinder, color: [0.15, 0.75, 0.85, 0.95], outline: false },
        midpointProbe,
        arenaRect,
      ],
    },
    {
      label: 'Edges Only Mode (Pure Geometric Crease & Boundary Lines)',
      entities: [
        { ...model1Box, fill: false, outline: true, outlineColor: [1.0, 0.3, 0.3, 1.0] },
        { ...model2Cylinder, fill: false, outline: true, outlineColor: [0.2, 1.0, 1.0, 1.0] },
        midpointProbe,
        arenaRect,
      ],
    },
    {
      label: 'Independent Asymmetric Rendering (Model 1 Edges, Model 2 Surfaces)',
      entities: [
        { ...model1Box, fill: false, outline: true, outlineColor: [1.0, 0.5, 0.2, 1.0] },
        { ...model2Cylinder, fill: true, outline: false, color: [0.2, 0.9, 0.5, 0.95] },
        { ...midpointProbe, color: [0.8, 0.3, 0.9, 0.95] },
        arenaRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. FXAA (Fast Approximate Anti-Aliasing - NYC 3D Buildings)
function generateFxaa() {
  const buildingBox = {
    id: 'fxaa-building-box',
    name: 'NYC Facade Anti-Aliasing Benchmark Structure',
    show: true,
    positionDegrees: [-74.0060, 40.7128, 25.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 50.0],
    },
    color: [0.25, 0.7, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      stage: 'FXAA',
      subpixelAntiAliasing: true,
    },
  };

  const edgePillar = {
    id: 'fxaa-edge-detector-pillar',
    name: 'High-Frequency Edge Contrast Probe Pillar',
    show: true,
    positionDegrees: [-74.0052, 40.7135, 22.5],
    shape: {
      type: 'cylinder',
      length: 45.0,
      topRadius: 15.0,
      bottomRadius: 15.0,
    },
    color: [0.95, 0.55, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      edgeThreshold: 0.125,
      edgeThresholdMin: 0.05,
    },
  };

  const sampleSphere = {
    id: 'fxaa-sample-sphere',
    name: 'Subpixel Luminance Gradient Calibration Sphere',
    show: true,
    positionDegrees: [-74.0068, 40.7121, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.85, 0.25, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      postProcessStage: 'postProcessStages.fxaa',
    },
  };

  const viewportRect = {
    id: 'fxaa-viewport-rect',
    name: 'NYC Tileset Antialiased Viewport Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.010, 40.709, -74.002, 40.716],
    },
    color: [0.15, 0.25, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.65, 0.95, 0.8],
    outlineWidth: 1.0,
    properties: {
      tilesetIonAssetId: 75343,
      terrain: 'WorldTerrain',
    },
  };

  const entities = [buildingBox, edgePillar, sampleSphere, viewportRect];

  const variants = [
    {
      label: 'Baseline: Fast Approximate Anti-Aliasing Enabled (High-Quality Edges)',
      entities: [
        buildingBox,
        edgePillar,
        sampleSphere,
        viewportRect,
      ],
    },
    {
      label: 'FXAA Bypass Mode (Aliased Raw Pixel Edges)',
      entities: [
        { ...buildingBox, outlineColor: [1.0, 0.3, 0.3, 1.0], outlineWidth: 1.0 },
        edgePillar,
        sampleSphere,
        viewportRect,
      ],
    },
    {
      label: 'High-Contrast Architectural Silhouette Infill (Amber Glow)',
      entities: [
        buildingBox,
        { ...edgePillar, color: [0.95, 0.75, 0.1, 0.95], outlineColor: [1.0, 0.95, 0.5, 1.0] },
        sampleSphere,
        viewportRect,
      ],
    },
    {
      label: 'Urban Sky & Terrain Anti-Aliased Horizon (Emerald Outline)',
      entities: [
        buildingBox,
        edgePillar,
        { ...sampleSphere, color: [0.2, 0.85, 0.45, 0.95], outlineColor: [0.65, 1.0, 0.8, 1.0] },
        { ...viewportRect, color: [0.1, 0.3, 0.25, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Geometry and Appearances - Dev (US Southwest PerInstanceColorAppearance Showcase)
function generateGeometryAndAppearancesDev() {
  const starBox = {
    id: 'geom-app-star-box',
    name: 'Star Profile Extruded Geometry Anchor Box',
    show: true,
    positionDegrees: [-105.0, 20.0, 20000.0],
    shape: {
      type: 'box',
      dimensions: [30000.0, 30000.0, 40000.0],
    },
    color: [0.85, 0.3, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      appearance: 'PerInstanceColorAppearance',
      extrudedShape: 'star',
    },
  };

  const tubeCylinder = {
    id: 'geom-app-tube-cylinder',
    name: 'Tube Geometry PolylineVolume Extrusion Column',
    show: true,
    positionDegrees: [-104.5, 20.4, 25000.0],
    shape: {
      type: 'cylinder',
      length: 50000.0,
      topRadius: 18000.0,
      bottomRadius: 18000.0,
    },
    color: [0.2, 0.8, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      geometry: 'PolylineVolumeGeometry',
      tubeRadius: 40000.0,
    },
  };

  const coplanarSphere = {
    id: 'geom-app-coplanar-sphere',
    name: 'Flat Appearance Geometry Instance Sphere',
    show: true,
    positionDegrees: [-105.5, 19.6, 20000.0],
    shape: {
      type: 'ellipsoid',
      radii: [16000.0, 16000.0, 16000.0],
    },
    color: [0.3, 0.55, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      renderState: { lineWidth: 1.0 },
      flat: true,
    },
  };

  const showcaseRect = {
    id: 'geom-app-showcase-rect',
    name: 'Geometry and Appearances Test Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-106.0, 19.2, -104.0, 20.8],
    },
    color: [0.3, 0.2, 0.4, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.45, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      primitiveCount: 2,
      closedAppearance: true,
    },
  };

  const entities = [starBox, tubeCylinder, coplanarSphere, showcaseRect];

  const variants = [
    {
      label: 'Baseline: Per-Instance Color Appearances with Star & Tube Extrusions',
      entities: [
        starBox,
        tubeCylinder,
        coplanarSphere,
        showcaseRect,
      ],
    },
    {
      label: 'Translucent Appearance Override (Semi-Transparent Multi-Instance)',
      entities: [
        { ...starBox, color: [0.95, 0.4, 0.2, 0.5] },
        { ...tubeCylinder, color: [0.2, 0.85, 0.5, 0.5] },
        coplanarSphere,
        showcaseRect,
      ],
    },
    {
      label: 'Wireframe Line Width Diagnostic (Flat Color Appearance)',
      entities: [
        starBox,
        tubeCylinder,
        { ...coplanarSphere, fill: false, outline: true, outlineColor: [1.0, 0.85, 0.3, 1.0] },
        showcaseRect,
      ],
    },
    {
      label: 'High-Contrast Composite Extrusion Infill (Crimson & Amber Profiles)',
      entities: [
        { ...starBox, color: [0.9, 0.15, 0.35, 0.95] },
        tubeCylinder,
        coplanarSphere,
        { ...showcaseRect, color: [0.45, 0.3, 0.1, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Globe Interior (Earth Concentric Layers & Translucency Cutout)
function generateGlobeInterior() {
  const coreBox = {
    id: 'globe-interior-core-box',
    name: 'Inner Core Solid Geophysical Mass Model',
    show: true,
    positionDegrees: [-75.0, 25.0, 300000.0],
    shape: {
      type: 'box',
      dimensions: [400000.0, 400000.0, 600000.0],
    },
    color: [0.95, 0.85, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      layer: 'Inner Core',
      radiusMeters: 1250000,
    },
  };

  const axisCylinder = {
    id: 'globe-interior-axis-cylinder',
    name: 'Outer Core Convective Liquid Column Anchor',
    show: true,
    positionDegrees: [-73.5, 26.5, 400000.0],
    shape: {
      type: 'cylinder',
      length: 800000.0,
      topRadius: 180000.0,
      bottomRadius: 180000.0,
    },
    color: [0.95, 0.5, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      layer: 'Outer Core',
      radiusMeters: 3450000,
    },
  };

  const mantleSphere = {
    id: 'globe-interior-mantle-sphere',
    name: 'Silicate Mantle Geosphere Representation Probe',
    show: true,
    positionDegrees: [-76.5, 23.5, 350000.0],
    shape: {
      type: 'ellipsoid',
      radii: [250000.0, 250000.0, 250000.0],
    },
    color: [0.85, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      layer: 'Mantle',
      radiusMeters: 6300000,
    },
  };

  const windowRect = {
    id: 'globe-interior-window-rect',
    name: 'Translucency Cutout Inspection Window',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-80.0, 20.0, -70.0, 30.0],
    },
    color: [0.15, 0.35, 0.6, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      translucencyEnabled: true,
      frontFaceAlpha: 0.25,
    },
  };

  const entities = [coreBox, axisCylinder, mantleSphere, windowRect];

  const variants = [
    {
      label: 'Baseline: Inner Core Primary Focus (Radiant Yellow Core)',
      entities: [
        coreBox,
        axisCylinder,
        mantleSphere,
        windowRect,
      ],
    },
    {
      label: 'Outer Core Dynamo Shell Mode (Vibrant Orange Infill)',
      entities: [
        coreBox,
        { ...axisCylinder, color: [1.0, 0.6, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        mantleSphere,
        windowRect,
      ],
    },
    {
      label: 'Mantle Convection Layer Display (Crimson Subsurface)',
      entities: [
        coreBox,
        axisCylinder,
        { ...mantleSphere, color: [0.95, 0.25, 0.3, 0.95], outlineColor: [1.0, 0.65, 0.7, 1.0] },
        windowRect,
      ],
    },
    {
      label: 'Translucency Mask Rectangle Window (Cyan Cutout)',
      entities: [
        coreBox,
        axisCylinder,
        mantleSphere,
        { ...windowRect, color: [0.1, 0.8, 0.75, 0.35], outlineColor: [0.4, 1.0, 0.95, 0.9] },
      ],
    },
  ];

  return { entities, variants };
}

function writeRecipe(id, data) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(data.entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants: data.variants }, null, 2) + '\n');
  console.log(`Generated ${id}: ${data.entities.length} entities, ${data.variants.length} variants`);
}

writeRecipe('edge-visibility-dev', generateEdgeVisibilityDev());
writeRecipe('fxaa', generateFxaa());
writeRecipe('geometry-and-appearances-dev', generateGeometryAndAppearancesDev());
writeRecipe('globe-interior', generateGlobeInterior());
