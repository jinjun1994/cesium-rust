import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Classification Types (Chester County, Pennsylvania)
function generateClassificationTypes() {
  const buildingVolume = {
    id: 'class-types-volume-box',
    name: 'Classified Building Volume Prism',
    show: true,
    positionDegrees: [-75.5965, 40.0395, 30.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 60.0],
    },
    color: [0.9, 0.2, 0.2, 0.75],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'BOTH',
      facility: 'Chester Industrial Facility',
      status: 'active',
    },
  };

  const corridorPillar = {
    id: 'class-types-corridor-pillar',
    name: 'Right-of-Way Corridor Benchmark Pillar',
    show: true,
    positionDegrees: [-75.5985, 40.0405, 25.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.95, 0.85, 0.15, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'TERRAIN',
      corridor: 'Main Line Transit Alignment',
    },
  };

  const centroidSphere = {
    id: 'class-types-centroid-sphere',
    name: 'Classified Parcel Centroid Marker',
    show: true,
    positionDegrees: [-75.5945, 40.0380, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [22.0, 22.0, 22.0],
    },
    color: [0.2, 0.8, 0.9, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      node: 'Survey Control Mon 408',
    },
  };

  const boundaryRect = {
    id: 'class-types-boundary-rect',
    name: 'Classification Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.602, 40.035, -75.593, 40.042],
    },
    color: [0.25, 0.25, 0.6, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.6, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      zone: 'Chester County Special Planning District',
    },
  };

  const entities = [buildingVolume, corridorPillar, centroidSphere, boundaryRect];

  const variants = [
    {
      label: 'Baseline: Classify Both (Terrain & 3D Tiles)',
      entities: [
        buildingVolume,
        corridorPillar,
        centroidSphere,
        boundaryRect,
      ],
    },
    {
      label: 'Classify Terrain Only (Green & Earth Tone Highlight)',
      entities: [
        { ...buildingVolume, color: [0.2, 0.8, 0.4, 0.85], outlineColor: [0.6, 1.0, 0.8, 1.0] },
        corridorPillar,
        centroidSphere,
        boundaryRect,
      ],
    },
    {
      label: 'Classify 3D Tiles Only (Electric Cyan Mesh Infill)',
      entities: [
        buildingVolume,
        { ...corridorPillar, color: [0.1, 0.85, 0.95, 0.9], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        centroidSphere,
        boundaryRect,
      ],
    },
    {
      label: 'Textured Material Emulation (Checker Gold Pattern)',
      entities: [
        { ...buildingVolume, color: [0.95, 0.75, 0.1, 0.9], outlineColor: [1.0, 0.95, 0.4, 1.0] },
        { ...corridorPillar, color: [0.85, 0.65, 0.15, 0.9] },
        centroidSphere,
        boundaryRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Client-side Snapping - Dev (BIM Facility, Asset 4665927)
function generateClientSideSnappingDev() {
  const anchorPoint = {
    id: 'client-snap-anchor-point',
    name: 'First Snap Measurement Anchor Pin',
    show: true,
    positionDegrees: [-122.3845, 37.7882, 35.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [1.0, 0.9, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'VERTEX',
      state: 'MEASURING_START',
    },
  };

  const targetPoint = {
    id: 'client-snap-target-point',
    name: 'Second Snap Measurement Target Column',
    show: true,
    positionDegrees: [-122.3832, 37.7890, 42.0],
    shape: {
      type: 'cylinder',
      length: 45.0,
      topRadius: 15.0,
      bottomRadius: 15.0,
    },
    color: [1.0, 0.55, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'SURFACE_NORMAL',
      state: 'MEASURING_END',
    },
  };

  const edgeGuide = {
    id: 'client-snap-edge-guide',
    name: 'Structural Edge Snapping Guide Member',
    show: true,
    positionDegrees: [-122.3838, 37.7886, 38.0],
    shape: {
      type: 'box',
      dimensions: [70.0, 18.0, 18.0],
    },
    color: [0.3, 0.9, 0.3, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.8, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'EDGE_ONLY',
      edgeTolerancePixels: 6.0,
    },
  };

  const meshExtent = {
    id: 'client-snap-mesh-extent',
    name: 'Snapping Search Boundary Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.386, 37.787, -122.382, 37.790],
    },
    color: [0.2, 0.3, 0.5, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      assetId: 4665927,
      debugShowBoundingVolume: true,
    },
  };

  const entities = [anchorPoint, targetPoint, edgeGuide, meshExtent];

  const variants = [
    {
      label: 'Baseline: Edges Only Snapping (Yellow Snap Anchors)',
      entities: [
        anchorPoint,
        targetPoint,
        edgeGuide,
        meshExtent,
      ],
    },
    {
      label: 'Surfaces + Edges Snapping (Cyan Surface & Yellow Edges)',
      entities: [
        anchorPoint,
        { ...targetPoint, color: [0.15, 0.85, 0.95, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        edgeGuide,
        meshExtent,
      ],
    },
    {
      label: 'Surfaces Only Snapping (Orange Surface Projection)',
      entities: [
        anchorPoint,
        targetPoint,
        { ...edgeGuide, color: [0.95, 0.45, 0.1, 0.9], outlineColor: [1.0, 0.7, 0.3, 1.0] },
        meshExtent,
      ],
    },
    {
      label: 'Show Tile Bounds Active (Neon Green Bounding Extents)',
      entities: [
        anchorPoint,
        targetPoint,
        edgeGuide,
        { ...meshExtent, color: [0.25, 0.75, 0.35, 0.5], outlineColor: [0.4, 1.0, 0.5, 1.0] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Clipping Performance - Dev (Philadelphia Downtown Grid)
function generateClippingPerformanceDev() {
  const centerCylinder = {
    id: 'clip-perf-center-cylinder',
    name: 'Central Circular Excavation Clipping Zone',
    show: true,
    positionDegrees: [-75.1524, 39.9470, 30.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 28.0,
      bottomRadius: 28.0,
    },
    color: [0.95, 0.2, 0.3, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      numPoints: 16,
      spacing: 0.21,
      target: 'Terrain',
    },
  };

  const gridBox1 = {
    id: 'clip-perf-grid-box-1',
    name: 'Parametric Array Excavation Cell Box',
    show: true,
    positionDegrees: [-75.1480, 39.9510, 25.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 55.0, 50.0],
    },
    color: [0.98, 0.65, 0.1, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      gridIndex: 1,
      quality: 1.0,
    },
  };

  const gridMarker = {
    id: 'clip-perf-grid-marker',
    name: 'Performance Telemetry Sampling Probe',
    show: true,
    positionDegrees: [-75.1560, 39.9430, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [20.0, 20.0, 20.0],
    },
    color: [0.65, 0.25, 0.9, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.7, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      fpsDisplay: true,
      debugDistanceTexture: false,
    },
  };

  const extentRect = {
    id: 'clip-perf-extent-rect',
    name: 'Parametric Clipping Coverage Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.165, 39.935, -75.140, 39.960],
    },
    color: [0.15, 0.45, 0.5, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.8, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      gridSize: 2,
      scale: 1.0,
    },
  };

  const entities = [centerCylinder, gridBox1, gridMarker, extentRect];

  const variants = [
    {
      label: 'Baseline: Terrain Target Clipping (Crimson & Amber Grid)',
      entities: [
        centerCylinder,
        gridBox1,
        gridMarker,
        extentRect,
      ],
    },
    {
      label: '3D Tiles Target Clipping (Electric Cyan Volume Mask)',
      entities: [
        { ...centerCylinder, color: [0.15, 0.85, 0.95, 0.9], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        gridBox1,
        gridMarker,
        extentRect,
      ],
    },
    {
      label: 'Debug DistanceTexture Mode (High-Contrast Violet Infill)',
      entities: [
        centerCylinder,
        gridBox1,
        { ...gridMarker, color: [0.85, 0.2, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        extentRect,
      ],
    },
    {
      label: 'High-Density Quality Mode (Golden High-Precision Contour)',
      entities: [
        centerCylinder,
        { ...gridBox1, color: [1.0, 0.8, 0.15, 0.9], outlineColor: [1.0, 0.95, 0.6, 1.0] },
        gridMarker,
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Clipping Regions (Denver Downtown Development Lot)
function generateClippingRegions() {
  const buildingInset = {
    id: 'clip-reg-building-inset',
    name: 'Proposed Architectural Development Tower Inset',
    show: true,
    positionDegrees: [-105.0085, 39.7505, 45.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 65.0, 90.0],
    },
    color: [0.3, 0.65, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 1670818,
      status: 'proposed',
    },
  };

  const coreCylinder = {
    id: 'clip-reg-core-cylinder',
    name: 'Excavation Core Foundation Shaft',
    show: true,
    positionDegrees: [-105.0080, 39.7510, 30.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 22.0,
      bottomRadius: 22.0,
    },
    color: [0.2, 0.8, 0.45, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      depthMeters: 45.0,
      foundation: 'drilledShaft',
    },
  };

  const surveyBenchmark = {
    id: 'clip-reg-survey-benchmark',
    name: 'Denver Urban Lot Survey Control Mon',
    show: true,
    positionDegrees: [-105.0092, 39.7498, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 18.0, 18.0],
    },
    color: [1.0, 0.7, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      controlPoint: 'CO-DEN-LOT-29',
    },
  };

  const lotPolygonRect = {
    id: 'clip-reg-lot-polygon-rect',
    name: 'Denver Downtown Lot Excavation Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.010, 39.749, -105.007, 39.752],
    },
    color: [0.55, 0.15, 0.25, 0.4],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.4, 0.5, 0.85],
    outlineWidth: 1.0,
    properties: {
      clippingEnabled: true,
      inverse: false,
    },
  };

  const entities = [buildingInset, coreCylinder, surveyBenchmark, lotPolygonRect];

  const variants = [
    {
      label: 'Baseline: 3D Tiles Lot Clipping with Proposed Building',
      entities: [
        buildingInset,
        coreCylinder,
        surveyBenchmark,
        lotPolygonRect,
      ],
    },
    {
      label: 'Terrain Surface Clipping Mode (Emerald Core Foundation)',
      entities: [
        buildingInset,
        { ...coreCylinder, color: [0.1, 0.95, 0.55, 0.95], outlineColor: [0.8, 1.0, 0.9, 1.0] },
        surveyBenchmark,
        lotPolygonRect,
      ],
    },
    {
      label: 'Inverse Clipping Mode (Hollow Exterior Void / Amber Highlight)',
      entities: [
        buildingInset,
        coreCylinder,
        { ...surveyBenchmark, color: [1.0, 0.4, 0.1, 0.95], outlineColor: [1.0, 0.8, 0.4, 1.0] },
        lotPolygonRect,
      ],
    },
    {
      label: 'Building Inset Isolated Focus (High-Contrast Tower Shading)',
      entities: [
        { ...buildingInset, color: [0.2, 0.75, 1.0, 0.95], outlineColor: [0.9, 1.0, 1.0, 1.0] },
        coreCylinder,
        surveyBenchmark,
        lotPolygonRect,
      ],
    },
  ];

  return { entities, variants };
}

function writeCase(id, { entities, variants }) {
  fs.writeFileSync(path.join(outDir, `${id}.json`), JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(path.join(outDir, `${id}.variants.json`), JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json and ${id}.variants.json`);
}

writeCase('classification-types', generateClassificationTypes());
writeCase('client-side-snapping-dev', generateClientSideSnappingDev());
writeCase('clipping-performance-dev', generateClippingPerformanceDev());
writeCase('clipping-regions', generateClippingRegions());
