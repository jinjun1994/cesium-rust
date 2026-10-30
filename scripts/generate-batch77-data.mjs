import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Ion Snapping - Dev
function generateIonSnappingDev() {
  const bimBox = {
    id: 'ion-snap-bim-column-box',
    name: 'BIM CAD Database Structural Element Snapping Target',
    show: true,
    positionDegrees: [-75.165, 39.952, 60.0],
    shape: {
      type: 'box',
      dimensions: [30.0, 30.0, 80.0],
    },
    color: [0.0, 0.9, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.6, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      assetId: 5161569,
      snapMode: 'NEAREST',
      snapAperture: 16,
    },
  };

  const apertureCylinder = {
    id: 'ion-snap-aperture-cylinder',
    name: 'Screen Space Snap Aperture Search Cylinder Probe',
    show: true,
    positionDegrees: [-75.162, 39.955, 45.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.95, 0.2, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapTolerancePixels: 16,
      surfaceSnapping: true,
    },
  };

  const hitpointSphere = {
    id: 'ion-snap-hitpoint-sphere',
    name: 'Snapped Element Geometry Hit Marker Screen Probe',
    show: true,
    positionDegrees: [-75.168, 39.950, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.0, 5.0, 5.0],
    },
    color: [1.0, 0.75, 0.8, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.95, 1.0],
    outlineWidth: 1.0,
    properties: {
      pickedSwatch: '#00ff00',
      snapSwatch: '#ff0000',
    },
  };

  const extentRect = {
    id: 'ion-snap-bim-envelope-rect',
    name: 'BIM Facility Cadastral Snapping Bounds Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.175, 39.945, -75.155, 39.960],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 0.75, 0.9],
    outlineWidth: 1.0,
    properties: {
      depthTestAgainstTerrain: true,
      pickTranslucentDepth: true,
    },
  };

  const entities = [bimBox, apertureCylinder, hitpointSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Ion BIM/CAD Snapping Service (Nearest Point Mode)',
      entities: [bimBox, apertureCylinder, hitpointSphere, extentRect],
    },
    {
      label: 'Keypoints Snapping Mode (Edge / Vertex / Corner Snapping Target)',
      entities: [
        { ...bimBox, color: [0.95, 0.65, 0.15, 0.95], outlineColor: [1.0, 0.85, 0.6, 1.0] },
        apertureCylinder,
        hitpointSphere,
        extentRect,
      ],
    },
    {
      label: 'Center Snapping Mode (Face / Volume Geometric Centroid Snap)',
      entities: [
        bimBox,
        { ...apertureCylinder, color: [0.25, 0.75, 0.95, 0.95], outlineColor: [0.7, 0.95, 1.0, 1.0] },
        hitpointSphere,
        extentRect,
      ],
    },
    {
      label: 'High Tolerance Aperture Mode (Expanded 32px Snap Search Volume)',
      entities: [
        bimBox,
        apertureCylinder,
        { ...hitpointSphere, color: [0.95, 0.25, 0.85, 0.95], outlineColor: [1.0, 0.7, 0.95, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.3, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. iTwin Feature Service
function generateITwinFeatureService() {
  const meshBox = {
    id: 'itwin-reality-mesh-box',
    name: 'Reality Mesh AI-Derived Feature Extraction Bounding Box',
    show: true,
    positionDegrees: [-122.419, 37.774, 50.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 60.0],
    },
    color: [0.2, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      iTwinId: '04ba725f-f3c0-4f30-8014-a4488cbd612d',
      featureService: 'GeospatialFeatures',
    },
  };

  const postCylinder = {
    id: 'itwin-lamp-post-cylinder',
    name: 'Lamp Post Geospatial Feature Vector Column',
    show: true,
    positionDegrees: [-122.417, 37.776, 40.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 6.0,
      bottomRadius: 6.0,
    },
    color: [0.95, 0.95, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.8, 0.8, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      featureType: 'Lamp_post',
      icon: 'lighthouse',
    },
  };

  const trafficSphere = {
    id: 'itwin-traffic-light-sphere',
    name: 'Traffic Light Sensor Node Spatial Probe',
    show: true,
    positionDegrees: [-122.421, 37.772, 45.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.5, 5.5, 5.5],
    },
    color: [0.86, 0.08, 0.24, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.6, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      featureType: 'Traffic_light',
      icon: 'circle-stroked',
    },
  };

  const extentRect = {
    id: 'itwin-feature-extent-rect',
    name: 'iTwin Reality Survey Geospatial Area Extent Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.430, 37.765, -122.405, 37.785],
    },
    color: [0.25, 0.45, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.95, 0.6, 0.9],
    outlineWidth: 1.0,
    properties: {
      collections: ['points', 'lines', 'areas'],
    },
  };

  const entities = [meshBox, postCylinder, trafficSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Bentley iTwin Feature Service (Points, Lines, Areas & Mesh)',
      entities: [meshBox, postCylinder, trafficSphere, extentRect],
    },
    {
      label: 'Point Features Isolation Mode (Trees, Lamps, Traffic Signals)',
      entities: [
        { ...meshBox, color: [0.35, 0.85, 0.45, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        postCylinder,
        trafficSphere,
        extentRect,
      ],
    },
    {
      label: 'Linear Infrastructure Markings Mode (Contours, Lane Markings, Kerbs)',
      entities: [
        meshBox,
        { ...postCylinder, color: [0.15, 0.85, 0.95, 0.95], outlineColor: [0.65, 0.95, 1.0, 1.0] },
        trafficSphere,
        extentRect,
      ],
    },
    {
      label: 'Birdseye Viewpoint Aerial Survey Extent (Cadastral Zoned Boundaries)',
      entities: [
        meshBox,
        postCylinder,
        { ...trafficSphere, color: [0.95, 0.6, 0.2, 0.95], outlineColor: [1.0, 0.8, 0.6, 1.0] },
        { ...extentRect, color: [0.35, 0.3, 0.2, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. iTwin Gaussian Splats
function generateITwinGaussianSplats() {
  const damBox = {
    id: 'splats-dam-cluster-box',
    name: '3D Gaussian Splatting Dam Reality Mesh Anchor Box',
    show: true,
    positionDegrees: [-121.285, 37.957, 70.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 45.0, 65.0],
    },
    color: [0.9, 0.55, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      realityDataId: '4afd5d69-3ba7-491d-86cd-3d86d43db907',
      type: 'GaussianSplats',
    },
  };

  const alignCylinder = {
    id: 'splats-alignment-cylinder',
    name: 'Model Matrix World Terrain Alignment Datum Column',
    show: true,
    positionDegrees: [-121.282, 37.960, 55.0],
    shape: {
      type: 'cylinder',
      length: 65.0,
      topRadius: 7.5,
      bottomRadius: 7.5,
    },
    color: [0.3, 0.75, 0.9, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.7, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      translationOffset: [-1.0, 9.0, -8.0],
      terrainAlignment: 'WorldTerrain',
    },
  };

  const centroidSphere = {
    id: 'splats-centroid-sphere',
    name: 'Gaussian Covariance Centroid Kernel Spatial Probe',
    show: true,
    positionDegrees: [-121.288, 37.954, 60.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.0, 6.0, 6.0],
    },
    color: [0.85, 0.3, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      splatKernel: '3D_Gaussian',
      headingPitchRange: [0.0, -0.5, 100.0],
    },
  };

  const extentRect = {
    id: 'splats-survey-envelope-rect',
    name: 'Bentley Reality Data Geographic Extent Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-121.300, 37.945, -121.270, 37.970],
    },
    color: [0.35, 0.3, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.75, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      surveyArea: 'Dam_Infrastructure',
    },
  };

  const entities = [damBox, alignCylinder, centroidSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: 3D Gaussian Splats Reality Data on Cesium World Terrain',
      entities: [damBox, alignCylinder, centroidSphere, extentRect],
    },
    {
      label: 'Dam Crest Close Inspection Mode (Heading Pitch Range Zoom)',
      entities: [
        { ...damBox, color: [0.95, 0.75, 0.25, 0.95], outlineColor: [1.0, 0.9, 0.65, 1.0] },
        alignCylinder,
        centroidSphere,
        extentRect,
      ],
    },
    {
      label: 'Terrain Subsurface Matrix Offset Calibration (Z-Offset -8m Mode)',
      entities: [
        damBox,
        { ...alignCylinder, color: [0.25, 0.85, 0.55, 0.95], outlineColor: [0.75, 1.0, 0.85, 1.0] },
        centroidSphere,
        extentRect,
      ],
    },
    {
      label: 'Survey Footprint Cadastral Boundary Mode (High-Density Splat Area)',
      entities: [
        damBox,
        alignCylinder,
        { ...centroidSphere, color: [0.35, 0.55, 0.95, 0.95], outlineColor: [0.75, 0.85, 1.0, 1.0] },
        { ...extentRect, color: [0.25, 0.35, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Japan Buildings
function generateJapanBuildings() {
  const tokyoBox = {
    id: 'japan-tokyo-tower-box',
    name: 'Tokyo Tower 3D Building Dataset Landmark Anchor Box',
    show: true,
    positionDegrees: [139.745, 35.658, 120.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 150.0],
    },
    color: [0.95, 0.25, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      landmark: 'Tokyo Tower',
      buildingAssetId: 2602291,
      terrainAssetId: 2767062,
    },
  };

  const lightCylinder = {
    id: 'japan-camera-light-cylinder',
    name: 'Camera-Aligned Directional Light Vector Calibration Column',
    show: true,
    positionDegrees: [139.748, 35.661, 80.0],
    shape: {
      type: 'cylinder',
      length: 70.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.95, 0.9, 0.3, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      lightType: 'DirectionalLight',
      intensity: 2.0,
      cameraSync: true,
    },
  };

  const nijoSphere = {
    id: 'japan-nijo-castle-sphere',
    name: 'Kyoto Nijo Castle Landmark Geolocation Sensor Probe',
    show: true,
    positionDegrees: [135.748, 35.014, 60.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.5, 6.5, 6.5],
    },
    color: [0.3, 0.85, 0.6, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.85, 1.0],
    outlineWidth: 1.0,
    properties: {
      poi: 'Kyoto Nijo Castle',
      osmMetadata: true,
    },
  };

  const extentRect = {
    id: 'japan-metropolitan-extent-rect',
    name: 'Japan Regional Building Survey Cadastral Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [135.500, 34.800, 140.000, 35.800],
    },
    color: [0.3, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.6, 0.6, 0.9],
    outlineWidth: 1.0,
    properties: {
      coverage: 'Metropolitan Japan (Tokyo - Kyoto)',
    },
  };

  const entities = [tokyoBox, lightCylinder, nijoSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Japan 3D Buildings & Regional Terrain (Tokyo Tower Landmark)',
      entities: [tokyoBox, lightCylinder, nijoSphere, extentRect],
    },
    {
      label: 'Kyoto Nijo Castle Cultural Heritage Landmark Viewpoint',
      entities: [
        { ...tokyoBox, color: [0.85, 0.35, 0.75, 0.95], outlineColor: [1.0, 0.7, 0.95, 1.0] },
        lightCylinder,
        nijoSphere,
        extentRect,
      ],
    },
    {
      label: 'Camera Dynamic Directional Lighting Highlighting Building Facades',
      entities: [
        tokyoBox,
        { ...lightCylinder, color: [0.3, 0.75, 0.95, 0.95], outlineColor: [0.75, 0.95, 1.0, 1.0] },
        nijoSphere,
        extentRect,
      ],
    },
    {
      label: 'Mt. Fuji - Gotemba Regional Topography Elevation Mode',
      entities: [
        tokyoBox,
        lightCylinder,
        { ...nijoSphere, color: [0.95, 0.8, 0.25, 0.95], outlineColor: [1.0, 0.9, 0.65, 1.0] },
        { ...extentRect, color: [0.2, 0.35, 0.3, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 77 fixtures
const datasets = {
  'ion-snapping-dev': generateIonSnappingDev(),
  'itwin-feature-service': generateITwinFeatureService(),
  'itwin-gaussian-splats': generateITwinGaussianSplats(),
  'japan-buildings': generateJapanBuildings(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
