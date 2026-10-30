import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Gaussian Splatting Comparison
function generateSplattingComparison() {
  const leftCylinder = {
    id: 'gs-compare-left-pivot-cylinder',
    name: 'Primary Gaussian Radiance Field Left Pivot Cylinder',
    show: true,
    positionDegrees: [-75.165, 39.952, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      leftIonAssetId: 3667784,
      splitDirection: 'LEFT',
      viewHeadingPitchRange: [-50.0, -20.0, 100.0],
    },
  };

  const dividerBox = {
    id: 'gs-compare-split-divider-box',
    name: 'Dual-View Split Screen Divider Boundary Box',
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
      splitPosition: 0.5,
      interactiveSlider: true,
    },
  };

  const rightSphere = {
    id: 'gs-compare-right-kernel-sphere',
    name: 'Secondary Gaussian Radiance Field Right Kernel Sphere',
    show: true,
    positionDegrees: [-75.172, 39.946, 2200.0],
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
      rightIonAssetId: 3443919,
      splitDirection: 'RIGHT',
    },
  };

  const coverageRect = {
    id: 'gs-compare-coverage-rect',
    name: 'Radiance Field Comparison Spatial Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.19, 39.93, -75.14, 39.97],
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
      terrainDraping: true,
      captureComparisonMode: 'Gaussian 3D vs Hybrid',
    },
  };

  const entities = [leftCylinder, dividerBox, rightSphere, coverageRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: 50% Center Split (Radiance Field 1 vs 2)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Split Position 25% (Dominant Secondary Radiance Field)',
        entities: entities.map((e) => {
          if (e.id === 'gs-compare-right-kernel-sphere') {
            return { ...e, color: [0.95, 0.35, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Split Position 75% (Dominant Primary Radiance Field)',
        entities: entities.map((e) => {
          if (e.id === 'gs-compare-left-pivot-cylinder') {
            return { ...e, color: [0.15, 0.85, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Split Screen Disabled (Unified Full View)',
        entities: entities.map((e) => {
          if (e.id === 'gs-compare-split-divider-box') {
            return { ...e, color: [0.95, 0.85, 0.15, 0.85] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 2. 3D Tiles Inspector
function generateInspector() {
  const wireframeCylinder = {
    id: 'inspector-wireframe-cylinder',
    name: '3D Tiles Wireframe Inspection Anchor Cylinder',
    show: true,
    positionDegrees: [-74.006, 40.7128, 2500.0],
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
      ionAssetId: 75343,
      enableDebugWireframe: true,
      debugColorizeTiles: true,
    },
  };

  const boundingBox = {
    id: 'inspector-bounding-box',
    name: 'Tile Hierarchical Bounding Volume Box',
    show: true,
    positionDegrees: [-73.998, 40.719, 2000.0],
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
      debugShowBoundingVolume: true,
      debugShowContentBoundingVolume: true,
    },
  };

  const colorizeSphere = {
    id: 'inspector-tile-colorize-sphere',
    name: 'Tile Level Colorization Palette Reference Sphere',
    show: true,
    positionDegrees: [-74.015, 40.705, 2200.0],
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
      debugShowGeometricError: true,
      freezeTileLoading: false,
    },
  };

  const nycRect = {
    id: 'inspector-nyc-downtown-rect',
    name: 'NYC Financial District Urban Extent Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.03, 40.69, -73.98, 40.73],
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
      depthTestAgainstTerrain: true,
      dataSource: 'NYC OpenData 3D Buildings',
    },
  };

  const entities = [wireframeCylinder, boundingBox, colorizeSphere, nycRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Colorize Tiles Enabled & Wireframe Mode',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Bounding Volumes Visualized (Oriented Bounding Boxes)',
        entities: entities.map((e) => {
          if (e.id === 'inspector-bounding-box') {
            return { ...e, color: [0.2, 0.95, 0.85, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Freeze Tile Loading & Screen Space Error Debugging',
        entities: entities.map((e) => {
          if (e.id === 'inspector-tile-colorize-sphere') {
            return { ...e, color: [0.95, 0.45, 0.25, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Standard Photorealistic Shading (Debug Overlays Off)',
        entities: entities.map((e) => {
          if (e.id === 'inspector-wireframe-cylinder') {
            return { ...e, color: [0.7, 0.7, 0.7, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 3. 3D Tiles Interactivity
function generateInteractivity() {
  const pinCylinder = {
    id: 'interactivity-selected-pin-cylinder',
    name: 'Spatial Interaction Anchor & Pin Cylinder',
    show: true,
    positionDegrees: [-75.612, 40.042, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.15, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      rightClickAction: 'annotate',
      middleClickAction: 'hide',
      pickPositionSupported: true,
    },
  };

  const hoverBox = {
    id: 'interactivity-hover-bbox',
    name: 'Dynamic Feature Hover Bounding Box',
    show: true,
    positionDegrees: [-75.605, 40.048, 2000.0],
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
      hoverHighlightColor: 'yellow',
      depthTestAgainstTerrain: true,
    },
  };

  const annotationSphere = {
    id: 'interactivity-annotation-sphere',
    name: 'Spatial Annotation 3D Label Anchor Sphere',
    show: true,
    positionDegrees: [-75.618, 40.035, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.85, 0.25, 0.55, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      labelCollectionActive: true,
      annotationCount: 1,
    },
  };

  const surveyRect = {
    id: 'interactivity-survey-rect',
    name: 'Interactive Cadastral Survey Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.63, 40.03, -75.59, 40.06],
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
      knockoutBindings: true,
      middleClickActionActive: true,
    },
  };

  const entities = [pinCylinder, hoverBox, annotationSphere, surveyRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: Right Click Annotate & Middle Click Hide',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Annotations Active (3D Text Overlay Markers)',
        entities: entities.map((e) => {
          if (e.id === 'interactivity-annotation-sphere') {
            return { ...e, color: [0.95, 0.2, 0.3, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Hidden Features Mode (Selective Architectural Culling)',
        entities: entities.map((e) => {
          if (e.id === 'interactivity-hover-bbox') {
            return { ...e, color: [0.35, 0.45, 0.55, 0.5] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Reset Interactivity (All Features Restored)',
        entities: entities.map((e) => {
          if (e.id === 'interactivity-selected-pin-cylinder') {
            return { ...e, color: [0.2, 0.95, 0.4, 0.95] };
          }
          return { ...e };
        }),
      },
    ],
  };

  return { entities, variants };
}

// 4. 3D Tiles Interior (San Miguel Architectural Interior)
function generateInterior() {
  const pillarCylinder = {
    id: 'interior-san-miguel-pillar-cylinder',
    name: 'San Miguel Indoor Courtyard Column Pillar Cylinder',
    show: true,
    positionDegrees: [-100.917, 20.914, 2500.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.95, 0.55, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 125737,
      sceneModel: 'San Miguel Interior Architectural Mesh',
      author: 'Guillermo M. Leal Llaguno',
    },
  };

  const roomBox = {
    id: 'interior-courtyard-room-box',
    name: 'Interior Architectural Chamber Enclosure Box',
    show: true,
    positionDegrees: [-100.91, 20.92, 2000.0],
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
      initialDestination: [-1111583.37, -5855888.15, 2262561.44],
      initialOrientationHpr: [100.0, -15.0, 0.0],
    },
  };

  const lightSphere = {
    id: 'interior-spatial-light-sphere',
    name: 'Indoor Skylight Natural Lighting Hemisphere',
    show: true,
    positionDegrees: [-100.924, 20.908, 2200.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 1800.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.95, 0.95, 0.35, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: {
      ambientOcclusion: true,
      indoorCollisionDetection: false,
    },
  };

  const floorRect = {
    id: 'interior-architectural-floor-rect',
    name: 'Interior Ground Floor Layout Boundary Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-100.935, 20.9, -100.895, 20.935],
      height: 2000.0,
      extrudedHeight: 4000.0,
      rotation: 0.0,
    },
    color: [0.45, 0.75, 0.35, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.95, 0.55, 0.9],
    outlineWidth: 1.0,
    properties: {
      floorAreaApprox: '850 sq meters',
      indoorNavigationEnabled: true,
    },
  };

  const entities = [pillarCylinder, roomBox, lightSphere, floorRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Baseline: San Miguel Indoor Courtyard Perspective',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Enhanced Indoor Skylight & Warm Ambiance',
        entities: entities.map((e) => {
          if (e.id === 'interior-spatial-light-sphere') {
            return { ...e, color: [1.0, 0.85, 0.2, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Structural Wireframe Accents on Columns & Arches',
        entities: entities.map((e) => {
          if (e.id === 'interior-san-miguel-pillar-cylinder') {
            return { ...e, color: [0.2, 0.7, 0.95, 0.95] };
          }
          return { ...e };
        }),
      },
      {
        label: 'Overhead Architectural Cutaway Perspective',
        entities: entities.map((e) => {
          if (e.id === 'interior-courtyard-room-box') {
            return { ...e, color: [0.85, 0.35, 0.65, 0.9] };
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

// Generate all Batch 58 recipes
console.log('Generating Batch 58 Sandcastle data...');
writeData('3d-tiles-gaussian-splatting-comparison', generateSplattingComparison());
writeData('3d-tiles-inspector', generateInspector());
writeData('3d-tiles-interactivity', generateInteractivity());
writeData('3d-tiles-interior', generateInterior());
console.log('Batch 58 data generation complete.');
