// Generate test fixtures for Batch 87 recipes:
// 1. terrain-entity-batching-dev
// 2. terrain-exaggeration
// 3. terrain-performance-dev
// 4. terrain-tweaks-dev

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: terrain-entity-batching-dev
// Regional anchor: Mt. St. Helens / Rainier region [-122.25, 46.25]
const batchingBaseline = [
  {
    id: 'batching-concave-polygon-box',
    name: 'Concave Surface Polygon Bounding Hull',
    show: true,
    positionDegrees: [-122.18, 46.22, 1200.0],
    shape: {
      type: 'box',
      dimensions: [4500.0, 4500.0, 600.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.7, 0.95, 0.85],
    outlineColor: [0.05, 0.35, 0.55, 1.0]
  },
  {
    id: 'batching-grid-cluster-anchor',
    name: '10x10 Batched Classification Squares Anchor',
    show: true,
    positionDegrees: [-122.2534, 46.2522, 1000.0],
    shape: {
      type: 'cylinder',
      length: 800.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.55, 0.15, 0.75],
    outlineColor: [0.55, 0.25, 0.05, 1.0]
  },
  {
    id: 'batching-checkerboard-rect',
    name: 'Checkerboard Surface Material Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.17778, 46.36169, -120.17778, 48.36169],
      height: 800.0,
      extrudedHeight: 1200.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.2, 0.65],
    outlineColor: [0.65, 0.45, 0.05, 1.0]
  },
  {
    id: 'batching-textured-ellipse-cylinder',
    name: 'Rotated Texture Classification Ellipse Column',
    show: true,
    positionDegrees: [-121.7071, 45.9438, 900.0],
    shape: {
      type: 'ellipsoid',
      radii: [1500.0, 3000.0, 900.0]
    },
    fill: true,
    outline: true,
    color: [0.35, 0.85, 0.45, 0.8],
    outlineColor: [0.1, 0.45, 0.15, 1.0]
  }
];

const batchingVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Batched 10x10 Surface Grid & Textured Terrain Primitives',
      entities: batchingBaseline
    },
    {
      label: 'Dense 100-Entity Terrain Overlay Stress Test (High Density Mode)',
      entities: [
        {
          ...batchingBaseline[0],
          id: 'batching-dense-grid-core',
          name: 'High-Density Terrain Classification Mesh',
          color: [0.85, 0.25, 0.9, 0.9]
        },
        batchingBaseline[1],
        batchingBaseline[2],
        batchingBaseline[3]
      ]
    },
    {
      label: 'High-Contrast Orange/Yellow Checkerboard Mode',
      entities: [
        batchingBaseline[0],
        batchingBaseline[1],
        {
          ...batchingBaseline[2],
          id: 'batching-vibrant-checker-rect',
          name: 'Vibrant Dual-Tone Checkerboard Partition',
          color: [1.0, 0.6, 0.0, 0.8]
        },
        batchingBaseline[3]
      ]
    },
    {
      label: 'Rotated Ellipse & Surface Decal Focus',
      entities: [
        batchingBaseline[0],
        batchingBaseline[1],
        batchingBaseline[2],
        {
          ...batchingBaseline[3],
          id: 'batching-rotated-decal-marker',
          name: 'Isolated 45-Degree St-Rotated Ellipse Decal',
          color: [0.2, 0.9, 0.8, 0.95]
        }
      ]
    }
  ]
};

// Recipe 2: terrain-exaggeration
// Alpine / Everest elevation benchmark (approx lon 86.6, lat 27.25)
const exaggerationBaseline = [
  {
    id: 'exaggeration-target-ellipsoid',
    name: 'Clamped Red Target Ellipsoid (400m Radii)',
    show: true,
    positionDegrees: [86.595, 27.258, 2400.0],
    shape: {
      type: 'ellipsoid',
      radii: [400.0, 400.0, 400.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.15, 0.15, 1.0],
    outlineColor: [0.55, 0.05, 0.05, 1.0]
  },
  {
    id: 'exaggeration-datum-plane',
    name: '2400m Relative Exaggeration Datum Plane',
    show: true,
    positionDegrees: [86.595, 27.258, 2400.0],
    shape: {
      type: 'box',
      dimensions: [6000.0, 6000.0, 50.0]
    },
    fill: true,
    outline: true,
    color: [0.3, 0.75, 0.95, 0.5],
    outlineColor: [0.1, 0.35, 0.65, 0.9]
  },
  {
    id: 'exaggeration-band-indicator',
    name: 'Elevation Band Transition Indicator Column',
    show: true,
    positionDegrees: [86.61, 27.27, 2450.0],
    shape: {
      type: 'cylinder',
      length: 600.0,
      topRadius: 250.0,
      bottomRadius: 250.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.9, 0.4, 0.75],
    outlineColor: [0.05, 0.45, 0.15, 1.0]
  },
  {
    id: 'exaggeration-region-rect',
    name: 'Exaggerated Valley Survey Sector',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.52, 27.20, 86.68, 27.32],
      height: 1800.0,
      extrudedHeight: 2500.0
    },
    fill: true,
    outline: true,
    color: [0.4, 0.5, 0.7, 0.4],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const exaggerationVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: 2.0x Vertical Exaggeration with 2400m Relative Datum & Elevation Bands',
      entities: exaggerationBaseline
    },
    {
      label: 'Extreme 4.0x Vertical Exaggeration Mode (High-Relief Alpine Ruggedness)',
      entities: [
        exaggerationBaseline[0],
        {
          ...exaggerationBaseline[1],
          id: 'exaggeration-extreme-datum',
          name: 'Elevated Extreme 4x Datum Indicator',
          color: [0.95, 0.45, 0.1, 0.7]
        },
        exaggerationBaseline[2],
        exaggerationBaseline[3]
      ]
    },
    {
      label: 'Standard 1.0x Flat True-Elevation Baseline (No Exaggeration)',
      entities: [
        exaggerationBaseline[0],
        exaggerationBaseline[1],
        {
          ...exaggerationBaseline[2],
          id: 'exaggeration-flat-indicator',
          name: 'Flat Datum Unscaled Marker',
          color: [0.65, 0.65, 0.65, 0.8]
        },
        exaggerationBaseline[3]
      ]
    },
    {
      label: 'Zero-Datum 2.5x Exaggeration Mode (Sea-Level Scaled Reference)',
      entities: [
        exaggerationBaseline[0],
        exaggerationBaseline[1],
        exaggerationBaseline[2],
        {
          ...exaggerationBaseline[3],
          id: 'exaggeration-zero-datum-rect',
          name: 'Sea Level Datum Valley Bound',
          color: [0.15, 0.7, 0.85, 0.55]
        }
      ]
    }
  ]
};

// Recipe 3: terrain-performance-dev
// Mount Everest Summit and Corridor [86.925, 27.988]
const performanceBaseline = [
  {
    id: 'everest-summit-marker',
    name: 'Mount Everest 8848m Summit Marker',
    show: true,
    positionDegrees: [86.925, 27.988, 8848.0],
    shape: {
      type: 'ellipsoid',
      radii: [350.0, 350.0, 500.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.15, 1.0],
    outlineColor: [0.55, 0.45, 0.05, 1.0]
  },
  {
    id: 'everest-horizontal-flight-box',
    name: 'Horizontal Flight Corridor Trajectory Anchor',
    show: true,
    positionDegrees: [86.91, 27.97, 8200.0],
    shape: {
      type: 'box',
      dimensions: [4000.0, 1500.0, 400.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.95, 0.8],
    outlineColor: [0.05, 0.35, 0.55, 1.0]
  },
  {
    id: 'everest-khumbu-corridor-cylinder',
    name: 'Khumbu Icefall Approach Inspection Column',
    show: true,
    positionDegrees: [86.87, 27.99, 7200.0],
    shape: {
      type: 'cylinder',
      length: 1200.0,
      topRadius: 600.0,
      bottomRadius: 600.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.95, 0.55, 0.75],
    outlineColor: [0.05, 0.45, 0.2, 1.0]
  },
  {
    id: 'everest-base-camp-staging-rect',
    name: 'South Base Camp Performance Sector Grid',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.82, 27.94, 86.98, 28.02],
      height: 5200.0,
      extrudedHeight: 5800.0
    },
    fill: true,
    outline: true,
    color: [0.45, 0.55, 0.75, 0.45],
    outlineColor: [0.9, 0.95, 1.0, 0.9]
  }
];

const performanceVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Everest Horizontal Flight Path Performance Benchmark',
      entities: performanceBaseline
    },
    {
      label: 'Everest Orthogonal Top-Down Inspection Mode (Nadir Tile Streaming)',
      entities: [
        {
          ...performanceBaseline[0],
          id: 'everest-nadir-summit',
          name: 'Nadir Inspection Peak Marker',
          color: [0.95, 0.3, 0.15, 1.0]
        },
        performanceBaseline[1],
        performanceBaseline[2],
        performanceBaseline[3]
      ]
    },
    {
      label: 'Everest 45-Degree Oblique Performance Sweep (Multi-LOD Frustum Load)',
      entities: [
        performanceBaseline[0],
        {
          ...performanceBaseline[1],
          id: 'everest-oblique-flight',
          name: '45-Degree Oblique Descent Vector',
          color: [0.85, 0.35, 0.9, 0.85]
        },
        performanceBaseline[2],
        performanceBaseline[3]
      ]
    },
    {
      label: 'Rapid High-LOD Zoom Stress Test (High Throughput Request Queue)',
      entities: [
        performanceBaseline[0],
        performanceBaseline[1],
        performanceBaseline[2],
        {
          ...performanceBaseline[3],
          id: 'everest-zoom-stress-rect',
          name: 'Max-LOD Tile Queue Pressure Sector',
          color: [0.15, 0.95, 0.85, 0.6]
        }
      ]
    }
  ]
};

// Recipe 4: terrain-tweaks-dev
// Regional anchor: Global tile streaming workbench [-105.0, 40.0]
const tweaksBaseline = [
  {
    id: 'tweaks-descendant-limit-box',
    name: 'Loading Descendant Limit (20) Envelope Box',
    show: true,
    positionDegrees: [-105.0, 40.0, 1500.0],
    shape: {
      type: 'box',
      dimensions: [3000.0, 3000.0, 800.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.7, 0.95, 0.8],
    outlineColor: [0.05, 0.3, 0.55, 1.0]
  },
  {
    id: 'tweaks-ancestor-preload-column',
    name: 'Ancestor Tile Preload Hierarchy Column',
    show: true,
    positionDegrees: [-104.95, 40.05, 2000.0],
    shape: {
      type: 'cylinder',
      length: 1200.0,
      topRadius: 400.0,
      bottomRadius: 600.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.75, 0.15, 0.8],
    outlineColor: [0.55, 0.4, 0.05, 1.0]
  },
  {
    id: 'tweaks-sibling-preload-sphere',
    name: 'Sibling Adjacent Tile Preload Sphere',
    show: true,
    positionDegrees: [-105.05, 39.95, 1800.0],
    shape: {
      type: 'ellipsoid',
      radii: [500.0, 500.0, 500.0]
    },
    fill: true,
    outline: true,
    color: [0.3, 0.9, 0.4, 0.8],
    outlineColor: [0.08, 0.45, 0.15, 1.0]
  },
  {
    id: 'tweaks-fill-highlight-rect',
    name: 'Fill Highlight Color Debug Tile Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.15, 39.88, -104.85, 40.12],
      height: 1000.0,
      extrudedHeight: 1400.0
    },
    fill: true,
    outline: true,
    color: [1.0, 1.0, 0.0, 0.5],
    outlineColor: [0.95, 0.85, 0.1, 0.9]
  }
];

const tweaksVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Standard Descendant Limit (20) & Balanced Ancestor/Sibling Preload',
      entities: tweaksBaseline
    },
    {
      label: 'High-Throughput Aggressive Preload (Limit 40, Cyan Highlight)',
      entities: [
        {
          ...tweaksBaseline[0],
          id: 'tweaks-aggressive-box',
          name: 'Aggressive 40-Descendant Limit Envelope',
          color: [0.1, 0.85, 0.95, 0.85]
        },
        tweaksBaseline[1],
        tweaksBaseline[2],
        {
          ...tweaksBaseline[3],
          id: 'tweaks-cyan-highlight-rect',
          name: 'Cyan Fill Highlight Inspection Area',
          color: [0.0, 1.0, 1.0, 0.6]
        }
      ]
    },
    {
      label: 'Low-Latency Minimal Preload (Limit 5, Magenta Highlight)',
      entities: [
        {
          ...tweaksBaseline[0],
          id: 'tweaks-minimal-box',
          name: 'Conservative 5-Descendant Limit Envelope',
          color: [0.9, 0.25, 0.8, 0.85]
        },
        tweaksBaseline[1],
        tweaksBaseline[2],
        {
          ...tweaksBaseline[3],
          id: 'tweaks-magenta-highlight-rect',
          name: 'Magenta Fill Highlight Inspection Area',
          color: [1.0, 0.0, 0.5, 0.5]
        }
      ]
    },
    {
      label: 'Production Clean Mode (Fill Highlight Disabled)',
      entities: [
        tweaksBaseline[0],
        tweaksBaseline[1],
        tweaksBaseline[2],
        {
          ...tweaksBaseline[3],
          id: 'tweaks-disabled-highlight-rect',
          name: 'Standard Clean Ground Surface (No Fill Debug Color)',
          color: [0.4, 0.5, 0.6, 0.35],
          outlineColor: [0.7, 0.8, 0.9, 0.6]
        }
      ]
    }
  ]
};

// Write files to tests/sandcastle/full/
const writePair = (id, baseline, variants) => {
  const jsonPath = path.join(root, 'tests/sandcastle/full', `${id}.json`);
  const variantsPath = path.join(root, 'tests/sandcastle/full', `${id}.variants.json`);
  fs.writeFileSync(jsonPath, JSON.stringify(baseline, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify(variants, null, 2) + '\n');
  console.log(`Generated ${id}: ${baseline.length} entities, ${variants.variants.length} variants`);
};

writePair('terrain-entity-batching-dev', batchingBaseline, batchingVariants);
writePair('terrain-exaggeration', exaggerationBaseline, exaggerationVariants);
writePair('terrain-performance-dev', performanceBaseline, performanceVariants);
writePair('terrain-tweaks-dev', tweaksBaseline, tweaksVariants);
console.log('Batch 87 fixtures successfully generated.');
