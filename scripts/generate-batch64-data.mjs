import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Aerometrex San Francisco (Street-Level Enhanced Photogrammetry)
function generateAerometrexSanFrancisco() {
  const ferryBuilding = {
    id: 'sf-ferry-building-tower',
    name: 'San Francisco Ferry Building Landmark Tower (~6mm/px)',
    show: true,
    positionDegrees: [-122.3937, 37.7955, 65.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 40.0, 90.0],
    },
    color: [0.86, 0.78, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      location: 'The Embarcadero / Market St',
      resolution: '~6mm per pixel',
      assetId: 1415196,
    },
  };

  const pier39 = {
    id: 'sf-pier39-marina',
    name: 'Pier 39 Marine Boardwalk (2cm/px)',
    show: true,
    positionDegrees: [-122.4098, 37.8087, 50.0],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 35.0,
      bottomRadius: 35.0,
    },
    color: [0.35, 0.68, 0.88, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      location: 'Fisherman Wharf / Pier 39',
      resolution: '2 cm per pixel',
    },
  };

  const skylinePyramid = {
    id: 'sf-skyline-transamerica',
    name: 'Transamerica Financial Skyline Tower',
    show: true,
    positionDegrees: [-122.401, 37.792, 140.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 260.0],
    },
    color: [0.92, 0.92, 0.96, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      location: 'Montgomery Street Financial Core',
      heightMeters: 260.0,
    },
  };

  const lombardStreet = {
    id: 'sf-lombard-crooked-hill',
    name: 'Lombard Street Russian Hill Crooked Section',
    show: true,
    positionDegrees: [-122.418, 37.802, 90.0],
    shape: {
      type: 'ellipsoid',
      radii: [32.0, 32.0, 22.0],
    },
    color: [0.88, 0.38, 0.28, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      location: 'Russian Hill Switchbacks',
      resolution: '2 cm per pixel',
    },
  };

  const sfBayFootprint = {
    id: 'sf-bay-waterfront-bounds',
    name: 'San Francisco Waterfront Photogrammetry Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.425, 37.788, -122.385, 37.815],
    },
    color: [0.2, 0.45, 0.65, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'waterfrontBounds',
    },
  };

  const entities = [ferryBuilding, pier39, skylinePyramid, lombardStreet, sfBayFootprint];

  const variants = [
    {
      label: 'Baseline: San Francisco Waterfront & Landmarks',
      entities: [
        ferryBuilding,
        pier39,
        skylinePyramid,
        lombardStreet,
        sfBayFootprint,
      ],
    },
    {
      label: 'Ferry Building & Embarcadero (~6mm/px)',
      entities: [
        { ...ferryBuilding, color: [0.98, 0.85, 0.3, 0.95] },
        pier39,
        skylinePyramid,
        lombardStreet,
        sfBayFootprint,
      ],
    },
    {
      label: 'Pier 39 Marina (2cm/px High Resolution)',
      entities: [
        ferryBuilding,
        { ...pier39, color: [0.2, 0.9, 0.95, 0.95] },
        skylinePyramid,
        lombardStreet,
        sfBayFootprint,
      ],
    },
    {
      label: 'Financial District Skyline & Lombard Russian Hill',
      entities: [
        ferryBuilding,
        pier39,
        { ...skylinePyramid, color: [0.95, 0.3, 0.3, 0.95] },
        { ...lombardStreet, color: [0.3, 0.9, 0.4, 0.95] },
        sfBayFootprint,
      ],
    },
  ];

  return { entities, variants };
}

// 2. ArcGIS Tiled Elevation Terrain (Esri WorldElevation3D LERC / Terrain3D)
function generateArcgisTiledElevation() {
  const surveyPillar = {
    id: 'arcgis-elevation-pillar',
    name: 'Grand Canyon South Rim Elevation Benchmark Pillar',
    show: true,
    positionDegrees: [-112.14, 36.05, 2250.0],
    shape: {
      type: 'cylinder',
      length: 800.0,
      topRadius: 100.0,
      bottomRadius: 100.0,
    },
    color: [0.95, 0.35, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      benchmark: 'South Rim Survey Point',
      service: 'WorldElevation3D/Terrain3D',
      format: 'LERC Tile Service',
    },
  };

  const rimLookout = {
    id: 'arcgis-canyon-rim-box',
    name: 'Mather Point Rim Lookout Formation',
    show: true,
    positionDegrees: [-112.12, 36.06, 2180.0],
    shape: {
      type: 'box',
      dimensions: [320.0, 220.0, 160.0],
    },
    color: [0.85, 0.65, 0.42, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      geologicalUnit: 'Kaibab Limestone Rim',
      elevationMeters: 2180.0,
    },
  };

  const riverDatum = {
    id: 'arcgis-colorado-river-datum',
    name: 'Inner Gorge Colorado River Elevation Datum',
    show: true,
    positionDegrees: [-112.13, 36.08, 1250.0],
    shape: {
      type: 'ellipsoid',
      radii: [160.0, 160.0, 110.0],
    },
    color: [0.2, 0.65, 0.88, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      elevationMeters: 750.0,
      depthFromRimMeters: 1430.0,
    },
  };

  const coverageRect = {
    id: 'arcgis-coverage-boundary',
    name: 'ArcGIS Tiled Elevation Local Coverage Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-112.22, 36.01, -112.04, 36.12],
    },
    color: [0.32, 0.48, 0.38, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.75, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      projection: 'WebMercator / WGS84 Elevation Tiled',
    },
  };

  const entities = [surveyPillar, rimLookout, riverDatum, coverageRect];

  const variants = [
    {
      label: 'Grand Canyon Elevation Profile (Global Terrain3D LERC)',
      entities: [
        surveyPillar,
        rimLookout,
        riverDatum,
        coverageRect,
      ],
    },
    {
      label: 'High-Plateau Rim Elevation Benchmarks',
      entities: [
        { ...surveyPillar, color: [0.98, 0.85, 0.2, 0.95] },
        { ...rimLookout, color: [0.95, 0.4, 0.2, 0.95] },
        riverDatum,
        coverageRect,
      ],
    },
    {
      label: 'Inner Gorge Riverbed Contour Verification',
      entities: [
        surveyPillar,
        rimLookout,
        { ...riverDatum, color: [0.15, 0.95, 0.85, 0.95] },
        coverageRect,
      ],
    },
    {
      label: 'All Elevation Grid Ground Control Points',
      entities: [
        { ...surveyPillar, color: [1.0, 0.2, 0.2, 0.95] },
        { ...rimLookout, color: [0.2, 0.8, 1.0, 0.95] },
        { ...riverDatum, color: [0.3, 1.0, 0.4, 0.95] },
        coverageRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. ArcticDEM (Cesium ion Asset 3956, 2m high-resolution photogrammetric arctic elevation)
function generateArcticDem() {
  const denaliSummit = {
    id: 'arctic-denali-summit',
    name: 'Denali (Mount McKinley) 6190m Summit Peak',
    show: true,
    positionDegrees: [-151.006, 63.069, 5800.0],
    shape: {
      type: 'cylinder',
      length: 1000.0,
      topRadius: 150.0,
      bottomRadius: 150.0,
    },
    color: [0.92, 0.94, 0.98, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      peak: 'South Peak (Denali)',
      elevationMeters: 6190.0,
      source: 'ArcticDEM Release 4',
      assetId: 3956,
    },
  };

  const glacierFormation = {
    id: 'arctic-kahiltna-glacier',
    name: 'Kahiltna Glacier Trunk Formation',
    show: true,
    positionDegrees: [-151.15, 62.95, 2800.0],
    shape: {
      type: 'box',
      dimensions: [600.0, 400.0, 250.0],
    },
    color: [0.55, 0.82, 0.95, 0.92],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      feature: 'Valley Glacier System',
      elevationMeters: 2800.0,
    },
  };

  const moraineRidge = {
    id: 'arctic-lateral-moraine',
    name: 'Glacial Lateral Moraine Elevation Ridge',
    show: true,
    positionDegrees: [-151.08, 62.92, 2100.0],
    shape: {
      type: 'ellipsoid',
      radii: [250.0, 250.0, 180.0],
    },
    color: [0.68, 0.72, 0.78, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.9, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: {
      feature: 'Lateral Moraine Debris Ridge',
      elevationMeters: 2100.0,
    },
  };

  const arcticCoverageRect = {
    id: 'arctic-strip-coverage-rect',
    name: 'ArcticDEM Stereoscopic Strip Coverage Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-151.35, 62.88, -150.85, 63.15],
    },
    color: [0.28, 0.45, 0.58, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.7, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      dataset: 'Polar Geospatial Center ArcticDEM 2m',
    },
  };

  const entities = [denaliSummit, glacierFormation, moraineRidge, arcticCoverageRect];

  const variants = [
    {
      label: 'Baseline: Denali & Alaska Range High Elevation Profile',
      entities: [
        denaliSummit,
        glacierFormation,
        moraineRidge,
        arcticCoverageRect,
      ],
    },
    {
      label: 'Denali Summit Focus (Highest Peak 6,190m)',
      entities: [
        { ...denaliSummit, color: [1.0, 0.95, 0.4, 0.95] },
        glacierFormation,
        moraineRidge,
        arcticCoverageRect,
      ],
    },
    {
      label: 'Anchorage Chugach Mountains & Glacier Trunk',
      entities: [
        denaliSummit,
        { ...glacierFormation, color: [0.3, 0.9, 0.95, 0.95] },
        moraineRidge,
        arcticCoverageRect,
      ],
    },
    {
      label: 'Subarctic Lateral Moraine & Braided Plain',
      entities: [
        denaliSummit,
        glacierFormation,
        { ...moraineRidge, color: [0.95, 0.5, 0.3, 0.95] },
        arcticCoverageRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Cesium Inspector (Cesium Inspector mixin, depthTestAgainstTerrain, Primitive Box, Rectangle, Billboard)
function generateCesiumInspector() {
  const primitiveBox = {
    id: 'inspector-primitive-box',
    name: 'Red Translucent Primitive Box (Closed Appearance)',
    show: true,
    positionDegrees: [-105.0, 45.0, 250000.0],
    shape: {
      type: 'box',
      dimensions: [400000.0, 300000.0, 500000.0],
    },
    color: [1.0, 0.0, 0.0, 0.5],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      appearance: 'PerInstanceColorAppearance',
      closed: true,
      vertexFormat: 'VERTEX_FORMAT',
    },
  };

  const elevatedRect = {
    id: 'inspector-elevated-rectangle',
    name: 'Blue Elevated Extruded Rectangle Geometry',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-100.0, 30.0, -93.0, 37.0],
      height: 100000.0,
    },
    color: [0.0, 0.0, 1.0, 1.0],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      height: 100000.0,
      appearance: 'PerInstanceColorAppearance',
    },
  };

  const inspectorMarker = {
    id: 'inspector-billboard-marker',
    name: 'Cesium Logo Billboard Diagnostic Marker',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 150000.0],
    shape: {
      type: 'ellipsoid',
      radii: [75000.0, 75000.0, 75000.0],
    },
    color: [0.15, 0.85, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      image: '../images/Cesium_Logo_overlay.png',
      collection: 'BillboardCollection',
    },
  };

  const terrainRefCylinder = {
    id: 'inspector-terrain-reference',
    name: 'Depth Test Against Terrain Reference Cylinder',
    show: true,
    positionDegrees: [-88.0, 36.0, 120000.0],
    shape: {
      type: 'cylinder',
      length: 240000.0,
      topRadius: 60000.0,
      bottomRadius: 60000.0,
    },
    color: [0.95, 0.65, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      depthTestAgainstTerrain: true,
    },
  };

  const entities = [primitiveBox, elevatedRect, inspectorMarker, terrainRefCylinder];

  const variants = [
    {
      label: 'Inspector Diagnostics: Wireframe & Primitives',
      entities: [
        primitiveBox,
        elevatedRect,
        inspectorMarker,
        terrainRefCylinder,
      ],
    },
    {
      label: 'Inspect Translucent Box Primitive Geometry',
      entities: [
        { ...primitiveBox, color: [1.0, 0.3, 0.3, 0.85] },
        elevatedRect,
        inspectorMarker,
        terrainRefCylinder,
      ],
    },
    {
      label: 'Inspect Elevated Rectangle Extruded Surface',
      entities: [
        primitiveBox,
        { ...elevatedRect, color: [0.2, 0.5, 1.0, 0.9] },
        inspectorMarker,
        terrainRefCylinder,
      ],
    },
    {
      label: 'Inspect Billboard Overlay & Depth Buffer',
      entities: [
        primitiveBox,
        elevatedRect,
        { ...inspectorMarker, color: [0.95, 0.85, 0.2, 0.95] },
        terrainRefCylinder,
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

writeCase('aerometrex-san-francisco', generateAerometrexSanFrancisco());
writeCase('arcgis-tiled-elevation-terrain', generateArcgisTiledElevation());
writeCase('arcticdem', generateArcticDem());
writeCase('cesium-inspector', generateCesiumInspector());
