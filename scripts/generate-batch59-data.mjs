import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles NGA GPM Visualization
function generateNgaGpm() {
  const datumCylinder = {
    id: 'gpm-reference-datum-cylinder',
    name: 'NGA GPM Reference Geodetic Datum Cylinder',
    show: true,
    positionDegrees: [44.385, 33.315, 2500.0],
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
      specification: 'NGA GPM v1.2',
      dataProvider: 'Maxar Evaluation',
      maximumScreenSpaceError: 4.0,
    },
  };

  const meshBox = {
    id: 'gpm-precision-mesh-box',
    name: 'Grid Precision Mesh Elevation Matrix Box',
    show: true,
    positionDegrees: [44.392, 33.322, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.55, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      depthPlaneEllipsoidOffset: 10000,
      cacheBytesBudget: 2147483648,
    },
  };

  const covarianceSphere = {
    id: 'gpm-covariance-ellipsoid',
    name: '1-Sigma Positional Error Covariance Ellipsoid',
    show: true,
    positionDegrees: [44.378, 33.308, 2200.0],
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
      errorMetric: 'Covariance Matrix [Cxx, Cyy, Czz]',
      confidenceLevel: '95%',
    },
  };

  const surveyRect = {
    id: 'gpm-evaluation-survey-rect',
    name: 'NGA GPM Precision Assessment Extent Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [44.36, 33.29, 44.41, 33.34],
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
      depthPlaneOffsetActive: true,
      staticCredit: 'Maxar GPM Evaluation',
    },
  };

  const entities = [datumCylinder, meshBox, covarianceSphere, surveyRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Default NGA GPM Mesh & Error Covariance',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Precision Error Ellipsoids (1-Sigma Covariance)',
        entities: entities.map((e) => {
          if (e.id === 'gpm-covariance-ellipsoid') {
            return { ...e, color: [0.2, 0.95, 0.45, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Maxar High-Resolution Grid Elevation Profile',
        entities: entities.map((e) => {
          if (e.id === 'gpm-precision-mesh-box') {
            return { ...e, color: [0.95, 0.35, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Strict SSE Refinement Mode (SSE = 2.0)',
        entities: entities.map((e) => {
          if (e.id === 'gpm-reference-datum-cylinder') {
            return { ...e, color: [0.3, 0.85, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles Performance Testing Dev
function generatePerfTesting() {
  const originCylinder = {
    id: 'perf-benchmark-origin-cylinder',
    name: 'Streaming Performance Benchmark Anchor Cylinder',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.85, 0.25, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      flightDuration: 8.0,
      benchmarkMode: 'Camera Tour',
    },
  };

  const waypointBox = {
    id: 'perf-benchmark-tour-waypoint-box',
    name: 'Camera Tour Trajectory Waypoint Target Box',
    show: true,
    positionDegrees: [-75.605, 40.048, 2000.0],
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
      tourLoadsMetric: true,
      lastTotalLoaded: 128,
    },
  };

  const heatmapSphere = {
    id: 'perf-heatmap-property-sphere',
    name: 'Tile Streaming Heatmap Property Palette Sphere',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.85, 0.15, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      heatmapTileProperty: 'geometricError',
      rangeMinMax: [-1.0, 1.0],
    },
  };

  const boundaryRect = {
    id: 'perf-streaming-boundary-rect',
    name: 'Tile Cache Throughput Testing Spatial Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.35, 0.65, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.85, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      cacheThrottlingTest: true,
      tourTimeSeconds: 24.5,
    },
  };

  const entities = [originCylinder, waypointBox, heatmapSphere, boundaryRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Default Performance Benchmark Camera Tour',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Heatmap Property Colorization (Geometric Error)',
        entities: entities.map((e) => {
          if (e.id === 'perf-heatmap-property-sphere') {
            return { ...e, color: [0.95, 0.35, 0.15, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'High-Frequency Camera Flight Stress Test',
        entities: entities.map((e) => {
          if (e.id === 'perf-benchmark-tour-waypoint-box') {
            return { ...e, color: [0.15, 0.95, 0.75, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Cache Exhaustion & Memory Pressure Inspection',
        entities: entities.map((e) => {
          if (e.id === 'perf-streaming-boundary-rect') {
            return { ...e, color: [0.85, 0.15, 0.45, 0.8] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Photogrammetry
function generatePhotogrammetry() {
  const coreCylinder = {
    id: 'photogrammetry-mesh-core-cylinder',
    name: 'Photogrammetric Mesh Center Landmark Cylinder',
    show: true,
    positionDegrees: [144.963, -37.813, 2500.0],
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
      ionAssetId: 40866,
      meshProvider: 'Aerometrex Melbourne Photogrammetry',
      worldTerrainDraped: true,
    },
  };

  const blockBox = {
    id: 'photogrammetry-textured-block-box',
    name: 'High-Fidelity PBR Texture Mapping Sample Block',
    show: true,
    positionDegrees: [144.97, -37.808, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.95, 0.65, 0.25, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      diffuseTextureChannel: true,
      ambientOcclusionDraped: true,
    },
  };

  const surveySphere = {
    id: 'photogrammetry-drone-survey-sphere',
    name: 'Aerial Drone Survey Radiance Dome Sphere',
    show: true,
    positionDegrees: [144.956, -37.818, 2200.0],
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
      sensorType: 'Oblique Multi-Camera Aerial Rig',
      groundSamplingDistance: '2.5cm/pixel',
    },
  };

  const corridorRect = {
    id: 'photogrammetry-flight-corridor-rect',
    name: 'Metropolitan Aerial Survey Flight Corridor Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [144.94, -37.83, 144.99, -37.79],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.75, 0.35, 0.85, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.55, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      zoomToTargetActive: true,
      coordinateSystem: 'MGA Zone 55 / GDA2020',
    },
  };

  const entities = [coreCylinder, blockBox, surveySphere, corridorRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Ion Asset 40866 Photogrammetry Mesh',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Photorealistic PBR Sunlight & Shadow Draping',
        entities: entities.map((e) => {
          if (e.id === 'photogrammetry-textured-block-box') {
            return { ...e, color: [1.0, 0.8, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Wireframe Mesh Geometry Accents',
        entities: entities.map((e) => {
          if (e.id === 'photogrammetry-mesh-core-cylinder') {
            return { ...e, color: [0.2, 0.95, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Macro Panoramic City Overview Perspective',
        entities: entities.map((e) => {
          if (e.id === 'photogrammetry-flight-corridor-rect') {
            return { ...e, color: [0.85, 0.2, 0.4, 0.8] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles Photogrammetry Classification
function generateClassification() {
  const parcelPillar = {
    id: 'photo-class-parcel-pillar-cylinder',
    name: 'Classified Cadastral Parcel Anchor Pillar',
    show: true,
    positionDegrees: [144.963, -37.813, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.2, 0.2, 0.65],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.4, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      classificationType: 'CESIUM_3D_TILE',
      classificationUrl: 'Classification/Photogrammetry/tileset.json',
      targetMeshAssetId: 40866,
    },
  };

  const volumeBox = {
    id: 'photo-class-volume-box',
    name: '3D Classification Prism Zoning Volume Box',
    show: true,
    positionDegrees: [144.97, -37.808, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2500.0, 2500.0, 2200.0],
    },
    color: [0.2, 0.85, 0.35, 0.65],
    fill: true,
    outline: true,
    outlineColor: [0.4, 1.0, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      styleColorExpression: 'rgba(255, 0, 0, 0.5)',
      b3dmClassificationOverB3dm: true,
    },
  };

  const zoneSphere = {
    id: 'photo-class-semantic-zone-sphere',
    name: 'Urban Planning Functional Zone Hemisphere',
    show: true,
    positionDegrees: [144.956, -37.818, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.25, 0.55, 0.95, 0.75],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      zoneCategory: 'Commercial High-Density',
      taxAssessmentActive: true,
    },
  };

  const drapedRect = {
    id: 'photo-class-draped-boundary-rect',
    name: 'Classified Cadastral Survey Footprint Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [144.94, -37.83, 144.99, -37.79],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.85, 0.65, 0.15, 0.6],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.35, 0.9],
    outlineWidth: 1.0,
    properties: {
      terrainClassificationType: 'BOTH',
      featureSelectionHighlighted: true,
    },
  };

  const entities = [parcelPillar, volumeBox, zoneSphere, drapedRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Red Translucent 3D Tile Classification Overlay',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Green Zoning Classification Overlay (rgba(0, 255, 0, 0.5))',
        entities: entities.map((e) => {
          if (e.id === 'photo-class-volume-box') {
            return { ...e, color: [0.1, 0.95, 0.3, 0.75] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Blue Water & Open Space Classification Overlay',
        entities: entities.map((e) => {
          if (e.id === 'photo-class-zone-sphere') {
            return { ...e, color: [0.15, 0.5, 0.95, 0.8] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Classification Suppressed (Pure Photogrammetry)',
        entities: entities.map((e) => {
          if (e.id === 'photo-class-parcel-pillar-cylinder') {
            return { ...e, color: [0.8, 0.8, 0.8, 0.9] };
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

// Generate all Batch 59 recipes
console.log('Generating Batch 59 Sandcastle data...');
writeData('3d-tiles-nga-gpm-visualization', generateNgaGpm());
writeData('3d-tiles-performance-testing-dev', generatePerfTesting());
writeData('3d-tiles-photogrammetry', generatePhotogrammetry());
writeData('3d-tiles-photogrammetry-classification', generateClassification());
console.log('Batch 59 data generation complete.');
