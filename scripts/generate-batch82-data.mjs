import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Particle System Weather
function generateParticleSystemWeather() {
  const snowCloudDome = {
    id: 'weather-snow-cloud-dome',
    name: 'High-Altitude Snow Cloud Volumetric Dome',
    show: true,
    positionDegrees: [87.1912, 27.8334, 5500.0],
    shape: {
      type: 'ellipsoid',
      radii: [50.0, 50.0, 30.0],
    },
    color: [0.9, 0.95, 1.0, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      mode: 'Snow',
      emissionRate: 7000.0,
      fogDensity: 0.001,
      snowflakeSize: 12.0,
    },
  };

  const rainCurtainBox = {
    id: 'weather-rain-curtain-box',
    name: 'Heavy Torrential Precipitation Curtain Bounds',
    show: true,
    positionDegrees: [87.1925, 27.8345, 4800.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 70.0],
    },
    color: [0.27, 0.5, 0.7, 0.8],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      mode: 'Rain',
      emissionRate: 9000.0,
      gravityDown: -1050.0,
      fogDensity: 0.00025,
    },
  };

  const cameraMountCylinder = {
    id: 'weather-camera-mount-cylinder',
    name: 'Alpine Weather Station Observation Datum Mast',
    show: true,
    positionDegrees: [87.1905, 27.8322, 5200.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 5.0,
      bottomRadius: 7.0,
    },
    color: [0.85, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      heading: 4.731,
      pitch: -0.320,
      cameraAltitudeMeters: 16267.0,
    },
  };

  const extentRect = {
    id: 'weather-extent-rect',
    name: 'Himalayan Weather Simulation Cadastral Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [87.180, 27.820, 87.205, 27.845],
    },
    color: [0.2, 0.3, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      snowRadius: 100000.0,
      rainRadius: 100000.0,
    },
  };

  const entities = [snowCloudDome, rainCurtainBox, cameraMountCylinder, extentRect];

  const variants = [
    {
      label: 'Baseline: Alpine Snow Storm Mode (7000.0 Rate Snowflakes, Dense Fog 0.001)',
      entities: [
        snowCloudDome,
        { ...rainCurtainBox, show: false },
        cameraMountCylinder,
        extentRect,
      ],
    },
    {
      label: 'Heavy Torrential Downpour Rain Mode (9000.0 Rate Droplets, Gravity -1050.0 m/s)',
      entities: [
        { ...snowCloudDome, show: false },
        { ...rainCurtainBox, show: true },
        cameraMountCylinder,
        extentRect,
      ],
    },
    {
      label: 'Freezing Sleet and Blizzard Mode (High-Velocity Wind-Blown Mixed Precipitation)',
      entities: [
        {
          ...snowCloudDome,
          show: true,
          shape: { type: 'ellipsoid', radii: [75.0, 75.0, 45.0] },
          color: [0.8, 0.9, 1.0, 0.9],
        },
        {
          ...rainCurtainBox,
          show: true,
          color: [0.35, 0.6, 0.8, 0.75],
        },
        cameraMountCylinder,
        extentRect,
      ],
    },
    {
      label: 'Clear Sky Post-Storm Alpine Observation (Direct Sunlight, Zero Fog Baseline)',
      entities: [
        { ...snowCloudDome, show: false },
        { ...rainCurtainBox, show: false },
        cameraMountCylinder,
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. PBR Lighting - Dev
function generatePbrLightingDev() {
  const helmetBox = {
    id: 'pbr-helmet-model-box',
    name: 'Damaged Helmet Complex PBR Asset Bounding Hull',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 200.0],
    shape: {
      type: 'box',
      dimensions: [14.0, 12.0, 15.0],
    },
    color: [0.75, 0.7, 0.65, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 2681021,
      name: 'Damaged Helmet',
      roughness: 0.4,
      metallic: 0.9,
    },
  };

  const specularSphere = {
    id: 'pbr-environment-specular-sphere',
    name: 'Mirror Ball Environment Specular Reflection Probe',
    show: true,
    positionDegrees: [-123.0740, 44.0508, 205.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0],
    },
    color: [0.9, 0.92, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      specularFactor: [1.0, 1.0],
      groundAlbedo: 0.31,
      gamma: 1.0,
    },
  };

  const lightCylinder = {
    id: 'pbr-lighting-rig-cylinder',
    name: 'Directional Studio Key Light Azimuth Standard Column',
    show: true,
    positionDegrees: [-123.0748, 44.0500, 195.0],
    shape: {
      type: 'cylinder',
      length: 30.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [1.0, 0.85, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      directionalLightIntensity: 2.0,
      atmosphereScatteringIntensity: 2.0,
    },
  };

  const extentRect = {
    id: 'pbr-inspection-platform-rect',
    name: 'PBR Physical Laboratory Cadastral Ground Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.085, 44.042, -123.065, 44.058],
    },
    color: [0.2, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.75, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      enableLighting: true,
      hdr: true,
      shadows: true,
    },
  };

  const entities = [helmetBox, specularSphere, lightCylinder, extentRect];

  const variants = [
    {
      label: 'Baseline: Full PBR Image-Based Lighting (Diffuse and Specular Environment Maps)',
      entities: [
        helmetBox,
        specularSphere,
        lightCylinder,
        extentRect,
      ],
    },
    {
      label: 'Diffuse Only Environment Lighting Mode (Factor [1.0, 0.0], Matte Reflection)',
      entities: [
        helmetBox,
        {
          ...specularSphere,
          color: [0.7, 0.75, 0.8, 0.9],
          properties: { ...specularSphere.properties, specularFactor: [1.0, 0.0] },
        },
        lightCylinder,
        extentRect,
      ],
    },
    {
      label: 'Specular Mirror Chrome Reflection Mode (Factor [0.0, 1.0], High Contrast Metallic)',
      entities: [
        helmetBox,
        {
          ...specularSphere,
          color: [1.0, 1.0, 1.0, 0.95],
          properties: { ...specularSphere.properties, specularFactor: [0.0, 1.0] },
        },
        lightCylinder,
        extentRect,
      ],
    },
    {
      label: 'Direct Sunlight Studio Mode (Zero Environment Factor, Intense Directional Key Light)',
      entities: [
        helmetBox,
        { ...specularSphere, show: false },
        {
          ...lightCylinder,
          color: [1.0, 0.95, 0.5, 1.0],
          properties: { ...lightCylinder.properties, directionalLightIntensity: 4.0 },
        },
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. Per-Feature Post-Processing
function generatePerFeaturePostProcessing() {
  const characterBox = {
    id: 'perfeature-model-character-box',
    name: 'Cesium_Man Articulated Character Model Hull',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 10.0],
    shape: {
      type: 'box',
      dimensions: [8.0, 6.0, 18.0],
    },
    color: [0.25, 0.75, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'Cesium_Man.glb',
      trackedEntity: true,
    },
  };

  const haloEllipsoid = {
    id: 'perfeature-silhouette-halo-ellipsoid',
    name: 'Per-Feature Lime Silhouette Glow Perimeter Shell',
    show: true,
    positionDegrees: [-123.0741, 44.0506, 15.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 12.0, 15.0],
    },
    color: [0.2, 1.0, 0.2, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      stage: 'Silhouette',
      silhouetteColor: '#00ff00',
      lineWidth: 2.0,
    },
  };

  const tonalCylinder = {
    id: 'perfeature-tonal-cylinder',
    name: 'Quantized Black-and-White Shading Test Column',
    show: true,
    positionDegrees: [-123.0748, 44.0501, 8.0],
    shape: {
      type: 'cylinder',
      length: 22.0,
      topRadius: 3.5,
      bottomRadius: 3.5,
    },
    color: [0.6, 0.6, 0.6, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.9, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      stage: 'BlackAndWhite',
      gradations: 5.0,
    },
  };

  const extentRect = {
    id: 'perfeature-bounds-rect',
    name: 'Screen Space Mouse-Move Feature Picking Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.084, 44.043, -123.064, 44.057],
    },
    color: [0.2, 0.3, 0.25, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.9, 0.5, 0.8],
    outlineWidth: 1.0,
    properties: {
      interactiveMouseMove: true,
    },
  };

  const entities = [characterBox, haloEllipsoid, tonalCylinder, extentRect];

  const variants = [
    {
      label: 'Baseline: Mouse-Over Lime Silhouette Stage (Per-Feature Highlight Boundary Halo)',
      entities: [
        characterBox,
        haloEllipsoid,
        { ...tonalCylinder, show: false },
        extentRect,
      ],
    },
    {
      label: 'Mouse-Over Black and White Quantization Stage (5-Step Grayscale Shading)',
      entities: [
        characterBox,
        { ...haloEllipsoid, show: false },
        { ...tonalCylinder, show: true },
        extentRect,
      ],
    },
    {
      label: 'Dual Stage Composite Mode (Simultaneous Silhouette Edge & Monochromatic Body)',
      entities: [
        characterBox,
        haloEllipsoid,
        tonalCylinder,
        extentRect,
      ],
    },
    {
      label: 'Pass-Through Unfiltered Base Model Render (Direct PBR Character View)',
      entities: [
        characterBox,
        { ...haloEllipsoid, show: false },
        { ...tonalCylinder, show: false },
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. Physically-Based Materials
function generatePhysicallyBasedMaterials() {
  const vehicleBox = {
    id: 'pbm-ground-vehicle-box',
    name: 'GroundVehicle Heavy Heavy-Duty Model Bounding Hull',
    show: true,
    positionDegrees: [-103.97688, 27.07507, 30.0],
    shape: {
      type: 'box',
      dimensions: [18.0, 8.0, 7.0],
    },
    color: [0.85, 0.45, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.3, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelUri: 'GroundVehicle.glb',
      scale: 8.0,
      heightReference: 'CLAMP_TO_GROUND',
    },
  };

  const reflectionCylinder = {
    id: 'pbm-sun-reflection-cylinder',
    name: 'Solar Angle Specular Highlight Reflection Column',
    show: true,
    positionDegrees: [-103.9772, 27.0753, 35.0],
    shape: {
      type: 'cylinder',
      length: 25.0,
      topRadius: 4.0,
      bottomRadius: 4.0,
    },
    color: [1.0, 0.95, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      lightAngleDeg: 80.5,
      timeUtc: '2017-07-11T20:00:00Z',
      dynamicLighting: 'SUNLIGHT',
    },
  };

  const scatterSphere = {
    id: 'pbm-roughness-probe-sphere',
    name: 'PBR Microfacet Roughness Scatter Test Sphere',
    show: true,
    positionDegrees: [-103.9765, 27.0748, 28.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0],
    },
    color: [0.35, 0.65, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      viewMode: 'Front reflection',
      shadows: true,
      multiplier: 1000,
    },
  };

  const extentRect = {
    id: 'pbm-terrain-corridor-rect',
    name: 'Desert Terrain Test Track Inspection Cadastral Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-103.987, 27.067, -103.967, 27.083],
    },
    color: [0.35, 0.25, 0.15, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.6, 0.4, 0.8],
    outlineWidth: 1.0,
    properties: {
      enableLighting: true,
      depthTestAgainstTerrain: true,
    },
  };

  const entities = [vehicleBox, reflectionCylinder, scatterSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Front Specular Glint Reflection View (Sun Angle 20:00 UTC, Sunlight Dynamic Lighting)',
      entities: [
        vehicleBox,
        reflectionCylinder,
        scatterSphere,
        extentRect,
      ],
    },
    {
      label: 'Three Quarters Sunrise Side Shading View (11:00 UTC, Warm Low-Angle Raking Light)',
      entities: [
        vehicleBox,
        {
          ...reflectionCylinder,
          color: [1.0, 0.6, 0.3, 0.95],
          properties: { ...reflectionCylinder.properties, timeUtc: '2017-07-11T11:00:00Z', lightAngleDeg: 67.6 },
        },
        scatterSphere,
        extentRect,
      ],
    },
    {
      label: 'Top Down Direct Solar Noon Reflection View (12:00 UTC, High-Intensity Zenith Shadows)',
      entities: [
        vehicleBox,
        {
          ...reflectionCylinder,
          color: [1.0, 1.0, 0.8, 1.0],
          properties: { ...reflectionCylinder.properties, timeUtc: '2017-07-11T12:00:00Z', lightAngleDeg: 68.3 },
        },
        scatterSphere,
        extentRect,
      ],
    },
    {
      label: 'Upward Angle Dusk Horizon Rim Glow View (23:00 UTC, Golden Hour Atmospheric Scatter)',
      entities: [
        vehicleBox,
        {
          ...reflectionCylinder,
          color: [0.95, 0.4, 0.2, 0.95],
          properties: { ...reflectionCylinder.properties, timeUtc: '2017-07-11T23:00:00Z', lightAngleDeg: 236.4 },
        },
        scatterSphere,
        extentRect,
      ],
    },
  ];

  return { entities, variants };
}

const generators = {
  'particle-system-weather': generateParticleSystemWeather,
  'pbr-lighting-dev': generatePbrLightingDev,
  'per-feature-post-processing': generatePerFeaturePostProcessing,
  'physically-based-materials': generatePhysicallyBasedMaterials,
};

for (const [id, gen] of Object.entries(generators)) {
  const { entities, variants } = gen();
  const fullPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(fullPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}: ${entities.length} entities, ${variants.length} variants`);
}
