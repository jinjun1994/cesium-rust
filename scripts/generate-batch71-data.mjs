import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Globe Materials – 3D Tiles Terrain (Mount Everest / Himalayas)
function generateGlobeMaterials3dTilesTerrain() {
  const summitBox = {
    id: 'globe-mat-3dtiles-summit-box',
    name: 'Mount Everest Hypsometric Summit Benchmark',
    show: true,
    positionDegrees: [86.9250, 27.9880, 8848.0],
    shape: {
      type: 'box',
      dimensions: [32.0, 32.0, 48.0],
    },
    color: [0.95, 0.95, 0.98, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      fabricType: 'ElevationLand',
      elevationRamp: 'AlpineSnow',
    },
  };

  const valleyCylinder = {
    id: 'globe-mat-3dtiles-valley-cylinder',
    name: 'Glacial Valley Land-Water Separation Column',
    show: true,
    positionDegrees: [86.9242, 27.9890, 5200.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0,
    },
    color: [0.35, 0.55, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      waterMask: true,
      alpha: '1.0 - waterMask.alpha',
    },
  };

  const hillshadeProbe = {
    id: 'globe-mat-3dtiles-hillshade-probe',
    name: 'Directional Hillshade Surface Normal Probe',
    show: true,
    positionDegrees: [86.9260, 27.9870, 6000.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.95, 0.75, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      light: 'DirectionalLight',
      vertexNormals: true,
    },
  };

  const extentRect = {
    id: 'globe-mat-3dtiles-extent-rect',
    name: '3D Tiles Terrain Hypsometric Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.920, 27.984, 86.930, 27.992],
    },
    color: [0.2, 0.35, 0.25, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.75, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'Cesium3DTilesTerrainProvider',
      ionAssetId: 3923568,
    },
  };

  const entities = [summitBox, valleyCylinder, hillshadeProbe, extentRect];

  const variants = [
    {
      label: 'Baseline: ElevationLand Shading with Water Mask Separation (Hillshade)',
      entities: [
        summitBox,
        valleyCylinder,
        hillshadeProbe,
        extentRect,
      ],
    },
    {
      label: 'Inverse Water Mask Land Infill (Emerald High-Contrast Ramp)',
      entities: [
        { ...summitBox, color: [0.2, 0.85, 0.4, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        valleyCylinder,
        hillshadeProbe,
        extentRect,
      ],
    },
    {
      label: 'Alpine Elevation Ramp Accent (Amber Snowcap Highlight)',
      entities: [
        summitBox,
        { ...valleyCylinder, color: [0.95, 0.65, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        hillshadeProbe,
        extentRect,
      ],
    },
    {
      label: 'Unshaded Raw 3D Tiles Terrain Surface (Clean Normal Profile)',
      entities: [
        summitBox,
        valleyCylinder,
        { ...hillshadeProbe, color: [0.4, 0.6, 0.95, 0.95], outlineColor: [0.75, 0.85, 1.0, 1.0] },
        { ...extentRect, color: [0.15, 0.2, 0.3, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Globe Materials – Water Mask Elevation Map (Grand Canyon Colorado Plateau)
function generateGlobeMaterialsWaterMaskElevationMap() {
  const plateauBox = {
    id: 'globe-water-plateau-box',
    name: 'Colorado Plateau Hypsometric Anchor',
    show: true,
    positionDegrees: [-112.1120, 36.0540, 2150.0],
    shape: {
      type: 'box',
      dimensions: [36.0, 36.0, 50.0],
    },
    color: [0.85, 0.45, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      minHeight: -414.0,
      maxHeight: 8777.0,
    },
  };

  const riverCylinder = {
    id: 'globe-water-river-cylinder',
    name: 'Colorado River Water Mask Channel Column',
    show: true,
    positionDegrees: [-112.1110, 36.0550, 750.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 16.0,
      bottomRadius: 16.0,
    },
    color: [0.15, 0.65, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      riverWaterMask: true,
      landAlpha: 1.0,
    },
  };

  const hypsometricProbe = {
    id: 'globe-water-hypsometric-probe',
    name: 'Hypsometric Elevation Ramp Calibration Probe',
    show: true,
    positionDegrees: [-112.1132, 36.0532, 1500.0],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 15.0, 15.0],
    },
    color: [0.9, 0.8, 0.3, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      linearGradientRamp: 6,
    },
  };

  const canyonRect = {
    id: 'globe-water-canyon-rect',
    name: 'Grand Canyon Hypsometric Inspection Area',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-112.118, 36.050, -112.106, 36.058],
    },
    color: [0.25, 0.2, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.5, 0.8, 0.8],
    outlineWidth: 1.0,
    properties: {
      terrain: 'WorldTerrain',
      requestWaterMask: true,
    },
  };

  const entities = [plateauBox, riverCylinder, hypsometricProbe, canyonRect];

  const variants = [
    {
      label: 'Baseline: Elevation Land Color Ramp with Ocean Alpha Masking',
      entities: [
        plateauBox,
        riverCylinder,
        hypsometricProbe,
        canyonRect,
      ],
    },
    {
      label: 'Hypsometric Tinting Mode (Vibrant Continental Gradient)',
      entities: [
        { ...plateauBox, color: [0.95, 0.3, 0.45, 0.95], outlineColor: [1.0, 0.7, 0.8, 1.0] },
        riverCylinder,
        hypsometricProbe,
        canyonRect,
      ],
    },
    {
      label: 'High-Altitude Snowcap Isolation (Alpine White Peak Infill)',
      entities: [
        plateauBox,
        { ...riverCylinder, color: [0.85, 0.95, 1.0, 0.95], outlineColor: [1.0, 1.0, 1.0, 1.0] },
        hypsometricProbe,
        canyonRect,
      ],
    },
    {
      label: 'Coastal Bathymetry & Water Surface Boundary (Cyan Shoreline)',
      entities: [
        plateauBox,
        riverCylinder,
        { ...hypsometricProbe, color: [0.15, 0.85, 0.75, 0.95], outlineColor: [0.65, 1.0, 0.95, 1.0] },
        { ...canyonRect, color: [0.1, 0.25, 0.4, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Globe Translucency (Seattle Subsurface Utility Network)
function generateGlobeTranslucency() {
  const stationBox = {
    id: 'globe-trans-station-box',
    name: 'Underground Transit Terminal Vault',
    show: true,
    positionDegrees: [-122.3320, 47.6060, -25.0],
    shape: {
      type: 'box',
      dimensions: [30.0, 30.0, 24.0],
    },
    color: [0.95, 0.3, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      subsurfaceDepth: -25.0,
      frontFaceAlpha: 0.5,
    },
  };

  const tunnelCylinder = {
    id: 'globe-trans-tunnel-cylinder',
    name: 'Deep Tunnel Utility Corridor Anchor',
    show: true,
    positionDegrees: [-122.3312, 47.6068, -40.0],
    shape: {
      type: 'cylinder',
      length: 45.0,
      topRadius: 12.0,
      bottomRadius: 12.0,
    },
    color: [0.2, 0.75, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      backFaceAlpha: 0.5,
      undergroundColor: 'transparent',
    },
  };

  const shaftProbe = {
    id: 'globe-trans-shaft-probe',
    name: 'Vertical Geotechnical Access Shaft Probe',
    show: true,
    positionDegrees: [-122.3328, 47.6052, -15.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 12.0, 12.0],
    },
    color: [0.95, 0.75, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      collisionDetection: false,
    },
  };

  const corridorRect = {
    id: 'globe-trans-corridor-rect',
    name: 'Seattle Translucent Surface Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.336, 47.602, -122.328, 47.610],
    },
    color: [0.15, 0.3, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.65, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      translucencyEnabled: true,
    },
  };

  const entities = [stationBox, tunnelCylinder, shaftProbe, corridorRect];

  const variants = [
    {
      label: 'Baseline: Semi-Transparent Globe Surface (50% Alpha Subsurface)',
      entities: [
        stationBox,
        tunnelCylinder,
        shaftProbe,
        corridorRect,
      ],
    },
    {
      label: 'Near-Surface Inspection Window (High-Alpha Crust Translucency)',
      entities: [
        { ...stationBox, color: [0.85, 0.2, 0.9, 0.95], outlineColor: [1.0, 0.6, 1.0, 1.0] },
        tunnelCylinder,
        shaftProbe,
        corridorRect,
      ],
    },
    {
      label: 'Deep Underground Borehole Projection (Neon Amber Core)',
      entities: [
        stationBox,
        { ...tunnelCylinder, color: [0.95, 0.55, 0.1, 0.95], outlineColor: [1.0, 0.85, 0.4, 1.0] },
        shaftProbe,
        corridorRect,
      ],
    },
    {
      label: 'Opaque Surface Default View (Solid Terrain Boundary)',
      entities: [
        stationBox,
        tunnelCylinder,
        { ...shaftProbe, color: [0.25, 0.85, 0.45, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        { ...corridorRect, color: [0.2, 0.35, 0.2, 0.3] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. glTF PBR Next (Clear Coat, Specular, Anisotropy & IBL)
function generateGltfPbrExtensions() {
  const clearcoatBox = {
    id: 'gltf-pbr-clearcoat-box',
    name: 'Clear Coat Multilayer Varnish Structural Box',
    show: true,
    positionDegrees: [-123.0740, 44.0504, 25.0],
    shape: {
      type: 'box',
      dimensions: [28.0, 28.0, 36.0],
    },
    color: [0.85, 0.6, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'KHR_materials_clearcoat',
      assetId: 2584329,
    },
  };

  const anisotropyCylinder = {
    id: 'gltf-pbr-anisotropy-cylinder',
    name: 'Brushed Metal Anisotropic Microfacet Pillar',
    show: true,
    positionDegrees: [-123.0732, 44.0510, 20.0],
    shape: {
      type: 'cylinder',
      length: 35.0,
      topRadius: 14.0,
      bottomRadius: 14.0,
    },
    color: [0.3, 0.85, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'KHR_materials_anisotropy',
      iblCoefficients: 9,
    },
  };

  const specularSphere = {
    id: 'gltf-pbr-specular-sphere',
    name: 'Dielectric Specular Transmission Test Sphere',
    show: true,
    positionDegrees: [-123.0748, 44.0498, 18.0],
    shape: {
      type: 'ellipsoid',
      radii: [14.0, 14.0, 14.0],
    },
    color: [0.8, 0.3, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      extension: 'KHR_materials_specular',
      environmentMap: true,
    },
  };

  const studioRect = {
    id: 'gltf-pbr-studio-rect',
    name: 'PBR Next Material Calibration Studio Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.078, 44.047, -123.070, 44.054],
    },
    color: [0.2, 0.25, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.65, 0.85, 0.8],
    outlineWidth: 1.0,
    properties: {
      lighting: 'ImageBasedLighting',
      sphericalHarmonics: true,
    },
  };

  const entities = [clearcoatBox, anisotropyCylinder, specularSphere, studioRect];

  const variants = [
    {
      label: 'Baseline: Clear Coat PBR Next Material (Spherical Harmonics IBL)',
      entities: [
        clearcoatBox,
        anisotropyCylinder,
        specularSphere,
        studioRect,
      ],
    },
    {
      label: 'Specular Environment Refraction Mode (Vibrant Gloss Profile)',
      entities: [
        { ...clearcoatBox, color: [0.95, 0.4, 0.2, 0.95], outlineColor: [1.0, 0.7, 0.5, 1.0] },
        anisotropyCylinder,
        specularSphere,
        studioRect,
      ],
    },
    {
      label: 'Metallic Anisotropy Brushed Surface (Amber Radiant Highlight)',
      entities: [
        clearcoatBox,
        { ...anisotropyCylinder, color: [0.95, 0.7, 0.1, 0.95], outlineColor: [1.0, 0.9, 0.5, 1.0] },
        specularSphere,
        studioRect,
      ],
    },
    {
      label: 'Direct Sunlight Only Lighting Mode (High-Contrast Facet Shading)',
      entities: [
        clearcoatBox,
        anisotropyCylinder,
        { ...specularSphere, color: [0.2, 0.7, 0.95, 0.95], outlineColor: [0.65, 0.9, 1.0, 1.0] },
        { ...studioRect, color: [0.35, 0.25, 0.2, 0.3] },
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

writeRecipe('globe-materials-3d-tiles-terrain', generateGlobeMaterials3dTilesTerrain());
writeRecipe('globe-materials-water-mask-elevation-map', generateGlobeMaterialsWaterMaskElevationMap());
writeRecipe('globe-translucency', generateGlobeTranslucency());
writeRecipe('gltf-pbr-extensions', generateGltfPbrExtensions());
