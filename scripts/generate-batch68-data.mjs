import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Custom Post Process (Eugene, Oregon - Mosaic Filter)
function generateCustomPostProcess() {
  const subjectBox = {
    id: 'custom-post-mosaic-subject',
    name: 'Mosaic Post Process Subject Model',
    show: true,
    positionDegrees: [-123.0745, 44.0504, 25.0],
    shape: {
      type: 'box',
      dimensions: [36.0, 36.0, 52.0],
    },
    color: [0.2, 0.75, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      postProcessStage: 'MosaicBlurKernel',
      kernelWidth: 16,
    },
  };

  const kernelPillar = {
    id: 'custom-post-kernel-pillar',
    name: 'Convolution Sampling Kernel Anchor',
    show: true,
    positionDegrees: [-123.0738, 44.0510, 20.0],
    shape: {
      type: 'cylinder',
      length: 40.0,
      topRadius: 20.0,
      bottomRadius: 20.0,
    },
    color: [0.65, 0.35, 0.9, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      filterPass: '16x16 Box Average',
    },
  };

  const pixelProbe = {
    id: 'custom-post-pixel-probe',
    name: 'Screen Space Pixel Ratio Sampling Probe',
    show: true,
    positionDegrees: [-123.0752, 44.0498, 18.0],
    shape: {
      type: 'ellipsoid',
      radii: [18.0, 18.0, 18.0],
    },
    color: [0.98, 0.65, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      samplingStep: '8.0 * czm_pixelRatio / viewport.zw',
    },
  };

  const viewportRect = {
    id: 'custom-post-viewport-rect',
    name: 'Mosaic Post Process Viewport Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.077, 44.048, -123.072, 44.053],
    },
    color: [0.25, 0.15, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.4, 0.9, 0.8],
    outlineWidth: 1.0,
    properties: {
      pass: 'PostProcessStage',
    },
  };

  const entities = [subjectBox, kernelPillar, pixelProbe, viewportRect];

  const variants = [
    {
      label: 'Baseline: 16x16 Kernel Mosaic Post Process (Cyan & Violet)',
      entities: [
        subjectBox,
        kernelPillar,
        pixelProbe,
        viewportRect,
      ],
    },
    {
      label: 'High-Density Pixel Step Mode (Amber Radiant Highlight)',
      entities: [
        { ...subjectBox, color: [1.0, 0.5, 0.1, 0.95], outlineColor: [1.0, 0.85, 0.4, 1.0] },
        kernelPillar,
        pixelProbe,
        viewportRect,
      ],
    },
    {
      label: 'Inverted Contrast Mosaic Diagnostic (Neon Lime Infill)',
      entities: [
        subjectBox,
        { ...kernelPillar, color: [0.3, 0.95, 0.4, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        pixelProbe,
        viewportRect,
      ],
    },
    {
      label: 'Bypass Stage Pass-Through View (Clean Geometry Outline)',
      entities: [
        subjectBox,
        kernelPillar,
        { ...pixelProbe, color: [0.2, 0.9, 1.0, 0.95], outlineColor: [0.8, 1.0, 1.0, 1.0] },
        viewportRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. Custom Primitive - Dev (Null Island Procedural Waves)
function generateCustomPrimitiveDev() {
  const wavePlate = {
    id: 'custom-prim-wave-plate',
    name: 'Interference Wave Procedural Surface Mesh',
    show: true,
    positionDegrees: [0.0, 0.0, 20.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 25.0],
    },
    color: [0.15, 0.85, 0.8, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      halfWidthMeters: 10000,
      faceResolution: 200,
      primitiveType: 'TRIANGLES',
    },
  };

  const dispPillar = {
    id: 'custom-prim-disp-pillar',
    name: 'Vertex Sinusoidal Wave Amplitude Column',
    show: true,
    positionDegrees: [0.005, 0.005, 25.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 22.0,
      bottomRadius: 22.0,
    },
    color: [0.95, 0.78, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      amplitude: 1000.0,
      frequencyK: [0.0004, 0.0001],
    },
  };

  const texSphere = {
    id: 'custom-prim-1d-tex-sphere',
    name: '1D Color Palette Texture Probe',
    show: true,
    positionDegrees: [-0.005, -0.005, 20.0],
    shape: {
      type: 'ellipsoid',
      radii: [20.0, 20.0, 20.0],
    },
    color: [0.9, 0.2, 0.75, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      textureType: '1D HSL Palette',
      paletteWidth: 256,
    },
  };

  const domainRect = {
    id: 'custom-prim-domain-rect',
    name: 'Custom Primitive Procedural Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-0.015, -0.015, 0.015, 0.015],
    },
    color: [0.1, 0.25, 0.5, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.6, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      gridAreaMeters2: 400000000,
    },
  };

  const entities = [wavePlate, dispPillar, texSphere, domainRect];

  const variants = [
    {
      label: 'Baseline: Dual Wave Interference Surface (Teal & Gold)',
      entities: [
        wavePlate,
        dispPillar,
        texSphere,
        domainRect,
      ],
    },
    {
      label: 'High-Frequency Standing Wave Mode (Magenta Peak Shading)',
      entities: [
        { ...wavePlate, color: [0.85, 0.2, 0.9, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        dispPillar,
        texSphere,
        domainRect,
      ],
    },
    {
      label: '1D Color Texture Palette Gradient (Electric Emerald Infill)',
      entities: [
        wavePlate,
        { ...dispPillar, color: [0.2, 0.9, 0.45, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        texSphere,
        domainRect,
      ],
    },
    {
      label: 'Wireframe Quad Topology Inspection (Neon White Outlines)',
      entities: [
        wavePlate,
        dispPillar,
        { ...texSphere, color: [0.95, 0.95, 0.95, 0.95], outlineColor: [1.0, 1.0, 1.0, 1.0] },
        domainRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Custom Shaders 3D Tiles (NYC Lower Manhattan - Asset 75343)
function generateCustomShaders3dTiles() {
  const towerBox = {
    id: 'custom-shader-tower-box',
    name: 'Camera Distance Colorized Tower Member',
    show: true,
    positionDegrees: [-74.0188, 40.6911, 120.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 240.0],
    },
    color: [0.15, 0.45, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      shader: 'ColorByDistanceToCamera',
      ionAssetId: 75343,
    },
  };

  const stripesCylinder = {
    id: 'custom-shader-stripes-cylinder',
    name: 'Animated Texture Stripes Column',
    show: true,
    positionDegrees: [-74.0150, 40.6930, 45.0],
    shape: {
      type: 'cylinder',
      length: 90.0,
      topRadius: 25.0,
      bottomRadius: 25.0,
    },
    color: [0.95, 0.85, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      textureUniform: 'cesium_stripes.png',
      animated: true,
    },
  };

  const cameraProbe = {
    id: 'custom-shader-camera-probe',
    name: 'Eye Coordinates Position Benchmark Sphere',
    show: true,
    positionDegrees: [-74.0220, 40.6890, 40.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 25.0, 25.0],
    },
    color: [0.2, 0.85, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      attribute: 'positionEC.z',
    },
  };

  const nycBoundsRect = {
    id: 'custom-shader-nyc-bounds-rect',
    name: '3D Tiles NYC Dataset Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.025, 40.685, -74.010, 40.700],
    },
    color: [0.2, 0.3, 0.45, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      lightingModel: 'UNLIT',
    },
  };

  const entities = [towerBox, stripesCylinder, cameraProbe, nycBoundsRect];

  const variants = [
    {
      label: 'Baseline: Color Tiles by Distance to Camera (Blue/Green Gradient)',
      entities: [
        towerBox,
        stripesCylinder,
        cameraProbe,
        nycBoundsRect,
      ],
    },
    {
      label: 'Animated Procedural Texture Stripes (Yellow & Indigo Bands)',
      entities: [
        { ...towerBox, color: [0.9, 0.75, 0.1, 0.95], outlineColor: [1.0, 0.9, 0.4, 1.0] },
        stripesCylinder,
        cameraProbe,
        nycBoundsRect,
      ],
    },
    {
      label: 'Unlit Standard PBR Material (White Architectural Facade)',
      entities: [
        towerBox,
        { ...stripesCylinder, color: [0.85, 0.9, 0.95, 0.95], outlineColor: [1.0, 1.0, 1.0, 1.0] },
        cameraProbe,
        nycBoundsRect,
      ],
    },
    {
      label: 'High-Contrast Camera Range Mask (Neon Amber Depth Bands)',
      entities: [
        towerBox,
        stripesCylinder,
        { ...cameraProbe, color: [1.0, 0.45, 0.05, 0.95], outlineColor: [1.0, 0.8, 0.4, 1.0] },
        nycBoundsRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Custom Shaders Models (Eugene, Oregon - Vehicle & Balloon)
function generateCustomShadersModels() {
  const vehicleBox = {
    id: 'custom-model-vehicle-box',
    name: 'Ground Vehicle Target Model Volume',
    show: true,
    positionDegrees: [-123.0745, 44.0504, 30.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 75.0, 40.0],
    },
    color: [0.9, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'GroundVehicle.glb',
      customShader: 'TextureUniformShader',
    },
  };

  const balloonCylinder = {
    id: 'custom-model-balloon-cylinder',
    name: 'Procedural Checkerboard Balloon Shell',
    show: true,
    positionDegrees: [-123.0735, 44.0512, 45.0],
    shape: {
      type: 'cylinder',
      length: 80.0,
      topRadius: 30.0,
      bottomRadius: 20.0,
    },
    color: [0.98, 0.7, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'CesiumBalloon.glb',
      customShader: 'CheckerboardMaskShader',
    },
  };

  const waveSphere = {
    id: 'custom-model-wave-sphere',
    name: 'Sinusoid Displacement Wave Sphere',
    show: true,
    positionDegrees: [-123.0755, 44.0495, 30.0],
    shape: {
      type: 'ellipsoid',
      radii: [22.0, 22.0, 22.0],
    },
    color: [0.7, 0.25, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.6, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'PointCloudWave.glb',
      customShader: 'PointCloudWaveShader',
    },
  };

  const stageRect = {
    id: 'custom-model-stage-rect',
    name: 'Model Custom Shader Testing Arena Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.078, 44.047, -123.071, 44.054],
    },
    color: [0.25, 0.25, 0.35, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.6, 0.8, 0.8],
    outlineWidth: 1.0,
    properties: {
      translucencyMode: 'TRANSLUCENT',
    },
  };

  const entities = [vehicleBox, balloonCylinder, waveSphere, stageRect];

  const variants = [
    {
      label: 'Baseline: Custom Animated Texture (Racing Red & Stripes)',
      entities: [
        vehicleBox,
        balloonCylinder,
        waveSphere,
        stageRect,
      ],
    },
    {
      label: 'Procedural Checkerboard Mask (Amber & Ebony Quads)',
      entities: [
        { ...vehicleBox, color: [0.95, 0.55, 0.1, 0.95], outlineColor: [1.0, 0.8, 0.4, 1.0] },
        balloonCylinder,
        waveSphere,
        stageRect,
      ],
    },
    {
      label: 'Translucent Diagonal Alpha Gradient (Semi-Transparent Violet)',
      entities: [
        vehicleBox,
        { ...balloonCylinder, color: [0.6, 0.2, 0.85, 0.6], outlineColor: [0.8, 0.5, 1.0, 1.0] },
        waveSphere,
        stageRect,
      ],
    },
    {
      label: 'Vertex Normal Expansion Deformation (Expanded Neon Cyan)',
      entities: [
        vehicleBox,
        balloonCylinder,
        { ...waveSphere, color: [0.1, 0.9, 0.95, 0.95], outlineColor: [0.7, 1.0, 1.0, 1.0] },
        stageRect,
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

writeCase('custom-post-process', generateCustomPostProcess());
writeCase('custom-primitive-dev', generateCustomPrimitiveDev());
writeCase('custom-shaders-3d-tiles', generateCustomShaders3dTiles());
writeCase('custom-shaders-models', generateCustomShadersModels());
