import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Material with Custom GLSL
function generateMaterialWithCustomGlsl() {
  const customBox = {
    id: 'glsl-custom-material-box',
    name: 'Custom Fabric Shader Dynamic Appearance Box',
    show: true,
    positionDegrees: [114.502, 30.498, 50.0],
    shape: {
      type: 'box',
      dimensions: [25.0, 25.0, 25.0],
    },
    color: [1.0, 1.0, 0.0, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      fabricType: 'CustomBoxShader',
      materialCacheKey: 'my-box-material-appearance',
      boxSize: 25.0,
      rotateZ: true,
    },
  };

  const uniformCylinder = {
    id: 'glsl-uniform-time-cylinder',
    name: 'Fabric Uniform Time Cycle Calibration Pillar',
    show: true,
    positionDegrees: [114.505, 30.501, 40.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 6.0,
      bottomRadius: 6.0,
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      uniformVariable: 'time',
      frameNumber: 1.0,
    },
  };

  const phongSphere = {
    id: 'glsl-phong-lighting-sphere',
    name: 'czm_phong Custom FragColor Shading Probe',
    show: true,
    positionDegrees: [114.499, 30.495, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.0, 5.0, 5.0],
    },
    color: [0.95, 0.35, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      diffuseShader: 'czm_material.diffuse',
      alphaLineScan: true,
    },
  };

  const extentRect = {
    id: 'glsl-material-bounds-rect',
    name: 'Custom GLSL Appearance Cadastral Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [114.490, 30.485, 114.515, 30.510],
    },
    color: [0.25, 0.35, 0.3, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.9, 0.3, 0.9],
    outlineWidth: 1.0,
    properties: {
      eastNorthUpToFixedFrame: true,
    },
  };

  const entities = [customBox, uniformCylinder, phongSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Material Appearance with Custom Fabric GLSL (Yellow Base Color)',
      entities: [customBox, uniformCylinder, phongSphere, extentRect],
    },
    {
      label: 'Sinusoidal Time Cycle RGB Chromatic Modulation Mode (Dynamic CustomColor)',
      entities: [
        { ...customBox, color: [0.2, 0.95, 0.35, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        uniformCylinder,
        phongSphere,
        extentRect,
      ],
    },
    {
      label: 'Dual Scanning Bright Alpha Line Scan Bands (Transparent Cutoff Mode)',
      entities: [
        customBox,
        { ...uniformCylinder, color: [0.95, 0.55, 0.15, 0.95], outlineColor: [1.0, 0.8, 0.6, 1.0] },
        phongSphere,
        extentRect,
      ],
    },
    {
      label: 'Continuous Z-Axis Vertex Rotation Mode (mat3 rotateZ Angular Inspection)',
      entities: [
        customBox,
        uniformCylinder,
        { ...phongSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. Montreal Point Cloud
function generateMontrealPointCloud() {
  const downtownBox = {
    id: 'mtl-downtown-cloud-box',
    name: 'Montreal Downtown High-Density 20cm Point Cloud Box',
    show: true,
    positionDegrees: [-73.567, 45.502, 80.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 75.0],
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      resolution: '20cm',
      year: 2015,
      city: 'Montreal',
    },
  };

  const royalCylinder = {
    id: 'mtl-mount-royal-cylinder',
    name: 'Mount Royal Park Topography Datum Column',
    show: true,
    positionDegrees: [-73.564, 45.505, 65.0],
    shape: {
      type: 'cylinder',
      length: 65.0,
      topRadius: 7.5,
      bottomRadius: 7.5,
    },
    color: [0.3, 0.85, 0.5, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      poi: 'Mount Royal',
      terrain: 'WorldTerrain',
    },
  };

  const oldPortSphere = {
    id: 'mtl-old-port-sphere',
    name: 'Old Port Historic Waterfront Sensor Probe',
    show: true,
    positionDegrees: [-73.570, 45.499, 55.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.95, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      poi: 'Old Port of Montreal',
      stLawrenceRiver: true,
    },
  };

  const extentRect = {
    id: 'mtl-metro-cadastral-rect',
    name: 'Greater Montreal Point Cloud Cadastral Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-73.585, 45.490, -73.550, 45.515],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 0.75, 0.9],
    outlineWidth: 1.0,
    properties: {
      surveyArea: 'Metropolitan_Montreal_Lidar',
    },
  };

  const entities = [downtownBox, royalCylinder, oldPortSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Montreal 20cm Lidar Point Cloud (Overview Viewpoint)',
      entities: [downtownBox, royalCylinder, oldPortSphere, extentRect],
    },
    {
      label: 'Mount Royal Park Foliage Topography Classification Mode',
      entities: [
        { ...downtownBox, color: [0.35, 0.9, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        royalCylinder,
        oldPortSphere,
        extentRect,
      ],
    },
    {
      label: 'Old Port Historic Maritime Waterfront Close-Up FlyTo',
      entities: [
        downtownBox,
        { ...royalCylinder, color: [0.25, 0.85, 0.95, 0.95], outlineColor: [0.7, 0.95, 1.0, 1.0] },
        oldPortSphere,
        extentRect,
      ],
    },
    {
      label: 'St. Lawrence River Coastal Shoreline Survey Bounds',
      entities: [
        downtownBox,
        royalCylinder,
        { ...oldPortSphere, color: [0.85, 0.35, 0.95, 0.95], outlineColor: [1.0, 0.75, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. Moon
function generateMoon() {
  const apolloBox = {
    id: 'moon-apollo11-tranquility-box',
    name: 'Apollo 11 Sea of Tranquility Landmark Anchor Box',
    show: true,
    positionDegrees: [23.473, 0.674, 500.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 60.0],
    },
    color: [0.85, 0.85, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ellipsoid: 'Ellipsoid.MOON',
      landingSite: 'Apollo 11 (Statio Tranquillitatis)',
      terrainAssetId: 2684829,
    },
  };

  const artemisCylinder = {
    id: 'moon-artemis3-southpole-cylinder',
    name: 'Artemis 3 South Pole Candidate Landing Site Column',
    show: true,
    positionDegrees: [0.000, -85.000, 400.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 7.5,
      bottomRadius: 7.5,
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      mission: 'Artemis 3',
      region: 'Lunar South Pole',
      geojsonAssetId: 2683531,
    },
  };

  const mareSphere = {
    id: 'moon-mare-serenitatis-sphere',
    name: 'LROC Global Lunar Mare Basalt Boundary Probe',
    show: true,
    positionDegrees: [17.500, 28.000, 450.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.1, 0.42, 0.44, 0.95], // [26, 106, 113]
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.8, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      mare: 'Mare Serenitatis',
      boundariesAssetId: 2683530,
    },
  };

  const extentRect = {
    id: 'moon-survey-envelope-rect',
    name: 'Lunar Reconnaissance Orbiter Survey Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [23.400, 0.600, 23.550, 0.750],
    },
    color: [0.25, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.85, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      coverage: 'Lunar High-Resolution Cartography',
    },
  };

  const entities = [apolloBox, artemisCylinder, mareSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Moon 3D Tiles Lunar Terrain (Apollo 11 Tranquility Base)',
      entities: [apolloBox, artemisCylinder, mareSphere, extentRect],
    },
    {
      label: 'Artemis 3 South Polar Shadowed Craters Exploration Mode',
      entities: [
        { ...apolloBox, color: [0.35, 0.85, 0.95, 0.95], outlineColor: [0.75, 1.0, 1.0, 1.0] },
        artemisCylinder,
        mareSphere,
        extentRect,
      ],
    },
    {
      label: 'LROC Global Mare Boundaries Clamped GeoJSON Overlay Mode',
      entities: [
        apolloBox,
        { ...artemisCylinder, color: [0.2, 0.75, 0.55, 0.95], outlineColor: [0.65, 1.0, 0.8, 1.0] },
        mareSphere,
        extentRect,
      ],
    },
    {
      label: 'Lunar Highland Anorthosite Regolith Reflectance Analysis',
      entities: [
        apolloBox,
        artemisCylinder,
        { ...mareSphere, color: [0.95, 0.75, 0.25, 0.95], outlineColor: [1.0, 0.9, 0.65, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. MSAA
function generateMsaa() {
  const modelBox = {
    id: 'msaa-facility-target-box',
    name: 'MSAA Anti-Aliased Subpixel Polygon Target Box',
    show: true,
    positionDegrees: [-123.074, 44.050, 300.0],
    shape: {
      type: 'box',
      dimensions: [35.0, 35.0, 50.0],
    },
    color: [0.3, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      msaaSamples: 8,
      msaaSupported: true,
    },
  };

  const sampleCylinder = {
    id: 'msaa-subpixel-sample-cylinder',
    name: 'Multi-Sample Subpixel Rasterization Datum Column',
    show: true,
    positionDegrees: [-123.071, 44.053, 280.0],
    shape: {
      type: 'cylinder',
      length: 55.0,
      topRadius: 7.0,
      bottomRadius: 7.0,
    },
    color: [0.95, 0.45, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      sampleMask: 8,
      aliasingReduction: 'MSAA_8X',
    },
  };

  const edgeSphere = {
    id: 'msaa-silhouetted-edge-sphere',
    name: 'Silhouetted Wireframe Edge Anti-Aliasing Probe',
    show: true,
    positionDegrees: [-123.077, 44.047, 290.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.35, 0.9, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      headingPitchRoll: [135, 0, 0],
      terrain: 'WorldTerrain',
    },
  };

  const extentRect = {
    id: 'msaa-testbed-cadastral-rect',
    name: 'MSAA Graphical Benchmark Testbed Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.085, 44.040, -123.060, 44.060],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.7, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      benchmarkScene: 'MSAA_8x_Terrain_Testbed',
    },
  };

  const entities = [modelBox, sampleCylinder, edgeSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: MSAA 8x Multisample Anti-Aliasing Active (Smooth Edges)',
      entities: [modelBox, sampleCylinder, edgeSphere, extentRect],
    },
    {
      label: 'MSAA 4x Balanced Performance Anti-Aliasing Mode (4 Samples/Pixel)',
      entities: [
        { ...modelBox, color: [0.95, 0.85, 0.25, 0.95], outlineColor: [1.0, 0.95, 0.7, 1.0] },
        sampleCylinder,
        edgeSphere,
        extentRect,
      ],
    },
    {
      label: 'MSAA 2x Lightweight Edge Smoothing Mode (2 Samples/Pixel)',
      entities: [
        modelBox,
        { ...sampleCylinder, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        edgeSphere,
        extentRect,
      ],
    },
    {
      label: 'MSAA 1x Aliasing Baseline Pass-Through Mode (No Multi-Sampling)',
      entities: [
        modelBox,
        sampleCylinder,
        { ...edgeSphere, color: [0.25, 0.85, 0.85, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 79 fixtures
const datasets = {
  'material-with-custom-glsl': generateMaterialWithCustomGlsl(),
  'montreal-point-cloud': generateMontrealPointCloud(),
  'moon': generateMoon(),
  'msaa': generateMsaa(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
