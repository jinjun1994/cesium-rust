#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 47 recipes:
 * 1. drawing-on-terrain (Interactive terrain drawing: polylines, polygons, and vertex points)
 * 2. rotatable-2d-map (Rotatable 2D map view: SCENE2D mode with heading rotations)
 * 3. polylines-on-terrain-dev (Grand Canyon terrain ground polylines and classification rectangle)
 * 4. elevation-band-material (Himalayan elevation contour bands and gradient material layers)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

const logoPng =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

function generateDrawingOnTerrain() {
  const drawnPolyline = {
    id: 'drawing-polyline',
    name: 'Drawn Terrain Polyline',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.25, 46.18, 0.0],
          [-122.20, 46.20, 0.0],
          [-122.15, 46.19, 0.0],
          [-122.10, 46.22, 0.0],
        ],
        width: 3.0,
        arcType: 'GEODESIC',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { drawingMode: 'line', clampToGround: true },
  };

  const drawnPolygon = {
    id: 'drawing-polygon',
    name: 'Drawn Terrain Polygon',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-122.22, 46.17, 0.0],
        [-122.16, 46.17, 0.0],
        [-122.14, 46.21, 0.0],
        [-122.20, 46.21, 0.0],
      ],
      holes: [],
      height: 0.0,
      extrudedHeight: null,
    },
    color: [0.3, 0.8, 1.0, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { drawingMode: 'polygon', fillOpacity: 0.7 },
  };

  const vertexPt1 = {
    id: 'drawing-pt-1',
    name: 'Vertex 1',
    show: true,
    positionDegrees: [-122.25, 46.18, 0.0],
    shape: {
      type: 'point',
      pixelSize: 6.0,
      outlineWidth: 0.0,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
    },
    color: [1.0, 0.3, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { vertexIndex: 1 },
  };

  const vertexPt2 = {
    id: 'drawing-pt-2',
    name: 'Vertex 2',
    show: true,
    positionDegrees: [-122.10, 46.22, 0.0],
    shape: {
      type: 'point',
      pixelSize: 6.0,
      outlineWidth: 0.0,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
    },
    color: [1.0, 0.3, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { vertexIndex: 2 },
  };

  const entities = [drawnPolyline, drawnPolygon, vertexPt1, vertexPt2];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Both Line and Polygon on Terrain',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Line Drawing Mode Only',
        entities: entities.map((e) => {
          if (e.id === 'drawing-polygon') return { ...e, show: false };
          return { ...e, show: true };
        }),
      },
      {
        label: 'Polygon Drawing Mode Only',
        entities: entities.map((e) => {
          if (e.id === 'drawing-polyline') return { ...e, show: false };
          return { ...e, show: true };
        }),
      },
      {
        label: 'Custom High-Visibility Styles',
        entities: entities.map((e) => {
          if (e.id === 'drawing-polyline') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.9, 0.0, 1.0],
              shape: { ...e.shape, data: { ...e.shape.data, width: 5.0 } },
            };
          }
          if (e.id === 'drawing-polygon') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.9, 0.4, 0.8],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(fullDir, 'drawing-on-terrain.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(fullDir, 'drawing-on-terrain.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated drawing-on-terrain.json and .variants.json');
}

function generateRotatable2dMap() {
  const pin = {
    id: 'rotatable-2d-pin',
    name: 'Boston Regional Landmark',
    show: true,
    positionDegrees: [-73.0, 42.0, 0.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.2,
      width: 32,
      height: 32,
    },
    color: [1.0, 0.2, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { landmark: 'Boston Region', mode: 'SCENE2D' },
  };

  const coverageRect = {
    id: 'rotatable-2d-rect',
    name: 'New England Inspection Zone',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-76.0, 40.0, -70.0, 44.0],
      height: 0.0,
      extrudedHeight: null,
      rotation: 0.0,
    },
    color: [0.1, 0.6, 1.0, 0.35],
    fill: true,
    outline: true,
    outlineColor: [0.0, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { region: 'New England', bounds: '[-76, 40, -70, 44]' },
  };

  const parallelGuide = {
    id: 'rotatable-2d-parallel',
    name: '42N Reference Parallel',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-78.0, 42.0, 0.0],
          [-73.0, 42.0, 0.0],
          [-68.0, 42.0, 0.0],
        ],
        width: 3.0,
        arcType: 'RHUMB',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: {
          type: 'dash',
          gapColor: [0, 0, 0, 0],
          dashLength: 16.0,
          dashPattern: 255,
        },
      },
    },
    color: [1.0, 0.85, 0.1, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'parallel', latitude: 42.0 },
  };

  const entities = [pin, coverageRect, parallelGuide];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Heading -45 Degrees (Northwest Incline)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Heading 0 Degrees (Standard North)',
        entities: entities.map((e) => {
          if (e.id === 'rotatable-2d-rect') {
            return {
              ...e,
              color: [0.2, 0.8, 0.3, 0.35],
              outlineColor: [0.1, 0.9, 0.4, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Heading 90 Degrees (Eastward Orientation)',
        entities: entities.map((e) => {
          if (e.id === 'rotatable-2d-pin') {
            return {
              ...e,
              color: [1.0, 0.7, 0.0, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Heading 180 Degrees (Southward Orientation)',
        entities: entities.map((e) => {
          if (e.id === 'rotatable-2d-parallel') {
            return {
              ...e,
              color: [1.0, 0.2, 0.5, 1.0],
              shape: { ...e.shape, data: { ...e.shape.data, width: 5.0 } },
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(fullDir, 'rotatable-2d-map.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(fullDir, 'rotatable-2d-map.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated rotatable-2d-map.json and .variants.json');
}

function generatePolylinesOnTerrainDev() {
  const hairpinPositions = [
    [-112.134, 36.055, 0.0],
    [-112.088, 36.098, 0.0],
    [-112.133, 36.169, 0.0],
    [-112.108, 36.200, 0.0],
    [-112.139, 36.169, 0.0],
    [-112.115, 36.201, 0.0],
    [-112.159, 36.179, 0.0],
    [-112.147, 36.217, 0.0],
    [-112.209, 36.195, 0.0],
  ];

  const loopPositions = [
    [-111.945, 36.276, 0.0],
    [-111.910, 36.080, 0.0],
    [-111.804, 36.137, 0.0],
    [-111.855, 36.260, 0.0],
    [-111.691, 36.051, 0.0],
  ];

  const polyline1 = {
    id: 'terrain-poly-hairpin',
    name: 'Grand Canyon Hairpin Trail',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: hairpinPositions,
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 0.3, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { trail: 'Hairpin Trail', lengthKm: 18.5 },
  };

  const polyline2 = {
    id: 'terrain-poly-loop',
    name: 'Grand Canyon Eastern Loop',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: loopPositions,
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: {
          type: 'glow',
          glowPower: 0.25,
          taperPower: 1.0,
        },
      },
    },
    color: [0.0, 0.8, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { trail: 'Eastern Loop', lengthKm: 24.0 },
  };

  const groundRect = {
    id: 'terrain-ground-rect',
    name: 'Grand Canyon Floor Zone',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-112.134, 36.055, -112.084, 36.105],
      height: 0.0,
      extrudedHeight: null,
      rotation: 0.0,
    },
    color: [0.1, 0.9, 0.4, 0.4],
    fill: true,
    outline: true,
    outlineColor: [0.1, 1.0, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: { zone: 'Classification Area' },
  };

  const entities = [polyline1, polyline2, groundRect];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Grand Canyon Trails and Ground Rectangle',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Main Hairpin Trail Focus',
        entities: entities.map((e) => {
          if (e.id === 'terrain-poly-hairpin') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.8, 0.0, 1.0],
              shape: { ...e.shape, data: { ...e.shape.data, width: 8.0 } },
            };
          }
          if (e.id === 'terrain-ground-rect') return { ...e, show: false };
          return { ...e, show: true };
        }),
      },
      {
        label: 'Eastern Loop Trail Focus',
        entities: entities.map((e) => {
          if (e.id === 'terrain-poly-loop') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.9, 0.5, 1.0],
              shape: { ...e.shape, data: { ...e.shape.data, width: 8.0 } },
            };
          }
          if (e.id === 'terrain-ground-rect') return { ...e, show: false };
          return { ...e, show: true };
        }),
      },
      {
        label: 'Composite Terrain Classification',
        entities: entities.map((e) => {
          if (e.id === 'terrain-ground-rect') {
            return {
              ...e,
              show: true,
              color: [0.8, 0.2, 0.9, 0.6],
              outlineColor: [1.0, 0.4, 1.0, 1.0],
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'polylines-on-terrain-dev.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'polylines-on-terrain-dev.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated polylines-on-terrain-dev.json and .variants.json');
}

function generateElevationBandMaterial() {
  // Himalayan Everest region: lon 86.925, lat 27.988
  const baseBand = {
    id: 'elevation-band-base',
    name: 'Himalayan Base Zone (4200m - 7000m)',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.85, 27.92, 87.00, 28.05],
      height: 4200.0,
      extrudedHeight: 7000.0,
      rotation: 0.0,
    },
    color: [0.0, 0.2, 0.5, 0.5],
    fill: true,
    outline: true,
    outlineColor: [0.1, 0.4, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: { elevationStart: 4200, elevationEnd: 7000 },
  };

  const midBand = {
    id: 'elevation-band-mid',
    name: 'Himalayan Mid Band (7000m - 7500m)',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.88, 27.95, 86.97, 28.02],
      height: 7000.0,
      extrudedHeight: 7500.0,
      rotation: 0.0,
    },
    color: [0.1, 0.8, 0.3, 0.6],
    fill: true,
    outline: true,
    outlineColor: [0.2, 1.0, 0.4, 1.0],
    outlineWidth: 1.0,
    properties: { elevationStart: 7000, elevationEnd: 7500 },
  };

  const highBand = {
    id: 'elevation-band-high',
    name: 'Himalayan High Band (7500m - 8000m)',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.90, 27.97, 86.95, 28.00],
      height: 7500.0,
      extrudedHeight: 8000.0,
      rotation: 0.0,
    },
    color: [1.0, 0.8, 0.0, 0.7],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.1, 1.0],
    outlineWidth: 1.0,
    properties: { elevationStart: 7500, elevationEnd: 8000 },
  };

  const summitBand = {
    id: 'elevation-band-summit',
    name: 'Everest Summit Zone (8000m+)',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [86.92, 27.985, 86.935, 27.995],
      height: 8000.0,
      extrudedHeight: 8848.0,
      rotation: 0.0,
    },
    color: [1.0, 0.2, 0.1, 0.8],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { elevationStart: 8000, elevationEnd: 8848 },
  };

  const entities = [baseBand, midBand, highBand, summitBand];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'All Elevation Bands (4200m - 8848m)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'High Altitude Focus (7500m+)',
        entities: entities.map((e) => {
          if (e.id === 'elevation-band-high' || e.id === 'elevation-band-summit') {
            return { ...e, show: true };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Everest Summit Peak Only (8000m - 8848m)',
        entities: entities.map((e) => {
          if (e.id === 'elevation-band-summit') {
            return {
              ...e,
              show: true,
              color: [1.0, 1.0, 1.0, 0.9],
              outlineColor: [1.0, 0.8, 0.0, 1.0],
            };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Gradient Multi-Tiered Visualization',
        entities: entities.map((e) => {
          if (e.id === 'elevation-band-base') {
            return { ...e, show: true, color: [0.1, 0.3, 0.6, 0.6] };
          }
          if (e.id === 'elevation-band-mid') {
            return { ...e, show: true, color: [0.0, 0.9, 0.7, 0.7] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'elevation-band-material.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'elevation-band-material.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated elevation-band-material.json and .variants.json');
}

generateDrawingOnTerrain();
generateRotatable2dMap();
generatePolylinesOnTerrainDev();
generateElevationBandMaterial();
