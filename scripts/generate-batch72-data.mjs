import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. glTF Planar Fill - Dev
function generateGltfPlanarFillDev() {
  const panelBox = {
    id: 'gltf-planar-panel-box',
    name: 'Planar Polygon Fill Structural Panel',
    show: true,
    positionDegrees: [0.0, 0.0, 80.0],
    shape: {
      type: 'box',
      dimensions: [24.0, 24.0, 36.0],
    },
    color: [0.95, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'BENTLEY_materials_planar_fill',
      wireframeFill: 'NONE',
      backgroundFill: false,
      behind: false,
      featureId: 42,
    },
  };

  const sketchCylinder = {
    id: 'gltf-planar-sketch-cylinder',
    name: 'Planar Hatching & Sketch Geometry Pillar',
    show: true,
    positionDegrees: [0.0003, 0.0003, 70.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 12.0,
      bottomRadius: 12.0,
    },
    color: [0.1, 0.85, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'BENTLEY_materials_planar_fill',
      wireframeFill: 'ALWAYS',
      backgroundFill: false,
      behind: true,
      featureId: 99,
    },
  };

  const probeSphere = {
    id: 'gltf-planar-probe-sphere',
    name: 'CAD Silhouette Background Infill Probe',
    show: true,
    positionDegrees: [-0.0003, -0.0003, 60.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.1, 0.6, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'BENTLEY_materials_planar_fill',
      wireframeFill: 'TOGGLE',
      backgroundFill: true,
      behind: false,
    },
  };

  const backdropRect = {
    id: 'gltf-planar-backdrop-rect',
    name: 'Planar Fill Calibration Testbed Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-0.001, -0.001, 0.001, 0.001],
    },
    color: [0.15, 0.15, 0.2, 0.4],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.4, 0.5, 0.8],
    outlineWidth: 1.0,
    properties: {
      edgeDisplayMode: 'SURFACES_AND_EDGES',
      backdrop: true,
    },
  };

  const entities = [panelBox, sketchCylinder, probeSphere, backdropRect];

  const variants = [
    {
      label: 'Baseline: Multi-Configuration Planar Fill CAD Display (Surfaces & Edges)',
      entities: [
        panelBox,
        sketchCylinder,
        probeSphere,
        backdropRect,
      ],
    },
    {
      label: 'Pure Wireframe Fill Diagnostic Mode (Wireframe Only)',
      entities: [
        { ...panelBox, color: [0.95, 0.85, 0.15, 0.95], outlineColor: [1.0, 0.95, 0.5, 1.0] },
        sketchCylinder,
        probeSphere,
        backdropRect,
      ],
    },
    {
      label: 'Background Fill Masking Mode (Color Coded Silhouette)',
      entities: [
        panelBox,
        { ...sketchCylinder, color: [0.95, 0.3, 0.7, 0.95], outlineColor: [1.0, 0.65, 0.85, 1.0] },
        probeSphere,
        backdropRect,
      ],
    },
    {
      label: 'Coplanar Behind Layer Priority Mode (Infill Behind Outlines)',
      entities: [
        panelBox,
        sketchCylinder,
        { ...probeSphere, color: [0.2, 0.9, 0.75, 0.95], outlineColor: [0.65, 1.0, 0.9, 1.0] },
        { ...backdropRect, color: [0.25, 0.2, 0.3, 0.4] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Google 2D Tiles
function generateGoogle2dTiles() {
  const phillyBox = {
    id: 'google-2d-philly-box',
    name: 'Philadelphia City Hall Center Benchmark',
    show: true,
    positionDegrees: [-75.1652, 39.9526, 120.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.2, 0.6, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      assetId: 3830184,
      imageryProvider: 'Google2DImageryProvider',
    },
  };

  const towerCylinder = {
    id: 'google-2d-tower-cylinder',
    name: 'Comcast Technology Center Tower Column',
    show: true,
    positionDegrees: [-75.1640, 39.9535, 100.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.95, 0.7, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      geocoder: 'GOOGLE',
      terrainAssetId: 1,
    },
  };

  const centerSphere = {
    id: 'google-2d-center-sphere',
    name: 'Market Street Urban Navigational Probe',
    show: true,
    positionDegrees: [-75.1665, 39.9515, 80.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.3, 0.85, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      keepExpanded: true,
      coordinateSystem: 'WGS84',
    },
  };

  const viewportRect = {
    id: 'google-2d-viewport-rect',
    name: 'Google 2D Tiles Philadelphia Metropolitan Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.280, 39.867, -74.956, 40.138],
    },
    color: [0.15, 0.3, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      southWest: [-75.280266, 39.867004],
      northEast: [-74.955763, 40.137992],
    },
  };

  const entities = [phillyBox, towerCylinder, centerSphere, viewportRect];

  const variants = [
    {
      label: 'Baseline: Google 2D Global Photorealistic Imagery (Philadelphia Focus)',
      entities: [
        phillyBox,
        towerCylinder,
        centerSphere,
        viewportRect,
      ],
    },
    {
      label: 'High-Zoom Urban Core Inspection Mode (Emerald Landmark Highlights)',
      entities: [
        { ...phillyBox, color: [0.2, 0.9, 0.5, 0.95], outlineColor: [0.65, 1.0, 0.8, 1.0] },
        towerCylinder,
        centerSphere,
        viewportRect,
      ],
    },
    {
      label: 'Delaware River Waterfront Shading (Azure Hydrological Tints)',
      entities: [
        phillyBox,
        { ...towerCylinder, color: [0.15, 0.5, 0.95, 0.95], outlineColor: [0.6, 0.8, 1.0, 1.0] },
        centerSphere,
        viewportRect,
      ],
    },
    {
      label: 'Regional Tri-State Overview Perspective (Amber Bound Envelope)',
      entities: [
        phillyBox,
        towerCylinder,
        { ...centerSphere, color: [0.95, 0.45, 0.15, 0.95], outlineColor: [1.0, 0.75, 0.5, 1.0] },
        { ...viewportRect, color: [0.35, 0.25, 0.15, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Google 2D Tiles with Custom Styles
function generateGoogle2dTilesWithCustomStyles() {
  const satBox = {
    id: 'google-custom-sat-box',
    name: 'Satellite Base Imagery Calibrated Block',
    show: true,
    positionDegrees: [-75.1652, 39.9526, 120.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.1, 0.75, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.95, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      assetId: 3830184,
      mapType: 'satellite',
    },
  };

  const roadCylinder = {
    id: 'google-custom-road-cylinder',
    name: 'Cyan Roadmap Overlay Vector Column',
    show: true,
    positionDegrees: [-75.1640, 39.9535, 100.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.95, 0.85, 0.1, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      overlayLayerType: 'layerRoadmap',
      hue: '#00ffe6',
      saturation: -20,
    },
  };

  const accentSphere = {
    id: 'google-custom-accent-sphere',
    name: 'Simplified Road Geometry Stylized Marker',
    show: true,
    positionDegrees: [-75.1665, 39.9515, 80.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.95, 0.35, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      featureType: 'road',
      elementType: 'geometry',
      lightness: 100,
      visibility: 'simplified',
    },
  };

  const overlayRect = {
    id: 'google-custom-overlay-rect',
    name: 'Styled Roadmap & Satellite Overlay Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.280, 39.867, -74.956, 40.138],
    },
    color: [0.1, 0.35, 0.3, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.75, 0.7, 0.8],
    outlineWidth: 1.0,
    properties: {
      stylesApplied: 2,
      destination: 'Philadelphia',
    },
  };

  const entities = [satBox, roadCylinder, accentSphere, overlayRect];

  const variants = [
    {
      label: 'Baseline: Satellite Base with Cyan Road Network Overlay (Hue & Saturation)',
      entities: [
        satBox,
        roadCylinder,
        accentSphere,
        overlayRect,
      ],
    },
    {
      label: 'Simplified High-Lightness Roadway Geometry (Pure White Vectors)',
      entities: [
        { ...satBox, color: [0.95, 0.95, 0.98, 0.95], outlineColor: [1.0, 1.0, 1.0, 1.0] },
        roadCylinder,
        accentSphere,
        overlayRect,
      ],
    },
    {
      label: 'Monochrome High-Contrast Cartographic Infill (Amber Transit Accents)',
      entities: [
        satBox,
        { ...roadCylinder, color: [0.95, 0.55, 0.1, 0.95], outlineColor: [1.0, 0.8, 0.4, 1.0] },
        accentSphere,
        overlayRect,
      ],
    },
    {
      label: 'Night Mode Darkened Base with Vibrant Highway Veins (Crimson Arterials)',
      entities: [
        satBox,
        roadCylinder,
        { ...accentSphere, color: [0.9, 0.15, 0.25, 0.95], outlineColor: [1.0, 0.5, 0.6, 1.0] },
        { ...overlayRect, color: [0.25, 0.15, 0.2, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Google Earth Enterprise
function generateGoogleEarthEnterprise() {
  const serverBox = {
    id: 'gee-server-node-box',
    name: 'Google Earth Enterprise Server Node Benchmark',
    show: true,
    positionDegrees: [-122.4194, 37.7749, 100.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 80.0],
    },
    color: [0.25, 0.5, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      serverUrl: 'http://www.earthenterprise.org/3d',
      proxy: '/proxy/',
      metadataClass: 'GoogleEarthEnterpriseMetadata',
    },
  };

  const meshCylinder = {
    id: 'gee-terrain-mesh-cylinder',
    name: 'GEE Terrain Provider Surface Normal Pillar',
    show: true,
    positionDegrees: [-122.4180, 37.7760, 90.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.3, 0.8, 0.4, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'GoogleEarthEnterpriseTerrainProvider',
      fromMetadata: true,
    },
  };

  const probeSphere = {
    id: 'gee-imagery-probe-sphere',
    name: 'GEE Imagery Layer & Black Marble Sampling Sphere',
    show: true,
    positionDegrees: [-122.4210, 37.7735, 80.0],
    shape: {
      type: 'ellipsoid',
      radii: [16.0, 16.0, 16.0],
    },
    color: [0.95, 0.7, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      imageryProvider: 'GoogleEarthEnterpriseImageryProvider',
      layerType: 'blackMarble',
    },
  };

  const bayareaRect = {
    id: 'gee-bayarea-coverage-rect',
    name: 'San Francisco Bay Regional Coverage Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.0, 36.0, -121.7, 39.0],
    },
    color: [0.15, 0.25, 0.4, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.6, 0.8, 0.8],
    outlineWidth: 1.0,
    properties: {
      west: -123.0,
      south: 36.0,
      east: -121.7,
      north: 39.0,
    },
  };

  const entities = [serverBox, meshCylinder, probeSphere, bayareaRect];

  const variants = [
    {
      label: 'Baseline: GEE Terrain & Black Marble Imagery Service (San Francisco Bay)',
      entities: [
        serverBox,
        meshCylinder,
        probeSphere,
        bayareaRect,
      ],
    },
    {
      label: 'Enterprise Elevation Mesh Wireframe Mode (Emerald Topography)',
      entities: [
        { ...serverBox, color: [0.2, 0.85, 0.55, 0.95], outlineColor: [0.65, 1.0, 0.85, 1.0] },
        meshCylinder,
        probeSphere,
        bayareaRect,
      ],
    },
    {
      label: 'Night Imagery Layer Amplification (Golden Urban Glow)',
      entities: [
        serverBox,
        { ...meshCylinder, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.95, 0.6, 1.0] },
        probeSphere,
        bayareaRect,
      ],
    },
    {
      label: 'High-Contrast Coastal Enterprise Boundary (Crimson Extent Outline)',
      entities: [
        serverBox,
        meshCylinder,
        { ...probeSphere, color: [0.9, 0.25, 0.35, 0.95], outlineColor: [1.0, 0.6, 0.7, 1.0] },
        { ...bayareaRect, color: [0.3, 0.15, 0.2, 0.35] },
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

writeRecipe('gltf-planar-fill-dev', generateGltfPlanarFillDev());
writeRecipe('google-2d-tiles', generateGoogle2dTiles());
writeRecipe('google-2d-tiles-with-custom-styles', generateGoogle2dTilesWithCustomStyles());
writeRecipe('google-earth-enterprise', generateGoogleEarthEnterprise());
