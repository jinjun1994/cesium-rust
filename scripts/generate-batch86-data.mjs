// Generate test fixtures for Batch 86 recipes:
// 1. styled-gltf-lines-dev
// 2. styled-gltf-points-dev
// 3. terrain
// 4. terrain-clipping-planes

import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve('.');

// Recipe 1: styled-gltf-lines-dev
const styledLinesBaseline = [
  {
    id: 'styledlines-bounding-box',
    name: 'BENTLEY Materials Line Style Model Bounding Hull',
    show: true,
    positionDegrees: [0.0, 0.0, 20.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 40.0]
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.95, 1.0],
    outlineColor: [0.05, 0.35, 0.45, 1.0]
  },
  {
    id: 'styledlines-vector-cylinder',
    name: 'Edge Display Mode Optical Vector Column',
    show: true,
    positionDegrees: [0.0, 0.0, 60.0],
    shape: {
      type: 'cylinder',
      length: 60.0,
      topRadius: 12.0,
      bottomRadius: 12.0
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.2, 0.8],
    outlineColor: [0.45, 0.38, 0.05, 1.0]
  },
  {
    id: 'styledlines-anchor-sphere',
    name: 'Null Island Model Origin Alignment Sphere',
    show: true,
    positionDegrees: [0.0, 0.0, 0.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.3, 0.2, 1.0],
    outlineColor: [0.45, 0.08, 0.05, 1.0]
  },
  {
    id: 'styledlines-staging-rect',
    name: 'Styled Line Geometry Base Reference Grid',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-0.001, -0.001, 0.001, 0.001],
      height: 0.0,
      extrudedHeight: 2.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.35, 0.45, 0.6],
    outlineColor: [0.8, 0.85, 0.95, 1.0]
  }
];

const styledLinesVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: BENTLEY_materials_line_style Screen-Pixel Width (Surfaces & Edges)',
      entities: styledLinesBaseline
    },
    {
      label: '16-Bit Dash Pattern Stippled Geometry Mode (Dynamic Stippling)',
      entities: [
        {
          ...styledLinesBaseline[0],
          id: 'styledlines-stippled-box',
          name: 'Stippled Dashed Edge Geometry Core',
          color: [0.85, 0.3, 0.9, 1.0]
        },
        styledLinesBaseline[1],
        styledLinesBaseline[2],
        styledLinesBaseline[3]
      ]
    },
    {
      label: 'High-Contrast Fluorescent Wireframe Mode (Vibrant Edge Accents)',
      entities: [
        styledLinesBaseline[0],
        {
          ...styledLinesBaseline[1],
          id: 'styledlines-fluorescent-column',
          name: 'Fluorescent Edge Vector Column',
          color: [0.2, 0.95, 0.3, 0.9]
        },
        styledLinesBaseline[2],
        styledLinesBaseline[3]
      ]
    },
    {
      label: 'Oblique Perspective Edge Inspection Mode (LookAt Zoom Framing)',
      entities: [
        styledLinesBaseline[0],
        styledLinesBaseline[1],
        {
          ...styledLinesBaseline[2],
          id: 'styledlines-inspection-sphere',
          name: 'Oblique LookAt Focus Datum',
          shape: {
            type: 'ellipsoid',
            radii: [14.0, 14.0, 14.0]
          },
          color: [0.95, 0.65, 0.1, 1.0]
        },
        styledLinesBaseline[3]
      ]
    }
  ]
};

// Recipe 2: styled-gltf-points-dev
const styledPointsBaseline = [
  {
    id: 'styledpoints-bounding-box',
    name: 'BENTLEY Materials Point Style Multi-Diameter Envelope',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 25.0],
    shape: {
      type: 'box',
      dimensions: [50.0, 50.0, 50.0]
    },
    fill: true,
    outline: true,
    color: [0.95, 0.85, 0.1, 1.0],
    outlineColor: [0.45, 0.4, 0.05, 1.0]
  },
  {
    id: 'styledpoints-vector-cylinder',
    name: 'Variable Diameter Point Cloud Elevation Column',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 75.0],
    shape: {
      type: 'cylinder',
      length: 70.0,
      topRadius: 15.0,
      bottomRadius: 15.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.75, 0.9, 0.85],
    outlineColor: [0.05, 0.3, 0.45, 1.0]
  },
  {
    id: 'styledpoints-anchor-sphere',
    name: 'Eugene Oregon Point Cluster Origin Marker',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 0.0],
    shape: {
      type: 'ellipsoid',
      radii: [10.0, 10.0, 10.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.2, 0.2, 1.0],
    outlineColor: [0.4, 0.05, 0.05, 1.0]
  },
  {
    id: 'styledpoints-staging-rect',
    name: 'Multi-Size Styled Points Ground Staging Pad',
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
    color: [0.35, 0.4, 0.45, 0.6],
    outlineColor: [0.8, 0.85, 0.9, 1.0]
  }
];

const styledPointsVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: BENTLEY_materials_point_style Variable Diameters (r5, g8, b14, y10)',
      entities: styledPointsBaseline
    },
    {
      label: 'Uniform Micro-Point Inspection Mode (High Resolution Density)',
      entities: [
        {
          ...styledPointsBaseline[0],
          id: 'styledpoints-micro-box',
          name: 'Micro-Point Dense Cloud Core',
          color: [0.3, 0.9, 0.5, 1.0]
        },
        styledPointsBaseline[1],
        styledPointsBaseline[2],
        styledPointsBaseline[3]
      ]
    },
    {
      label: 'Large-Diameter Beacon Points Mode (Fluorescent Halo Markers)',
      entities: [
        styledPointsBaseline[0],
        {
          ...styledPointsBaseline[1],
          id: 'styledpoints-beacon-column',
          name: 'Fluorescent Beacon Shaft',
          color: [0.95, 0.4, 0.7, 0.85]
        },
        styledPointsBaseline[2],
        styledPointsBaseline[3]
      ]
    },
    {
      label: 'Color-Coded Intensity Attribute Point Mode (Multivariate Palette)',
      entities: [
        styledPointsBaseline[0],
        styledPointsBaseline[1],
        {
          ...styledPointsBaseline[2],
          id: 'styledpoints-intensity-sphere',
          name: 'Multivariate Intensity Attribute Beacon',
          shape: {
            type: 'ellipsoid',
            radii: [16.0, 16.0, 16.0]
          },
          color: [0.15, 0.8, 0.85, 1.0]
        },
        styledPointsBaseline[3]
      ]
    }
  ]
};

// Recipe 3: terrain
const terrainBaseline = [
  {
    id: 'terrain-elevation-box',
    name: 'Mount Fuji Topographic Elevation Peak Anchor',
    show: true,
    positionDegrees: [138.73, 35.36, 3776.0],
    shape: {
      type: 'box',
      dimensions: [6000.0, 6000.0, 3776.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.92, 0.95, 1.0],
    outlineColor: [0.4, 0.45, 0.5, 1.0]
  },
  {
    id: 'terrain-ridge-cylinder',
    name: 'Stratovolcano Topographic Ridge Spine Pillar',
    show: true,
    positionDegrees: [138.75, 35.38, 2500.0],
    shape: {
      type: 'cylinder',
      length: 4000.0,
      topRadius: 1200.0,
      bottomRadius: 2000.0
    },
    fill: true,
    outline: true,
    color: [0.6, 0.55, 0.5, 1.0],
    outlineColor: [0.3, 0.25, 0.2, 1.0]
  },
  {
    id: 'terrain-water-sphere',
    name: 'Fuji Five Lakes Water Mask Reflectance Sphere',
    show: true,
    positionDegrees: [138.70, 35.33, 1000.0],
    shape: {
      type: 'ellipsoid',
      radii: [1500.0, 1500.0, 1500.0]
    },
    fill: true,
    outline: true,
    color: [0.1, 0.4, 0.8, 1.0],
    outlineColor: [0.05, 0.2, 0.5, 1.0]
  },
  {
    id: 'terrain-fuji-rect',
    name: 'Mount Fuji Regional Topographic Elevation Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [138.65, 35.30, 138.80, 35.45],
      height: 500.0,
      extrudedHeight: 800.0
    },
    fill: true,
    outline: true,
    color: [0.35, 0.45, 0.35, 0.6],
    outlineColor: [0.8, 0.9, 0.8, 1.0]
  }
];

const terrainVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Cesium World Terrain (Water Mask, Vertex Normals & Sun Lighting)',
      entities: terrainBaseline
    },
    {
      label: 'Flat Ellipsoid WGS84 Reference Baseline Mode (Zero Topographic Elevation)',
      entities: [
        {
          ...terrainBaseline[0],
          id: 'terrain-ellipsoid-flat-box',
          name: 'Flat Ellipsoid WGS84 Geodetic Core',
          positionDegrees: [138.73, 35.36, 100.0],
          shape: {
            type: 'box',
            dimensions: [6000.0, 6000.0, 200.0]
          },
          color: [0.4, 0.7, 0.9, 1.0]
        },
        terrainBaseline[1],
        terrainBaseline[2],
        terrainBaseline[3]
      ]
    },
    {
      label: 'Custom Sine Wave Mathematical Procedural Heightmap Mode (Periodic Relief)',
      entities: [
        terrainBaseline[0],
        {
          ...terrainBaseline[1],
          id: 'terrain-sine-ridge-shaft',
          name: 'Sine Wave Periodic Ridge Spine',
          color: [0.85, 0.5, 0.3, 1.0]
        },
        terrainBaseline[2],
        terrainBaseline[3]
      ]
    },
    {
      label: 'Dawn Sun Angle High-Relief Valley Shadow Mode (Dynamic Solar Shading)',
      entities: [
        terrainBaseline[0],
        terrainBaseline[1],
        {
          ...terrainBaseline[2],
          id: 'terrain-solar-dawn-sphere',
          name: 'Solar Shading Dynamic Incidence Probe',
          shape: {
            type: 'ellipsoid',
            radii: [2000.0, 2000.0, 2000.0]
          },
          color: [0.95, 0.75, 0.2, 1.0]
        },
        terrainBaseline[3]
      ]
    }
  ]
};

// Recipe 4: terrain-clipping-planes
const terrainClipBaseline = [
  {
    id: 'terrainclip-excavation-box',
    name: 'Terrain Excavation Clipping Planes Boundary Box',
    show: true,
    positionDegrees: [-122.99885, 45.31705, 1400.0],
    shape: {
      type: 'box',
      dimensions: [1400.0, 1400.0, 2800.0]
    },
    fill: true,
    outline: true,
    color: [1.0, 1.0, 1.0, 0.3],
    outlineColor: [1.0, 1.0, 1.0, 1.0]
  },
  {
    id: 'terrainclip-character-cylinder',
    name: 'Excavation Depth Alignment Guide Pillar',
    show: true,
    positionDegrees: [-122.99885, 45.31705, 2900.0],
    shape: {
      type: 'cylinder',
      length: 200.0,
      topRadius: 60.0,
      bottomRadius: 60.0
    },
    fill: true,
    outline: true,
    color: [0.2, 0.8, 0.4, 1.0],
    outlineColor: [0.05, 0.4, 0.15, 1.0]
  },
  {
    id: 'terrainclip-depth-sphere',
    name: 'Subterranean Pit Floor Depth Datum Probe',
    show: true,
    positionDegrees: [-122.99885, 45.31705, 0.0],
    shape: {
      type: 'ellipsoid',
      radii: [150.0, 150.0, 150.0]
    },
    fill: true,
    outline: true,
    color: [0.9, 0.3, 0.2, 1.0],
    outlineColor: [0.45, 0.1, 0.05, 1.0]
  },
  {
    id: 'terrainclip-envelope-rect',
    name: 'Globe Surface Excavation Trench Perimeter',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.015, 45.305, -122.985, 45.330],
      height: 1000.0,
      extrudedHeight: 1200.0
    },
    fill: true,
    outline: true,
    color: [0.3, 0.35, 0.4, 0.6],
    outlineColor: [0.95, 0.95, 1.0, 1.0]
  }
];

const terrainClipVariants = {
  initial: 0,
  variants: [
    {
      label: 'Baseline: Cesium Man Excavation Pit Clipping (4 Planes, White Translucent Box)',
      entities: terrainClipBaseline
    },
    {
      label: 'St. Helens Volcanic Crater Polygon Clipping Mode (Irregular Multi-Plane Chasm)',
      entities: [
        {
          ...terrainClipBaseline[0],
          id: 'terrainclip-sthelens-chasm-box',
          name: 'St. Helens Crater Irregular Chasm Volume',
          positionDegrees: [-122.19, 46.20, 2000.0],
          shape: {
            type: 'box',
            dimensions: [3000.0, 3000.0, 1500.0]
          },
          color: [0.85, 0.4, 0.2, 0.4]
        },
        terrainClipBaseline[1],
        terrainClipBaseline[2],
        terrainClipBaseline[3]
      ]
    },
    {
      label: 'Grand Canyon Isolated Plateau Multi-Plane Mode (Island Terrain Mesa)',
      entities: [
        terrainClipBaseline[0],
        {
          ...terrainClipBaseline[1],
          id: 'terrainclip-mesa-column',
          name: 'Grand Canyon Isolated Mesa Column',
          shape: {
            type: 'cylinder',
            length: 800.0,
            topRadius: 200.0,
            bottomRadius: 300.0
          },
          color: [0.9, 0.6, 0.2, 0.9]
        },
        terrainClipBaseline[2],
        terrainClipBaseline[3]
      ]
    },
    {
      label: 'Inverted Negative Space Subsurface Chamber Mode (Edge Styling Highlight)',
      entities: [
        terrainClipBaseline[0],
        terrainClipBaseline[1],
        {
          ...terrainClipBaseline[2],
          id: 'terrainclip-chamber-dome',
          name: 'Negative Space Subsurface Dome',
          shape: {
            type: 'ellipsoid',
            radii: [300.0, 300.0, 300.0]
          },
          color: [0.2, 0.7, 0.95, 0.85]
        },
        terrainClipBaseline[3]
      ]
    }
  ]
};

// Write files
const files = [
  ['tests/sandcastle/full/styled-gltf-lines-dev.json', styledLinesBaseline],
  ['tests/sandcastle/full/styled-gltf-lines-dev.variants.json', styledLinesVariants],
  ['tests/sandcastle/full/styled-gltf-points-dev.json', styledPointsBaseline],
  ['tests/sandcastle/full/styled-gltf-points-dev.variants.json', styledPointsVariants],
  ['tests/sandcastle/full/terrain.json', terrainBaseline],
  ['tests/sandcastle/full/terrain.variants.json', terrainVariants],
  ['tests/sandcastle/full/terrain-clipping-planes.json', terrainClipBaseline],
  ['tests/sandcastle/full/terrain-clipping-planes.variants.json', terrainClipVariants]
];

for (const [relPath, data] of files) {
  const fullPath = path.join(root, relPath);
  fs.writeFileSync(fullPath, JSON.stringify(data, null, 2) + '\n');
  console.log('wrote', relPath);
}
