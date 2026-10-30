import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Imagery Assets available from ion
function generateImageryAssetsAvailableFromIon() {
  const phillyBox = {
    id: 'ion-imagery-philly-box',
    name: 'Philadelphia City Hall Ion Imagery Landmark',
    show: true,
    positionDegrees: [-75.165, 39.952, 60.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 90.0],
    },
    color: [0.95, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      provider: 'IonImageryProvider',
      assetId: 3830182,
      layerName: 'Google Maps 2D Satellite',
    },
  };

  const azureCylinder = {
    id: 'ion-imagery-azure-cylinder',
    name: 'Azure Maps Roads Alignment Calibration Pillar',
    show: true,
    positionDegrees: [-75.162, 39.955, 45.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 7.5,
      bottomRadius: 7.5,
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      provider: 'AzureMapsRoads',
      assetId: 3891169,
    },
  };

  const sentinelSphere = {
    id: 'ion-imagery-sentinel-sphere',
    name: 'Sentinel-2 Multispectral Radiance Sensor Probe',
    show: true,
    positionDegrees: [-75.168, 39.950, 55.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.35, 0.9, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      provider: 'Sentinel-2',
      assetId: 3954,
    },
  };

  const extentRect = {
    id: 'ion-imagery-philly-rect',
    name: 'Philadelphia Metropolitan Extent Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.280, 39.867, -74.956, 40.138],
    },
    color: [0.25, 0.3, 0.4, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.65, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      metroArea: 'Greater Philadelphia',
    },
  };

  const entities = [phillyBox, azureCylinder, sentinelSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Google Maps 2D Satellite from Cesium ion (Philly Metropolitan Extent)',
      entities: [phillyBox, azureCylinder, sentinelSphere, extentRect],
    },
    {
      label: 'Azure Maps Aerial High-Resolution Inspection Mode (Urban Road Network)',
      entities: [
        { ...phillyBox, color: [0.25, 0.75, 0.95, 0.95], outlineColor: [0.7, 0.95, 1.0, 1.0] },
        azureCylinder,
        sentinelSphere,
        extentRect,
      ],
    },
    {
      label: 'Sentinel-2 Global Multispectral Surface Cover (Natural Color Band)',
      entities: [
        phillyBox,
        { ...azureCylinder, color: [0.35, 0.9, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        sentinelSphere,
        extentRect,
      ],
    },
    {
      label: 'Google Maps 2D Contour Elevation Topography (Topographic Contours)',
      entities: [
        phillyBox,
        azureCylinder,
        { ...sentinelSphere, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.9, 0.55, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Imagery Layers Split
function generateImageryLayersSplit() {
  const nightBox = {
    id: 'split-night-city-box',
    name: 'Earth at Night Radiance Split Anchor Box',
    show: true,
    positionDegrees: [31.235, 30.044, 80.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 50.0, 75.0],
    },
    color: [0.95, 0.85, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      splitDirection: 'LEFT',
      splitPosition: 0.5,
      nightImagery: 'EarthAtNight',
    },
  };

  const streetCylinder = {
    id: 'split-street-divider-cylinder',
    name: 'World Street Map Seam Divider Column',
    show: true,
    positionDegrees: [31.238, 30.047, 60.0],
    shape: {
      type: 'cylinder',
      length: 65.0,
      topRadius: 8.5,
      bottomRadius: 8.5,
    },
    color: [0.35, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      splitDirection: 'NONE',
      streetProvider: 'World_Street_Map',
    },
  };

  const sliderSphere = {
    id: 'split-slider-marker-sphere',
    name: 'Screen Space Split Slider Interactive Anchor',
    show: true,
    positionDegrees: [31.232, 30.041, 70.0],
    shape: {
      type: 'ellipsoid',
      radii: [7.0, 7.0, 7.0],
    },
    color: [0.95, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      sliderDragging: true,
      pinchInteractive: true,
    },
  };

  const splitRect = {
    id: 'split-nile-valley-rect',
    name: 'Nile Valley Imagery Split Comparison Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [31.100, 29.900, 31.400, 30.200],
    },
    color: [0.2, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.85, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      region: 'Cairo_Nile_Delta',
    },
  };

  const entities = [nightBox, streetCylinder, sliderSphere, splitRect];

  const variants = [
    {
      label: 'Baseline: Split Earth at Night & World Street Map (Midway Slider 50%)',
      entities: [nightBox, streetCylinder, sliderSphere, splitRect],
    },
    {
      label: 'Nighttime Radiance Dominated Left Split (Slider 80% Reveal)',
      entities: [
        { ...nightBox, color: [0.95, 0.7, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        streetCylinder,
        sliderSphere,
        splitRect,
      ],
    },
    {
      label: 'Daylight Cartographic Street Map Dominated (Slider 20% Reveal)',
      entities: [
        nightBox,
        { ...streetCylinder, color: [0.25, 0.85, 0.45, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        sliderSphere,
        splitRect,
      ],
    },
    {
      label: 'High-Contrast Continental Coastline Boundary (Diagnostic Split Seam)',
      entities: [
        nightBox,
        streetCylinder,
        { ...sliderSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...splitRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Imagery Layers Texture Filters
function generateImageryLayersTextureFilters() {
  const filterBox = {
    id: 'filter-linear-base-box',
    name: 'NaturalEarthII Bilinear Texture Anchor Box',
    show: true,
    positionDegrees: [-82.0, 45.0, 100.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 80.0],
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      minificationFilter: 'LINEAR',
      magnificationFilter: 'LINEAR',
    },
  };

  const nearestCylinder = {
    id: 'filter-nearest-pixel-cylinder',
    name: 'Nearest Neighbor Pixel Filtering Calibration Column',
    show: true,
    positionDegrees: [-81.9, 45.1, 80.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.95, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      minificationFilter: 'NEAREST',
      magnificationFilter: 'NEAREST',
      splitDirection: 'RIGHT',
    },
  };

  const seamSphere = {
    id: 'filter-texture-seam-sphere',
    name: 'Texture Filtering Interpolation Probe Sphere',
    show: true,
    positionDegrees: [-82.1, 44.9, 90.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.5, 6.5, 6.5],
    },
    color: [0.95, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      textureSampling: 'PointVsBilinear',
    },
  };

  const extentRect = {
    id: 'filter-great-lakes-rect',
    name: 'Great Lakes Michigan Texture Filter Evaluation Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-84.0, 43.0, -80.0, 47.0],
    },
    color: [0.2, 0.35, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.95, 0.85, 0.9],
    outlineWidth: 1.0,
    properties: {
      geographicArea: 'Great_Lakes_Basin',
    },
  };

  const entities = [filterBox, nearestCylinder, seamSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Split Linear Smooth vs Nearest Pixelated Filtering (Texture Filters)',
      entities: [filterBox, nearestCylinder, seamSphere, extentRect],
    },
    {
      label: 'Pure Nearest Neighbor Pixel Art Cartography Mode (Hard Pixel Boundary)',
      entities: [
        { ...filterBox, color: [0.95, 0.35, 0.35, 0.95], outlineColor: [1.0, 0.7, 0.7, 1.0] },
        nearestCylinder,
        seamSphere,
        extentRect,
      ],
    },
    {
      label: 'Pure Bilinear Filtering Smooth Gradient Mode (Anti-Aliased Interpolation)',
      entities: [
        filterBox,
        { ...nearestCylinder, color: [0.35, 0.9, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        seamSphere,
        extentRect,
      ],
    },
    {
      label: 'Great Lakes Shoreline High-Contrast Contrast Mode (Nyquist Texture Inspection)',
      entities: [
        filterBox,
        nearestCylinder,
        { ...seamSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. iModel Mesh Export Service
function generateIModelMeshExportService() {
  const imodelBox = {
    id: 'imodel-facility-core-box',
    name: 'Bentley iModel Industrial Facility Anchor Box',
    show: true,
    positionDegrees: [-75.620, 40.035, 80.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 35.0, 70.0],
    },
    color: [0.85, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      platform: 'Bentley_iTwin',
      hoverHighlight: '#eeff41',
      serviceType: 'MeshExportService',
    },
  };

  const steelCylinder = {
    id: 'imodel-steel-framing-cylinder',
    name: 'Structural Steel Framing Truss Column',
    show: true,
    positionDegrees: [-75.618, 40.038, 65.0],
    shape: {
      type: 'cylinder',
      length: 55.0,
      topRadius: 7.0,
      bottomRadius: 7.0,
    },
    color: [0.93, 1.0, 0.25, 0.95], // #eeff41
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      discipline: 'StructuralSteel',
      elementId: 'Truss_Column_A1',
    },
  };

  const tooltipSphere = {
    id: 'imodel-hover-tooltip-sphere',
    name: 'Feature Hover Tooltip Screen Space Probe',
    show: true,
    positionDegrees: [-75.622, 40.032, 75.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      htmlOverlay: 'backdrop',
      mouseoverPicking: true,
    },
  };

  const extentRect = {
    id: 'imodel-project-boundary-rect',
    name: 'Bentley Project Cadastral Boundary Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.635, 40.025, -75.605, 40.045],
    },
    color: [0.35, 0.35, 0.25, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.9, 0.35, 0.9],
    outlineWidth: 1.0,
    properties: {
      projectExtent: 'Exton_Industrial_Campus',
    },
  };

  const entities = [imodelBox, steelCylinder, tooltipSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Bentley iModel Infrastructure Mesh (BIM Architectural Elements)',
      entities: [imodelBox, steelCylinder, tooltipSphere, extentRect],
    },
    {
      label: 'Hover Yellow Feature Highlight Mode (#eeff41 Selection Tint)',
      entities: [
        { ...imodelBox, color: [0.93, 1.0, 0.25, 0.95], outlineColor: [1.0, 1.0, 0.7, 1.0] },
        steelCylinder,
        tooltipSphere,
        extentRect,
      ],
    },
    {
      label: 'Structural Steel Framing Isolation (Engineering Discipline Mesh)',
      entities: [
        imodelBox,
        { ...steelCylinder, color: [0.25, 0.85, 0.45, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        tooltipSphere,
        extentRect,
      ],
    },
    {
      label: 'Spatial Bounding Volume Extent Mode (Project Cadastral Footprint)',
      entities: [
        imodelBox,
        steelCylinder,
        { ...tooltipSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 76 fixtures
const datasets = {
  'imagery-assets-available-from-ion': generateImageryAssetsAvailableFromIon(),
  'imagery-layers-split': generateImageryLayersSplit(),
  'imagery-layers-texture-filters': generateImageryLayersTextureFilters(),
  'imodel-mesh-export-service': generateIModelMeshExportService(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
