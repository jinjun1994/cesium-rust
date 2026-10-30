// Generate test fixtures for Batch 88 recipes:
// 1. time-dynamic-point-cloud
// 2. timeline
// 3. underground-color
// 4. vector-3d-tiles

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: time-dynamic-point-cloud
// Regional anchor: Point cloud test staging [-105.0, 40.0]
const pointcloudBaseline = [
  {
    id: 'tdpc-cluster-bounding-box',
    name: 'Time-Dynamic Point Cloud Bounding Envelope',
    show: true,
    positionDegrees: [-105.0, 40.0, 100.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 30.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.95, 0.85],
    outlineColor: [0.05, 0.35, 0.55, 1.0]
  },
  {
    id: 'tdpc-temporal-trajectory-cylinder',
    name: 'Point Stream Temporal Velocity Vector',
    show: true,
    positionDegrees: [-105.0, 40.0, 120.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 8.0,
      bottomRadius: 10.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.75, 0.15, 0.8],
    outlineColor: [0.55, 0.4, 0.05, 1.0]
  },
  {
    id: 'tdpc-pnts-sample-sphere',
    name: 'PNTS Frame Interval Centroid Marker',
    show: true,
    positionDegrees: [-105.0005, 40.0005, 95.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.25, 0.35, 0.9],
    outlineColor: [0.45, 0.08, 0.12, 1.0]
  },
  {
    id: 'tdpc-spatial-extent-rect',
    name: 'Time-Dynamic Point Sensor Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.002, 39.998, -104.998, 40.002],
      height: 50.0,
      extrudedHeight: 80.0
    },
    fill: true,
    outline: true,
    color: [0.35, 0.45, 0.7, 0.55],
    outlineColor: [0.8, 0.85, 0.95, 0.9]
  }
];

const pointcloudVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Time-Dynamic Point Cloud Temporal Sequence (0.pnts ~ 4.pnts)',
      entities: pointcloudBaseline
    },
    {
      label: 'High-Density 10px Point Size Shading Mode (Vibrant Point Diameter)',
      entities: [
        {
          ...pointcloudBaseline[0],
          id: 'tdpc-high-res-box',
          name: 'High-Density Point Cloud Bounding Box',
          color: [0.85, 0.2, 0.95, 0.9]
        },
        pointcloudBaseline[1],
        pointcloudBaseline[2],
        pointcloudBaseline[3]
      ]
    },
    {
      label: 'Fast Scrubbing 4x Temporal Multiplier Mode',
      entities: [
        pointcloudBaseline[0],
        {
          ...pointcloudBaseline[1],
          id: 'tdpc-fast-velocity-column',
          name: 'Accelerated Stream Vector Column',
          color: [0.15, 0.95, 0.45, 0.85]
        },
        pointcloudBaseline[2],
        pointcloudBaseline[3]
      ]
    },
    {
      label: 'Static Bounding Hull & Frustum Framing Mode',
      entities: [
        pointcloudBaseline[0],
        pointcloudBaseline[1],
        pointcloudBaseline[2],
        {
          ...pointcloudBaseline[3],
          id: 'tdpc-framed-rect',
          name: 'Pinned Sensor Inspection Boundary',
          color: [0.95, 0.6, 0.1, 0.7]
        }
      ]
    }
  ]
};

// Recipe 2: timeline
// Temporal coordination demo workbench [-75.0, 40.0]
const timelineBaseline = [
  {
    id: 'timeline-start-interval-box',
    name: 'Start Interval 1-Hour Track Hull (Red)',
    show: true,
    positionDegrees: [-75.0, 40.0, 500.0],
    shape: {
      type: 'box',
      dimensions: [5000.0, 5000.0, 500.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.15, 0.15, 0.85],
    outlineColor: [0.55, 0.05, 0.05, 1.0]
  },
  {
    id: 'timeline-end-interval-cylinder',
    name: 'End Interval 1-Hour Track Column (Lime)',
    show: true,
    positionDegrees: [-74.95, 40.05, 800.0],
    shape: {
      type: 'cylinder',
      length: 1000.0,
      topRadius: 800.0,
      bottomRadius: 800.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.95, 0.2, 0.85],
    outlineColor: [0.05, 0.45, 0.05, 1.0]
  },
  {
    id: 'timeline-middle-span-sphere',
    name: 'Middle Quarter Interval Track Sphere (DeepSkyBlue)',
    show: true,
    positionDegrees: [-75.05, 39.95, 700.0],
    shape: {
      type: 'ellipsoid',
      radii: [1200.0, 1200.0, 1200.0]
    },
    fill: true,
    outline: true,
    color: [0.0, 0.75, 1.0, 0.85],
    outlineColor: [0.0, 0.35, 0.55, 1.0]
  },
  {
    id: 'timeline-active-scrub-rect',
    name: 'Active Timeline Playback Sector Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.12, 39.90, -74.88, 40.10],
      height: 100.0,
      extrudedHeight: 400.0
    },
    fill: true,
    outline: true,
    color: [0.45, 0.55, 0.7, 0.5],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const timelineVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: 24-Hour Timeline with Red/Lime/SkyBlue Interval Tracks',
      entities: timelineBaseline
    },
    {
      label: 'Accelerated 120x Clock Multiplier Mode (High-Speed Time-Lapse)',
      entities: [
        timelineBaseline[0],
        {
          ...timelineBaseline[1],
          id: 'timeline-accelerated-end',
          name: 'Fast-Forward Time Marker Column',
          color: [0.95, 0.85, 0.15, 0.9]
        },
        timelineBaseline[2],
        timelineBaseline[3]
      ]
    },
    {
      label: 'High-Span Multi-Day Inspection Mode',
      entities: [
        timelineBaseline[0],
        timelineBaseline[1],
        {
          ...timelineBaseline[2],
          id: 'timeline-multiday-sphere',
          name: 'Multi-Day Temporal Horizon Sphere',
          color: [0.85, 0.3, 0.9, 0.9]
        },
        timelineBaseline[3]
      ]
    },
    {
      label: 'Interactive Clock Scrubbing Focus Mode',
      entities: [
        timelineBaseline[0],
        timelineBaseline[1],
        timelineBaseline[2],
        {
          ...timelineBaseline[3],
          id: 'timeline-scrub-sector',
          name: 'Precision Scrubbing Sector Bound',
          color: [0.2, 0.85, 0.75, 0.65]
        }
      ]
    }
  ]
};

// Recipe 3: underground-color
// Parc Lead Mine underground site in Wales [-3.82518, 53.11728, -500.0]
const undergroundBaseline = [
  {
    id: 'underground-mine-shaft-box',
    name: 'Parc Lead Mine Main Excavation Chamber',
    show: true,
    positionDegrees: [-3.82518, 53.11728, -500.0],
    shape: {
      type: 'box',
      dimensions: [400.0, 400.0, 200.0]
    },
    fill: true,
    outline: true,
    color: [0.45, 0.5, 0.55, 0.85],
    outlineColor: [0.2, 0.25, 0.3, 1.0]
  },
  {
    id: 'underground-adit-cylinder',
    name: 'Subterranean Ventilation Shaft Adit',
    show: true,
    positionDegrees: [-3.823, 53.118, -450.0],
    shape: {
      type: 'cylinder',
      length: 500.0,
      topRadius: 80.0,
      bottomRadius: 80.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.95, 0.8],
    outlineColor: [0.05, 0.35, 0.55, 1.0]
  },
  {
    id: 'underground-ore-body-sphere',
    name: 'Subsurface Mineral Vein Centroid',
    show: true,
    positionDegrees: [-3.827, 53.116, -550.0],
    shape: {
      type: 'ellipsoid',
      radii: [150.0, 150.0, 250.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.7, 0.15, 0.9],
    outlineColor: [0.55, 0.35, 0.05, 1.0]
  },
  {
    id: 'underground-surface-collar-rect',
    name: 'Surface Collar Entry Boundary Area',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-3.835, 53.110, -3.815, 53.125],
      height: 100.0,
      extrudedHeight: 250.0
    },
    fill: true,
    outline: true,
    color: [0.35, 0.6, 0.4, 0.5],
    outlineColor: [0.75, 0.9, 0.8, 0.9]
  }
];

const undergroundVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: LightSlateGray Underground Color with Distance Alpha Blend',
      entities: undergroundBaseline
    },
    {
      label: 'Solid Black Subterranean Occlusion Mode (High Depth Contrast)',
      entities: [
        {
          ...undergroundBaseline[0],
          id: 'underground-black-chamber',
          name: 'Pure Black Subterranean Void Chamber',
          color: [0.1, 0.1, 0.12, 0.95]
        },
        undergroundBaseline[1],
        undergroundBaseline[2],
        undergroundBaseline[3]
      ]
    },
    {
      label: 'Translucent Blue Deep Subsurface Mode (Hydrology Inspection)',
      entities: [
        undergroundBaseline[0],
        {
          ...undergroundBaseline[1],
          id: 'underground-hydro-cylinder',
          name: 'Deep Aquifer Flow Conduit Column',
          color: [0.1, 0.5, 0.95, 0.85]
        },
        undergroundBaseline[2],
        undergroundBaseline[3]
      ]
    },
    {
      label: 'Pure Surface Terrain Uncolored Baseline (Globe Color Disabled)',
      entities: [
        undergroundBaseline[0],
        undergroundBaseline[1],
        undergroundBaseline[2],
        {
          ...undergroundBaseline[3],
          id: 'underground-uncolored-rect',
          name: 'Transparent Ground Surface Perimeter',
          color: [0.5, 0.5, 0.6, 0.35],
          outlineColor: [0.8, 0.85, 0.95, 0.6]
        }
      ]
    }
  ]
};

// Recipe 4: vector-3d-tiles
// Downtown Portland/Seattle vector urban district [-122.332, 47.606]
const vectorBaseline = [
  {
    id: 'vector-skyscraper-tall-box',
    name: 'Skyscraper Landmark (>300ft Roof Height)',
    show: true,
    positionDegrees: [-122.332, 47.606, 160.0],
    shape: {
      type: 'box',
      dimensions: [120.0, 120.0, 320.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.15, 0.15, 0.95],
    outlineColor: [0.55, 0.05, 0.05, 1.0]
  },
  {
    id: 'vector-midrise-building-cylinder',
    name: 'Mid-Rise Commercial Tower (100-299ft)',
    show: true,
    positionDegrees: [-122.330, 47.608, 90.0],
    shape: {
      type: 'cylinder',
      length: 180.0,
      topRadius: 60.0,
      bottomRadius: 60.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.55, 0.1, 0.9],
    outlineColor: [0.55, 0.25, 0.05, 1.0]
  },
  {
    id: 'vector-historic-heritage-sphere',
    name: 'Pre-1900 Heritage Cultural Monument',
    show: true,
    positionDegrees: [-122.335, 47.604, 30.0],
    shape: {
      type: 'ellipsoid',
      radii: [60.0, 60.0, 60.0]
    },
    fill: true,
    outline: true,
    color: [0.35, 0.0, 0.6, 0.9],
    outlineColor: [0.15, 0.0, 0.3, 1.0]
  },
  {
    id: 'vector-downtown-district-rect',
    name: 'Urban Commercial Vector Cadastral Sector',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-122.345, 47.595, -122.320, 47.615],
      height: 10.0,
      extrudedHeight: 80.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.6, 0.75, 0.55],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const vectorVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Construction Year 8-Color Categorical Palette (styleByYear)',
      entities: vectorBaseline
    },
    {
      label: 'Roof Height Sequential 6-Bin Gradient Palette (styleByHeight)',
      entities: [
        {
          ...vectorBaseline[0],
          id: 'vector-height-skyscraper',
          name: 'Peak Height Class (>100ft Navy Blue)',
          color: [0.15, 0.2, 0.6, 0.95]
        },
        vectorBaseline[1],
        vectorBaseline[2],
        vectorBaseline[3]
      ]
    },
    {
      label: 'Skyscraper Red/Orange Prominence Highlight Mode (styleHighlightTall)',
      entities: [
        vectorBaseline[0],
        {
          ...vectorBaseline[1],
          id: 'vector-tall-midrise',
          name: 'Prominent Orange Tall Class',
          color: [1.0, 0.55, 0.0, 0.9]
        },
        vectorBaseline[2],
        vectorBaseline[3]
      ]
    },
    {
      label: 'Unstyled Neutral Wireframe Vector Inspection (Base Urban Footprint)',
      entities: [
        vectorBaseline[0],
        vectorBaseline[1],
        vectorBaseline[2],
        {
          ...vectorBaseline[3],
          id: 'vector-neutral-district',
          name: 'Monochrome District Master Plan',
          color: [0.55, 0.55, 0.6, 0.45],
          outlineColor: [0.8, 0.85, 0.9, 0.8]
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

writePair('time-dynamic-point-cloud', pointcloudBaseline, pointcloudVariants);
writePair('timeline', timelineBaseline, timelineVariants);
writePair('underground-color', undergroundBaseline, undergroundVariants);
writePair('vector-3d-tiles', vectorBaseline, vectorVariants);
console.log('Batch 88 fixtures successfully generated.');
