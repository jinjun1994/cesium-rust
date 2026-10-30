import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Feature Styling (Seattle OSM Buildings)
function generateFeatureStyling() {
  const needleCylinder = {
    id: 'feature-styling-seattle-needle-cylinder',
    name: 'Seattle Space Needle Spatial Datum Cylinder',
    show: true,
    positionDegrees: [-122.3493, 47.6205, 2500.0],
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
      location: 'Seattle, WA',
      targetTileset: 'Cesium OSM Buildings',
      styleLanguage: '3D Tiles Styling 1.0',
    },
  };

  const gradientBox = {
    id: 'feature-styling-height-gradient-box',
    name: 'Height-Dependent HSL Color Gradient Box',
    show: true,
    positionDegrees: [-122.341, 47.615, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.25, 0.55, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      heightExpression: 'color("hsl(" + (${cesium#estimatedHeight} / 300) + ", 0.9, 0.5)")',
      minHeight: 0.0,
      maxHeight: 300.0,
    },
  };

  const materialSphere = {
    id: 'feature-styling-material-sphere',
    name: 'Building Material Classification Hemisphere',
    show: true,
    positionDegrees: [-122.355, 47.61, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.85, 0.85, 0.2, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      materialCategories: ['glass', 'concrete', 'brick', 'metal', 'wood'],
      defaultColor: 'color("cyan")',
    },
  };

  const downtownRect = {
    id: 'feature-styling-downtown-rect',
    name: 'Downtown Seattle Metropolitan Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.37, 47.59, -122.32, 47.63],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.25, 0.85, 0.45, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.45, 1.0, 0.65, 0.9],
    outlineWidth: 1.0,
    properties: {
      filterFilter: '${cesium#estimatedHeight} > 20',
      showExpression: 'true',
    },
  };

  const entities = [needleCylinder, gradientBox, materialSphere, downtownRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Default Seattle OSM Buildings Shading',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Color by Height Gradient (Warm HSL Spectrum)',
        entities: entities.map((e) => {
          if (e.id === 'feature-styling-height-gradient-box') {
            return { ...e, color: [0.95, 0.55, 0.1, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Color by Building Material (Discrete Palette)',
        entities: entities.map((e) => {
          if (e.id === 'feature-styling-material-sphere') {
            return { ...e, color: [0.3, 0.7, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Filter Height Threshold (${height} > 50m Skyscrapers)',
        entities: entities.map((e) => {
          if (e.id === 'feature-styling-downtown-rect') {
            return { ...e, color: [0.85, 0.15, 0.35, 0.85] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles Formats (Inspector & Multi-Format Showcase)
function generateFormats() {
  const monolithCylinder = {
    id: 'formats-b3dm-monolith-cylinder',
    name: 'B3DM Batched 3D Model Primary Cylinder',
    show: true,
    positionDegrees: [-75.615, 40.045, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.55, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      format: 'b3dm',
      sampleTileset: 'Tilesets/Tileset/tileset.json',
      inspectorMixin: true,
    },
  };

  const instizedBox = {
    id: 'formats-i3dm-instanced-box',
    name: 'I3DM Instanced 3D Model Forest Matrix Box',
    show: true,
    positionDegrees: [-75.608, 40.052, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.25, 0.85, 0.55, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      format: 'i3dm',
      sampleTileset: 'Instanced/InstancedOrientation/tileset.json',
      instanceCount: 25,
    },
  };

  const cloudSphere = {
    id: 'formats-pnts-cloud-sphere',
    name: 'PNTS Point Cloud Spectral Cluster Sphere',
    show: true,
    positionDegrees: [-75.622, 40.038, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.65, 0.95, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      format: 'pnts',
      sampleTileset: 'PointCloud/PointCloudRGB/tileset.json',
      semantics: 'RGB, POSITION, NORMAL',
    },
  };

  const campusRect = {
    id: 'formats-composite-campus-rect',
    name: 'CMPT Composite Tile & Translucent Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.635, 40.03, -75.595, 40.065],
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
      format: 'cmpt',
      translucentResource: 'Batched/BatchedTranslucent/tileset.json',
      shadows: true,
    },
  };

  const entities = [monolithCylinder, instizedBox, cloudSphere, campusRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Standard B3DM Opaque Buildings Showcase',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Translucent Batched 3D Model (BatchedTranslucent)',
        entities: entities.map((e) => {
          if (e.id === 'formats-b3dm-monolith-cylinder') {
            return { ...e, color: [0.85, 0.35, 0.25, 0.5] };
          }
          return { ...e };
        }),
      },
      {
        label: 'PNTS Point Cloud RGB Shading Focus',
        entities: entities.map((e) => {
          if (e.id === 'formats-pnts-cloud-sphere') {
            return { ...e, color: [0.1, 0.95, 0.8, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'I3DM Instanced Trees & Structures Matrix',
        entities: entities.map((e) => {
          if (e.id === 'formats-i3dm-instanced-box') {
            return { ...e, color: [0.15, 0.9, 0.3, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Gaussian Splats with LOD
function generateSplatsWithLod() {
  const coreCylinder = {
    id: 'splats-lod-core-cylinder',
    name: 'Gaussian Splats Scene Center Anchor Pillar',
    show: true,
    positionDegrees: [-122.1381, 47.6445, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.65, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      boundingSphereRadius: 50.0,
      viewOffsetHeadingPitchRange: [100.0, -25.0, 500.0],
      terrainEnabled: true,
    },
  };

  const octreeBox = {
    id: 'splats-lod-octree-box',
    name: 'Spatial Octree Hierarchical Level of Detail Volume',
    show: true,
    positionDegrees: [-122.132, 47.65, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.25, 0.75, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      maxScreenSpaceError: 16.0,
      progressiveStreaming: true,
    },
  };

  const radianceSphere = {
    id: 'splats-lod-radiance-sphere',
    name: 'Radiance Field Covariance Ellipsoid Boundary',
    show: true,
    positionDegrees: [-122.144, 47.639, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.85, 0.25, 0.75, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      splatRadiusScale: 1.0,
      pointBudget: 2000000,
    },
  };

  const captureRect = {
    id: 'splats-lod-capture-rect',
    name: 'Photogrammetric Drone Capture Area Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.16, 47.63, -122.11, 47.66],
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
      captureType: 'Aerial 3DGS Photogrammetry',
      ionAssetSupported: true,
    },
  };

  const entities = [coreCylinder, octreeBox, radianceSphere, captureRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Full LOD Gaussian Splats Radiance Field',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High-Density Splats (Max Point Budget: 4,000,000)',
        entities: entities.map((e) => {
          if (e.id === 'splats-lod-radiance-sphere') {
            return { ...e, color: [0.95, 0.35, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Coarse Low-LOD Preview (SSE = 32)',
        entities: entities.map((e) => {
          if (e.id === 'splats-lod-octree-box') {
            return { ...e, color: [0.95, 0.85, 0.2, 0.9] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Splat Radius Scaled (1.5x Dilation)',
        entities: entities.map((e) => {
          if (e.id === 'splats-lod-core-cylinder') {
            return { ...e, color: [0.2, 0.9, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles Gaussian Splatting
function generateGaussianSplatting() {
  const focalCylinder = {
    id: 'gaussian-splatting-focal-cylinder',
    name: '3DGS Focal Center Landmark Cylinder',
    show: true,
    positionDegrees: [-75.165, 39.952, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.25, 0.55, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.75, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 3667783,
      headingPitchRange: [0.0, -15.0, 200.0],
      radianceModel: '3D Gaussian Splats',
    },
  };

  const volumeBox = {
    id: 'gaussian-splatting-volume-box',
    name: 'Depth-Sorted Splat Rasterization Volume',
    show: true,
    positionDegrees: [-75.158, 39.958, 2000.0],
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
      sortOrder: 'Back-to-Front',
      blendEquation: 'Alpha Premultiplied',
    },
  };

  const ellipsoidKernel = {
    id: 'gaussian-splatting-ellipsoid-kernel',
    name: 'Covariance Ellipsoid 3D Kernel Envelope',
    show: true,
    positionDegrees: [-75.172, 39.946, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.35, 0.85, 0.35, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      sphericalHarmonicsDegree: 3,
      covarianceRank: 3,
    },
  };

  const coverageRect = {
    id: 'gaussian-splatting-coverage-rect',
    name: 'High-Resolution Scan Extent Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.19, 39.93, -75.14, 39.97],
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
      captureDate: '2024-Q3',
      pointCountApprox: 1200000,
    },
  };

  const entities = [focalCylinder, volumeBox, ellipsoidKernel, coverageRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Default 3D Gaussian Splatting Scene',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Spherical Harmonic Color Boost (Vivid Radiance)',
        entities: entities.map((e) => {
          if (e.id === 'gaussian-splatting-ellipsoid-kernel') {
            return { ...e, color: [0.4, 0.95, 0.4, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Low-Opacity Splat Inspection Mode (Alpha = 0.5)',
        entities: entities.map((e) => {
          if (e.id === 'gaussian-splatting-volume-box') {
            return { ...e, color: [0.95, 0.45, 0.25, 0.5] };
          }
          return { ...e };
        }),
      },
      {
        label: 'High-Contrast Feature Silhouette Overlay',
        entities: entities.map((e) => {
          if (e.id === 'gaussian-splatting-focal-cylinder') {
            return { ...e, color: [0.95, 0.9, 0.1, 0.95] };
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

// Generate all Batch 57 recipes
console.log('Generating Batch 57 Sandcastle data...');
writeData('3d-tiles-feature-styling', generateFeatureStyling());
writeData('3d-tiles-formats', generateFormats());
writeData('3d-tiles-gaussian-splats-with-lod', generateSplatsWithLod());
writeData('3d-tiles-gaussian-splatting', generateGaussianSplatting());
console.log('Batch 57 data generation complete.');
