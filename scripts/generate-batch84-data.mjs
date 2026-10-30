// Generate test fixtures for Batch 84 recipes:
// 1. procedural-terrain
// 2. projection
// 3. sample-height-from-3d-tiles
// 4. scene-rendering-performance

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: procedural-terrain
const proceduralTerrainBaseline = [
  {
    id: 'procedural-terrain-peak-box',
    name: 'FBM Value Noise Peak Mountain Core',
    show: true,
    positionDegrees: [86.56, 27.56, 4500.0],
    shape: {
      type: 'box',
      dimensions: [12000.0, 12000.0, 9000.0]
    },
    fill: true,
    outline: true,
    color: [0.85, 0.9, 0.95, 1.0],
    outlineColor: [0.4, 0.45, 0.5, 1.0]
  },
  {
    id: 'procedural-terrain-ridge-cylinder',
    name: 'Fractal Elevation Ridge Spine',
    show: true,
    positionDegrees: [86.58, 27.58, 3000.0],
    shape: {
      type: 'cylinder',
      length: 6000.0,
      topRadius: 1500.0,
      bottomRadius: 2500.0
    },
    fill: true,
    outline: true,
    color: [0.55, 0.6, 0.65, 1.0],
    outlineColor: [0.3, 0.35, 0.4, 1.0]
  },
  {
    id: 'procedural-terrain-valley-sphere',
    name: 'Procedural Valley Datum Sphere',
    show: true,
    positionDegrees: [86.54, 27.54, 1500.0],
    shape: {
      type: 'ellipsoid',
      radii: [1800.0, 1800.0, 1800.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.45, 0.35, 1.0],
    outlineColor: [0.1, 0.25, 0.18, 1.0]
  },
  {
    id: 'procedural-terrain-bounds-rect',
    name: 'Custom Heightmap Procedural Tile Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.4, 27.4, 86.7, 27.7],
      height: 0.0,
      extrudedHeight: 500.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.35, 0.4, 0.6],
    outlineColor: [0.8, 0.85, 0.9, 1.0]
  }
];

const proceduralTerrainVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Value Noise FBM Mountain Mode (Octaves Accumulation, Peak 9000m)',
      entities: proceduralTerrainBaseline
    },
    {
      label: 'Mathematical Sine Wave Ridges Mode (Periodic Sine Terrain, Peak 4000m)',
      entities: [
        {
          ...proceduralTerrainBaseline[0],
          id: 'procedural-terrain-sine-peak-box',
          name: 'Sine Wave Periodic Ridge Peak Structure',
          positionDegrees: [86.56, 27.56, 2000.0],
          shape: {
            type: 'box',
            dimensions: [10000.0, 10000.0, 4000.0]
          },
          color: [0.4, 0.7, 0.95, 1.0]
        },
        proceduralTerrainBaseline[1],
        proceduralTerrainBaseline[2],
        proceduralTerrainBaseline[3]
      ]
    },
    {
      label: 'High-Frequency Ripple Procedural Terrain (High Octaves 6000m Crest)',
      entities: [
        proceduralTerrainBaseline[0],
        {
          ...proceduralTerrainBaseline[1],
          id: 'procedural-terrain-ripple-spine',
          name: 'High Frequency Octave Wave Spine',
          shape: {
            type: 'cylinder',
            length: 8000.0,
            topRadius: 1000.0,
            bottomRadius: 3000.0
          },
          color: [0.75, 0.5, 0.8, 1.0]
        },
        proceduralTerrainBaseline[2],
        proceduralTerrainBaseline[3]
      ]
    },
    {
      label: 'Smooth Rolling Dunes Procedural Terrain (Gentle Gradient Low Frequency)',
      entities: [
        proceduralTerrainBaseline[0],
        proceduralTerrainBaseline[1],
        {
          ...proceduralTerrainBaseline[2],
          id: 'procedural-terrain-dune-dome',
          name: 'Smooth Dune Low Gradient Dome',
          shape: {
            type: 'ellipsoid',
            radii: [3000.0, 3000.0, 1200.0]
          },
          color: [0.85, 0.75, 0.45, 1.0]
        },
        proceduralTerrainBaseline[3]
      ]
    }
  ]
};

// Recipe 2: projection
const projectionBaseline = [
  {
    id: 'projection-milktruck-box',
    name: 'Cesium Milk Truck Model Bounding Hull',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 12.0],
    shape: {
      type: 'box',
      dimensions: [28.0, 14.0, 16.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.85, 0.2, 1.0],
    outlineColor: [0.4, 0.35, 0.05, 1.0]
  },
  {
    id: 'projection-frustum-cylinder',
    name: 'Orthographic Projection Optical Vector Column',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 50.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 18.0,
      bottomRadius: 18.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.6, 0.9, 0.75],
    outlineColor: [0.1, 0.3, 0.6, 1.0]
  },
  {
    id: 'projection-anchor-sphere',
    name: 'Local Tangent Coordinate Frame Origin Marker',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 0.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0]
    },
    fill: true,
    outline: true,
    color: [1.0, 0.2, 0.2, 1.0],
    outlineColor: [0.5, 0.05, 0.05, 1.0]
  },
  {
    id: 'projection-staging-rect',
    name: 'Ground Staging Pad and Reference Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.076, 44.049, -123.073, 44.052],
      height: 0.0,
      extrudedHeight: 2.0
    },
    fill: true,
    outline: true,
    color: [0.4, 0.4, 0.45, 0.7],
    outlineColor: [0.8, 0.8, 0.85, 1.0]
  }
];

const projectionVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Orthographic Parallel Projection Mode (Isometric Milk Truck View)',
      entities: projectionBaseline
    },
    {
      label: 'Perspective Projection Mode (Natural Focal Depth & Vanishing Perspective)',
      entities: [
        {
          ...projectionBaseline[0],
          id: 'projection-perspective-truck-box',
          name: 'Perspective Vanishing Point Aligned Truck Hull',
          color: [0.2, 0.85, 0.4, 1.0]
        },
        projectionBaseline[1],
        projectionBaseline[2],
        projectionBaseline[3]
      ]
    },
    {
      label: 'Close-Up Telephoto Orthographic Inspection (Zero Parallax Distortion)',
      entities: [
        projectionBaseline[0],
        {
          ...projectionBaseline[1],
          id: 'projection-telephoto-cylinder',
          name: 'Narrow Telephoto Parallel Column',
          shape: {
            type: 'cylinder',
            length: 80.0,
            topRadius: 10.0,
            bottomRadius: 10.0
          },
          color: [0.95, 0.4, 0.2, 0.8]
        },
        projectionBaseline[2],
        projectionBaseline[3]
      ]
    },
    {
      label: 'Wide-Angle Aerial Perspective Context View (Wide FOV Planetary Arc)',
      entities: [
        projectionBaseline[0],
        projectionBaseline[1],
        {
          ...projectionBaseline[2],
          id: 'projection-wideangle-sphere',
          name: 'Wide FOV Angular Coverage Dome',
          shape: {
            type: 'ellipsoid',
            radii: [16.0, 16.0, 16.0]
          },
          color: [0.65, 0.2, 0.85, 1.0]
        },
        projectionBaseline[3]
      ]
    }
  ]
};

// Recipe 3: sample-height-from-3d-tiles
const sampleHeightBaseline = [
  {
    id: 'sampleheight-profile-polyline-box',
    name: 'Chester County 3D Tiles Clamped Profile Transect',
    show: true,
    positionDegrees: [-75.59675, 40.03911, 100.0],
    shape: {
      type: 'box',
      dimensions: [120.0, 60.0, 30.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.1, 1.0],
    outlineColor: [0.4, 0.35, 0.05, 1.0]
  },
  {
    id: 'sampleheight-transect-cylinder',
    name: 'Interpolated Borehole Elevation Sample Column',
    show: true,
    positionDegrees: [-75.59647, 40.03872, 110.0],
    shape: {
      type: 'cylinder',
      length: 35.0,
      topRadius: 6.0,
      bottomRadius: 6.0
    },
    fill: true,
    outline: true,
    color: [0.9, 0.2, 0.2, 1.0],
    outlineColor: [0.45, 0.05, 0.05, 1.0]
  },
  {
    id: 'sampleheight-datum-sphere',
    name: 'End Target Height Sample Anchor Probe',
    show: true,
    positionDegrees: [-75.59703, 40.03950, 110.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.3, 1.0],
    outlineColor: [0.05, 0.4, 0.1, 1.0]
  },
  {
    id: 'sampleheight-tileset-rect',
    name: 'Exton BIM / Photogrammetry Tileset Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.599, 40.037, -75.594, 40.041],
      height: 70.0,
      extrudedHeight: 85.0
    },
    fill: true,
    outline: true,
    color: [0.25, 0.35, 0.5, 0.65],
    outlineColor: [0.8, 0.85, 0.95, 1.0]
  }
];

const sampleHeightVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Clamped Profile Sampling (30 Interpolated Red Spheres & Yellow Polyline)',
      entities: sampleHeightBaseline
    },
    {
      label: 'High-Density 60 Points Topographic Elevation Profile (Detailed Roof Clamping)',
      entities: [
        {
          ...sampleHeightBaseline[0],
          id: 'sampleheight-highdensity-profile-box',
          name: 'High Density 60 Point Detailed Roof Profile',
          color: [0.2, 0.9, 0.8, 1.0]
        },
        sampleHeightBaseline[1],
        sampleHeightBaseline[2],
        sampleHeightBaseline[3]
      ]
    },
    {
      label: 'Sparse 10 Points Structural Span Inspection (Rapid Elevation Check)',
      entities: [
        sampleHeightBaseline[0],
        {
          ...sampleHeightBaseline[1],
          id: 'sampleheight-sparse-column',
          name: 'Rapid Structural Span Column',
          shape: {
            type: 'cylinder',
            length: 50.0,
            topRadius: 10.0,
            bottomRadius: 10.0
          },
          color: [0.95, 0.5, 0.1, 1.0]
        },
        sampleHeightBaseline[2],
        sampleHeightBaseline[3]
      ]
    },
    {
      label: 'Orthogonal Cross-Sectional Clamped Elevation Transect (Transverse Crosscut)',
      entities: [
        sampleHeightBaseline[0],
        sampleHeightBaseline[1],
        {
          ...sampleHeightBaseline[2],
          id: 'sampleheight-crosscut-sphere',
          name: 'Transverse Profile Center Intersection Sphere',
          shape: {
            type: 'ellipsoid',
            radii: [12.0, 12.0, 12.0]
          },
          color: [0.7, 0.2, 0.9, 1.0]
        },
        sampleHeightBaseline[3]
      ]
    }
  ]
};

// Recipe 4: scene-rendering-performance
const sceneRenderBaseline = [
  {
    id: 'scenerender-demand-box',
    name: 'Explicit Request Render Demand Box (CornflowerBlue Idle, Yellow Hover)',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 15000.0],
    shape: {
      type: 'box',
      dimensions: [1000000.0, 1000000.0, 30000.0]
    },
    fill: true,
    outline: true,
    color: [0.392, 0.584, 0.929, 1.0],
    outlineColor: [0.15, 0.25, 0.45, 1.0]
  },
  {
    id: 'scenerender-aircraft-cylinder',
    name: 'CesiumAir Dynamic Flight Simulation Vector Pillar',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 5000.0],
    shape: {
      type: 'cylinder',
      length: 4000.0,
      topRadius: 1500.0,
      bottomRadius: 2500.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.9, 0.2, 1.0],
    outlineColor: [0.45, 0.4, 0.05, 1.0]
  },
  {
    id: 'scenerender-cadence-sphere',
    name: 'Simulation Time Render Tick Indicator Probe',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 60000.0],
    shape: {
      type: 'ellipsoid',
      radii: [40000.0, 40000.0, 40000.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.85, 0.4, 1.0],
    outlineColor: [0.05, 0.45, 0.15, 1.0]
  },
  {
    id: 'scenerender-envelope-rect',
    name: 'East Coast Explicit Render Evaluation Area',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-82.0, 35.0, -70.0, 45.0],
      height: 0.0,
      extrudedHeight: 5000.0
    },
    fill: true,
    outline: true,
    color: [0.25, 0.3, 0.45, 0.55],
    outlineColor: [0.75, 0.8, 0.9, 1.0]
  }
];

const sceneRenderVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Interactive Hover Demand Mode (Yellow Highlight On Mouse Move)',
      entities: sceneRenderBaseline
    },
    {
      label: 'Animated Aircraft Simulation Scenario (Maximum Render Time Change Step)',
      entities: [
        {
          ...sceneRenderBaseline[0],
          id: 'scenerender-aircraft-track-box',
          name: 'Tracked Aircraft Simulation Corridor',
          positionDegrees: [-123.0744619, 44.0503706, 5000.0],
          shape: {
            type: 'box',
            dimensions: [6000.0, 3000.0, 2000.0]
          },
          color: [0.95, 0.3, 0.2, 1.0]
        },
        sceneRenderBaseline[1],
        sceneRenderBaseline[2],
        sceneRenderBaseline[3]
      ]
    },
    {
      label: '3D Tileset Static Urban Scene (Zero Energy Render Quiescence)',
      entities: [
        sceneRenderBaseline[0],
        {
          ...sceneRenderBaseline[1],
          id: 'scenerender-static-tileset-cylinder',
          name: 'Static Tileset Resting State Column',
          positionDegrees: [-75.59777, 40.03883, 100.0],
          shape: {
            type: 'cylinder',
            length: 100.0,
            topRadius: 30.0,
            bottomRadius: 30.0
          },
          color: [0.3, 0.75, 0.85, 1.0]
        },
        sceneRenderBaseline[2],
        sceneRenderBaseline[3]
      ]
    },
    {
      label: 'Explicit Background API Invalidation (scene.requestRender Force Pass)',
      entities: [
        sceneRenderBaseline[0],
        sceneRenderBaseline[1],
        {
          ...sceneRenderBaseline[2],
          id: 'scenerender-invalidation-burst-sphere',
          name: 'Forced Invalidation Burst Beacon',
          shape: {
            type: 'ellipsoid',
            radii: [50000.0, 50000.0, 50000.0]
          },
          color: [0.95, 0.8, 0.1, 1.0]
        },
        sceneRenderBaseline[3]
      ]
    }
  ]
};

// Write files
const files = [
  ['tests/sandcastle/full/procedural-terrain.json', proceduralTerrainBaseline],
  ['tests/sandcastle/full/procedural-terrain.variants.json', proceduralTerrainVariants],
  ['tests/sandcastle/full/projection.json', projectionBaseline],
  ['tests/sandcastle/full/projection.variants.json', projectionVariants],
  ['tests/sandcastle/full/sample-height-from-3d-tiles.json', sampleHeightBaseline],
  ['tests/sandcastle/full/sample-height-from-3d-tiles.variants.json', sampleHeightVariants],
  ['tests/sandcastle/full/scene-rendering-performance.json', sceneRenderBaseline],
  ['tests/sandcastle/full/scene-rendering-performance.variants.json', sceneRenderVariants]
];

for (const [relPath, data] of files) {
  const fullPath = path.join(root, relPath);
  fs.writeFileSync(fullPath, JSON.stringify(data, null, 2) + '\n');
  console.log('wrote', relPath);
}
