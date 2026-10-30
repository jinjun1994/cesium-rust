import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. AEC Isolate by Category (Snowdon Towers sample)
function generateIsolateByCategory() {
  const wallBox = {
    id: 'aec-isolate-wall',
    name: 'Snowdon Towers Architectural Walls (-2000011)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 245.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 6.0, 25.0],
    },
    color: [0.82, 0.76, 0.7, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: -2000011,
      categoryName: 'Walls',
      revitCategory: 'OST_Walls',
    },
  };

  const floorBox = {
    id: 'aec-isolate-floor',
    name: 'Snowdon Towers Slabs and Floors (-2000032)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 233.0],
    shape: {
      type: 'box',
      dimensions: [55.0, 45.0, 2.5],
    },
    color: [0.65, 0.68, 0.72, 0.9],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: -2000032,
      categoryName: 'Floors',
      revitCategory: 'OST_Floors',
    },
  };

  const stairsCylinder = {
    id: 'aec-isolate-stairs',
    name: 'Snowdon Towers Spiral Stairs Core (-2000120)',
    show: true,
    positionDegrees: [-79.8863, 40.0219, 245.0],
    shape: {
      type: 'cylinder',
      length: 26.0,
      topRadius: 5.0,
      bottomRadius: 5.0,
    },
    color: [0.85, 0.52, 0.22, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: -2000120,
      categoryName: 'Stairs',
      revitCategory: 'OST_Stairs',
    },
  };

  const doorsBox = {
    id: 'aec-isolate-doors',
    name: 'Snowdon Towers Entrance Doors (-2000023)',
    show: true,
    positionDegrees: [-79.8868, 40.0214, 236.0],
    shape: {
      type: 'box',
      dimensions: [3.0, 8.0, 6.0],
    },
    color: [0.2, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: -2000023,
      categoryName: 'Doors',
      revitCategory: 'OST_Doors',
    },
  };

  const lightingSphere = {
    id: 'aec-isolate-lighting',
    name: 'Snowdon Towers Lighting Luminaires (-2001120)',
    show: true,
    positionDegrees: [-79.8865, 40.0218, 252.0],
    shape: {
      type: 'ellipsoid',
      radii: [3.5, 3.5, 3.5],
    },
    color: [0.98, 0.92, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: -2001120,
      categoryName: 'Lighting Fixtures',
      revitCategory: 'OST_LightingFixtures',
    },
  };

  const siteBounds = {
    id: 'aec-isolate-site-bounds',
    name: 'Snowdon Towers Clipping Site Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8877, 40.0214, -79.8855, 40.0231],
    },
    color: [0.28, 0.48, 0.38, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'siteClippingBounds',
    },
  };

  const entities = [wallBox, floorBox, stairsCylinder, doorsBox, lightingSphere, siteBounds];

  const variants = [
    {
      label: 'Show All Categories',
      entities: [
        { ...wallBox, show: true },
        { ...floorBox, show: true },
        { ...stairsCylinder, show: true },
        { ...doorsBox, show: true },
        { ...lightingSphere, show: true },
        siteBounds,
      ],
    },
    {
      label: 'Isolate Walls (OST_Walls -2000011)',
      entities: [
        { ...wallBox, show: true },
        { ...floorBox, show: false },
        { ...stairsCylinder, show: false },
        { ...doorsBox, show: false },
        { ...lightingSphere, show: false },
        siteBounds,
      ],
    },
    {
      label: 'Isolate Floors (OST_Floors -2000032)',
      entities: [
        { ...wallBox, show: false },
        { ...floorBox, show: true },
        { ...stairsCylinder, show: false },
        { ...doorsBox, show: false },
        { ...lightingSphere, show: false },
        siteBounds,
      ],
    },
    {
      label: 'Isolate Stairs (OST_Stairs -2000120)',
      entities: [
        { ...wallBox, show: false },
        { ...floorBox, show: false },
        { ...stairsCylinder, show: true },
        { ...doorsBox, show: false },
        { ...lightingSphere, show: false },
        siteBounds,
      ],
    },
    {
      label: 'Isolate Doors (OST_Doors -2000023)',
      entities: [
        { ...wallBox, show: false },
        { ...floorBox, show: false },
        { ...stairsCylinder, show: false },
        { ...doorsBox, show: true },
        { ...lightingSphere, show: false },
        siteBounds,
      ],
    },
    {
      label: 'Isolate Lighting (OST_LightingFixtures -2001120)',
      entities: [
        { ...wallBox, show: false },
        { ...floorBox, show: false },
        { ...stairsCylinder, show: false },
        { ...doorsBox, show: false },
        { ...lightingSphere, show: true },
        siteBounds,
      ],
    },
  ];

  return { entities, variants };
}

// 2. AEC Metadata Styling (HVAC Pipe length filter with ghost architecture)
function generateMetadataStyling() {
  const ghostArchitecture = {
    id: 'aec-meta-ghost-building',
    name: 'Snowdon Towers Ghost Architecture Shell',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 246.0],
    shape: {
      type: 'box',
      dimensions: [56.0, 48.0, 32.0],
    },
    color: [0.68, 0.85, 0.95, 0.18],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.9, 1.0, 0.45],
    outlineWidth: 1.0,
    properties: {
      role: 'ghostArchitecture',
      styleFormula: "color('lightblue', 0.05)",
    },
  };

  const pipeMain = {
    id: 'aec-meta-hvac-main',
    name: 'HVAC Main Supply Trunk (Length 42m)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 252.0],
    shape: {
      type: 'cylinder',
      length: 42.0,
      topRadius: 3.5,
      bottomRadius: 3.5,
    },
    color: [0.95, 0.25, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      length: 42.0,
      ductType: 'Primary Supply Air Trunk',
      sectionMm: '600x400',
    },
  };

  const pipeBranch = {
    id: 'aec-meta-hvac-branch',
    name: 'HVAC Branch Supply Duct (Length 18m)',
    show: true,
    positionDegrees: [-79.8863, 40.0219, 246.0],
    shape: {
      type: 'cylinder',
      length: 24.0,
      topRadius: 2.8,
      bottomRadius: 2.8,
    },
    color: [0.95, 0.25, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      length: 18.0,
      ductType: 'Secondary Distribution Duct',
      sectionMm: '350x250',
    },
  };

  const pipeTerminal = {
    id: 'aec-meta-hvac-terminal',
    name: 'HVAC Diffuser Terminal Run (Length 6m)',
    show: true,
    positionDegrees: [-79.8868, 40.0215, 240.0],
    shape: {
      type: 'box',
      dimensions: [12.0, 4.0, 4.0],
    },
    color: [0.95, 0.25, 0.2, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.5, 0.5, 1.0],
    outlineWidth: 1.0,
    properties: {
      length: 6.0,
      ductType: 'Ceiling Diffuser Terminal Run',
      sectionMm: '200x200',
    },
  };

  const siteFootprint = {
    id: 'aec-meta-site-footprint',
    name: 'Snowdon Towers MEP Site Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8876, 40.0214, -79.8856, 40.0230],
    },
    color: [0.25, 0.42, 0.35, 0.3],
    fill: true,
    outline: true,
    outlineColor: [0.35, 0.65, 0.5, 0.7],
    outlineWidth: 1.0,
    properties: {
      role: 'siteFootprint',
    },
  };

  const entities = [ghostArchitecture, pipeMain, pipeBranch, pipeTerminal, siteFootprint];

  const variants = [
    {
      label: 'Highlight All Pipe Lengths (0m - 50m)',
      entities: [
        ghostArchitecture,
        { ...pipeMain, color: [0.95, 0.25, 0.2, 0.95] },
        { ...pipeBranch, color: [0.95, 0.25, 0.2, 0.95] },
        { ...pipeTerminal, color: [0.95, 0.25, 0.2, 0.95] },
        siteFootprint,
      ],
    },
    {
      label: 'Highlight Short Terminal Runs (0m - 10m)',
      entities: [
        ghostArchitecture,
        { ...pipeMain, color: [0.3, 0.6, 0.85, 0.6] },
        { ...pipeBranch, color: [0.3, 0.6, 0.85, 0.6] },
        { ...pipeTerminal, color: [0.95, 0.25, 0.2, 0.95] },
        siteFootprint,
      ],
    },
    {
      label: 'Highlight Medium Branch Ducts (10m - 25m)',
      entities: [
        ghostArchitecture,
        { ...pipeMain, color: [0.3, 0.6, 0.85, 0.6] },
        { ...pipeBranch, color: [0.95, 0.25, 0.2, 0.95] },
        { ...pipeTerminal, color: [0.3, 0.6, 0.85, 0.6] },
        siteFootprint,
      ],
    },
    {
      label: 'Highlight Long Supply Mains (25m - 50m)',
      entities: [
        ghostArchitecture,
        { ...pipeMain, color: [0.95, 0.25, 0.2, 0.95] },
        { ...pipeBranch, color: [0.3, 0.6, 0.85, 0.6] },
        { ...pipeTerminal, color: [0.3, 0.6, 0.85, 0.6] },
        siteFootprint,
      ],
    },
  ];

  return { entities, variants };
}

// 3. AEC Snapping (Hybrid Server-Client Snapping & Edge Feedback)
function generateSnapping() {
  const bimColumn = {
    id: 'aec-snap-column',
    name: 'BIM Structural Column Geometry Target',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 245.0],
    shape: {
      type: 'box',
      dimensions: [10.0, 10.0, 35.0],
    },
    color: [0.72, 0.78, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.2, 0.5, 0.9, 1.0],
    outlineWidth: 1.0,
    properties: {
      edgeDisplayMode: 'SurfacesAndEdges',
      snapTarget: true,
      assetId: 5161569,
    },
  };

  const hoverEdgeMarker = {
    id: 'aec-snap-hover-marker',
    name: 'Cyan Hover Edge Snap Indicator',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 268.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0],
    },
    color: [0.0, 1.0, 1.0, 1.0], // Cyan
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'hoverEdge',
      pixelSize: 10,
    },
  };

  const clientSnapMarker = {
    id: 'aec-snap-client-marker',
    name: 'Lime Client Instant Snap Marker',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 268.0],
    shape: {
      type: 'ellipsoid',
      radii: [7.0, 7.0, 7.0],
    },
    color: [0.2, 1.0, 0.2, 1.0], // Lime
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'clientSnap',
      pixelSize: 8,
    },
  };

  const serverSnapMarker = {
    id: 'aec-snap-server-marker',
    name: 'Red Server Geometry-Refined Snap Marker',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 268.0],
    shape: {
      type: 'ellipsoid',
      radii: [9.0, 9.0, 9.0],
    },
    color: [1.0, 0.15, 0.15, 1.0], // Red
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'serverSnap',
      pixelSize: 14,
    },
  };

  const terrainHoverMarker = {
    id: 'aec-snap-terrain-marker',
    name: 'Gray Terrain Cursor Feedback',
    show: true,
    positionDegrees: [-79.8864, 40.0215, 245.0],
    shape: {
      type: 'ellipsoid',
      radii: [8.0, 8.0, 8.0],
    },
    color: [0.55, 0.55, 0.55, 0.9], // Gray
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      snapMode: 'hoverTerrain',
    },
  };

  const foundationRectangle = {
    id: 'aec-snap-foundation-pad',
    name: 'BIM Snapping Foundation Slab Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8872, 40.0213, -79.8858, 40.0222],
    },
    color: [0.3, 0.42, 0.52, 0.4],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.6, 0.75, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'foundationPad',
    },
  };

  const entities = [
    bimColumn,
    hoverEdgeMarker,
    clientSnapMarker,
    serverSnapMarker,
    terrainHoverMarker,
    foundationRectangle,
  ];

  const variants = [
    {
      label: 'Edge Snapping Mode (Cyan Edge Hover + Lime Client + Red Server)',
      entities: [
        bimColumn,
        { ...hoverEdgeMarker, show: true, color: [0.0, 1.0, 1.0, 1.0] },
        { ...clientSnapMarker, show: true },
        { ...serverSnapMarker, show: true, color: [1.0, 0.15, 0.15, 1.0] },
        { ...terrainHoverMarker, show: false },
        foundationRectangle,
      ],
    },
    {
      label: 'Surface Snapping Mode (Yellow Surface Hover + Surface Snap)',
      entities: [
        bimColumn,
        { ...hoverEdgeMarker, show: true, color: [1.0, 0.95, 0.1, 1.0] }, // Yellow
        { ...clientSnapMarker, show: true },
        { ...serverSnapMarker, show: true, color: [1.0, 0.4, 0.1, 1.0] }, // Orange/Surface
        { ...terrainHoverMarker, show: false },
        foundationRectangle,
      ],
    },
    {
      label: 'Terrain Hover Mode (Gray Feedback, Reject Server Snap)',
      entities: [
        bimColumn,
        { ...hoverEdgeMarker, show: false },
        { ...clientSnapMarker, show: false },
        { ...serverSnapMarker, show: false },
        { ...terrainHoverMarker, show: true },
        foundationRectangle,
      ],
    },
    {
      label: 'All Snapping Calibration Markers Enabled',
      entities: [
        bimColumn,
        { ...hoverEdgeMarker, show: true },
        { ...clientSnapMarker, show: true },
        { ...serverSnapMarker, show: true },
        { ...terrainHoverMarker, show: true },
        foundationRectangle,
      ],
    },
  ];

  return { entities, variants };
}

// 4. AEC Tileset Metadata Filtering (Parallel metadata arrays, Solo & Hide-by-Pick)
function generateTilesetMetadataFiltering() {
  const colCylinder = {
    id: 'aec-filter-col-cylinder',
    name: 'Cast-In-Place Concrete Columns (Cat 101, Sub 201)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 245.0],
    shape: {
      type: 'cylinder',
      length: 26.0,
      topRadius: 2.0,
      bottomRadius: 2.0,
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.5, 0.85, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: 101,
      categoryName: 'Structural Columns',
      subcategoryId: 201,
      subcategoryName: 'Cast-In-Place Concrete',
    },
  };

  const curtainWallBox = {
    id: 'aec-filter-mullions-box',
    name: 'Curtain Wall System & Mullions (Cat 102, Sub 202)',
    show: true,
    positionDegrees: [-79.8863, 40.0219, 246.0],
    shape: {
      type: 'box',
      dimensions: [22.0, 1.5, 26.0],
    },
    color: [0.35, 0.88, 0.92, 0.88],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.98, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: 102,
      categoryName: 'Curtain Walls',
      subcategoryId: 202,
      subcategoryName: 'Curtain Wall Mullions',
    },
  };

  const floorSlabBox = {
    id: 'aec-filter-floor-slab',
    name: 'Structural Concrete Floor Slabs (Cat 103, Sub 203)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 233.0],
    shape: {
      type: 'box',
      dimensions: [48.0, 48.0, 2.5],
    },
    color: [0.72, 0.75, 0.8, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.9, 0.92, 0.98, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: 103,
      categoryName: 'Floors',
      subcategoryId: 203,
      subcategoryName: 'Concrete Slab',
    },
  };

  const roofDeckBox = {
    id: 'aec-filter-roof-deck',
    name: 'Composite Steel Roof Deck (Cat 104, Sub 204)',
    show: true,
    positionDegrees: [-79.8866, 40.0217, 258.0],
    shape: {
      type: 'box',
      dimensions: [46.0, 46.0, 1.8],
    },
    color: [0.55, 0.58, 0.65, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.8, 0.88, 1.0],
    outlineWidth: 1.0,
    properties: {
      categoryId: 104,
      categoryName: 'Roofs',
      subcategoryId: 204,
      subcategoryName: 'Steel Deck',
    },
  };

  const filterSiteGrid = {
    id: 'aec-filter-site-grid',
    name: 'Snowdon Towers Advanced Metadata Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-79.8876, 40.0213, -79.8856, 40.0229],
    },
    color: [0.3, 0.42, 0.36, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.68, 0.55, 0.8],
    outlineWidth: 1.0,
    properties: {
      role: 'metadataSiteGrid',
    },
  };

  const entities = [colCylinder, curtainWallBox, floorSlabBox, roofDeckBox, filterSiteGrid];

  const variants = [
    {
      label: 'All Categories and Subcategories Visible',
      entities: [
        { ...colCylinder, show: true },
        { ...curtainWallBox, show: true },
        { ...floorSlabBox, show: true },
        { ...roofDeckBox, show: true },
        filterSiteGrid,
      ],
    },
    {
      label: 'Solo Structural Columns (CTRL+Click Solo Cat 101)',
      entities: [
        { ...colCylinder, show: true },
        { ...curtainWallBox, show: false },
        { ...floorSlabBox, show: false },
        { ...roofDeckBox, show: false },
        filterSiteGrid,
      ],
    },
    {
      label: 'Filter Subcategories (Hide Mullions Sub 202)',
      entities: [
        { ...colCylinder, show: true },
        { ...curtainWallBox, show: false },
        { ...floorSlabBox, show: true },
        { ...roofDeckBox, show: true },
        filterSiteGrid,
      ],
    },
    {
      label: 'Hide by Pick Mode (Hide Roof & Floors)',
      entities: [
        { ...colCylinder, show: true },
        { ...curtainWallBox, show: true },
        { ...floorSlabBox, show: false },
        { ...roofDeckBox, show: false },
        filterSiteGrid,
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

writeCase('aec-isolate-by-category', generateIsolateByCategory());
writeCase('aec-metadata-styling', generateMetadataStyling());
writeCase('aec-snapping', generateSnapping());
writeCase('aec-tileset-metadata-filtering', generateTilesetMetadataFiltering());
