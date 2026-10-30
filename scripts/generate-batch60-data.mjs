import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Picking Dev
function generatePickingDev() {
  const anchorCylinder = {
    id: 'picking-dev-anchor-cylinder',
    name: 'Multi-Source 3D Tiles Picking Anchor Cylinder',
    show: true,
    positionDegrees: [-122.4194, 37.7749, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.15, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      supportedTilesets: ['Google P3DT', 'Cesium OSM Buildings', 'Photogrammetry'],
      pickingMethod: 'scene.pick',
    },
  };

  const probeBox = {
    id: 'picking-dev-probe-box',
    name: 'Interactive Ray-Cast Depth Probe Volume Box',
    show: true,
    positionDegrees: [-122.412, 37.781, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.85, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      globeDisabled: true,
      onlyUsingWithGoogleGeocoder: true,
    },
  };

  const hitSphere = {
    id: 'picking-dev-hit-sphere',
    name: 'Surface Intersection Normal Vector Reference Sphere',
    show: true,
    positionDegrees: [-122.426, 37.768, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.85, 0.25, 0.35, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      normalExtraction: true,
      screenSpaceHandlerActive: true,
    },
  };

  const footprintRect = {
    id: 'picking-dev-footprint-rect',
    name: 'San Francisco Downtown Picking Test Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.44, 37.75, -122.39, 37.79],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.35, 0.85, 0.45, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 1.0, 0.65, 0.9],
    outlineWidth: 1.0,
    properties: {
      renderMode: 'Pure 3D Tiles without Globe',
      baseLayerPicker: false,
    },
  };

  const entities = [anchorCylinder, probeBox, hitSphere, footprintRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Google Photorealistic 3D Tiles Picking',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Cesium OSM Buildings Picking & Semantic Inspector',
        entities: entities.map((e) => {
          if (e.id === 'picking-dev-probe-box') {
            return { ...e, color: [0.2, 0.95, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Photogrammetry Mesh Picking & Surface Normal Visualization',
        entities: entities.map((e) => {
          if (e.id === 'picking-dev-hit-sphere') {
            return { ...e, color: [0.95, 0.45, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Strict Depth Buffer Query Mode',
        entities: entities.map((e) => {
          if (e.id === 'picking-dev-anchor-cylinder') {
            return { ...e, color: [0.25, 0.9, 0.35, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles Point Cloud
function generatePointCloud() {
  const centerCylinder = {
    id: 'pnts-columbia-center-cylinder',
    name: 'Columbia Robotics Lab Laser Scan Datum Cylinder',
    show: true,
    positionDegrees: [2.928, 46.227, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.25, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 16421,
      scanner: 'Alejandro Troccoli and Matei Ciocarlie',
      lab: 'Columbia University Robotics Lab',
    },
  };

  const volumeBox = {
    id: 'pnts-scan-volume-box',
    name: 'Point Cloud LiDAR Scan Envelope Box',
    show: true,
    positionDegrees: [2.935, 46.233, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.45, 0.25, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      cameraDestination: [4401744.64, 225051.41, 4595420.37],
      cameraHpr: [5.6467, -0.2766, 6.2811],
    },
  };

  const sensorSphere = {
    id: 'pnts-sensor-dome-sphere',
    name: 'Spherical Laser Range Sensor Field Dome Sphere',
    show: true,
    positionDegrees: [2.921, 46.221, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.85, 0.45, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      pointFormat: 'PNTS with RGB & Normal',
      attenuation: true,
    },
  };

  const siteRect = {
    id: 'pnts-campus-site-rect',
    name: 'Terrestrial LiDAR Site Survey Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [2.91, 46.21, 2.95, 46.25],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.65, 0.35, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.55, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      terrainDraping: true,
      pointCountApprox: 5000000,
    },
  };

  const entities = [centerCylinder, volumeBox, sensorSphere, siteRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Columbia University Robotics Lab Point Cloud',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High-Contrast Density Gradient Shading',
        entities: entities.map((e) => {
          if (e.id === 'pnts-scan-volume-box') {
            return { ...e, color: [0.95, 0.85, 0.15, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Point Size Attenuation Scaling (Geometric Error)',
        entities: entities.map((e) => {
          if (e.id === 'pnts-sensor-dome-sphere') {
            return { ...e, color: [0.2, 0.95, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Depth-Tested Surface Normals Overlay',
        entities: entities.map((e) => {
          if (e.id === 'pnts-columbia-center-cylinder') {
            return { ...e, color: [0.2, 0.9, 0.35, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Point Cloud Classification
function generatePointCloudClassification() {
  const markerCylinder = {
    id: 'pnts-class-marker-cylinder',
    name: 'Classified Point Cloud Survey Station Cylinder',
    show: true,
    positionDegrees: [2.928, 46.227, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.25, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      pointCloudAssetId: 16421,
      classificationType: 'CESIUM_3D_TILE',
      classificationUrl: 'Classification/PointCloud/tileset.json',
    },
  };

  const prismBox = {
    id: 'pnts-class-prism-box',
    name: '3D Geometry Classification Prism Volume Box',
    show: true,
    positionDegrees: [2.935, 46.233, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.25, 0.85, 0.45, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 1.0, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      styleExpression: 'color("rgba(255, 0, 0, 0.6)")',
      experimentalGeometryTiles: true,
    },
  };

  const semanticSphere = {
    id: 'pnts-class-semantic-sphere',
    name: 'Semantic Classification Label Reference Sphere',
    show: true,
    positionDegrees: [2.921, 46.221, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.75, 0.15, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      classes: ['Architecture', 'Ground', 'Vegetation', 'Artifacts'],
    },
  };

  const boundaryRect = {
    id: 'pnts-class-boundary-rect',
    name: 'Point Cloud Semantic Zoning Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [2.91, 46.21, 2.95, 46.25],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.55, 0.35, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.55, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      spatialIndex: 'Morton Z-Order',
      classificationActive: true,
    },
  };

  const entities = [markerCylinder, prismBox, semanticSphere, boundaryRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Semantic Zoning Classification on Point Cloud',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Building Structure Classification (Red Highlight)',
        entities: entities.map((e) => {
          if (e.id === 'pnts-class-prism-box') {
            return { ...e, color: [0.95, 0.2, 0.3, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Ground & Vegetation Classification (Green Tint)',
        entities: entities.map((e) => {
          if (e.id === 'pnts-class-semantic-sphere') {
            return { ...e, color: [0.15, 0.9, 0.35, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Unclassified Point Cloud Base Shading',
        entities: entities.map((e) => {
          if (e.id === 'pnts-class-marker-cylinder') {
            return { ...e, color: [0.7, 0.7, 0.7, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles Point Cloud Shading
function generatePointCloudShading() {
  const helensCylinder = {
    id: 'pnts-shading-st-helens-cylinder',
    name: 'Mount St. Helens Volcanic Peak Anchor Cylinder',
    show: true,
    positionDegrees: [-122.195, 46.191, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.45, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.65, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      dataset: 'St. Helens',
      eyeDomeLighting: true,
      geometricErrorScale: 1.0,
    },
  };

  const attenuationBox = {
    id: 'pnts-shading-attenuation-box',
    name: 'Dynamic Point Attenuation Calibration Box',
    show: true,
    positionDegrees: [-122.188, 46.198, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.25, 0.65, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      maximumAttenuation: 16.0,
      baseResolution_meters: 0.05,
    },
  };

  const edlSphere = {
    id: 'pnts-shading-edl-sphere',
    name: 'Eye Dome Lighting Depth Gradient Sphere',
    show: true,
    positionDegrees: [-122.202, 46.184, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.85, 0.55, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      edlStrength: 1.0,
      edlRadius: 1.0,
    },
  };

  const surveyRect = {
    id: 'pnts-shading-survey-rect',
    name: 'Volcanic Caldera Point Cloud Survey Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.22, 46.17, -122.17, 46.21],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.75, 0.25, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.45, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      shadingSupported: true,
      maximumScreenSpaceError: 16.0,
    },
  };

  const entities = [helensCylinder, attenuationBox, edlSphere, surveyRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: St. Helens Topography with Eye Dome Lighting (EDL)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Eye Dome Lighting Disabled (Raw Point Shading)',
        entities: entities.map((e) => {
          if (e.id === 'pnts-shading-edl-sphere') {
            return { ...e, color: [0.65, 0.65, 0.65, 0.85] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Geometric Error Attenuation Active (Smooth Surface)',
        entities: entities.map((e) => {
          if (e.id === 'pnts-shading-attenuation-box') {
            return { ...e, color: [0.95, 0.35, 0.55, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Historic Church Architectural Dataset Mode',
        entities: entities.map((e) => {
          if (e.id === 'pnts-shading-st-helens-cylinder') {
            return { ...e, color: [0.2, 0.85, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// Write helper
function writeData(id, data) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);
  fs.writeFileSync(jsonPath, JSON.stringify(data.entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify(data.variants, null, 2) + '\n');
  console.log(`Generated ${id}: ${data.entities.length} entities, ${data.variants.variants.length} variants`);
}

// Generate all Batch 60 recipes
console.log('Generating Batch 60 Sandcastle data...');
writeData('3d-tiles-picking-dev', generatePickingDev());
writeData('3d-tiles-point-cloud', generatePointCloud());
writeData('3d-tiles-point-cloud-classification', generatePointCloudClassification());
writeData('3d-tiles-point-cloud-shading', generatePointCloudShading());
console.log('Batch 60 data generation complete.');
