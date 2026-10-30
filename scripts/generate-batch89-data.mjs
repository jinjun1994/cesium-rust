// Generate test fixtures for Batch 89 recipes:
// 1. video
// 2. viewerless
// 3. volumecloud
// 4. voxel-picking

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: video
// Regional anchor: Video textured ellipsoid [-79.0, 39.0, 1000.0]
const videoBaseline = [
  {
    id: 'video-textured-ellipsoid',
    name: 'Video Stream Textured Ellipsoid Surface',
    show: true,
    positionDegrees: [-79.0, 39.0, 1000.0],
    shape: {
      type: 'ellipsoid',
      radii: [1000.0, 1000.0, 1000.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.75, 0.95, 0.9],
    outlineColor: [0.05, 0.35, 0.55, 1.0]
  },
  {
    id: 'video-playback-screen-box',
    name: 'Video Playback Staging Screen Hull',
    show: true,
    positionDegrees: [-78.98, 39.02, 1200.0],
    shape: {
      type: 'box',
      dimensions: [1200.0, 600.0, 800.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.15, 0.85],
    outlineColor: [0.55, 0.45, 0.05, 1.0]
  },
  {
    id: 'video-synchronizer-pillar',
    name: 'Clock VideoSynchronizer Frequency Column',
    show: true,
    positionDegrees: [-79.02, 38.98, 800.0],
    shape: {
      type: 'cylinder',
      length: 800.0,
      topRadius: 200.0,
      bottomRadius: 200.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.95, 0.45, 0.8],
    outlineColor: [0.08, 0.45, 0.15, 1.0]
  },
  {
    id: 'video-viewing-platform-rect',
    name: 'Audience Viewing Platform Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.03, 38.97, -78.97, 39.03],
      height: 50.0,
      extrudedHeight: 150.0
    },
    fill: true,
    outline: true,
    color: [0.45, 0.5, 0.7, 0.5],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const videoVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: 1x UV Video Texture Mapping on Ellipsoid',
      entities: videoBaseline
    },
    {
      label: '8x Repeat St-Tiled Video Array Mode (Tiled Mosaic Texture)',
      entities: [
        {
          ...videoBaseline[0],
          id: 'video-tiled-ellipsoid',
          name: '8x8 Mosaic Video Tiled Ellipsoid',
          color: [0.85, 0.25, 0.9, 0.9]
        },
        videoBaseline[1],
        videoBaseline[2],
        videoBaseline[3]
      ]
    },
    {
      label: 'Clock-Synchronized Video Frame Progression Mode',
      entities: [
        videoBaseline[0],
        {
          ...videoBaseline[1],
          id: 'video-synced-screen',
          name: 'Real-Time Sync Monitor Screen',
          color: [0.15, 0.95, 0.85, 0.9]
        },
        videoBaseline[2],
        videoBaseline[3]
      ]
    },
    {
      label: 'DOM Video Overlay Inset Comparison Mode',
      entities: [
        videoBaseline[0],
        videoBaseline[1],
        videoBaseline[2],
        {
          ...videoBaseline[3],
          id: 'video-overlay-rect',
          name: 'DOM Inset Overlay Area Sector',
          color: [0.95, 0.55, 0.1, 0.65]
        }
      ]
    }
  ]
};

// Recipe 2: viewerless
// Standalone Sandcastle demo without Cesium Viewer [0.0, 0.0, 500.0]
const viewerlessBaseline = [
  {
    id: 'viewerless-canvas-backdrop-box',
    name: 'Viewerless UI Staging Gradient Backdrop',
    show: true,
    positionDegrees: [0.0, 0.0, 500.0],
    shape: {
      type: 'box',
      dimensions: [4000.0, 4000.0, 400.0]
    },
    fill: true,
    outline: true,
    color: [0.35, 0.75, 0.95, 0.85],
    outlineColor: [0.1, 0.35, 0.55, 1.0]
  },
  {
    id: 'viewerless-ui-button-cylinder',
    name: 'Interactive Toolbar Button Action Target',
    show: true,
    positionDegrees: [0.05, 0.05, 800.0],
    shape: {
      type: 'cylinder',
      length: 600.0,
      topRadius: 500.0,
      bottomRadius: 500.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.95, 0.3, 0.85],
    outlineColor: [0.08, 0.45, 0.08, 1.0]
  },
  {
    id: 'viewerless-logo-mark-sphere',
    name: 'Cesium Logomark Brand Alignment Sphere',
    show: true,
    positionDegrees: [-0.05, -0.05, 700.0],
    shape: {
      type: 'ellipsoid',
      radii: [600.0, 600.0, 600.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.45, 0.9, 0.9],
    outlineColor: [0.05, 0.15, 0.45, 1.0]
  },
  {
    id: 'viewerless-viewport-rect',
    name: 'Standalone HTML Canvas Viewport Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-0.1, -0.1, 0.1, 0.1],
      height: 100.0,
      extrudedHeight: 300.0
    },
    fill: true,
    outline: true,
    color: [0.55, 0.65, 0.75, 0.5],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const viewerlessVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Viewerless Standalone UI Staging with Dual Gradient Backing',
      entities: viewerlessBaseline
    },
    {
      label: 'High-Contrast Toolbar Action Mode (Activated State)',
      entities: [
        viewerlessBaseline[0],
        {
          ...viewerlessBaseline[1],
          id: 'viewerless-active-button',
          name: 'Activated Button Cylinder Target',
          color: [0.95, 0.85, 0.15, 0.9]
        },
        viewerlessBaseline[2],
        viewerlessBaseline[3]
      ]
    },
    {
      label: 'Multi-Option Menu Selection State (Dynamic Option 1)',
      entities: [
        viewerlessBaseline[0],
        viewerlessBaseline[1],
        {
          ...viewerlessBaseline[2],
          id: 'viewerless-selected-sphere',
          name: 'Menu Option 1 Focus Sphere',
          color: [0.85, 0.25, 0.95, 0.9]
        },
        viewerlessBaseline[3]
      ]
    },
    {
      label: 'Clean Minimal Canvas Background Mode',
      entities: [
        viewerlessBaseline[0],
        viewerlessBaseline[1],
        viewerlessBaseline[2],
        {
          ...viewerlessBaseline[3],
          id: 'viewerless-clean-rect',
          name: 'Minimal Solid Layout Bounds',
          color: [0.45, 0.5, 0.55, 0.45]
        }
      ]
    }
  ]
};

// Recipe 3: volumecloud
// 3D Noise Raymarched Volumetric Cloud [113.0, 33.0, 20000.0]
const volumecloudBaseline = [
  {
    id: 'volcloud-raymarching-bounding-box',
    name: 'Volumetric Cloud Raymarching Bounding Box',
    show: true,
    positionDegrees: [113.0, 33.0, 20000.0],
    shape: {
      type: 'box',
      dimensions: [10000.0, 10000.0, 10000.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.92, 0.95, 0.75],
    outlineColor: [0.55, 0.6, 0.7, 0.9]
  },
  {
    id: 'volcloud-perlin-noise-core-sphere',
    name: '3D Improved Noise Density Nucleus Sphere',
    show: true,
    positionDegrees: [113.0, 33.0, 20000.0],
    shape: {
      type: 'ellipsoid',
      radii: [4000.0, 4000.0, 3500.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.96, 1.0, 0.85],
    outlineColor: [0.7, 0.75, 0.85, 1.0]
  },
  {
    id: 'volcloud-density-attenuation-cylinder',
    name: 'Vertical Vapor Absorption Column',
    show: true,
    positionDegrees: [113.02, 33.01, 19000.0],
    shape: {
      type: 'cylinder',
      length: 8000.0,
      topRadius: 2500.0,
      bottomRadius: 2500.0
    },
    fill: true,
    outline: true,
    color: [0.75, 0.85, 0.98, 0.7],
    outlineColor: [0.4, 0.5, 0.7, 1.0]
  },
  {
    id: 'volcloud-ground-shadow-footprint-rect',
    name: 'Cloud Shadow Ground Projection Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [112.92, 32.92, 113.08, 33.08],
      height: 100.0,
      extrudedHeight: 500.0
    },
    fill: true,
    outline: true,
    color: [0.35, 0.4, 0.5, 0.4],
    outlineColor: [0.7, 0.75, 0.85, 0.8]
  }
];

const volumecloudVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: 3D Improved Noise Raymarched Volumetric Cloud',
      entities: volumecloudBaseline
    },
    {
      label: 'Dense Cumulonimbus Storm Cell Mode (High Density Absorption)',
      entities: [
        {
          ...volumecloudBaseline[0],
          id: 'volcloud-dense-box',
          name: 'Heavy Storm Vapor Enclosure',
          color: [0.55, 0.6, 0.7, 0.9]
        },
        volumecloudBaseline[1],
        volumecloudBaseline[2],
        volumecloudBaseline[3]
      ]
    },
    {
      label: 'Wispy Cirrus Dispersed Cloud Mode (High Translucency)',
      entities: [
        volumecloudBaseline[0],
        {
          ...volumecloudBaseline[1],
          id: 'volcloud-cirrus-sphere',
          name: 'Diffuse Cirrus Crystal Nucleus',
          color: [0.95, 0.98, 1.0, 0.5]
        },
        volumecloudBaseline[2],
        volumecloudBaseline[3]
      ]
    },
    {
      label: 'Sunlight Scattering Multi-Phase Anisotropy Mode',
      entities: [
        volumecloudBaseline[0],
        volumecloudBaseline[1],
        volumecloudBaseline[2],
        {
          ...volumecloudBaseline[3],
          id: 'volcloud-scatter-rect',
          name: 'Forward-Scattering Sunlit Floor',
          color: [1.0, 0.9, 0.75, 0.55]
        }
      ]
    }
  ]
};

// Recipe 4: voxel-picking
// Voxel picking workbench [-105.0, 40.0, 2000.0]
const voxelPickingBaseline = [
  {
    id: 'voxel-selection-highlight-box',
    name: 'Raycast Hit Target Voxel Cell (Picked Highlight)',
    show: true,
    positionDegrees: [-105.0, 40.0, 2000.0],
    shape: {
      type: 'box',
      dimensions: [2000.0, 2000.0, 2000.0]
    },
    fill: true,
    outline: true,
    color: [1.0, 0.9, 0.1, 0.9],
    outlineColor: [0.65, 0.5, 0.05, 1.0]
  },
  {
    id: 'voxel-neighbor-cell-cylinder',
    name: 'Adjacent Sub-Voxel Volumetric Node',
    show: true,
    positionDegrees: [-104.97, 40.02, 2000.0],
    shape: {
      type: 'cylinder',
      length: 1800.0,
      topRadius: 600.0,
      bottomRadius: 600.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.85, 0.95, 0.85],
    outlineColor: [0.05, 0.4, 0.55, 1.0]
  },
  {
    id: 'voxel-multi-level-lod-sphere',
    name: 'Level-3 Voxel Octree Division Centroid',
    show: true,
    positionDegrees: [-105.03, 39.98, 2000.0],
    shape: {
      type: 'ellipsoid',
      radii: [1000.0, 1000.0, 1000.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.2, 0.75, 0.85],
    outlineColor: [0.45, 0.05, 0.35, 1.0]
  },
  {
    id: 'voxel-subdivision-grid-rect',
    name: 'Procedural Multi-Tile Voxel Domain Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.10, 39.90, -104.90, 40.10],
      height: 100.0,
      extrudedHeight: 800.0
    },
    fill: true,
    outline: true,
    color: [0.4, 0.55, 0.7, 0.5],
    outlineColor: [0.85, 0.9, 1.0, 0.9]
  }
];

const voxelPickingVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Procedural Multi-Tile Voxel Grid with Raycast Hit Highlighting',
      entities: voxelPickingBaseline
    },
    {
      label: 'Dense 4x4x4 Voxel Octree Micro-Cell Focus (High Precision Picking)',
      entities: [
        {
          ...voxelPickingBaseline[0],
          id: 'voxel-micro-cell-box',
          name: 'Micro-Cell Sub-Volume Target',
          color: [0.95, 0.25, 0.15, 0.95]
        },
        voxelPickingBaseline[1],
        voxelPickingBaseline[2],
        voxelPickingBaseline[3]
      ]
    },
    {
      label: 'Deep Core Voxel Strata Cutaway Mode (Geological Cross-Section)',
      entities: [
        voxelPickingBaseline[0],
        {
          ...voxelPickingBaseline[1],
          id: 'voxel-core-cylinder',
          name: 'Core Strata Core-Sample Column',
          color: [0.25, 0.95, 0.45, 0.9]
        },
        voxelPickingBaseline[2],
        voxelPickingBaseline[3]
      ]
    },
    {
      label: 'Unselected Clean Procedural Voxel Cloud Mode',
      entities: [
        voxelPickingBaseline[0],
        voxelPickingBaseline[1],
        voxelPickingBaseline[2],
        {
          ...voxelPickingBaseline[3],
          id: 'voxel-clean-grid-rect',
          name: 'Transparent Voxel Outer Bounding Grid',
          color: [0.5, 0.6, 0.7, 0.4]
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

writePair('video', videoBaseline, videoVariants);
writePair('viewerless', viewerlessBaseline, viewerlessVariants);
writePair('volumecloud', volumecloudBaseline, volumecloudVariants);
writePair('voxel-picking', voxelPickingBaseline, voxelPickingVariants);
console.log('Batch 89 fixtures successfully generated.');
