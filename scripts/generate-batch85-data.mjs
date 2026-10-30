// Generate test fixtures for Batch 85 recipes:
// 1. shadows-dev
// 2. shared-context
// 3. splitdirection
// 4. star-burst

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: shadows-dev
const shadowsDevBaseline = [
  {
    id: 'shadowsdev-tester-box',
    name: 'Shadow Tester Multi-Surface Calibration Hull',
    show: true,
    positionDegrees: [-76.3438, 39.8447, 50.0],
    shape: {
      type: 'box',
      dimensions: [100.0, 60.0, 40.0]
    },
    fill: true,
    outline: true,
    color: [0.85, 0.85, 0.85, 1.0],
    outlineColor: [0.2, 0.2, 0.2, 1.0]
  },
  {
    id: 'shadowsdev-light-column',
    name: 'Directional Light Shadow Frustum Optical Vector',
    show: true,
    positionDegrees: [-76.3438, 39.8447, 120.0],
    shape: {
      type: 'cylinder',
      length: 100.0,
      topRadius: 15.0,
      bottomRadius: 30.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.2, 0.8],
    outlineColor: [0.5, 0.4, 0.05, 1.0]
  },
  {
    id: 'shadowsdev-depth-sphere',
    name: 'Shadow Depth Bias Calibration Probe Sphere',
    show: true,
    positionDegrees: [-76.3438, 39.8447, 10.0],
    shape: {
      type: 'ellipsoid',
      radii: [12.0, 12.0, 12.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.7, 0.95, 1.0],
    outlineColor: [0.05, 0.3, 0.5, 1.0]
  },
  {
    id: 'shadowsdev-ground-rect',
    name: 'Pinnacle PA Shadow Receiver Terrain Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.347, 39.842, -76.340, 39.847],
      height: 0.0,
      extrudedHeight: 5.0
    },
    fill: true,
    outline: true,
    color: [0.35, 0.45, 0.35, 0.7],
    outlineColor: [0.7, 0.8, 0.7, 1.0]
  }
];

const shadowsDevVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Pinnacle PA 4-Cascade Shadow Mapping (Shadow Tester Model, Bias Tuning)',
      entities: shadowsDevBaseline
    },
    {
      label: 'Seneca Rocks Steep Terrain Self-Shadowing Mode (High-Angle Horizon Occlusion)',
      entities: [
        {
          ...shadowsDevBaseline[0],
          id: 'shadowsdev-seneca-steep-box',
          name: 'Seneca Rocks Steep Ridge Shadow Caster',
          positionDegrees: [-79.37, 38.83, 200.0],
          shape: {
            type: 'box',
            dimensions: [150.0, 40.0, 120.0]
          },
          color: [0.75, 0.6, 0.5, 1.0]
        },
        shadowsDevBaseline[1],
        shadowsDevBaseline[2],
        shadowsDevBaseline[3]
      ]
    },
    {
      label: 'Half Dome Deep Valley Cascade Horizon Mode (Soft Shadow Edge Filtering)',
      entities: [
        shadowsDevBaseline[0],
        {
          ...shadowsDevBaseline[1],
          id: 'shadowsdev-halfdome-column',
          name: 'Half Dome Granitic Optical Shaft',
          shape: {
            type: 'cylinder',
            length: 150.0,
            topRadius: 20.0,
            bottomRadius: 40.0
          },
          color: [0.6, 0.75, 0.9, 0.85]
        },
        shadowsDevBaseline[2],
        shadowsDevBaseline[3]
      ]
    },
    {
      label: 'High-Altitude Airborne Cesium Air Shadow Projection Mode (Long-Range Cast)',
      entities: [
        shadowsDevBaseline[0],
        shadowsDevBaseline[1],
        {
          ...shadowsDevBaseline[2],
          id: 'shadowsdev-highaltitude-sphere',
          name: 'Airborne Aircraft Shadow Focal Dome',
          shape: {
            type: 'ellipsoid',
            radii: [25.0, 25.0, 25.0]
          },
          color: [0.95, 0.4, 0.2, 1.0]
        },
        shadowsDevBaseline[3]
      ]
    }
  ]
};

// Recipe 2: shared-context
const sharedContextBaseline = [
  {
    id: 'sharedctx-green-cylinder',
    name: 'Green cylinder with black outline',
    show: true,
    positionDegrees: [-100.0, 40.0, 200000.0],
    shape: {
      type: 'cylinder',
      length: 400000.0,
      topRadius: 200000.0,
      bottomRadius: 200000.0
    },
    fill: true,
    outline: true,
    color: [0.0, 0.8, 0.0, 0.5],
    outlineColor: [0.0, 0.0, 0.0, 1.0]
  },
  {
    id: 'sharedctx-box-primitive',
    name: 'Shared WebGL Resource Box Primitive',
    show: true,
    positionDegrees: [-95.0, 40.0, 150000.0],
    shape: {
      type: 'box',
      dimensions: [250000.0, 250000.0, 300000.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.5, 0.9, 0.65],
    outlineColor: [0.1, 0.25, 0.5, 1.0]
  },
  {
    id: 'sharedctx-sphere-primitive',
    name: 'Shared WebGL Resource Sphere Primitive',
    show: true,
    positionDegrees: [-105.0, 40.0, 150000.0],
    shape: {
      type: 'ellipsoid',
      radii: [150000.0, 150000.0, 150000.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.35, 0.2, 0.7],
    outlineColor: [0.45, 0.1, 0.05, 1.0]
  },
  {
    id: 'sharedctx-dualview-rect',
    name: 'Dual Viewer 2D/3D Shared Staging Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.0, 36.0, -95.0, 44.0],
      height: 0.0,
      extrudedHeight: 5000.0
    },
    fill: true,
    outline: true,
    color: [0.25, 0.3, 0.4, 0.5],
    outlineColor: [0.8, 0.85, 0.95, 1.0]
  }
];

const sharedContextVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Shared Context Dual-Viewport (Green Semi-Transparent Cylinder, Black Outline)',
      entities: sharedContextBaseline
    },
    {
      label: 'Dual-Viewer Synchronized Cyan Box Primitive Mode (Cross-Context Texture Reuse)',
      entities: [
        sharedContextBaseline[0],
        {
          ...sharedContextBaseline[1],
          id: 'sharedctx-sync-cyan-box',
          name: 'Synchronized Cyan Texture Reused Box',
          color: [0.1, 0.9, 0.85, 0.8]
        },
        sharedContextBaseline[2],
        sharedContextBaseline[3]
      ]
    },
    {
      label: 'Shared Resource Multi-Frustum Sphere Geometry Mode (Shared Mesh Buffer)',
      entities: [
        sharedContextBaseline[0],
        sharedContextBaseline[1],
        {
          ...sharedContextBaseline[2],
          id: 'sharedctx-shared-mesh-sphere',
          name: 'Shared High-Poly Sphere Buffer Geometry',
          shape: {
            type: 'ellipsoid',
            radii: [200000.0, 200000.0, 200000.0]
          },
          color: [0.85, 0.7, 0.1, 0.85]
        },
        sharedContextBaseline[3]
      ]
    },
    {
      label: 'Independent High-Speed Buffer Invalidation Stress Test (Dynamic Pipeline Cycle)',
      entities: [
        {
          ...sharedContextBaseline[0],
          id: 'sharedctx-stress-cylinder',
          name: 'Dynamic Invalidation Active Cylinder',
          color: [0.95, 0.2, 0.6, 0.75]
        },
        sharedContextBaseline[1],
        sharedContextBaseline[2],
        sharedContextBaseline[3]
      ]
    }
  ]
};

// Recipe 3: splitdirection
const splitDirectionBaseline = [
  {
    id: 'splitdir-left-point-sphere',
    name: 'SkyBlue Left Partition Clamped Point',
    show: true,
    positionDegrees: [-75.0, 40.0, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [30000.0, 30000.0, 30000.0]
    },
    fill: true,
    outline: true,
    color: [0.529, 0.808, 0.922, 1.0], // SkyBlue
    outlineColor: [0.0, 0.8, 0.0, 1.0]   // Green outline
  },
  {
    id: 'splitdir-right-point-cylinder',
    name: 'Red Right Partition Indicator Column',
    show: true,
    positionDegrees: [-75.0, 35.0, 60000.0],
    shape: {
      type: 'cylinder',
      length: 120000.0,
      topRadius: 25000.0,
      bottomRadius: 25000.0
    },
    fill: true,
    outline: true,
    color: [0.9, 0.15, 0.15, 1.0],
    outlineColor: [0.4, 0.05, 0.05, 1.0]
  },
  {
    id: 'splitdir-swipe-center-box',
    name: 'Central Swipe Boundary Separator Box',
    show: true,
    positionDegrees: [-100.0, 37.5, 40000.0],
    shape: {
      type: 'box',
      dimensions: [50000.0, 800000.0, 80000.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.95, 0.95, 0.85],
    outlineColor: [0.1, 0.1, 0.1, 1.0]
  },
  {
    id: 'splitdir-canvas-rect',
    name: 'SplitDirection Screen Space Viewport Bounds',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-85.0, 33.0, -70.0, 42.0],
      height: 0.0,
      extrudedHeight: 5000.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.25, 0.35, 0.5],
    outlineColor: [0.8, 0.85, 0.9, 1.0]
  }
];

const splitDirectionVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Left/Right Split Screen Points & Billboards (SplitDirection Left vs Right)',
      entities: splitDirectionBaseline
    },
    {
      label: 'Model Geometry Split Comparison Mode (Cesium Air Split Left, Cesium Man Right)',
      entities: [
        {
          ...splitDirectionBaseline[0],
          id: 'splitdir-model-left-dome',
          name: 'Split Left Aircraft Envelope',
          color: [0.2, 0.85, 0.4, 1.0]
        },
        splitDirectionBaseline[1],
        splitDirectionBaseline[2],
        splitDirectionBaseline[3]
      ]
    },
    {
      label: 'Translucent Clamped Polyline & Polygon Partition Mode (Left Cyan vs Right Magenta)',
      entities: [
        splitDirectionBaseline[0],
        {
          ...splitDirectionBaseline[1],
          id: 'splitdir-poly-right-col',
          name: 'Split Right Magenta Clamped Column',
          color: [0.9, 0.2, 0.85, 1.0]
        },
        splitDirectionBaseline[2],
        splitDirectionBaseline[3]
      ]
    },
    {
      label: 'Central Swipe Boundary Diagnostic Mode (Active Splitter Position Tracking)',
      entities: [
        splitDirectionBaseline[0],
        splitDirectionBaseline[1],
        {
          ...splitDirectionBaseline[2],
          id: 'splitdir-swipe-active-box',
          name: 'High Contrast Splitter Bar',
          color: [0.95, 0.85, 0.1, 1.0]
        },
        splitDirectionBaseline[3]
      ]
    }
  ]
};

// Recipe 4: star-burst
const starBurstBaseline = [
  {
    id: 'starburst-hub-sphere',
    name: 'Clustered Billboards Origin Hub Probe Sphere',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 15.0],
    shape: {
      type: 'ellipsoid',
      radii: [15.0, 15.0, 15.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.3, 0.2, 1.0],
    outlineColor: [0.45, 0.05, 0.05, 1.0]
  },
  {
    id: 'starburst-radial-vector-cylinder',
    name: 'Radial Expansion Trajectory Vector Pillar',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 60.0],
    shape: {
      type: 'cylinder',
      length: 80.0,
      topRadius: 8.0,
      bottomRadius: 18.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.7, 0.95, 0.8],
    outlineColor: [0.05, 0.3, 0.5, 1.0]
  },
  {
    id: 'starburst-orbit-ring-box',
    name: 'Expanded Satellite Cluster Outer Bounding Hull',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 30.0],
    shape: {
      type: 'box',
      dimensions: [180.0, 180.0, 40.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.2, 0.75],
    outlineColor: [0.5, 0.4, 0.05, 1.0]
  },
  {
    id: 'starburst-facility-rect',
    name: 'Exton Facility Campus Billboard Cluster Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.601, 40.036, -75.594, 40.041],
      height: 0.0,
      extrudedHeight: 4.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.35, 0.45, 0.6],
    outlineColor: [0.75, 0.8, 0.9, 1.0]
  }
];

const starBurstVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Clustered Billboards Star-Burst Mode (30 Icons Radial Orbit Expansion)',
      entities: starBurstBaseline
    },
    {
      label: 'High-Density 60 Entity Expanding Galaxy Ring Mode (Wide Dispersion Arc)',
      entities: [
        {
          ...starBurstBaseline[0],
          id: 'starburst-galaxy-hub-dome',
          name: 'Galaxy High Density Hub Dome',
          shape: {
            type: 'ellipsoid',
            radii: [25.0, 25.0, 25.0]
          },
          color: [0.85, 0.2, 0.8, 1.0]
        },
        starBurstBaseline[1],
        starBurstBaseline[2],
        starBurstBaseline[3]
      ]
    },
    {
      label: 'Dual-Tier Concentric Shells Hierarchical Burst Mode (Inner & Outer Rings)',
      entities: [
        starBurstBaseline[0],
        {
          ...starBurstBaseline[1],
          id: 'starburst-dualtier-shaft',
          name: 'Concentric Two-Tier Expansion Shaft',
          shape: {
            type: 'cylinder',
            length: 120.0,
            topRadius: 12.0,
            bottomRadius: 24.0
          },
          color: [0.2, 0.85, 0.5, 0.85]
        },
        starBurstBaseline[2],
        starBurstBaseline[3]
      ]
    },
    {
      label: 'Collapsed Single-Point Anchor Resting Baseline Mode (Resting Pre-Burst)',
      entities: [
        starBurstBaseline[0],
        starBurstBaseline[1],
        {
          ...starBurstBaseline[2],
          id: 'starburst-collapsed-box',
          name: 'Compact Resting Cluster Core',
          shape: {
            type: 'box',
            dimensions: [40.0, 40.0, 20.0]
          },
          color: [0.3, 0.6, 0.95, 0.9]
        },
        starBurstBaseline[3]
      ]
    }
  ]
};

// Write files
const files = [
  ['tests/sandcastle/full/shadows-dev.json', shadowsDevBaseline],
  ['tests/sandcastle/full/shadows-dev.variants.json', shadowsDevVariants],
  ['tests/sandcastle/full/shared-context.json', sharedContextBaseline],
  ['tests/sandcastle/full/shared-context.variants.json', sharedContextVariants],
  ['tests/sandcastle/full/splitdirection.json', splitDirectionBaseline],
  ['tests/sandcastle/full/splitdirection.variants.json', splitDirectionVariants],
  ['tests/sandcastle/full/star-burst.json', starBurstBaseline],
  ['tests/sandcastle/full/star-burst.variants.json', starBurstVariants]
];

for (const [relPath, data] of files) {
  const fullPath = path.join(root, relPath);
  fs.writeFileSync(fullPath, JSON.stringify(data, null, 2) + '\n');
  console.log('wrote', relPath);
}
