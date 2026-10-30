import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Cesium OSM Buildings (New York Lower Manhattan)
function generateCesiumOsmBuildings() {
  const wtcTower = {
    id: 'osm-1wtc-tower',
    name: 'One World Trade Center Landmark Tower',
    show: true,
    positionDegrees: [-74.0134, 40.7127, 210.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 420.0],
    },
    color: [0.85, 0.9, 0.98, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      building: 'commercial',
      levels: 104,
      heightMeters: 417.0,
      source: 'Cesium OSM Buildings',
    },
  };

  const batteryPark = {
    id: 'osm-battery-park-pillar',
    name: 'Battery Park Esplanade Monument Pillar',
    show: true,
    positionDegrees: [-74.016, 40.703, 30.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 25.0,
      bottomRadius: 25.0,
    },
    color: [0.3, 0.72, 0.48, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      leisure: 'park',
      name: 'The Battery',
    },
  };

  const brooklynBridge = {
    id: 'osm-brooklyn-bridge-tower',
    name: 'Brooklyn Bridge Manhattan Anchorage Pier',
    show: true,
    positionDegrees: [-74.000, 40.707, 50.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 65.0, 90.0],
    },
    color: [0.82, 0.75, 0.68, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      man_made: 'bridge',
      material: 'limestone_granite',
    },
  };

  const libertyIsland = {
    id: 'osm-liberty-island-marker',
    name: 'Statue of Liberty National Monument Pedestal',
    show: true,
    positionDegrees: [-74.0445, 40.6892, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [40.0, 40.0, 48.0],
    },
    color: [0.35, 0.76, 0.68, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      historic: 'monument',
      heightMeters: 93.0,
    },
  };

  const manhattanBounds = {
    id: 'osm-manhattan-bounds-rect',
    name: 'Lower Manhattan OSM Building Footprint Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.05, 40.68, -73.99, 40.72],
    },
    color: [0.25, 0.35, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.6, 0.8, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'urbanFootprint',
    },
  };

  const entities = [wtcTower, batteryPark, brooklynBridge, libertyIsland, manhattanBounds];

  const variants = [
    {
      label: 'Baseline: Cesium OSM Buildings (Lower Manhattan)',
      entities: [
        wtcTower,
        batteryPark,
        brooklynBridge,
        libertyIsland,
        manhattanBounds,
      ],
    },
    {
      label: 'One World Trade Center Tower Focus',
      entities: [
        { ...wtcTower, color: [0.98, 0.85, 0.25, 0.95] },
        batteryPark,
        brooklynBridge,
        libertyIsland,
        manhattanBounds,
      ],
    },
    {
      label: 'Battery Park & Waterfront District',
      entities: [
        wtcTower,
        { ...batteryPark, color: [0.2, 0.95, 0.5, 0.95] },
        brooklynBridge,
        libertyIsland,
        manhattanBounds,
      ],
    },
    {
      label: 'Upper Harbor & Statue of Liberty Context',
      entities: [
        wtcTower,
        batteryPark,
        { ...brooklynBridge, color: [0.95, 0.4, 0.2, 0.95] },
        { ...libertyIsland, color: [0.2, 0.9, 1.0, 0.95] },
        manhattanBounds,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Cesium World Terrain (Yosemite Valley / Half Dome High Relief)
function generateCesiumWorldTerrain() {
  const halfDomeMonolith = {
    id: 'cwt-halfdome-monolith',
    name: 'Half Dome Monolithic Granite Summit (2694m)',
    show: true,
    positionDegrees: [-119.533, 37.746, 2694.0],
    shape: {
      type: 'cylinder',
      length: 600.0,
      topRadius: 180.0,
      bottomRadius: 180.0,
    },
    color: [0.88, 0.88, 0.92, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      elevationMeters: 2694.0,
      geology: 'Half Dome Granodiorite',
      provider: 'Cesium World Terrain',
    },
  };

  const elCapitanCliff = {
    id: 'cwt-elcapitan-cliff',
    name: 'El Capitan Vertical Granite Wall (2307m)',
    show: true,
    positionDegrees: [-119.637, 37.734, 2307.0],
    shape: {
      type: 'box',
      dimensions: [500.0, 350.0, 900.0],
    },
    color: [0.85, 0.82, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      elevationMeters: 2307.0,
      verticalDropMeters: 914.0,
    },
  };

  const mercedRiverBasin = {
    id: 'cwt-merced-river-basin',
    name: 'Yosemite Valley Merced River Drainage Basin',
    show: true,
    positionDegrees: [-119.585, 37.742, 1220.0],
    shape: {
      type: 'ellipsoid',
      radii: [250.0, 250.0, 150.0],
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      valleyFloorElevationMeters: 1219.0,
    },
  };

  const yosemiteBoundary = {
    id: 'cwt-yosemite-boundary-rect',
    name: 'Yosemite Valley High-Resolution Mesh Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-119.70, 37.68, -119.45, 37.80],
    },
    color: [0.28, 0.45, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 0.5, 0.8],
    outlineWidth: 1.0,
    properties: {
      terrainType: 'Quantized-Mesh 1.0 Terrain',
    },
  };

  const entities = [halfDomeMonolith, elCapitanCliff, mercedRiverBasin, yosemiteBoundary];

  const variants = [
    {
      label: 'Baseline: Cesium World Terrain (Yosemite Valley High Relief)',
      entities: [
        halfDomeMonolith,
        elCapitanCliff,
        mercedRiverBasin,
        yosemiteBoundary,
      ],
    },
    {
      label: 'Half Dome Monolith Summit View',
      entities: [
        { ...halfDomeMonolith, color: [0.98, 0.85, 0.35, 0.95] },
        elCapitanCliff,
        mercedRiverBasin,
        yosemiteBoundary,
      ],
    },
    {
      label: 'El Capitan Vertical Granite Wall',
      entities: [
        halfDomeMonolith,
        { ...elCapitanCliff, color: [0.95, 0.4, 0.25, 0.95] },
        mercedRiverBasin,
        yosemiteBoundary,
      ],
    },
    {
      label: 'Merced River Valley Basin & Meadows',
      entities: [
        halfDomeMonolith,
        elCapitanCliff,
        { ...mercedRiverBasin, color: [0.2, 0.9, 0.95, 0.95] },
        yosemiteBoundary,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Clamp to 3D Model (sampleHeight, GroundVehicle, Dynamic Clamping)
function generateClampTo3dModel() {
  const vehicleChassis = {
    id: 'clamp-vehicle-chassis',
    name: 'Ground Vehicle Articulated Model Chassis',
    show: true,
    positionDegrees: [-123.074, 44.050, 20.0],
    shape: {
      type: 'box',
      dimensions: [25.0, 16.0, 12.0],
    },
    color: [0.85, 0.32, 0.22, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      uri: 'SampleData/models/GroundVehicle/GroundVehicle.glb',
      clamped: true,
    },
  };

  const sampleHeightPoint = {
    id: 'clamp-sample-height-point',
    name: 'Real-Time Dynamic Clamping Sample Point',
    show: true,
    positionDegrees: [-123.074, 44.050, 26.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0],
    },
    color: [1.0, 0.95, 0.15, 0.95], // Yellow
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      pixelSize: 10,
      sampleHeightSupported: true,
      color: 'Yellow',
    },
  };

  const terrainRefMast = {
    id: 'clamp-reference-mast',
    name: 'Ground Proximity Laser Sensor Mast',
    show: true,
    positionDegrees: [-123.073, 44.051, 25.0],
    shape: {
      type: 'cylinder',
      length: 45.0,
      topRadius: 6.0,
      bottomRadius: 6.0,
    },
    color: [0.2, 0.72, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      sensorRangeMeters: 50.0,
    },
  };

  const groundTestBounds = {
    id: 'clamp-test-ground-bounds',
    name: 'Vehicle Proving Ground Testing Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.077, 44.047, -123.071, 44.053],
    },
    color: [0.35, 0.45, 0.38, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.7, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'provingGround',
    },
  };

  const entities = [vehicleChassis, sampleHeightPoint, terrainRefMast, groundTestBounds];

  const variants = [
    {
      label: 'Baseline: Clamped Ground Vehicle with Real-Time Height Sampling',
      entities: [
        vehicleChassis,
        sampleHeightPoint,
        terrainRefMast,
        groundTestBounds,
      ],
    },
    {
      label: 'Tracking Vehicle Chassis Focus',
      entities: [
        { ...vehicleChassis, color: [0.95, 0.85, 0.2, 0.95] },
        sampleHeightPoint,
        terrainRefMast,
        groundTestBounds,
      ],
    },
    {
      label: 'Sample Height Point Trajectory',
      entities: [
        vehicleChassis,
        { ...sampleHeightPoint, color: [0.2, 1.0, 0.3, 0.95] },
        terrainRefMast,
        groundTestBounds,
      ],
    },
    {
      label: 'Ground Clearance & Obstacle Test',
      entities: [
        vehicleChassis,
        sampleHeightPoint,
        { ...terrainRefMast, color: [1.0, 0.3, 0.3, 0.95] },
        groundTestBounds,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Classification (ClassificationPrimitive, 3D Tiles & Terrain Classification Volumes)
function generateClassification() {
  const buildingVolume = {
    id: 'class-building-volume',
    name: 'Building Structure Classification Box Volume',
    show: true,
    positionDegrees: [-75.5965, 39.8491, 120.0],
    shape: {
      type: 'box',
      dimensions: [24.0, 18.0, 24.0],
    },
    color: [1.0, 0.0, 0.0, 0.5], // Red alpha 0.5
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.4, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      id: 'volume',
    },
  };

  const treeVolume1 = {
    id: 'class-tree-volume-1',
    name: 'Vegetation Canopy Tree 1 Classification Ellipsoid',
    show: true,
    positionDegrees: [-75.5963, 39.8492, 120.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 15.0, 12.0],
    },
    color: [0.95, 0.39, 0.1, 0.6], // #F26419 alpha 0.5
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      id: 'volume 1',
    },
  };

  const treeVolume2 = {
    id: 'class-tree-volume-2',
    name: 'Vegetation Canopy Tree 2 Classification Ellipsoid',
    show: true,
    positionDegrees: [-75.5964, 39.8493, 120.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 15.0, 12.0],
    },
    color: [0.94, 0.23, 0.28, 0.6], // #F03A47 alpha 0.5
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      id: 'volume 2',
    },
  };

  const surveyMarker = {
    id: 'class-survey-marker',
    name: 'Photogrammetry Ground Datum Classification Cylinder',
    show: true,
    positionDegrees: [-75.5966, 39.8490, 125.0],
    shape: {
      type: 'cylinder',
      length: 30.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.0, 0.31, 1.0, 0.6], // #004FFF alpha 0.5
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      id: 'volume 3',
    },
  };

  const classSiteFootprint = {
    id: 'class-site-footprint-rect',
    name: 'Photogrammetry Survey Classification Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.5975, 39.8485, -75.5955, 39.8498],
    },
    color: [0.28, 0.42, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.68, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'classificationSiteBounds',
    },
  };

  const entities = [buildingVolume, treeVolume1, treeVolume2, surveyMarker, classSiteFootprint];

  const variants = [
    {
      label: 'Baseline: All Classification Volumes (Building & Trees)',
      entities: [
        buildingVolume,
        treeVolume1,
        treeVolume2,
        surveyMarker,
        classSiteFootprint,
      ],
    },
    {
      label: 'Highlight Building Volume (Red 50% Alpha)',
      entities: [
        { ...buildingVolume, color: [1.0, 0.2, 0.2, 0.9] },
        treeVolume1,
        treeVolume2,
        surveyMarker,
        classSiteFootprint,
      ],
    },
    {
      label: 'Highlight Vegetation Volumes (Orange & Crimson Trees)',
      entities: [
        buildingVolume,
        { ...treeVolume1, color: [1.0, 0.55, 0.1, 0.9] },
        { ...treeVolume2, color: [1.0, 0.3, 0.35, 0.9] },
        surveyMarker,
        classSiteFootprint,
      ],
    },
    {
      label: 'Interactive Hover Pick Emulation (Magenta Highlight)',
      entities: [
        { ...buildingVolume, color: [1.0, 0.0, 1.0, 0.8] }, // Magenta hover
        treeVolume1,
        treeVolume2,
        surveyMarker,
        classSiteFootprint,
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

writeCase('cesium-osm-buildings', generateCesiumOsmBuildings());
writeCase('cesium-world-terrain', generateCesiumWorldTerrain());
writeCase('clamp-to-3d-model', generateClampTo3dModel());
writeCase('classification', generateClassification());
