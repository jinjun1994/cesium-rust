import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. 3D Tiles Vertical Exaggeration
function generateVerticalExaggeration() {
  const peakCylinder = {
    id: 'vert-exag-peak-cylinder',
    name: 'Topographic Ridge Survey Pillar Cylinder',
    show: true,
    positionDegrees: [138.7274, 35.3606, 3776.0],
    shape: {
      type: 'cylinder',
      length: 12000.0,
      topRadius: 2500.0,
      bottomRadius: 2500.0,
    },
    color: [0.95, 0.35, 0.15, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.55, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      verticalExaggeration: 3.0,
      relativeHeight: 0.0,
      globeDisabled: true,
    },
  };

  const reliefBox = {
    id: 'vert-exag-relief-box',
    name: 'Exaggerated Relief Elevation Sample Box',
    show: true,
    positionDegrees: [138.74, 35.37, 3200.0],
    shape: {
      type: 'box',
      dimensions: [6000.0, 6000.0, 4500.0],
    },
    color: [0.15, 0.75, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      onlyUsingWithGoogleGeocoder: true,
      skyAtmosphereVisible: true,
    },
  };

  const alpineSphere = {
    id: 'vert-exag-alpine-sphere',
    name: 'Alpine Crest Geodetic Reference Sphere',
    show: true,
    positionDegrees: [138.71, 35.35, 3500.0],
    shape: {
      type: 'ellipsoid',
      radii: [3000.0, 3000.0, 3000.0],
    },
    color: [0.95, 0.85, 0.25, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.45, 1.0],
    outlineWidth: 1.0,
    properties: {
      photorealisticTileset: 'Google Photorealistic 3D Tiles',
    },
  };

  const terrainRect = {
    id: 'vert-exag-terrain-rect',
    name: 'Mount Fuji Terrain Exaggeration Inspection Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [138.65, 35.3, 138.8, 35.42],
      height: 1500.0,
      extrudedHeight: 5000.0,
      rotation: 0.0,
    },
    color: [0.25, 0.65, 0.35, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.85, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      cameraHeading: 5.794,
      cameraPitch: -0.303,
    },
  };

  const entities = [peakCylinder, reliefBox, alpineSphere, terrainRect];

  const variants = [
    {
      label: 'Baseline: 3x Vertical Exaggeration (scene.verticalExaggeration = 3.0)',
      entities,
    },
    {
      label: '5x Extreme Alpine Exaggeration (scene.verticalExaggeration = 5.0)',
      entities: [
        {
          ...peakCylinder,
          shape: {
            type: 'cylinder',
            length: 18000.0,
            topRadius: 3200.0,
            bottomRadius: 3200.0,
          },
          properties: { ...peakCylinder.properties, verticalExaggeration: 5.0 },
        },
        {
          ...reliefBox,
          color: [0.95, 0.15, 0.65, 0.95],
        },
        alpineSphere,
        terrainRect,
      ],
    },
    {
      label: '1x Realistic True-Scale Relief (scene.verticalExaggeration = 1.0)',
      entities: [
        {
          ...peakCylinder,
          shape: {
            type: 'cylinder',
            length: 4000.0,
            topRadius: 1800.0,
            bottomRadius: 1800.0,
          },
          properties: { ...peakCylinder.properties, verticalExaggeration: 1.0 },
        },
        reliefBox,
        {
          ...alpineSphere,
          color: [0.65, 0.85, 0.95, 0.9],
        },
        terrainRect,
      ],
    },
    {
      label: 'Exaggeration with Non-Zero Relative Height Offset (relativeHeight = 1500.0)',
      entities: [
        peakCylinder,
        {
          ...reliefBox,
          properties: { ...reliefBox.properties, relativeHeight: 1500.0 },
        },
        alpineSphere,
        terrainRect,
      ],
    },
  ];

  return { entities, variants };
}

// 2. AEC Architectural Design
function generateArchitecturalDesign() {
  const towerCylinder = {
    id: 'aec-design-tower-cylinder',
    name: 'Snowdon Towers Architectural Core Cylinder',
    show: true,
    positionDegrees: [-79.886626, 40.021649, 235.65],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 30.0,
      bottomRadius: 30.0,
    },
    color: [0.2, 0.45, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      modelName: 'Snowdon Towers (Autodesk Revit)',
      activeLayer: 'Architecture (Asset 2887123)',
      ambientOcclusion: true,
    },
  };

  const facadeBox = {
    id: 'aec-design-facade-box',
    name: 'Glass Curtain Wall Facade Envelope Box',
    show: true,
    positionDegrees: [-79.8863, 40.0219, 225.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 55.0, 80.0],
    },
    color: [0.35, 0.85, 0.95, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      facadeAssetId: 2887125,
      material: 'Structural Glass & Aluminum Mullions',
    },
  };

  const mepSphere = {
    id: 'aec-design-mep-sphere',
    name: 'MEP Services Distribution Node Sphere',
    show: true,
    positionDegrees: [-79.8869, 40.0214, 215.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 25.0, 25.0],
    },
    color: [0.95, 0.65, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.85, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      disciplines: ['Electrical', 'HVAC', 'Plumbing'],
      metadataPicking: 'HTML InfoBox Table',
    },
  };

  const siteRect = {
    id: 'aec-design-site-rect',
    name: 'Snowdon Site Clipping Polygon Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8878, 40.0214, -79.8854, 40.0231],
      height: 190.0,
      extrudedHeight: 280.0,
      rotation: 0.0,
    },
    color: [0.45, 0.75, 0.35, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.95, 0.55, 1.0],
    outlineWidth: 1.0,
    properties: {
      clippingPolygonsEnabled: true,
      googlePhotorealisticTilesetId: 2275207,
    },
  };

  const entities = [towerCylinder, facadeBox, mepSphere, siteRect];

  const variants = [
    {
      label: 'Baseline: Snowdon Towers Architecture & Facade Active with Site Clipping',
      entities,
    },
    {
      label: 'Structural & MEP Engineering Focus (Structural, HVAC, Plumbing Active)',
      entities: [
        {
          ...towerCylinder,
          color: [0.85, 0.35, 0.15, 0.95],
          properties: { ...towerCylinder.properties, activeLayer: 'Structural (Asset 2887130)' },
        },
        facadeBox,
        {
          ...mepSphere,
          color: [0.15, 0.95, 0.55, 0.95],
          shape: {
            type: 'ellipsoid',
            radii: [32.0, 32.0, 32.0],
          },
        },
        siteRect,
      ],
    },
    {
      label: 'Full Multi-Discipline BIM Convergence (All 7 Tileset Layers Active)',
      entities: [
        {
          ...towerCylinder,
          color: [0.95, 0.85, 0.15, 0.95],
        },
        {
          ...facadeBox,
          color: [0.25, 0.55, 0.95, 0.9],
        },
        {
          ...mepSphere,
          color: [0.95, 0.25, 0.25, 0.95],
        },
        siteRect,
      ],
    },
    {
      label: 'Pure Google Photorealistic Context without Architectural Inset',
      entities: [
        {
          ...towerCylinder,
          show: false,
        },
        {
          ...facadeBox,
          show: false,
        },
        {
          ...mepSphere,
          show: false,
        },
        siteRect,
      ],
    },
  ];

  return { entities, variants };
}

// 3. AEC Clipping
function generateAecClipping() {
  const designCylinder = {
    id: 'aec-clip-design-cylinder',
    name: 'Proposed Commercial Building Anchor Cylinder',
    show: true,
    positionDegrees: [-105.002, 39.754, 1600.0],
    shape: {
      type: 'cylinder',
      length: 150.0,
      topRadius: 40.0,
      bottomRadius: 40.0,
    },
    color: [0.15, 0.85, 0.55, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 1.0, 0.75, 1.0],
    outlineWidth: 1.0,
    properties: {
      proposedDesignAssetId: 2533124,
      footprintAssetId: 2533131,
    },
  };

  const volumeBox = {
    id: 'aec-clip-volume-box',
    name: 'Excavation Basement Inset Volume Box',
    show: true,
    positionDegrees: [-105.0015, 39.7545, 1580.0],
    shape: {
      type: 'box',
      dimensions: [80.0, 80.0, 60.0],
    },
    color: [0.95, 0.55, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      clippingTargetLocation: true,
      inverseClip: false,
    },
  };

  const sensorSphere = {
    id: 'aec-clip-sensor-sphere',
    name: 'Clipping Plane Normal Inspection Sphere',
    show: true,
    positionDegrees: [-105.0025, 39.7535, 1620.0],
    shape: {
      type: 'ellipsoid',
      radii: [35.0, 35.0, 35.0],
    },
    color: [0.35, 0.45, 0.95, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.65, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      cameraHeading: 95.0,
      cameraPitch: -75.0,
      cameraRange: 800.0,
    },
  };

  const footprintRect = {
    id: 'aec-clip-footprint-rect',
    name: 'Project Site Boundary Footprint Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-105.004, 39.752, -105.0, 39.756],
      height: 1550.0,
      extrudedHeight: 1720.0,
      rotation: 0.0,
    },
    color: [0.85, 0.25, 0.85, 0.65],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      clampToGround: true,
      homeButtonNavigation: true,
    },
  };

  const entities = [designCylinder, volumeBox, sensorSphere, footprintRect];

  const variants = [
    {
      label: 'Baseline: Proposed Building Inset with Normal Clipping and Footprint Active',
      entities,
    },
    {
      label: 'Inverse Clipping Mode (clippingPolygons.inverse = true: Cutout Rest of World)',
      entities: [
        designCylinder,
        {
          ...volumeBox,
          color: [0.95, 0.25, 0.25, 0.95],
          properties: { ...volumeBox.properties, inverseClip: true },
        },
        sensorSphere,
        footprintRect,
      ],
    },
    {
      label: 'Proposed Building Suppressed (Only Footprint Draped on Photorealistic Terrain)',
      entities: [
        {
          ...designCylinder,
          show: false,
        },
        volumeBox,
        sensorSphere,
        {
          ...footprintRect,
          color: [0.25, 0.95, 0.45, 0.85],
        },
      ],
    },
    {
      label: 'Pure Photorealistic Terrain without Clipping or Design Inset',
      entities: [
        {
          ...designCylinder,
          show: false,
        },
        {
          ...volumeBox,
          show: false,
        },
        {
          ...sensorSphere,
          show: false,
        },
        footprintRect,
      ],
    },
  ];

  return { entities, variants };
}

// 4. AEC Elements API Query
function generateElementsApiQuery() {
  const queryCylinder = {
    id: 'aec-elem-query-cylinder',
    name: 'Elements Query Focus Target Cylinder',
    show: true,
    positionDegrees: [-79.886626, 40.021649, 235.65],
    shape: {
      type: 'cylinder',
      length: 120.0,
      topRadius: 30.0,
      bottomRadius: 30.0,
    },
    color: [0.15, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      queryMode: 'All Walls (SQL: iFCClassName = "ifcwall")',
      assetId: 4823245,
      apiEndpoint: 'https://api.cesium.com',
    },
  };

  const wallBox = {
    id: 'aec-elem-wall-box',
    name: 'Matched Wall Element Spatial Prism Box',
    show: true,
    positionDegrees: [-79.8863, 40.0219, 225.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 55.0, 75.0],
    },
    color: [0.95, 0.85, 0.15, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.35, 1.0],
    outlineWidth: 1.0,
    properties: {
      hoverHighlighting: true,
      selectedColor: 'rgb(255, 230, 0)',
    },
  };

  const filterSphere = {
    id: 'aec-elem-filter-sphere',
    name: 'IFC Class Structural Filter Sphere',
    show: true,
    positionDegrees: [-79.8869, 40.0214, 215.0],
    shape: {
      type: 'ellipsoid',
      radii: [25.0, 25.0, 25.0],
    },
    color: [0.85, 0.25, 0.85, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.45, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      translucencyEnabled: true,
      depthTestAgainstTerrain: true,
    },
  };

  const boundsRect = {
    id: 'aec-elem-bounds-rect',
    name: 'Snowdon Towers Elements Extent Rectangle',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8878, 40.0214, -79.8854, 40.0231],
      height: 190.0,
      extrudedHeight: 280.0,
      rotation: 0.0,
    },
    color: [0.35, 0.55, 0.95, 0.7],
    fill: true,
    outline: true,
    outlineColor: [0.55, 0.75, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      resultListPanel: '#resultsList',
      paginationHeaderSupport: true,
    },
  };

  const entities = [queryCylinder, wallBox, filterSphere, boundsRect];

  const variants = [
    {
      label: 'Baseline: Query All Walls (SQL: iFCClassName = "ifcwall")',
      entities,
    },
    {
      label: 'Query Structural Elements (Columns, Beams, Slabs & Footings)',
      entities: [
        {
          ...queryCylinder,
          color: [0.95, 0.35, 0.15, 0.95],
          properties: { ...queryCylinder.properties, queryMode: 'Structural Elements' },
        },
        {
          ...wallBox,
          color: [0.85, 0.45, 0.15, 0.9],
        },
        filterSphere,
        boundsRect,
      ],
    },
    {
      label: 'Query Doors and Windows (Openings Filter)',
      entities: [
        queryCylinder,
        {
          ...wallBox,
          color: [0.15, 0.95, 0.45, 0.9],
          properties: { ...wallBox.properties, ifcClasses: ['ifcdoor', 'ifcwindow'] },
        },
        {
          ...filterSphere,
          color: [0.25, 0.85, 0.65, 0.9],
        },
        boundsRect,
      ],
    },
    {
      label: 'Query Large Spaces > 450 sq ft (Space Planning Filter)',
      entities: [
        queryCylinder,
        wallBox,
        {
          ...filterSphere,
          shape: {
            type: 'ellipsoid',
            radii: [35.0, 35.0, 35.0],
          },
          color: [0.95, 0.75, 0.15, 0.95],
          properties: { ...filterSphere.properties, areaFilter: '> 450 sq ft' },
        },
        boundsRect,
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

writeCase('3d-tiles-vertical-exaggeration', generateVerticalExaggeration());
writeCase('aec-architectural-design', generateArchitecturalDesign());
writeCase('aec-clipping', generateAecClipping());
writeCase('aec-elements-api-query', generateElementsApiQuery());
