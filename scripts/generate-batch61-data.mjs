import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Point Cloud Styling
function generatePointCloudStyling() {
  const probeCylinder = {
    id: 'pnts-style-probe-cylinder',
    name: 'Point Cloud Styling Probe Cylinder',
    show: true,
    positionDegrees: [-75.612094, 40.04253, 100.0],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 30.0,
      bottomRadius: 30.0,
    },
    color: [0.95, 0.25, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      activeStyle: 'Step Red/Blue',
      temperatureThreshold: 0.5,
    },
  };

  const volumeBox = {
    id: 'pnts-style-volume-box',
    name: 'Styling Filter Bounding Box',
    show: true,
    positionDegrees: [-75.6115, 40.043, 90.0],
    shape: {
      type: 'box',
      dimensions: [60.0, 60.0, 50.0],
    },
    color: [0.25, 0.45, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      pointSize: 5.0,
      pointCount: 1000,
    },
  };

  const tempSphere = {
    id: 'pnts-style-temperature-sphere',
    name: 'Temperature Gradient Field Sphere',
    show: true,
    positionDegrees: [-75.6125, 40.042, 95.0],
    shape: {
      type: 'ellipsoid',
      radii: [35.0, 35.0, 35.0],
    },
    color: [0.95, 0.75, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      evalExpression: "${temperature} > 0.5 ? color('red') : color('blue')",
    },
  };

  const footprintRect = {
    id: 'pnts-style-footprint-rect',
    name: 'Laser Footprint Coverage Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.614, 40.041, -75.61, 40.044],
      height: 60.0,
      extrudedHeight: 120.0,
      rotation: 0.0,
    },
    color: [0.35, 0.85, 0.45, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.95, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      tilesetUrl: '../../SampleData/Cesium3DTiles/PointCloud/PointCloudWithPerPointProperties/tileset.json',
      julianDate: 2457522.154792,
    },
  };

  const entities = [probeCylinder, volumeBox, tempSphere, footprintRect];

  const variants = [
    {
      label: 'Baseline: Step Red/Blue Temperature Styling (${temperature} > 0.5 ? red : blue)',
      entities,
    },
    {
      label: 'Color Ramp (10-Stage Temperature Conditions & Alpha)',
      entities: [
        {
          ...probeCylinder,
          color: [0.0, 0.8, 0.6, 1.0],
          properties: { ...probeCylinder.properties, activeStyle: 'Color Ramp' },
        },
        {
          ...volumeBox,
          color: [0.4, 1.0, 0.2, 0.5],
          properties: { ...volumeBox.properties, conditionStage: '0.3' },
        },
        {
          ...tempSphere,
          color: [1.0, 0.5, 0.0, 0.9],
          properties: { ...tempSphere.properties, conditionStage: '0.5' },
        },
        {
          ...footprintRect,
          color: [0.8, 0.0, 0.8, 0.7],
        },
      ],
    },
    {
      label: 'Position-Based Spatial Gradient (vec4(${POSITION}, 1.0))',
      entities: [
        {
          ...probeCylinder,
          color: [0.85, 0.35, 0.95, 0.95],
          properties: { ...probeCylinder.properties, activeStyle: 'Color based on position' },
        },
        volumeBox,
        {
          ...tempSphere,
          color: [0.15, 0.85, 0.95, 0.9],
          properties: { ...tempSphere.properties, coordinateGradient: 'normalized-xyz' },
        },
        footprintRect,
      ],
    },
    {
      label: 'Point Size & Multiple Defines Dynamic Styling',
      entities: [
        probeCylinder,
        {
          ...volumeBox,
          shape: {
            type: 'box',
            dimensions: [80.0, 80.0, 65.0],
          },
          properties: { ...volumeBox.properties, pointSize: 'temperature * 10' },
        },
        {
          ...tempSphere,
          shape: {
            type: 'ellipsoid',
            radii: [45.0, 45.0, 45.0],
          },
          properties: { ...tempSphere.properties, dynamicDefines: 'length, time' },
        },
        footprintRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. 3D Tiles Property LOD River
function generatePropertyLodRiver() {
  const flowCylinder = {
    id: 'river-lod-flow-cylinder',
    name: 'HydroRIVERS Gauge Station Cylinder',
    show: true,
    positionDegrees: [86.859391, 26.907291, 2000.0],
    shape: {
      type: 'cylinder',
      length: 5000.0,
      topRadius: 1500.0,
      bottomRadius: 1500.0,
    },
    color: [0.0, 0.0, 0.5, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.1, 0.2, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      ionAssetId: 5135960,
      ordFlowOrder: 1,
      flowClassification: 'Main Stem Mega River',
    },
  };

  const basinBox = {
    id: 'river-lod-basin-box',
    name: 'Catchment Basin Extent Box',
    show: true,
    positionDegrees: [86.865, 26.915, 1800.0],
    shape: {
      type: 'box',
      dimensions: [4000.0, 4000.0, 3000.0],
    },
    color: [0.12, 0.53, 0.9, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.25, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      ordFlowRange: '1-10',
      drapingEnabled: true,
      heightReference: 'CLAMP_TO_TERRAIN',
    },
  };

  const watershedSphere = {
    id: 'river-lod-watershed-sphere',
    name: 'Watershed Node Inspection Sphere',
    show: true,
    positionDegrees: [86.852, 26.9, 1900.0],
    shape: {
      type: 'ellipsoid',
      radii: [2000.0, 2000.0, 2000.0],
    },
    color: [0.0, 1.0, 1.0, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.3, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      lineWidthEquation: '5.0 - clamp(Number(${ORD_FLOW}), 0.0, 10.0) * 0.4',
    },
  };

  const corridorRect = {
    id: 'river-lod-corridor-rect',
    name: 'Himalayan River Corridor Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.82, 26.88, 86.9, 26.94],
      height: 1000.0,
      extrudedHeight: 2500.0,
      rotation: 0.0,
    },
    color: [0.2, 0.8, 0.2, 0.75],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.95, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      legendElement: '#ordFlowLegend',
      sourceDocumentation: 'HydroRIVERS_TechDoc_v10.pdf',
    },
  };

  const entities = [flowCylinder, basinBox, watershedSphere, corridorRect];

  const variants = [
    {
      label: 'Baseline: HydroRIVERS ORD_FLOW River Hierarchy (LOD 1-10 Rainbow Color Ramp)',
      entities,
    },
    {
      label: 'Major Rivers Emphasized (ORD_FLOW <= 3 Navy/Blue with Thick Lines)',
      entities: [
        {
          ...flowCylinder,
          color: [0.0, 0.1, 0.6, 1.0],
          shape: {
            type: 'cylinder',
            length: 6500.0,
            topRadius: 2000.0,
            bottomRadius: 2000.0,
          },
          properties: { ...flowCylinder.properties, highlightedOrder: '1-3' },
        },
        {
          ...basinBox,
          color: [0.0, 0.4, 0.9, 0.95],
          properties: { ...basinBox.properties, minWidth: 2.6 },
        },
        watershedSphere,
        corridorRect,
      ],
    },
    {
      label: 'Narrow Tributaries Accentuated (ORD_FLOW >= 7 Warm Hues)',
      entities: [
        flowCylinder,
        basinBox,
        {
          ...watershedSphere,
          color: [1.0, 0.4, 0.1, 0.95],
          properties: { ...watershedSphere.properties, accentuatedOrder: '7-10' },
        },
        {
          ...corridorRect,
          color: [0.95, 0.65, 0.15, 0.8],
        },
      ],
    },
    {
      label: 'Monochrome River Hierarchy with Variable Line Widths',
      entities: [
        {
          ...flowCylinder,
          color: [0.75, 0.75, 0.75, 0.95],
          properties: { ...flowCylinder.properties, styleMode: 'monochrome' },
        },
        {
          ...basinBox,
          color: [0.55, 0.55, 0.55, 0.9],
        },
        {
          ...watershedSphere,
          color: [0.9, 0.9, 0.9, 0.95],
        },
        corridorRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. 3D Tiles Split Dev
function generateSplitDev() {
  const seamCylinder = {
    id: 'split-dev-seam-cylinder',
    name: 'Split Viewport Partition Marker Cylinder',
    show: true,
    positionDegrees: [-75.612094, 40.04253, 100.0],
    shape: {
      type: 'cylinder',
      length: 150.0,
      topRadius: 35.0,
      bottomRadius: 35.0,
    },
    color: [0.15, 0.85, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      splitDirection: 'NONE',
      splitPosition: 0.5,
    },
  };

  const tilesetBox = {
    id: 'split-dev-tileset-box',
    name: 'Active 3D Tileset Spatial Volume Box',
    show: true,
    positionDegrees: [-75.6115, 40.043, 90.0],
    shape: {
      type: 'box',
      dimensions: [70.0, 70.0, 60.0],
    },
    color: [0.95, 0.65, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      selectedTileset: 'Tileset',
      supportedTilesetsCount: 14,
    },
  };

  const cameraSphere = {
    id: 'split-dev-camera-sphere',
    name: 'Split Camera Pivot Sphere',
    show: true,
    positionDegrees: [-75.6125, 40.042, 95.0],
    shape: {
      type: 'ellipsoid',
      radii: [40.0, 40.0, 40.0],
    },
    color: [0.85, 0.25, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      enableDebugWireframe: true,
      shadows: true,
    },
  };

  const boundaryRect = {
    id: 'split-dev-boundary-rect',
    name: 'Split Multi-Tileset Boundary Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-75.614, 40.041, -75.61, 40.044],
      height: 60.0,
      extrudedHeight: 120.0,
      rotation: 0.0,
    },
    color: [0.25, 0.55, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.75, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      sliderElement: '#slider',
      inspectorMixinEnabled: true,
    },
  };

  const entities = [seamCylinder, tilesetBox, cameraSphere, boundaryRect];

  const variants = [
    {
      label: 'Baseline: Tileset Default Split (SplitDirection.NONE, slider at 50%)',
      entities,
    },
    {
      label: 'Left Viewport Split (SplitDirection.LEFT, slider at 30%)',
      entities: [
        {
          ...seamCylinder,
          color: [0.15, 0.95, 0.45, 0.95],
          properties: { ...seamCylinder.properties, splitDirection: 'LEFT', splitPosition: 0.3 },
        },
        {
          ...tilesetBox,
          color: [0.25, 0.85, 0.35, 0.9],
        },
        cameraSphere,
        boundaryRect,
      ],
    },
    {
      label: 'Right Viewport Split (SplitDirection.RIGHT, slider at 70%)',
      entities: [
        {
          ...seamCylinder,
          color: [0.95, 0.35, 0.25, 0.95],
          properties: { ...seamCylinder.properties, splitDirection: 'RIGHT', splitPosition: 0.7 },
        },
        tilesetBox,
        {
          ...cameraSphere,
          color: [0.95, 0.55, 0.15, 0.9],
        },
        boundaryRect,
      ],
    },
    {
      label: 'Instanced Tileset Split with Shadow Caster Enabled',
      entities: [
        seamCylinder,
        {
          ...tilesetBox,
          shape: {
            type: 'box',
            dimensions: [90.0, 90.0, 75.0],
          },
          properties: { ...tilesetBox.properties, selectedTileset: 'Instanced' },
        },
        {
          ...cameraSphere,
          properties: { ...cameraSphere.properties, shadows: true, shadowMapSize: 2048 },
        },
        boundaryRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. 3D Tiles Terrain Classification
function generateTerrainClassification() {
  const viennaCylinder = {
    id: 'terrain-class-vienna-cylinder',
    name: 'Vienna Geocoded Anchor Cylinder',
    show: true,
    positionDegrees: [16.3738, 48.2082, 350.0],
    shape: {
      type: 'cylinder',
      length: 600.0,
      topRadius: 180.0,
      bottomRadius: 180.0,
    },
    color: [0.95, 0.95, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      geocoderQuery: 'Vienna, Austria',
      ionAssetId: 5737,
      classificationType: 'TERRAIN',
    },
  };

  const parcelBox = {
    id: 'terrain-class-parcel-box',
    name: 'Classified Urban Building Volume Box',
    show: true,
    positionDegrees: [16.376, 48.21, 320.0],
    shape: {
      type: 'box',
      dimensions: [350.0, 350.0, 250.0],
    },
    color: [1.0, 1.0, 1.0, 0.5],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 0.8],
    outlineWidth: 1.0,
    properties: {
      styleColor: 'rgba(255, 255, 255, 0.5)',
      format: 'Vector 3D Tiles (vctr)',
    },
  };

  const highlightSphere = {
    id: 'terrain-class-highlight-sphere',
    name: 'Vector Feature Highlight Sphere',
    show: true,
    positionDegrees: [16.371, 48.206, 330.0],
    shape: {
      type: 'ellipsoid',
      radii: [200.0, 200.0, 200.0],
    },
    color: [1.0, 1.0, 0.0, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      hoverHighlightColor: 'Color.YELLOW',
      interactionType: 'MOUSE_MOVE',
    },
  };

  const boundaryRect = {
    id: 'terrain-class-boundary-rect',
    name: 'Vienna Vector Classification Bounds Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [16.35, 48.19, 16.4, 48.23],
      height: 200.0,
      extrudedHeight: 450.0,
      rotation: 0.0,
    },
    color: [0.35, 0.65, 0.95, 0.6],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      terrainProvider: 'Terrain.fromWorldTerrain()',
      flightDuration: 0.0,
    },
  };

  const entities = [viennaCylinder, parcelBox, highlightSphere, boundaryRect];

  const variants = [
    {
      label: 'Baseline: Vienna Austria Vector Classification Draped on Terrain (rgba(255, 255, 255, 0.5))',
      entities,
    },
    {
      label: 'Hover Highlight State (Yellow Color Override on Picked Feature)',
      entities: [
        viennaCylinder,
        {
          ...parcelBox,
          color: [1.0, 1.0, 0.0, 0.85],
          properties: { ...parcelBox.properties, isHovered: true },
        },
        {
          ...highlightSphere,
          shape: {
            type: 'ellipsoid',
            radii: [260.0, 260.0, 260.0],
          },
          color: [1.0, 0.9, 0.0, 1.0],
        },
        boundaryRect,
      ],
    },
    {
      label: 'Colored Zoning Parcels (Green/Blue Draped Polygon Classification)',
      entities: [
        {
          ...viennaCylinder,
          color: [0.2, 0.8, 0.4, 0.75],
        },
        {
          ...parcelBox,
          color: [0.15, 0.55, 0.95, 0.7],
          properties: { ...parcelBox.properties, zoningType: 'Residential High Density' },
        },
        {
          ...highlightSphere,
          color: [0.3, 0.9, 0.5, 0.9],
        },
        boundaryRect,
      ],
    },
    {
      label: 'Classification Suppressed (Pure World Terrain View in Vienna)',
      entities: [
        {
          ...viennaCylinder,
          show: false,
        },
        {
          ...parcelBox,
          show: false,
        },
        {
          ...highlightSphere,
          show: false,
        },
        boundaryRect,
      ],
    },
  ];

  return { entities, variants };
}

function writeCase(id, { entities, variants }) {
  fs.writeFileSync(path.join(outDir, `${id}.json`), JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(path.join(outDir, `${id}.variants.json`), JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json and ${id}.variants.json`);
}

writeCase('3d-tiles-point-cloud-styling', generatePointCloudStyling());
writeCase('3d-tiles-property-lod-river', generatePropertyLodRiver());
writeCase('3d-tiles-split-dev', generateSplitDev());
writeCase('3d-tiles-terrain-classification', generateTerrainClassification());
