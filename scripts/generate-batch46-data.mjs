#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 46 recipes:
 * 1. gpx (GPS Exchange format with tracks, routes, waypoints, and symbology)
 * 2. kml-tours (KML Tours with Eiffel Tower camera trajectory and tour stops)
 * 3. export-kml (Export KML/KMZ with satellites, orbit lines, and milk truck model)
 * 4. ground-polyline-material-dev (Ground polylines on terrain with glow, dash, and outline materials)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

const logoPng =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

function generateGpx() {
  const trackMain = {
    id: 'gpx-track-main',
    name: 'Boulder Mountain Trail',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-105.28, 40.01, 1600.0],
          [-105.27, 40.02, 1750.0],
          [-105.25, 40.03, 1950.0],
          [-105.23, 40.04, 2100.0],
        ],
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: { type: 'solid' },
      },
    },
    color: [0.0, 0.9, 0.4, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'track', name: 'Boulder Trail' },
  };

  const wptStart = {
    id: 'gpx-wpt-start',
    name: 'Trailhead Waypoint',
    show: true,
    positionDegrees: [-105.28, 40.01, 1600.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 0.6, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'waypoint', desc: 'Trailhead' },
  };

  const wptSummit = {
    id: 'gpx-wpt-summit',
    name: 'Summit Waypoint',
    show: true,
    positionDegrees: [-105.25, 40.03, 1950.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [1.0, 0.3, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'waypoint', desc: 'Summit' },
  };

  const wptEnd = {
    id: 'gpx-wpt-end',
    name: 'Overlook Waypoint',
    show: true,
    positionDegrees: [-105.23, 40.04, 2100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 1.0, 0.4, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'waypoint', desc: 'Overlook' },
  };

  const trackDescent = {
    id: 'gpx-track-descent',
    name: 'Descent Ridge Trail',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-105.23, 40.04, 2100.0],
          [-105.22, 40.02, 1800.0],
          [-105.21, 40.00, 1600.0],
        ],
        width: 4.0,
        arcType: 'GEODESIC',
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
    color: [1.0, 0.8, 0.1, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { type: 'track', name: 'Descent Ridge' },
  };

  const entities = [trackMain, wptStart, wptSummit, wptEnd, trackDescent];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Track with Waypoints',
        entities: entities.map((e) => ({
          ...e,
          show: e.id !== 'gpx-track-descent',
        })),
      },
      {
        label: 'Route Navigation',
        entities: entities.map((e) => {
          if (e.id === 'gpx-track-main' || e.id === 'gpx-track-descent') {
            return { ...e, show: true };
          }
          if (e.id === 'gpx-wpt-start' || e.id === 'gpx-wpt-end') {
            return { ...e, show: true };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Waypoints Only',
        entities: entities.map((e) => {
          if (e.id.startsWith('gpx-wpt-')) return { ...e, show: true };
          return { ...e, show: false };
        }),
      },
      {
        label: 'Multiple Tracks & Symbology',
        entities: entities.map((e) => {
          if (e.id === 'gpx-track-main') {
            return {
              ...e,
              show: true,
              color: [0.95, 0.85, 0.05, 1.0],
              shape: {
                ...e.shape,
                data: { ...e.shape.data, width: 5.0 },
              },
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(fullDir, 'gpx.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(fullDir, 'gpx.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated gpx.json and gpx.variants.json');
}

function generateKmlTours() {
  const eiffel = {
    id: 'kml-tour-eiffel',
    name: 'Eiffel Tower Monument',
    show: true,
    positionDegrees: [2.2945, 48.8584, 300.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.2,
      width: 32,
      height: 32,
    },
    color: [1.0, 0.85, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'Eiffel Tower', city: 'Paris' },
  };

  const tourPath = {
    id: 'kml-tour-path',
    name: 'Tour Aerial Flight Path',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [2.2900, 48.8540, 200.0],
          [2.2945, 48.8584, 350.0],
          [2.2980, 48.8620, 250.0],
          [2.3020, 48.8650, 150.0],
        ],
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: false,
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
    properties: { tour: 'Eiffel FlyTo', duration: 45 },
  };

  const stopTrocadero = {
    id: 'kml-tour-stop-trocadero',
    name: 'Tour Stop 1 (Trocadero Terrace)',
    show: true,
    positionDegrees: [2.2900, 48.8540, 200.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 0.6, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { stop: 1, name: 'Trocadero' },
  };

  const stopSummit = {
    id: 'kml-tour-stop-summit',
    name: 'Tour Stop 2 (Summit View)',
    show: true,
    positionDegrees: [2.2945, 48.8584, 350.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [1.0, 0.3, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { stop: 2, name: 'Summit Vantage' },
  };

  const entities = [eiffel, tourPath, stopTrocadero, stopSummit];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Initial Tour Overview',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Tour Stop 1 (Trocadero Viewpoint)',
        entities: entities.map((e) => {
          if (e.id === 'kml-tour-stop-trocadero') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.9, 1.0, 1.0],
              shape: { ...e.shape, scale: 1.3 },
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Tour Stop 2 (Summit Orbit)',
        entities: entities.map((e) => {
          if (e.id === 'kml-tour-stop-summit') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.8, 0.0, 1.0],
              shape: { ...e.shape, scale: 1.3 },
            };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Complete Tour Experience',
        entities: entities.map((e) => {
          if (e.id === 'kml-tour-path') {
            return {
              ...e,
              show: true,
              color: [0.0, 1.0, 0.5, 1.0],
              shape: {
                ...e.shape,
                data: { ...e.shape.data, width: 6.0 },
              },
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(fullDir, 'kml-tours.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(fullDir, 'kml-tours.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated kml-tours.json and kml-tours.variants.json');
}

function generateExportKml() {
  const satGeoeye1 = {
    id: 'export-kml-sat-geoeye1',
    name: 'GeoEye-1 Satellite',
    show: true,
    positionDegrees: [-77.0, 38.9, 500000.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 0.8, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { platform: 'GeoEye-1', orbitType: 'LEO' },
  };

  const satOrbit = {
    id: 'export-kml-sat-orbit',
    name: 'GeoEye-1 Orbital Track',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-85.0, 35.0, 500000.0],
          [-77.0, 38.9, 500000.0],
          [-68.0, 42.0, 500000.0],
        ],
        width: 3.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        colors: [],
        colorsPerVertex: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 0.8, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { orbit: 'LEO Trajectory' },
  };

  const groundStation = {
    id: 'export-kml-ground-station',
    name: 'AGI Space Tracking Station',
    show: true,
    positionDegrees: [-75.6, 40.0, 100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 1.0, 0.4, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { facility: 'AGI Tracking', type: 'Earth Station' },
  };

  const milkTruck = {
    id: 'export-kml-milktruck',
    name: 'Cesium Milk Truck Model',
    show: true,
    positionDegrees: [-77.0, 37.0, 10.0],
    shape: {
      type: 'model',
      uri: 'models/CesiumMilkTruck/CesiumMilkTruck.glb',
      scale: 2.0,
      minimumPixelSize: 32,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { model: 'CesiumMilkTruck', cargo: 'Milk' },
  };

  const entities = [satGeoeye1, satOrbit, groundStation, milkTruck];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Satellites and Ground Assets',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Model Asset (Cesium Milk Truck)',
        entities: entities.map((e) => {
          if (e.id === 'export-kml-milktruck' || e.id === 'export-kml-ground-station') {
            return { ...e, show: true };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Space Assets (Satellites & Orbits)',
        entities: entities.map((e) => {
          if (e.id === 'export-kml-sat-geoeye1' || e.id === 'export-kml-sat-orbit') {
            return { ...e, show: true };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Full Combined KMZ Package',
        entities: entities.map((e) => {
          if (e.id === 'export-kml-sat-orbit') {
            return {
              ...e,
              show: true,
              color: [0.1, 0.9, 1.0, 1.0],
              shape: { ...e.shape, data: { ...e.shape.data, width: 5.0 } },
            };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(fullDir, 'export-kml.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(fullDir, 'export-kml.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated export-kml.json and export-kml.variants.json');
}

function generateGroundPolylineMaterialDev() {
  const polyGlow = {
    id: 'ground-poly-glow',
    name: 'Ground Polyline Glow Material',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.2558, 46.1955, 0.0],
          [-122.1058, 46.1955, 0.0],
        ],
        width: 10.0,
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
    properties: { materialType: 'PolylineGlow', location: 'Mount St. Helens' },
  };

  const polyDash = {
    id: 'ground-poly-dash',
    name: 'Ground Polyline Dash Material',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.2558, 46.1975, 0.0],
          [-122.1058, 46.1975, 0.0],
        ],
        width: 10.0,
        arcType: 'GEODESIC',
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
    color: [1.0, 0.9, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { materialType: 'PolylineDash', location: 'Mount St. Helens' },
  };

  const polyOutline = {
    id: 'ground-poly-outline',
    name: 'Ground Polyline Outline Material',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.2558, 46.1995, 0.0],
          [-122.1058, 46.1995, 0.0],
        ],
        width: 10.0,
        arcType: 'GEODESIC',
        clampToGround: true,
        colors: [],
        colorsPerVertex: false,
        material: {
          type: 'outline',
          outlineColor: [1.0, 1.0, 1.0, 1.0],
          outlineWidth: 2.0,
        },
      },
    },
    color: [1.0, 0.3, 0.1, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { materialType: 'PolylineOutline', location: 'Mount St. Helens' },
  };

  const entities = [polyGlow, polyDash, polyOutline];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'All Three Ground Polyline Materials',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Polyline Glow Focus',
        entities: entities.map((e) => {
          if (e.id === 'ground-poly-glow') {
            return {
              ...e,
              show: true,
              color: [0.0, 1.0, 0.8, 1.0],
              shape: {
                ...e.shape,
                data: { ...e.shape.data, width: 14.0 },
              },
            };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Polyline Dash Focus',
        entities: entities.map((e) => {
          if (e.id === 'ground-poly-dash') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.8, 0.0, 1.0],
              shape: {
                ...e.shape,
                data: { ...e.shape.data, width: 14.0 },
              },
            };
          }
          return { ...e, show: false };
        }),
      },
      {
        label: 'Polyline Outline Focus',
        entities: entities.map((e) => {
          if (e.id === 'ground-poly-outline') {
            return {
              ...e,
              show: true,
              color: [1.0, 0.2, 0.2, 1.0],
              shape: {
                ...e.shape,
                data: { ...e.shape.data, width: 14.0 },
              },
            };
          }
          return { ...e, show: false };
        }),
      },
    ],
  };

  fs.writeFileSync(
    path.join(fullDir, 'ground-polyline-material-dev.json'),
    JSON.stringify(entities, null, 2)
  );
  fs.writeFileSync(
    path.join(fullDir, 'ground-polyline-material-dev.variants.json'),
    JSON.stringify(variants, null, 2)
  );
  console.log('generated ground-polyline-material-dev.json and .variants.json');
}

generateGpx();
generateKmlTours();
generateExportKml();
generateGroundPolylineMaterialDev();
