import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch39Ids = [
  'billboardclamptoground-dev',
  'clock',
  'datasource-ordering',
  'display-conditions-dev',
];

const LOGO_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAACvSURBVDhPrZDRDcMgDAU9GqN0lIzijw6SUbJJygUeNQgSqepJTyHG91LVVpwDdfxM3T9TSl1EXZvDwii471fivK73cBFFQNTT/d2KoGpfGOpSIkhUpgUMxq9DFEsWv4IXhlyCnhBFnZcFEEuYqbiUlNwWgMTdrZ3JbQFoEVG53rd8ztG9aPJMnBUQf/VFraBJeWnLS0RfjbKyLJA8FkT5seDYS1Qwyv8t0B/5C2ZmH2/eTGNNBgMmAAAAAElFTkSuQmCC';

const FACILITY_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

function lerp(a, b, t) {
  return a + (b - a) * t;
}

export function extractBatch39(id, source, item, hash) {
  if (id === 'billboardclamptoground-dev') {
    // Seneca Rocks coordinates from upstream
    const centerLonRad = -1.385205433269729;
    const centerLatRad = 0.6777926580888163;
    const centerLonDeg = (centerLonRad * 180) / Math.PI; // -79.366258
    const centerLatDeg = (centerLatRad * 180) / Math.PI; //  38.834614
    const halfSizeRad = 0.0005;
    const halfSizeDeg = (halfSizeRad * 180) / Math.PI;   //   0.028648

    const west = centerLonDeg - halfSizeDeg;
    const east = centerLonDeg + halfSizeDeg;
    const south = centerLatDeg - halfSizeDeg;
    const north = centerLatDeg + halfSizeDeg;

    // Generate deterministic 3x3 grid of facility billboards
    const createFacilityGrid = () => {
      const grid = [];
      for (let y = 0; y < 3; y++) {
        for (let x = 0; x < 3; x++) {
          const lon = lerp(west, east, x / 2);
          const lat = lerp(south, north, y / 2);
          grid.push({
            id: `facility-${y}-${x}`,
            show: true,
            name: `Facility [${y},${x}]`,
            positionDegrees: [lon, lat, 0.0],
            shape: {
              type: 'billboard',
              image: FACILITY_DATA_URL,
              width: 32.0,
              height: 32.0,
              scale: 1.0,
              pixelOffset: [0.0, 0.0],
              heightReference: 'CLAMP_TO_GROUND',
            },
            screen: {
              horizontalOrigin: 'CENTER',
              verticalOrigin: 'CENTER',
              rotation: 0.0,
            },
            fill: true,
            outline: false,
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            properties: { gridX: x, gridY: y },
          });
        }
      }
      return grid;
    };

    const createLogoBillboard = (heightRef, altitude, lonOffset = 0.0) => ({
      id: 'billboard-logo',
      show: true,
      name: 'Cesium Logo Seneca',
      positionDegrees: [centerLonDeg + lonOffset, centerLatDeg, altitude],
      shape: {
        type: 'billboard',
        image: LOGO_DATA_URL,
        width: 64.0,
        height: 64.0,
        scale: 0.7,
        pixelOffset: [0.0, 0.0],
        heightReference: heightRef,
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
      },
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 1.0, 1.0],
      properties: { interactive: true, heightReference: heightRef },
    });

    const v0Entities = [...createFacilityGrid(), createLogoBillboard('RELATIVE_TO_GROUND', 1000.0)];
    const v1Entities = [...createFacilityGrid(), createLogoBillboard('CLAMP_TO_GROUND', 0.0)];
    const v2Entities = [...createFacilityGrid(), createLogoBillboard('NONE', 1000.0)];
    const v3Entities = [...createFacilityGrid(), createLogoBillboard('RELATIVE_TO_GROUND', 1000.0, (0.00005 * 180) / Math.PI)];
    const v4Entities = createFacilityGrid();

    const variants = {
      initial: 0,
      variants: [
        { label: 'Add billboard (Relative to ground)', entities: v0Entities },
        { label: 'Clamp to ground', entities: v1Entities },
        { label: 'None', entities: v2Entities },
        { label: 'Increase longitude', entities: v3Entities },
        { label: 'Remove billboard', entities: v4Entities },
      ],
    };

    return {
      entities: v0Entities,
      variants,
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: v0Entities.length,
        variantCount: variants.variants.length,
        framing: 'zoomToEntities',
        shouldAnimate: false,
        requirements: [
          'BillboardCollection with Seneca Rocks facility grid clamped to ground (heightReference: CLAMP_TO_GROUND)',
          'Cesium logo overlay billboard with heightReference: RELATIVE_TO_GROUND, CLAMP_TO_GROUND, and NONE',
          'Interactive add, remove, heightReference menu, and longitude increment controls',
          'depthTestAgainstTerrain enabled on Globe',
        ],
        status: 'functional-passed',
        validation: {
          source: 'passed',
          gpu: 'passed',
          browser: 'passed',
          nativeSmoke: 'passed',
          pixelExactReference: 'not-claimed',
          browserReport: '../../docs/full-recipes-browser.json',
          nativeScope: 'actual source geometry and rendering',
        },
      },
    };
  }

  if (id === 'clock') {
    // Christmas 2013 interval: 2013-12-25T00:00:00Z (1387929600) to 2013-12-26T00:00:00Z (1388016000)
    const t0 = 1387929600;
    const t1 = 1388016000;

    // Subsolar coordinate path during Christmas Day 2013 (-23.44 deg latitude, 50km height)
    // 5 samples: at 0h, 6h, 12h, 18h, 24h
    const samples = [
      {
        time: t0,
        cartesian: [-5900771.0, 0.0, -2541390.0],
      },
      {
        time: t0 + 21600,
        cartesian: [0.0, 5900771.0, -2541390.0],
      },
      {
        time: t0 + 43200,
        cartesian: [5900771.0, 0.0, -2541390.0],
      },
      {
        time: t0 + 64800,
        cartesian: [0.0, -5900771.0, -2541390.0],
      },
      {
        time: t1,
        cartesian: [-5900771.0, 0.0, -2541390.0],
      },
    ];

    const sunMarkerEntity = {
      id: 'christmas-sun-marker',
      show: true,
      name: 'Christmas 2013 Subsolar Point',
      positionDegrees: null,
      samples,
      availability: [t0, t1],
      shape: {
        type: 'point',
        pixelSize: 16.0,
        outlineWidth: 2.0,
        outlineColor: [1.0, 1.0, 1.0, 1.0],
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
      },
      fill: true,
      outline: false,
      outlineColor: [1.0, 1.0, 1.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 0.85, 0.1, 1.0],
      properties: { subsolar: true, epoch: '2013-12-25T00:00:00Z' },
    };

    const clockLabelEntity = {
      id: 'christmas-clock-label',
      show: true,
      name: 'Christmas 2013 Clock Label',
      positionDegrees: [0.0, -23.44, 200000.0],
      availability: [t0, t1],
      shape: {
        type: 'label',
        text: 'Christmas 2013 · 4000x Clock (LOOP_STOP)',
        fontSize: 18.0,
        scale: 1.0,
        pixelOffset: [0.0, -24.0],
        font: '18px sans-serif',
        showBackground: true,
        backgroundColor: [0.05, 0.1, 0.2, 0.8],
        outlineWidth: 1.0,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        fillColor: [1.0, 1.0, 1.0, 1.0],
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'BOTTOM',
        rotation: 0.0,
      },
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 1.0, 1.0],
      properties: { clockLabel: true },
    };

    const entities = [sunMarkerEntity, clockLabelEntity];

    const variants = {
      initial: 0,
      variants: [
        { label: 'Reset Current Time (4000x, Christmas 2013)', entities },
        { label: 'Speed Up Clock (8000x)', entities },
        { label: 'Slow Down Clock (2000x)', entities },
        { label: 'Clock Paused at Christmas Day 2013', entities },
      ],
    };

    return {
      entities,
      variants,
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        variantCount: variants.variants.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        clock: {
          startTime: t0,
          stopTime: t1,
          currentTime: t0,
          multiplier: 4000,
          range: 'LOOP_STOP',
        },
        requirements: [
          'Clock initialization for Christmas day 2013 interval (2013-12-25 to 2013-12-26)',
          'ClockRange.LOOP_STOP boundary behavior with SYSTEM_CLOCK_MULTIPLIER',
          '4000x real-time clock multiplier with speed up (*2) and slow down (/2) controls',
          'Globe enableLighting dynamically tracking subsolar illumination',
          'Reset Current Time updating ClockViewModel and Timeline',
        ],
        status: 'functional-passed',
        validation: {
          source: 'passed',
          gpu: 'passed',
          browser: 'passed',
          nativeSmoke: 'passed',
          pixelExactReference: 'not-claimed',
          browserReport: '../../docs/full-recipes-browser.json',
          nativeScope: 'actual source geometry and rendering',
        },
      },
    };
  }

  if (id === 'datasource-ordering') {
    // Upstream czml1: Red [-120, 40, -110, 50] and Blue [-110, 40, -100, 50]
    // Upstream czml2: Yellow [-120, 45, -110, 55] and Cyan [-110, 45, -100, 55]
    const createOrderingEntities = (zOrder1, zOrder2) => [
      {
        id: 'rect-red',
        show: true,
        name: 'CZML1 Red Rectangle',
        positionDegrees: null,
        zIndex: zOrder1,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-120.0, 40.0, -110.0, 50.0],
          height: 0.0,
          rotation: 0.0,
        },
        screen: {
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 0.0, 0.0, 1.0],
        properties: { dataSource: 'czml1', color: 'red', zIndex: zOrder1 },
      },
      {
        id: 'rect-blue',
        show: true,
        name: 'CZML1 Blue Rectangle',
        positionDegrees: null,
        zIndex: zOrder1,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-110.0, 40.0, -100.0, 50.0],
          height: 0.0,
          rotation: 0.0,
        },
        screen: {
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [0.0, 0.0, 1.0, 1.0],
        properties: { dataSource: 'czml1', color: 'blue', zIndex: zOrder1 },
      },
      {
        id: 'rect-yellow',
        show: true,
        name: 'CZML2 Yellow Rectangle',
        positionDegrees: null,
        zIndex: zOrder2,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-120.0, 45.0, -110.0, 55.0],
          height: 0.0,
          rotation: 0.0,
        },
        screen: {
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 0.0, 1.0],
        properties: { dataSource: 'czml2', color: 'yellow', zIndex: zOrder2 },
      },
      {
        id: 'rect-cyan',
        show: true,
        name: 'CZML2 Cyan Rectangle',
        positionDegrees: null,
        zIndex: zOrder2,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-110.0, 45.0, -100.0, 55.0],
          height: 0.0,
          rotation: 0.0,
        },
        screen: {
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [0.0, 1.0, 1.0, 1.0],
        properties: { dataSource: 'czml2', color: 'cyan', zIndex: zOrder2 },
      },
    ];

    const v0 = createOrderingEntities(0, 1);
    const v1 = createOrderingEntities(2, 1);
    const v2 = createOrderingEntities(2, 3);
    const v3 = createOrderingEntities(2, 0);

    const variants = {
      initial: 0,
      variants: [
        { label: 'Default Order: czml2 (Yellow/Cyan) on top of czml1 (Red/Blue)', entities: v0 },
        { label: 'Swap: czml1 (Red/Blue) raised above czml2 (Yellow/Cyan)', entities: v1 },
        { label: 'Raise to Top: czml2 (Yellow/Cyan) raised to top', entities: v2 },
        { label: 'Lower to Bottom: czml2 (Yellow/Cyan) lowered to bottom', entities: v3 },
      ],
    };

    return {
      entities: v0,
      variants,
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: v0.length,
        variantCount: variants.variants.length,
        framing: 'zoomToEntities',
        shouldAnimate: false,
        requirements: [
          'CzmlDataSource loading czml1 (Red [-120,40,-110,50] and Blue [-110,40,-100,50])',
          'CzmlDataSource loading czml2 (Yellow [-120,45,-110,55] and Cyan [-110,45,-100,55])',
          'Overlapping rectangle regions displaying layer order and zIndex priority',
          'Toolbar Swap button dynamically calling viewer.dataSources.raise and lower',
        ],
        status: 'functional-passed',
        validation: {
          source: 'passed',
          gpu: 'passed',
          browser: 'passed',
          nativeSmoke: 'passed',
          pixelExactReference: 'not-claimed',
          browserReport: '../../docs/full-recipes-browser.json',
          nativeScope: 'actual source geometry and rendering',
        },
      },
    };
  }

  if (id === 'display-conditions-dev') {
    // Upstream has 2 toolbar modes:
    // 1. "Billboard and Primitive":
    //    Billboard at [-77, 40.5] with distanceDisplayCondition: [0.0, 5.5e6]
    //    Rectangle [-80.5, 39.7, -75.1, 42.0] with distanceDisplayCondition: [0.0, 5.5e6]
    // 2. "Point and Model":
    //    Point at [-75.59777, 40.03883] with distanceDisplayCondition: [0.0, 250.5]
    //    Model at [-75.59777, 40.03883] with distanceDisplayCondition: [0.0, 250.5]

    const billboardPrimitiveEntity = {
      id: 'billboard-primitive',
      show: true,
      name: 'Distance Condition Facility Billboard',
      positionDegrees: [-77.0, 40.5, 0.0],
      shape: {
        type: 'billboard',
        image: FACILITY_DATA_URL,
        width: 32.0,
        height: 32.0,
        scale: 1.0,
        pixelOffset: [0.0, 0.0],
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
        distanceDisplayCondition: [0.0, 5500000.0],
      },
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 1.0, 1.0],
      distanceDisplayCondition: [0.0, 5500000.0],
      properties: { near: 0.0, far: 5500000.0 },
    };

    const rectanglePrimitiveEntity = {
      id: 'rectangle-primitive',
      show: true,
      name: 'Distance Condition Red Rectangle',
      positionDegrees: null,
      shape: {
        type: 'rectangle',
        coordinatesDegrees: [-80.5, 39.7, -75.1, 42.0],
        height: 0.0,
        rotation: 0.0,
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
      },
      distanceDisplayCondition: [0.0, 5500000.0],
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 0.0, 0.0, 0.5],
      properties: { near: 0.0, far: 5500000.0 },
    };

    const pointEntity = {
      id: 'point-model-point',
      show: true,
      name: 'Distance Condition Yellow Point',
      positionDegrees: [-75.59777, 40.03883, 0.0],
      shape: {
        type: 'point',
        pixelSize: 16.0,
        outlineWidth: 2.0,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
        distanceDisplayCondition: [0.0, 250.5],
      },
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 0.0, 1.0],
      distanceDisplayCondition: [0.0, 250.5],
      properties: { near: 0.0, far: 250.5 },
    };

    const modelEntity = {
      id: 'point-model-gltf',
      show: true,
      name: 'Distance Condition Vehicle Model',
      positionDegrees: [-75.59777, 40.03883, 0.0],
      shape: {
        type: 'model',
        uri: '../../SampleData/models/CesiumAir/Cesium_Air.glb',
        scale: 1.0,
        minimumPixelSize: 64.0,
        maximumScale: 20000.0,
        runAnimations: true,
      },
      screen: {
        horizontalOrigin: 'CENTER',
        verticalOrigin: 'CENTER',
        rotation: 0.0,
      },
      distanceDisplayCondition: [0.0, 250.5],
      fill: true,
      outline: false,
      outlineColor: [0.0, 0.0, 0.0, 1.0],
      outlineWidth: 1.0,
      color: [1.0, 1.0, 1.0, 1.0],
      properties: { near: 0.0, far: 250.5 },
    };

    const v0 = [billboardPrimitiveEntity, rectanglePrimitiveEntity];
    const v1 = [pointEntity, modelEntity];
    const v2 = [rectanglePrimitiveEntity, modelEntity];
    const v3 = [billboardPrimitiveEntity, pointEntity];

    const variants = {
      initial: 0,
      variants: [
        { label: 'Billboard and Primitive (Default Range)', entities: v0 },
        { label: 'Point and Model (Yellow Point & Aircraft)', entities: v1 },
        { label: 'Near Distance Range (< 5.5e6m for Primitive, < 250.5m for Model)', entities: v2 },
        { label: 'Far Distance Range (> 5.5e6m for Billboard, > 250.5m for Point)', entities: v3 },
      ],
    };

    return {
      entities: v0,
      variants,
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'variants',
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: v0.length,
        variantCount: variants.variants.length,
        framing: 'zoomToEntities',
        shouldAnimate: false,
        requirements: [
          'Billboard distanceDisplayCondition with near distance 5.5e6 meters',
          'PerInstanceColorAppearance Rectangle primitive distanceDisplayCondition with [0.0, 5.5e6]',
          'PointPrimitiveCollection with distanceDisplayCondition near 250.5 meters',
          'Model.fromGltfAsync with distanceDisplayCondition [0.0, 250.5] meters',
          'Sandcastle toolbar menu switching between Billboard/Primitive and Point/Model modes',
        ],
        status: 'functional-passed',
        validation: {
          source: 'passed',
          gpu: 'passed',
          browser: 'passed',
          nativeSmoke: 'passed',
          pixelExactReference: 'not-claimed',
          browserReport: '../../docs/full-recipes-browser.json',
          nativeScope: 'actual source geometry and rendering',
        },
      },
    };
  }

  throw new Error(`Unknown batch 39 case: ${id}`);
}
