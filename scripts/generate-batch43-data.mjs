#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 43 recipes:
 * 1. picking (Cartographic position label, billboard hover pick, drill-down 3 polygons, pick & modify)
 * 2. clamp-model-to-ground (CesiumMilkTruck model with clamp/relative/none heightReference modes)
 * 3. camera (San Diego landmark, Mount St. Helens heading/pitch/roll landmark, view rectangle)
 * 4. time-dependent-path-graphics (4-waypoint sampled aircraft path with portions/whole/glow/outline modes)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

function toCartesian(lonDeg, latDeg, altMeters) {
  const phi = (latDeg * Math.PI) / 180.0;
  const lambda = (lonDeg * Math.PI) / 180.0;
  const h = altMeters;
  const a = 6378137.0;
  const f = 1.0 / 298.257223563;
  const e2 = 2.0 * f - f * f;
  const N = a / Math.sqrt(1.0 - e2 * Math.sin(phi) * Math.sin(phi));
  const x = (N + h) * Math.cos(phi) * Math.cos(lambda);
  const y = (N + h) * Math.cos(phi) * Math.sin(lambda);
  const z = (N * (1.0 - e2) + h) * Math.sin(phi);
  return [x, y, z];
}

const logoPng =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

function generatePicking() {
  const labelEntity = {
    id: 'picking-label',
    name: 'Cartographic Position',
    show: true,
    positionDegrees: [-75.59777, 40.03883, 100.0],
    shape: {
      type: 'label',
      appearance: {
        style: 'FILL',
        showBackground: true,
        backgroundColor: [0.1, 0.1, 0.1, 0.7],
        backgroundPadding: [7, 5],
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
      },
      text: 'Lon: -75.60°\nLat:  40.04°',
      fontSize: 14,
      scale: 1.0,
      pixelOffset: [15, 0],
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'cartographicPosition' },
  };

  const billboardEntity = {
    id: 'picking-billboard',
    name: 'Facility Marker',
    show: false,
    positionDegrees: [-75.59777, 40.03883, 0.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 32,
      height: 32,
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'pickEntity' },
  };

  const redPoly = {
    id: 'drill-red',
    name: 'Red Overlay',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-70.0, 30.0, 0.0],
        [-60.0, 30.0, 0.0],
        [-60.0, 40.0, 0.0],
        [-70.0, 40.0, 0.0],
      ],
      height: 0,
      extrudedHeight: null,
    },
    color: [1.0, 0.0, 0.0, 0.5],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'drillPick' },
  };

  const bluePoly = {
    id: 'drill-blue',
    name: 'Blue Overlay',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-75.0, 34.0, 0.0],
        [-63.0, 34.0, 0.0],
        [-63.0, 40.0, 0.0],
        [-75.0, 40.0, 0.0],
      ],
      height: 0,
      extrudedHeight: null,
    },
    color: [0.0, 0.0, 1.0, 0.5],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'drillPick' },
  };

  const greenPoly = {
    id: 'drill-green',
    name: 'Green Overlay',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-67.0, 36.0, 0.0],
        [-55.0, 36.0, 0.0],
        [-55.0, 30.0, 0.0],
        [-67.0, 30.0, 0.0],
      ],
      height: 0,
      extrudedHeight: null,
    },
    color: [0.0, 1.0, 0.0, 0.5],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { feature: 'drillPick' },
  };

  const allEntities = [labelEntity, billboardEntity, redPoly, bluePoly, greenPoly];

  function makeVariant(activeIds, mutator) {
    return allEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      if (mutator) mutator(clone);
      return clone;
    });
  }

  const variants = [
    {
      label: 'Show Position on Mouse Over (Default)',
      entities: makeVariant(['picking-label']),
    },
    {
      label: 'Pick Entity (Billboard Hover)',
      entities: makeVariant(['picking-billboard'], e => {
        if (e.id === 'picking-billboard') {
          e.shape.scale = 2.0;
          e.color = [1.0, 1.0, 0.0, 1.0];
        }
      }),
    },
    {
      label: 'Drill-Down Picking (3 Polygons)',
      entities: makeVariant(['drill-red', 'drill-blue', 'drill-green']),
    },
    {
      label: 'Pick & Modify Entity',
      entities: makeVariant(['picking-billboard'], e => {
        if (e.id === 'picking-billboard') {
          e.shape.scale = 1.5;
          e.color = [0.2, 0.8, 1.0, 1.0];
          e.properties = { feature: 'pickAndModify', modified: true };
        }
      }),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'picking.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'picking.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated picking');
}

function generateClampModelToGround() {
  const url = '../../SampleData/models/CesiumMilkTruck/CesiumMilkTruck.glb';
  const truckLon = -75.5941;
  const truckLat = 40.0389;

  function makeEntities(clampingMode, heightRef, alt, scale) {
    return [
      {
        id: 'clamped-milk-truck',
        name: 'Cesium Milk Truck',
        show: true,
        positionDegrees: [truckLon, truckLat, alt],
        orientation: [-0.048, 0.035, 0.587, 0.807],
        shape: {
          type: 'model',
          uri: url,
          scale: scale,
          minimumPixelSize: 64,
          maximumScale: 2000,
        },
        heightReference: heightRef,
        color: [1.0, 1.0, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: {
          clampingMode: clampingMode,
          heightReference: heightRef,
          scale: scale,
        },
      },
      {
        id: 'truck-reference-pin',
        name: 'Ground Anchor Point',
        show: true,
        positionDegrees: [truckLon, truckLat, 0.0],
        shape: {
          type: 'point',
          pixelSize: 10,
          outlineColor: [1.0, 1.0, 1.0, 1.0],
          outlineWidth: 2,
        },
        color: [0.2, 0.6, 1.0, 1.0],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 0],
        outlineWidth: 1.0,
        properties: { target: 'clamped-milk-truck' },
      },
    ];
  }

  const variants = [
    {
      label: 'Clamp to Ground (Default)',
      entities: makeEntities('Clamp to Ground', 'CLAMP_TO_GROUND', 0.0, 2.5),
    },
    {
      label: 'Clamp to 3D Tiles',
      entities: makeEntities('Clamp to 3D Tiles', 'CLAMP_TO_GROUND', 0.0, 3.0),
    },
    {
      label: 'Relative to Ground (+50m)',
      entities: makeEntities('Relative to Ground', 'RELATIVE_TO_GROUND', 50.0, 2.5),
    },
    {
      label: 'No Clamping (Absolute Height 220m)',
      entities: makeEntities('No Clamping', 'NONE', 220.0, 2.5),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'clamp-model-to-ground.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'clamp-model-to-ground.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated clamp-model-to-ground');
}

function generateCamera() {
  const sanDiego = {
    id: 'san-diego-landmark',
    name: 'San Diego Destination',
    show: true,
    positionDegrees: [-117.16, 32.71, 0.0],
    shape: {
      type: 'point',
      pixelSize: 12,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 2,
    },
    color: [1.0, 0.2, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { mode: 'flyToSanDiego', targetAltitude: 15000.0 },
  };

  const stHelens = {
    id: 'st-helens-landmark',
    name: 'Mount St. Helens Landmark',
    show: false,
    positionDegrees: [-122.22, 46.12, 2549.0],
    shape: {
      type: 'point',
      pixelSize: 12,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 2,
    },
    color: [0.2, 0.8, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { mode: 'flyToHeadingPitchRoll', heading: 20, pitch: -35 },
  };

  const cameraRect = {
    id: 'camera-rectangle',
    name: 'View Rectangle',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-77.0, 38.0, -72.0, 42.0],
      height: 0,
      extrudedHeight: null,
      rotation: 0,
    },
    color: [0.2, 0.6, 1.0, 0.5],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { mode: 'viewRectangle' },
  };

  const baseEntities = [sanDiego, stHelens, cameraRect];

  function makeVariant(activeIds) {
    return baseEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      return clone;
    });
  }

  const variants = [
    { label: 'Fly to San Diego (Default)', entities: makeVariant(['san-diego-landmark']) },
    { label: 'Fly to HeadingPitchRoll', entities: makeVariant(['st-helens-landmark']) },
    { label: 'View Rectangle', entities: makeVariant(['camera-rectangle']) },
    {
      label: 'Camera LookAt (San Diego)',
      entities: makeVariant(['san-diego-landmark', 'st-helens-landmark']),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'camera.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'camera.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated camera');
}

function generateTimeDependentPathGraphics() {
  const waypoints = [
    { time: 0, lon: -70.0, lat: 20.0, alt: 150000.0 },
    { time: 60, lon: -75.0, lat: 15.0, alt: 160000.0 },
    { time: 120, lon: -78.0, lat: 24.0, alt: 140000.0 },
    { time: 180, lon: -83.0, lat: 10.0, alt: 170000.0 },
  ];

  const samples = waypoints.map(w => ({
    time: w.time,
    cartesian: toCartesian(w.lon, w.lat, w.alt),
  }));

  const pathPositions = waypoints.map(w => [w.lon, w.lat, w.alt]);

  function makeEntities(materialMode, polylineMaterial) {
    const pointEntity = {
      id: 'flight-aircraft-point',
      name: 'Flight Trajectory Aircraft',
      show: true,
      positionDegrees: null,
      availability: [0.0, 180.0],
      samples,
      shape: {
        type: 'point',
        pixelSize: 10,
        outlineColor: [1.0, 1.0, 1.0, 1.0],
        outlineWidth: 2,
      },
      color: [1.0, 0.84, 0.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { model: 'flight', materialMode, leadTime: 60, trailTime: 60 },
    };

    const pathEntity = {
      id: 'flight-path-polyline',
      name: 'Flight Path Corridor',
      show: true,
      positionDegrees: null,
      shape: {
        type: 'widePolyline',
        data: {
          positions: pathPositions,
          width: 8.0,
          arcType: 'GEODESIC',
          clampToGround: false,
          material: polylineMaterial,
        },
      },
      color: [0.0, 0.8, 1.0, 1.0],
      fill: true,
      outline: false,
      outlineColor: [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { materialMode },
    };

    return [pointEntity, pathEntity];
  }

  const glowDefault = {
    type: 'glow',
    color: [0.0, 0.8, 1.0, 1.0],
    glowPower: 0.25,
    taperPower: 1.0,
  };

  const glowCyan = {
    type: 'glow',
    color: [0.0, 1.0, 1.0, 1.0],
    glowPower: 0.4,
    taperPower: 1.0,
  };

  const outlineMat = {
    type: 'outline',
    outlineColor: [0.0, 0.0, 0.0, 1.0],
    outlineWidth: 2.0,
  };

  const variants = [
    {
      label: 'Portions Material Mode (Default)',
      entities: makeEntities('PORTIONS', glowDefault),
    },
    {
      label: 'Whole Material Mode',
      entities: makeEntities('WHOLE', glowDefault),
    },
    {
      label: 'Cyan Glow Material',
      entities: makeEntities('PORTIONS', glowCyan),
    },
    {
      label: 'Polyline Outline Material',
      entities: makeEntities('PORTIONS', outlineMat),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'time-dependent-path-graphics.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'time-dependent-path-graphics.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated time-dependent-path-graphics');
}

generatePicking();
generateClampModelToGround();
generateCamera();
generateTimeDependentPathGraphics();
