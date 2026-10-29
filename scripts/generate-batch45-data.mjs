#!/usr/bin/env node
/**
 * Data generator for Sandcastle Batch 45 recipes:
 * 1. clustering (Global science facilities with dynamic clustering badges and pin points)
 * 2. geojson-simplestyle (Simplestyle spec GeoJSON with markers, route polylines, and coverage polygons)
 * 3. geojson-and-topojson (US state polygon vector features with default, basic, extruded, and random colorings)
 * 4. kml (Global facilities placemarks, GDP per capita extruded polygons, and bike ride trajectory)
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const fullDir = path.join(root, 'tests/sandcastle/full');

const logoPng =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

function generateClustering() {
  const facility1 = {
    id: 'cluster-cern',
    name: 'CERN LHC',
    show: true,
    positionDegrees: [6.05, 46.23, 100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.0, 0.8, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { facility: 'CERN', country: 'CH' },
  };

  const facility2 = {
    id: 'cluster-fermilab',
    name: 'Fermilab',
    show: true,
    positionDegrees: [-88.25, 41.84, 100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [1.0, 0.6, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { facility: 'Fermilab', country: 'US' },
  };

  const facility3 = {
    id: 'cluster-kek',
    name: 'KEK Tsukuba',
    show: true,
    positionDegrees: [140.07, 36.15, 100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.2, 0.9, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { facility: 'KEK', country: 'JP' },
  };

  const facility4 = {
    id: 'cluster-desy',
    name: 'DESY Hamburg',
    show: true,
    positionDegrees: [9.88, 53.58, 100.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 28,
      height: 28,
    },
    color: [0.9, 0.3, 0.8, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { facility: 'DESY', country: 'DE' },
  };

  const clusterBadge = {
    id: 'cluster-badge-eu',
    name: 'Europe Cluster Badge (2)',
    show: true,
    positionDegrees: [7.96, 49.90, 500.0],
    shape: {
      type: 'label',
      appearance: {
        style: 'FILL',
        showBackground: true,
        backgroundColor: [0.1, 0.5, 0.9, 0.85],
        backgroundPadding: [10, 6],
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
      },
      text: '2 Facilities',
      fontSize: 14,
      scale: 1.0,
      pixelOffset: [0, -10],
    },
    color: [1.0, 1.0, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { clusterCount: 2, region: 'Europe' },
  };

  const allEntities = [facility1, facility2, facility3, facility4, clusterBadge];

  function makeVariant(activeIds) {
    return allEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      return clone;
    });
  }

  const defaultIds = ['cluster-cern', 'cluster-fermilab', 'cluster-kek', 'cluster-desy', 'cluster-badge-eu'];
  const rawIds = ['cluster-cern', 'cluster-fermilab', 'cluster-kek', 'cluster-desy'];

  const variants = [
    {
      label: 'Clustering Enabled (Default)',
      entities: makeVariant(defaultIds),
    },
    {
      label: 'High Density Clustering',
      entities: makeVariant(defaultIds),
    },
    {
      label: 'Unclustered Raw Facilities',
      entities: makeVariant(rawIds),
    },
    {
      label: 'Custom Pin Badges',
      entities: makeVariant(defaultIds),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'clustering.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'clustering.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated clustering');
}

function generateGeojsonSimplestyle() {
  const marker = {
    id: 'simplestyle-marker-sf',
    name: 'San Francisco Public Library',
    show: true,
    positionDegrees: [-122.416, 37.779, 10.0],
    shape: {
      type: 'billboard',
      image: logoPng,
      scale: 1.0,
      width: 32,
      height: 32,
    },
    color: [0.2, 0.6, 1.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { 'marker-symbol': 'library', 'marker-color': '#3399ff' },
  };

  const routePolyline = {
    id: 'simplestyle-corridor-route',
    name: 'Market Street Transit Corridor',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.420, 37.773, 5.0],
          [-122.415, 37.778, 5.0],
          [-122.408, 37.785, 5.0],
          [-122.398, 37.794, 5.0],
        ],
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: { type: 'solid' },
      },
    },
    color: [1.0, 0.4, 0.0, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { stroke: '#ff6600', 'stroke-width': 4 },
  };

  const districtPolygon = {
    id: 'simplestyle-civic-center-poly',
    name: 'Civic Center District',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-122.422, 37.776, 0.0],
        [-122.414, 37.776, 0.0],
        [-122.414, 37.782, 0.0],
        [-122.422, 37.782, 0.0],
      ],
      height: 0,
      extrudedHeight: null,
    },
    color: [0.1, 0.8, 0.4, 0.45],
    fill: true,
    outline: true,
    outlineColor: [0.0, 0.5, 0.2, 1.0],
    outlineWidth: 1.0,
    properties: { fill: '#19cc66', 'fill-opacity': 0.45, stroke: '#008033' },
  };

  const allEntities = [marker, routePolyline, districtPolygon];

  function makeVariant(activeIds) {
    return allEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      return clone;
    });
  }

  const allIds = ['simplestyle-marker-sf', 'simplestyle-corridor-route', 'simplestyle-civic-center-poly'];
  const polyIds = ['simplestyle-corridor-route', 'simplestyle-civic-center-poly'];
  const markerIds = ['simplestyle-marker-sf'];

  const variants = [
    {
      label: 'Default Simplestyle (Markers, Polygons, Lines)',
      entities: makeVariant(allIds),
    },
    {
      label: 'Polygons and Routes Highlighted',
      entities: makeVariant(polyIds),
    },
    {
      label: 'Markers Filtered',
      entities: makeVariant(markerIds),
    },
    {
      label: 'SCENE3D Perspective Mode',
      entities: makeVariant(allIds),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'geojson-simplestyle.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'geojson-simplestyle.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated geojson-simplestyle');
}

function generateGeojsonAndTopojson() {
  const calPoly = [
    [-124.4, 42.0, 0.0],
    [-120.0, 42.0, 0.0],
    [-120.0, 39.0, 0.0],
    [-114.6, 35.0, 0.0],
    [-117.1, 32.5, 0.0],
    [-124.4, 40.0, 0.0],
  ];

  const texPoly = [
    [-106.6, 32.0, 0.0],
    [-103.0, 32.0, 0.0],
    [-103.0, 36.5, 0.0],
    [-100.0, 36.5, 0.0],
    [-94.0, 33.5, 0.0],
    [-94.0, 29.7, 0.0],
    [-97.1, 26.0, 0.0],
    [-106.6, 32.0, 0.0],
  ];

  const nyPoly = [
    [-79.8, 42.0, 0.0],
    [-73.5, 42.0, 0.0],
    [-73.5, 45.0, 0.0],
    [-75.0, 45.0, 0.0],
    [-79.8, 42.0, 0.0],
  ];

  const flPoly = [
    [-87.5, 30.5, 0.0],
    [-85.0, 30.5, 0.0],
    [-80.0, 27.0, 0.0],
    [-80.5, 25.0, 0.0],
    [-82.5, 28.0, 0.0],
    [-87.5, 30.5, 0.0],
  ];

  function makeStateEntity(id, name, poly, color, outlineColor, height, extrudedHeight) {
    return {
      id,
      name,
      show: true,
      positionDegrees: null,
      shape: {
        type: 'polygon',
        positionsDegrees: poly,
        height: height || 0,
        extrudedHeight: extrudedHeight || null,
      },
      color,
      fill: true,
      outline: outlineColor !== null,
      outlineColor: outlineColor || [0, 0, 0, 0],
      outlineWidth: 1.0,
      properties: { state: name },
    };
  }

  // Variant 0: Default styling (blue-gray fill, white outline)
  const v0 = [
    makeStateEntity('state-california', 'California', calPoly, [0.3, 0.5, 0.7, 0.6], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-texas', 'Texas', texPoly, [0.3, 0.5, 0.7, 0.6], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-new-york', 'New York', nyPoly, [0.3, 0.5, 0.7, 0.6], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-florida', 'Florida', flPoly, [0.3, 0.5, 0.7, 0.6], [1, 1, 1, 1], 0, null),
  ];

  // Variant 1: Basic styling (hotpink outline, pink 0.5 fill)
  const v1 = [
    makeStateEntity('state-california', 'California', calPoly, [1.0, 0.75, 0.8, 0.5], [1.0, 0.1, 0.6, 1.0], 0, null),
    makeStateEntity('state-texas', 'Texas', texPoly, [1.0, 0.75, 0.8, 0.5], [1.0, 0.1, 0.6, 1.0], 0, null),
    makeStateEntity('state-new-york', 'New York', nyPoly, [1.0, 0.75, 0.8, 0.5], [1.0, 0.1, 0.6, 1.0], 0, null),
    makeStateEntity('state-florida', 'Florida', flPoly, [1.0, 0.75, 0.8, 0.5], [1.0, 0.1, 0.6, 1.0], 0, null),
  ];

  // Variant 2: Custom styling (extruded by population)
  const v2 = [
    makeStateEntity('state-california', 'California', calPoly, [0.2, 0.7, 0.3, 0.8], null, 0, 100000.0),
    makeStateEntity('state-texas', 'Texas', texPoly, [0.9, 0.6, 0.1, 0.8], null, 0, 80000.0),
    makeStateEntity('state-new-york', 'New York', nyPoly, [0.2, 0.5, 0.9, 0.8], null, 0, 60000.0),
    makeStateEntity('state-florida', 'Florida', flPoly, [0.8, 0.2, 0.5, 0.8], null, 0, 55000.0),
  ];

  // Variant 3: Random colors
  const v3 = [
    makeStateEntity('state-california', 'California', calPoly, [0.85, 0.25, 0.25, 0.75], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-texas', 'Texas', texPoly, [0.25, 0.75, 0.85, 0.75], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-new-york', 'New York', nyPoly, [0.85, 0.75, 0.25, 0.75], [1, 1, 1, 1], 0, null),
    makeStateEntity('state-florida', 'Florida', flPoly, [0.35, 0.85, 0.35, 0.75], [1, 1, 1, 1], 0, null),
  ];

  const variants = [
    { label: 'Default styling', entities: v0 },
    { label: 'Basic styling', entities: v1 },
    { label: 'Custom styling', entities: v2 },
    { label: 'Random colors', entities: v3 },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'geojson-and-topojson.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'geojson-and-topojson.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated geojson-and-topojson');
}

function generateKml() {
  const facilityPin = {
    id: 'kml-facility-pin',
    name: 'Global Facility Placemark',
    show: true,
    positionDegrees: [6.05, 46.23, 200.0],
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
    properties: { kmlType: 'placemark' },
  };

  const gdpPolygon = {
    id: 'kml-gdp-polygon',
    name: 'GDP per capita 2008',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'polygon',
      positionsDegrees: [
        [-5.0, 42.0, 0.0],
        [8.0, 42.0, 0.0],
        [8.0, 51.0, 0.0],
        [-5.0, 51.0, 0.0],
      ],
      height: 0,
      extrudedHeight: 45000.0,
    },
    color: [0.2, 0.6, 0.9, 0.6],
    fill: true,
    outline: true,
    outlineColor: [0.1, 0.3, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { kmlType: 'gdp' },
  };

  const bikeRider = {
    id: 'kml-bike-rider',
    name: 'Tour Rider',
    show: false,
    positionDegrees: [-122.42, 37.77, 50.0],
    shape: {
      type: 'point',
      pixelSize: 10,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
    },
    color: [1.0, 0.2, 0.2, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { kmlType: 'tour' },
  };

  const bikeTrack = {
    id: 'kml-bike-track',
    name: 'Bike Ride Track',
    show: false,
    positionDegrees: null,
    shape: {
      type: 'widePolyline',
      data: {
        positions: [
          [-122.43, 37.76, 30.0],
          [-122.42, 37.77, 50.0],
          [-122.41, 37.78, 40.0],
          [-122.40, 37.79, 20.0],
        ],
        width: 4.0,
        arcType: 'GEODESIC',
        clampToGround: false,
        material: { type: 'solid' },
      },
    },
    color: [0.0, 0.9, 0.3, 1.0],
    fill: true,
    outline: false,
    outlineColor: [0, 0, 0, 0],
    outlineWidth: 1.0,
    properties: { kmlType: 'tour' },
  };

  const allEntities = [facilityPin, gdpPolygon, bikeRider, bikeTrack];

  function makeVariant(activeIds) {
    return allEntities.map(e => {
      const clone = structuredClone(e);
      clone.show = activeIds.includes(clone.id);
      return clone;
    });
  }

  const facIds = ['kml-facility-pin'];
  const gdpIds = ['kml-gdp-polygon'];
  const bikeIds = ['kml-bike-rider', 'kml-bike-track'];
  const allIds = ['kml-facility-pin', 'kml-gdp-polygon', 'kml-bike-rider', 'kml-bike-track'];

  const variants = [
    {
      label: 'KML - Global Science Facilities',
      entities: makeVariant(facIds),
    },
    {
      label: 'KMZ with embedded data - GDP per capita',
      entities: makeVariant(gdpIds),
    },
    {
      label: 'gx KML extensions - Bike Ride',
      entities: makeVariant(bikeIds),
    },
    {
      label: 'Simple Placemarks and Paths',
      entities: makeVariant(allIds),
    },
  ];

  fs.writeFileSync(
    path.join(fullDir, 'kml.json'),
    JSON.stringify(variants[0].entities, null, 2),
  );
  fs.writeFileSync(
    path.join(fullDir, 'kml.variants.json'),
    JSON.stringify({ initial: 0, variants }, null, 2),
  );
  console.log('Generated kml');
}

generateClustering();
generateGeojsonSimplestyle();
generateGeojsonAndTopojson();
generateKml();
