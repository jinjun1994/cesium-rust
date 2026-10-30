import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Constant LOD - Dev (Exton, PA)
function generateConstantLodDev() {
  const checkerBox = {
    id: 'clod-checker-plane-box',
    name: 'Constant LOD Checkerboard Plane Member',
    show: true,
    positionDegrees: [-75.6867, 40.0658, 25.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 15.0],
    },
    color: [0.85, 0.85, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.2, 0.2, 0.2, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'ConstantLod_Checker.gltf',
      minimumPixelSize: 128,
      maximumScale: 20000,
    },
  };

  const grassCylinder = {
    id: 'clod-grass-canopy-cylinder',
    name: 'Grass Surface Texture Canopy Cylinder',
    show: true,
    positionDegrees: [-75.6855, 40.0665, 30.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 24.0,
      bottomRadius: 24.0,
    },
    color: [0.25, 0.75, 0.35, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'ConstantLod_Grass.gltf',
      surfaceType: 'vegetation',
    },
  };

  const normalSphere = {
    id: 'clod-normal-map-sphere',
    name: 'Normal Map Curvature Benchmark Sphere',
    show: true,
    positionDegrees: [-75.6880, 40.0650, 25.0],
    shape: {
      type: 'ellipsoid',
      radii: [20.0, 20.0, 20.0],
    },
    color: [0.2, 0.55, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'ConstantLod_NormalMap.gltf',
      hasNormalMap: true,
    },
  };

  const footprintRect = {
    id: 'clod-model-footprint-rect',
    name: 'Constant LOD Model Placement Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.690, 40.063, -75.683, 40.068],
    },
    color: [0.3, 0.35, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.7, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      testSite: 'Exton Pennsylvania CLOD Test Facility',
    },
  };

  const entities = [checkerBox, grassCylinder, normalSphere, footprintRect];

  const variants = [
    {
      label: 'Baseline: Checkerboard Constant LOD (Monochrome Grid)',
      entities: [
        checkerBox,
        grassCylinder,
        normalSphere,
        footprintRect,
      ],
    },
    {
      label: 'Grass Constant LOD Mode (Emerald Texture Infill)',
      entities: [
        { ...checkerBox, color: [0.2, 0.85, 0.4, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        grassCylinder,
        normalSphere,
        footprintRect,
      ],
    },
    {
      label: 'Checkerboard Emissive Glow (Amber Radiant Highlight)',
      entities: [
        checkerBox,
        { ...grassCylinder, color: [1.0, 0.75, 0.15, 0.95], outlineColor: [1.0, 0.9, 0.5, 1.0] },
        normalSphere,
        footprintRect,
      ],
    },
    {
      label: 'Angled Plane & Normal Map Transform (Cyan Rotated Profile)',
      entities: [
        checkerBox,
        grassCylinder,
        { ...normalSphere, color: [0.1, 0.9, 0.95, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        footprintRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Custom DataSource (WebGL Globe Population Spikes - Kanto Megacity)
function generateCustomDataSource() {
  const tokyoPillar = {
    id: 'custom-ds-tokyo-pillar',
    name: 'Tokyo Central Urban Population Density Pillar',
    show: true,
    positionDegrees: [139.6917, 35.6895, 60.0],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 25.0,
      bottomRadius: 25.0,
    },
    color: [0.95, 0.15, 0.5, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      seriesName: '1990',
      city: 'Tokyo',
      densityIndex: 0.92,
    },
  };

  const shinjukuPillar = {
    id: 'custom-ds-nyc-pillar',
    name: 'Shinjuku Skyscraper Cluster Density Pillar',
    show: true,
    positionDegrees: [139.7005, 35.6908, 55.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 50.0, 110.0],
    },
    color: [0.98, 0.55, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      seriesName: '1990',
      district: 'Shinjuku',
      densityIndex: 0.85,
    },
  };

  const yokohamaMarker = {
    id: 'custom-ds-london-marker',
    name: 'Yokohama Port Population Centroid Marker',
    show: true,
    positionDegrees: [139.6380, 35.4437, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 25.0, 25.0],
    },
    color: [0.2, 0.8, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      seriesName: '1990',
      city: 'Yokohama',
      densityIndex: 0.72,
    },
  };

  const globeBeltRect = {
    id: 'custom-ds-globe-baseline-rect',
    name: 'Tokyo Bay Megacity Cluster Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [139.55, 35.40, 139.85, 35.80],
    },
    color: [0.15, 0.2, 0.4, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.5, 0.8, 0.8],
    outlineWidth: 1.0,
    properties: {
      dataSource: 'WebGLGlobeDataSource',
      heightScale: 10000000,
    },
  };

  const entities = [tokyoPillar, shinjukuPillar, yokohamaMarker, globeBeltRect];

  const variants = [
    {
      label: 'Baseline: 1990 Population Series (Global Urban Pillars)',
      entities: [
        tokyoPillar,
        shinjukuPillar,
        yokohamaMarker,
        globeBeltRect,
      ],
    },
    {
      label: '1995 Population Growth Series (Amber & Crimson Infill)',
      entities: [
        tokyoPillar,
        { ...shinjukuPillar, color: [1.0, 0.25, 0.2, 0.95], outlineColor: [1.0, 0.7, 0.7, 1.0] },
        yokohamaMarker,
        globeBeltRect,
      ],
    },
    {
      label: '2000 Peak Millennium Series (Vibrant Violet & Rose)',
      entities: [
        { ...tokyoPillar, color: [0.8, 0.2, 0.95, 0.95], outlineColor: [1.0, 0.65, 1.0, 1.0] },
        shinjukuPillar,
        yokohamaMarker,
        globeBeltRect,
      ],
    },
    {
      label: 'Dynamic Height Scale Multiplier (2x Elevation Spikes)',
      entities: [
        tokyoPillar,
        shinjukuPillar,
        { ...yokohamaMarker, color: [0.3, 0.95, 0.6, 0.95], outlineColor: [0.8, 1.0, 0.9, 1.0] },
        globeBeltRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Custom Geocoder (Philadelphia Nominatim Search)
function generateCustomGeocoder() {
  const cityHallBox = {
    id: 'custom-geo-philly-box',
    name: 'Philadelphia City Hall Landmark Tower',
    show: true,
    positionDegrees: [-75.1636, 39.9524, 40.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 80.0],
    },
    color: [0.85, 0.78, 0.68, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      landmark: 'City Hall',
      geocoderQuery: 'Philadelphia',
    },
  };

  const parkwayPillar = {
    id: 'custom-geo-ben-franklin-pillar',
    name: 'Benjamin Franklin Parkway Anchor Pillar',
    show: true,
    positionDegrees: [-75.1720, 39.9580, 30.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 20.0,
      bottomRadius: 20.0,
    },
    color: [0.8, 0.5, 0.2, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      alignment: 'Parkway Diagonal',
    },
  };

  const searchPinSphere = {
    id: 'custom-geo-search-pin-sphere',
    name: 'Nominatim Geocoded Search Pin Marker',
    show: true,
    positionDegrees: [-75.1652, 39.9526, 25.0],
    shape: {
      type: 'ellipsoid',
      radii: [22.0, 22.0, 22.0],
    },
    color: [0.95, 0.15, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      geocoderService: 'OpenStreetMap Nominatim',
      displayName: 'Philadelphia, Pennsylvania, United States',
    },
  };

  const bboxRect = {
    id: 'custom-geo-nominatim-bbox-rect',
    name: 'Philadelphia Nominatim Result Bounding Box',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.280, 39.867, -74.955, 40.138],
    },
    color: [0.2, 0.35, 0.75, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.65, 1.0, 0.85],
    outlineWidth: 1.0,
    properties: {
      boundingbox: [39.867, 40.138, -75.280, -74.955],
    },
  };

  const entities = [cityHallBox, parkwayPillar, searchPinSphere, bboxRect];

  const variants = [
    {
      label: 'Baseline: Nominatim Search "Philadelphia" & Bounding Box',
      entities: [
        cityHallBox,
        parkwayPillar,
        searchPinSphere,
        bboxRect,
      ],
    },
    {
      label: 'Focus City Hall Landmark (Gold Tower Shading)',
      entities: [
        { ...cityHallBox, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.95, 0.5, 1.0] },
        parkwayPillar,
        searchPinSphere,
        bboxRect,
      ],
    },
    {
      label: 'Expanded Metro Extents (Emerald Regional Boundary)',
      entities: [
        cityHallBox,
        parkwayPillar,
        searchPinSphere,
        { ...bboxRect, color: [0.15, 0.5, 0.3, 0.45], outlineColor: [0.4, 0.95, 0.6, 0.9] },
      ],
    },
    {
      label: 'High-Contrast Search Pin Focus (Neon Amber Highlight)',
      entities: [
        cityHallBox,
        parkwayPillar,
        { ...searchPinSphere, color: [1.0, 0.65, 0.05, 0.95], outlineColor: [1.0, 0.9, 0.4, 1.0] },
        bboxRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Custom Per-Feature Post Process (Eugene, Oregon Model Stage)
function generateCustomPerFeaturePostProcess() {
  const subjectBox = {
    id: 'custom-post-subject-box',
    name: 'Tracked Model Target Subject Volume',
    show: true,
    positionDegrees: [-123.0745, 44.0504, 25.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 50.0],
    },
    color: [0.6, 0.6, 0.65, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.9, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'Cesium_Man.glb',
      postProcessStage: 'PerFeatureHighlight',
    },
  };

  const podiumCylinder = {
    id: 'custom-post-podium-cylinder',
    name: 'Model Inspection Podium Base',
    show: true,
    positionDegrees: [-123.0740, 44.0508, 18.0],
    shape: {
      type: 'cylinder',
      length: 36.0,
      topRadius: 22.0,
      bottomRadius: 22.0,
    },
    color: [0.35, 0.45, 0.55, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.75, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      role: 'stagePlatform',
    },
  };

  const hoverMarker = {
    id: 'custom-post-hover-marker',
    name: 'Feature Pick Hover Detection Marker',
    show: true,
    positionDegrees: [-123.0750, 44.0500, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 18.0, 18.0],
    },
    color: [0.15, 0.75, 0.75, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      shaderSelected: false,
    },
  };

  const stageFootprintRect = {
    id: 'custom-post-stage-footprint-rect',
    name: 'Post Process Stage Activation Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.077, 44.048, -123.072, 44.053],
    },
    color: [0.35, 0.2, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.45, 0.9, 0.85],
    outlineWidth: 1.0,
    properties: {
      fragmentShader: 'czm_selected per-feature color blend',
    },
  };

  const entities = [subjectBox, podiumCylinder, hoverMarker, stageFootprintRect];

  const variants = [
    {
      label: 'Baseline: Unselected Character Subject & Stage',
      entities: [
        subjectBox,
        podiumCylinder,
        hoverMarker,
        stageFootprintRect,
      ],
    },
    {
      label: 'Hover Selection Highlight (Red 50% Shader Overlay)',
      entities: [
        { ...subjectBox, color: [1.0, 0.2, 0.2, 0.95], outlineColor: [1.0, 0.7, 0.7, 1.0] },
        podiumCylinder,
        hoverMarker,
        stageFootprintRect,
      ],
    },
    {
      label: 'Multi-Feature Selection Highlight (Amber & Cyan Shading)',
      entities: [
        subjectBox,
        { ...podiumCylinder, color: [0.2, 0.85, 0.95, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        { ...hoverMarker, color: [1.0, 0.7, 0.1, 0.95], outlineColor: [1.0, 0.9, 0.5, 1.0] },
        stageFootprintRect,
      ],
    },
    {
      label: 'Stage Shader Tint Diagnostic (High-Contrast Neon Magenta)',
      entities: [
        { ...subjectBox, color: [0.95, 0.1, 0.8, 0.95], outlineColor: [1.0, 0.6, 0.9, 1.0] },
        podiumCylinder,
        hoverMarker,
        stageFootprintRect,
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

writeCase('constant-lod-dev', generateConstantLodDev());
writeCase('custom-datasource', generateCustomDataSource());
writeCase('custom-geocoder', generateCustomGeocoder());
writeCase('custom-per-feature-post-process', generateCustomPerFeaturePostProcess());
