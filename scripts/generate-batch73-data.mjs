import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Google Photorealistic 3D Tiles
function generateGooglePhotorealistic3dTiles() {
  const hqBox = {
    id: 'google-plex-hq-box',
    name: 'Googleplex Headquarters Main Facility',
    show: true,
    positionDegrees: [-122.0840, 37.4220, 30.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 60.0],
    },
    color: [0.95, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      photorealistic: true,
      geocoder: 'GOOGLE',
      globe: false,
    },
  };

  const towerCylinder = {
    id: 'google-plex-tower-cylinder',
    name: 'Googleplex Charleston Campus Atrium Column',
    show: true,
    positionDegrees: [-122.0825, 37.4230, 25.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 16.0,
      bottomRadius: 16.0,
    },
    color: [0.2, 0.6, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      photorealisticTileset: 'GooglePhotorealistic3DTileset',
      skyAtmosphere: true,
    },
  };

  const campusSphere = {
    id: 'google-plex-campus-sphere',
    name: 'Mountain View Photorealistic Geodetic Target',
    show: true,
    positionDegrees: [-122.0855, 37.4210, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.95, 0.75, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      ecefCoordinates: [-2693797.55, -4297135.52, 3854700.75],
    },
  };

  const extentRect = {
    id: 'google-plex-extent-rect',
    name: 'Googleplex Campus Photorealistic Ground Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.095, 37.415, -122.073, 37.430],
    },
    color: [0.2, 0.35, 0.25, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.75, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      location: 'Mountain View, CA',
    },
  };

  const entities = [hqBox, towerCylinder, campusSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Photorealistic 3D Tiles of Googleplex HQ (Photorealistic Mesh)',
      entities: [
        hqBox,
        towerCylinder,
        campusSphere,
        extentRect,
      ],
    },
    {
      label: 'Sunlit Aerial Inspection Perspective (High-Luminance Façades)',
      entities: [
        { ...hqBox, color: [0.95, 0.85, 0.2, 0.95], outlineColor: [1.0, 0.95, 0.6, 1.0] },
        towerCylinder,
        campusSphere,
        extentRect,
      ],
    },
    {
      label: 'Silicon Valley Metro Foothill Elevation (Azure Ambient Tone)',
      entities: [
        hqBox,
        { ...towerCylinder, color: [0.15, 0.75, 0.85, 0.95], outlineColor: [0.6, 0.95, 1.0, 1.0] },
        campusSphere,
        extentRect,
      ],
    },
    {
      label: 'High-Contrast Architectural Boundary (Amber Campus Envelope)',
      entities: [
        hqBox,
        towerCylinder,
        { ...campusSphere, color: [0.95, 0.45, 0.15, 0.95], outlineColor: [1.0, 0.75, 0.5, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.15, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Google Photorealistic 3D Tiles with Building Insert
function generateGooglePhotorealistic3dTilesWithBuildingInsert() {
  const buildingBox = {
    id: 'google-insert-building-box',
    name: 'Proposed High-Rise Building Insert Model',
    show: true,
    positionDegrees: [-105.0084, 39.7508, 1620.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 100.0],
    },
    color: [0.95, 0.85, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      buildingAssetId: 1670818,
      proposedBuilding: true,
    },
  };

  const lotCylinder = {
    id: 'google-insert-lot-cylinder',
    name: 'Development Lot Core Tower Column',
    show: true,
    positionDegrees: [-105.0075, 39.7515, 1600.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.25, 0.85, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      targetLot: true,
    },
  };

  const targetSphere = {
    id: 'google-insert-target-sphere',
    name: 'Denver Urban Planning Centroid Marker',
    show: true,
    positionDegrees: [-105.0092, 39.7501, 1590.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.95, 0.35, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      city: 'Denver',
      state: 'Colorado',
    },
  };

  const envelopeRect = {
    id: 'google-insert-envelope-rect',
    name: 'Denver Downtown Development Parcel Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.012, 39.747, -105.005, 39.754],
    },
    color: [0.25, 0.3, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      southWest: [-105.012, 39.747],
      northEast: [-105.005, 39.754],
    },
  };

  const entities = [buildingBox, lotCylinder, targetSphere, envelopeRect];

  const variants = [
    {
      label: 'Baseline: Proposed High-Rise Tower Insert with Classified Lot (Yellow Target)',
      entities: [
        buildingBox,
        lotCylinder,
        targetSphere,
        envelopeRect,
      ],
    },
    {
      label: 'Isolated Building Volume Visibility Mode (Opaque Architectural Insert)',
      entities: [
        { ...buildingBox, color: [0.2, 0.7, 0.95, 0.95], outlineColor: [0.6, 0.9, 1.0, 1.0] },
        lotCylinder,
        targetSphere,
        envelopeRect,
      ],
    },
    {
      label: 'Target Development Lot Highlight Mode (Amber Boundary Envelope)',
      entities: [
        buildingBox,
        { ...lotCylinder, color: [0.95, 0.55, 0.15, 0.95], outlineColor: [1.0, 0.8, 0.5, 1.0] },
        targetSphere,
        envelopeRect,
      ],
    },
    {
      label: 'Downtown Denver Urban Horizon Perspective (Azure City Context)',
      entities: [
        buildingBox,
        lotCylinder,
        { ...targetSphere, color: [0.35, 0.9, 0.65, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        { ...envelopeRect, color: [0.3, 0.2, 0.35, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Google Street View Panorama
function generateGoogleStreetViewPanorama() {
  const cameraBox = {
    id: 'streetview-pano-camera-box',
    name: 'Google Street View Camera Rig Base Node',
    show: true,
    positionDegrees: [-122.4194, 37.7749, 40.0],
    shape: {
      type: 'box',
      dimensions: [30.0, 30.0, 60.0],
    },
    color: [0.2, 0.6, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      viewType: 'PanoView',
      provider: 'GoogleStreetViewCubeMapPanoramaProvider',
    },
  };

  const podCylinder = {
    id: 'streetview-pano-pod-cylinder',
    name: 'Street View Sensor Array Mast Pillar',
    show: true,
    positionDegrees: [-122.4180, 37.7760, 35.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 14.0,
      bottomRadius: 14.0,
    },
    color: [0.95, 0.7, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      overlayLayerType: 'layerStreetview',
      searchRadiusMeters: 5,
    },
  };

  const bubbleSphere = {
    id: 'streetview-pano-bubble-sphere',
    name: '360-Degree Panorama Equirectangular Bubble',
    show: true,
    positionDegrees: [-122.4210, 37.7735, 30.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 18.0, 18.0],
    },
    color: [0.25, 0.85, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      cubeMapFaces: 6,
      headingTolerance: 0.05,
    },
  };

  const footprintRect = {
    id: 'streetview-pano-footprint-rect',
    name: 'Street View Urban Navigational Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.430, 37.765, -122.408, 37.785],
    },
    color: [0.15, 0.25, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.6, 0.75, 0.8],
    outlineWidth: 1.0,
    properties: {
      location: 'San Francisco Street View Coverage',
    },
  };

  const entities = [cameraBox, podCylinder, bubbleSphere, footprintRect];

  const variants = [
    {
      label: 'Baseline: 360-Degree Street View Panorama Bubble (Equirectangular Projection)',
      entities: [
        cameraBox,
        podCylinder,
        bubbleSphere,
        footprintRect,
      ],
    },
    {
      label: 'CubeMap Panorama Multi-Face Perspective (Directional Street Facets)',
      entities: [
        { ...cameraBox, color: [0.95, 0.85, 0.15, 0.95], outlineColor: [1.0, 0.95, 0.5, 1.0] },
        podCylinder,
        bubbleSphere,
        footprintRect,
      ],
    },
    {
      label: 'High-Altitude Map and Panorama Seamless Transition (Azure Horizon Blend)',
      entities: [
        cameraBox,
        { ...podCylinder, color: [0.2, 0.85, 0.85, 0.95], outlineColor: [0.65, 1.0, 1.0, 1.0] },
        bubbleSphere,
        footprintRect,
      ],
    },
    {
      label: 'Pedestrian Roadway Wayfinding Overlay (Amber Navigational Conduit)',
      entities: [
        cameraBox,
        podCylinder,
        { ...bubbleSphere, color: [0.95, 0.45, 0.2, 0.95], outlineColor: [1.0, 0.75, 0.55, 1.0] },
        { ...footprintRect, color: [0.35, 0.2, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Ground Primitive - Dev
function generateGroundPrimitiveDev() {
  const redBox = {
    id: 'ground-prim-red-polygon-box',
    name: 'Overlapping Terrain Polygon Benchmark Box',
    show: true,
    positionDegrees: [-122.195, 46.200, 2000.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.95, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      groundPrimitive: true,
      classificationType: 'TERRAIN',
      colorAttribute: 'Color.RED',
    },
  };

  const greenCylinder = {
    id: 'ground-prim-green-corridor-cylinder',
    name: 'Terrain-Draped Corridor Surface Cylinder',
    show: true,
    positionDegrees: [-122.185, 46.208, 1900.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.2, 0.85, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      arcType: 'RHUMB',
      colorAttribute: 'Color.GREEN',
    },
  };

  const blueSphere = {
    id: 'ground-prim-blue-circle-sphere',
    name: 'Draped Circle Geometry Radius Sphere',
    show: true,
    positionDegrees: [-122.205, 46.192, 1850.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.2, 0.55, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      colorAttribute: 'Color.BLUE',
      withAlpha: true,
    },
  };

  const terrainRect = {
    id: 'ground-prim-terrain-rect',
    name: 'Mount St. Helens Ground Primitive Drape Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.22, 46.18, -122.17, 46.22],
    },
    color: [0.2, 0.35, 0.25, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.75, 0.5, 0.8],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'WorldTerrain',
    },
  };

  const entities = [redBox, greenCylinder, blueSphere, terrainRect];

  const variants = [
    {
      label: 'Baseline: Overlapping Translucent Ground Primitives (Red, Green, Blue Drape)',
      entities: [
        redBox,
        greenCylinder,
        blueSphere,
        terrainRect,
      ],
    },
    {
      label: 'Terrain-Clamped Polygon Geometry Mode (Pure Red Ground Fill)',
      entities: [
        { ...redBox, color: [0.95, 0.8, 0.15, 0.95], outlineColor: [1.0, 0.9, 0.5, 1.0] },
        greenCylinder,
        blueSphere,
        terrainRect,
      ],
    },
    {
      label: 'Rhumb Arc Corridor and Circle Ground Geometry (Emerald & Azure Primitives)',
      entities: [
        redBox,
        { ...greenCylinder, color: [0.15, 0.8, 0.8, 0.95], outlineColor: [0.6, 1.0, 1.0, 1.0] },
        blueSphere,
        terrainRect,
      ],
    },
    {
      label: 'High-Contrast Ground Primitive Outlines (Amber Boundary Diagnostics)',
      entities: [
        redBox,
        greenCylinder,
        { ...blueSphere, color: [0.95, 0.45, 0.1, 0.95], outlineColor: [1.0, 0.75, 0.4, 1.0] },
        { ...terrainRect, color: [0.35, 0.2, 0.2, 0.35] },
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

writeRecipe('google-photorealistic-3d-tiles', generateGooglePhotorealistic3dTiles());
writeRecipe('google-photorealistic-3d-tiles-with-building-insert', generateGooglePhotorealistic3dTilesWithBuildingInsert());
writeRecipe('google-streetview-panorama', generateGoogleStreetViewPanorama());
writeRecipe('ground-primitive-dev', generateGroundPrimitiveDev());
