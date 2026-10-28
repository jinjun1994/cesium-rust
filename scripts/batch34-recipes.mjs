import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch34Ids = [
  'czml-polygon-interpolating-references',
  'distance-display-conditions',
  'callback-property',
  'map-pins',
];

const FACILITY_PNG =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQBAMAAADt3eJSAAAAMFBMVEVAQEDAwMDAoAD///+gYADAgACgoKSAYADgwAD///8AAAAAAAAAAAAAAAAAAAAAAAC6opgsAAAACnRSTlP///////////8AsswszwAAAH1JREFUeJw9xiEOwjAYhuF33/6KyQVPlh6AjOwWqBoOgMKScJo6LI5rIAg3IE0WFIK5CpIVQ3jUU0XygQgiH4d2D8R6WKx3TTRI3eoClnvSwxWMxPL5CRjdnHAgZt5TAKG7LydQc/O4zZkKD2N4XaV621MMIWACQUsBxM8/X+GkICjYO7GLAAAAAElFTkSuQmCC';

const PIN_BLUE =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAABSElEQVR4nO1ZyxXCMAwrPOaAMTjDNjAQbAPnjgGLwLUvr/5F8gst0bWNI1lu4qTD0NEWm4ygp8vrIz173g/UOWnBNNISGGLgADXESyBCtsjEDPJonCrlLOJziLoRdiCTfE38XQaJx20/Ss/O1/eROVfILis7GvESlhBvKblLiEne8763lKBVyEuGPW4KlwAtGygJbbzHBcgBRgbROJQSaglTgGQjK/tWPKuM1u/Ar6MLaI0uoDVMAVJTxe4qpXhWU7d+BzSwXEDiuARoNqIitPGeMwGlhGpFMBx0C7CyESXDOpGFbyU8PTp6Jo7cTKQc6tlLrIbwN8C+20TjQ2SYd0S1iYFWIZYbze5G0ckZ4yn7QC0JhoP/3QtNEc0m6/vpDkzhzSpzL+kOlLCyy97JuwNzkLKc0Ud1BySU2c7qYhfvQDqyf8suHl+fcpEZxMNAjwAAAABJRU5ErkJggg==';

const PIN_QUESTION =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAABS0lEQVR4nO1ZQRKDIAyETt+hz+j/j32GfoReHWpIwm4GbdmrGja7AQKmNDEWOSLosixFerbvO3VMWrAWaQmMZOAAPcRrIIk8kIEZ5NE4XZmziJ/B64bbgUjyPfGfESS2bXtLz9Z1fTHHctmlqdMiXkNLxFpK5hJikre8by0laBWykmF/d4TJppYaEomc81eJlFJO322Vk1ZKkAMMBdE4lBIaCTUBqXxY6mvxtMn8+w5cHTOB0ZgJjIaagLQTsrtKKV7oTnwFQAmwXEDimM8DPQ2dBUgjlxKphHoVZDhoTkBTw0uGdSJz30pYTkromdhzMxFyqGcvsS245wD7bhOND5Fh3hH1CgOtQiw3ht2NooMzvqfsA70kGA7+dy90hFdN1vyZDhxhVZW5l0wHamjqsnfy6cAZJJUj+qjpgIRa7agu9vYOhCP6t+zt8QGr5KNjVrhvRwAAAABJRU5ErkJggg==';

const PIN_GROCERY =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAABT0lEQVR4nO1Z0RXCIAwEn5O0Q7iYTqGLOUS7Cv72YUMScnm0mvttGy53AQJNKTAW2SPo9JwK9Wx9rNAxYcFapCkgkjEH6CFew5LIxTIwgrw1TlfmKOJ70LqhdsCTfE/8qweJ5b68qWfza74hx1LZxanTIl6DS0RaSuISQpKXvC8tJdMqJCWD/m4LkU0tNSgSOeevEiml7L7bKieulEwOIBS0xoGU0EiwCVDlg1Kfi8dN5t934OiIBEYjEhgNNgFqJ0R3lVQ81534CDAlgHLBEkd8Huhp6CSwNHIpgUqoV0GEg+IEODW0ZFAnMvWthOSkZD0Ta24mXA716CW2BfUcQN9tWuObyCDviHqFMa1CKDeG3Y1aB0d8D9kHekkgHPzvXmgLrZqo+RMObCFVFbmXhAM1OHXRO3k4sAdKZY8+KhygUKvt1cWe3gF3eP+WPT0++Eyd566XbqYAAAAASUVORK5CYII=';

const PIN_HOSPITAL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAYAAABXAvmHAAABU0lEQVR4nO1ZyxXCIBAEnyVoBdqDF61eL/agFWgPeM3DLPubfSTKXJMsszMLLCSlgb7IEUEfu3Ohnh3fN+iYsGAt0hQQybgDWIjX8CSy8QyMIO+NY8ocRXwOWjfUDkSSt8TfRpA4vK536tlzfzkhx1LZxanTIl6DS0RaSuISQpKXvC8tJdcqJCWD/m4KkU0tNSgSOeevEimlzL7bKieulFwOIBT0xoGUUE+wCVDlg1Kfi8dN5t93YOkYCfTGSKA32ASonRDdVVLxQnfiJcCVAMoFTxzxecDS0EngaeRSApWQVUGEg+IEODW0ZFAnMvWthOSk5D0Ta24mQg716CW2BfUcQN9teuO7yCDviKzCuFYhlBvd7ka9gyO+h+wDVhIIB/+7F5pCqyZq/gwHppCqitxLhgM1OHXRO/lwYA6UyhF91HCAQq12VBe7egfCEf1bdvX4AHxXmYtHRCGaAAAAAElFTkSuQmCC';

export function extractBatch34(id, source, item, hash) {
  class Color {
    constructor(r, g, b, a = 1) {
      this.rgba = [r, g, b, a];
    }
    withAlpha(a) {
      return new Color(...this.rgba.slice(0, 3), a);
    }
  }
  Object.assign(Color, {
    RED: new Color(1, 0, 0),
    GREEN: new Color(0, 1, 0),
    BLUE: new Color(0, 0, 1),
    WHITE: new Color(1, 1, 1),
    BLACK: new Color(0, 0, 0),
    YELLOW: new Color(1, 1, 0),
    ROYALBLUE: new Color(65 / 255, 105 / 255, 225 / 255, 1),
  });

  class Cartesian2 {
    constructor(x = 0, y = 0) {
      this.x = x;
      this.y = y;
    }
  }

  class Cartesian3 {
    constructor(x = 0, y = 0, z = 0) {
      this.x = x;
      this.y = y;
      this.z = z;
    }
    static fromDegrees(lon, lat, height = 0) {
      const phi = (lat * Math.PI) / 180;
      const lambda = (lon * Math.PI) / 180;
      const a = 6378137.0;
      const b = 6356752.314245;
      const e2 = 1 - (b * b) / (a * a);
      const sinPhi = Math.sin(phi);
      const cosPhi = Math.cos(phi);
      const N = a / Math.sqrt(1 - e2 * sinPhi * sinPhi);
      return {
        _degrees: [lon, lat, height],
        x: (N + height) * cosPhi * Math.cos(lambda),
        y: (N + height) * cosPhi * Math.sin(lambda),
        z: (N * (1 - e2) + height) * sinPhi,
      };
    }
    static fromDegreesArray(arr, ellipsoid, result) {
      const res = [];
      for (let i = 0; i < arr.length; i += 2) {
        res.push(Cartesian3.fromDegrees(arr[i], arr[i + 1]));
      }
      return res;
    }
    static fromRadians(lon, lat, height = 0) {
      return Cartesian3.fromDegrees((lon * 180) / Math.PI, (lat * 180) / Math.PI, height);
    }
  }

  class Cartographic {
    constructor(longitude = 0, latitude = 0, height = 0) {
      this.longitude = longitude;
      this.latitude = latitude;
      this.height = height;
    }
    static fromDegrees(lon, lat, height = 0) {
      return new Cartographic((lon * Math.PI) / 180, (lat * Math.PI) / 180, height);
    }
    static fromCartesian(cart) {
      if (cart._degrees) {
        return Cartographic.fromDegrees(...cart._degrees);
      }
      return new Cartographic(0, 0, 0);
    }
  }

  class EllipsoidGeodesic {
    constructor(start, end) {
      this.start = start;
      this.end = end;
      this.surfaceDistance = 9100.0;
    }
    setEndPoints(start, end) {
      this.start = start;
      this.end = end;
      this.surfaceDistance = 9100.0;
    }
    interpolateUsingFraction(fraction, result) {
      const lon = this.start.longitude + (this.end.longitude - this.start.longitude) * fraction;
      const lat = this.start.latitude + (this.end.latitude - this.start.latitude) * fraction;
      if (result) {
        result.longitude = lon;
        result.latitude = lat;
        result.height = 0;
        return result;
      }
      return new Cartographic(lon, lat, 0);
    }
  }

  class Rectangle {
    static fromDegrees(w, s, e, n) {
      return [w, s, e, n];
    }
  }

  class HeadingPitchRoll {
    constructor(heading = 0, pitch = 0, roll = 0) {
      this.heading = heading;
      this.pitch = pitch;
      this.roll = roll;
    }
  }

  const Transforms = {
    headingPitchRollQuaternion: (pos, hpr) => {
      return [0, 0, Math.sin(hpr.heading / 2), Math.cos(hpr.heading / 2)];
    },
  };

  class DistanceDisplayCondition {
    constructor(near = 0.0, far = Number.MAX_VALUE) {
      if (arguments.length === 1) {
        this.near = 0.0;
        this.far = near;
      } else {
        this.near = near;
        this.far = far;
      }
    }
  }

  class JulianDate {
    static now() {
      return 1344096000.0;
    }
    static secondsDifference(a, b) {
      return 100.0;
    }
  }

  class CallbackProperty {
    constructor(callback, isConstant) {
      this.callback = callback;
      this.isConstant = isConstant;
    }
    getValue(time, result) {
      return this.callback(time, result);
    }
  }

  const VerticalOrigin = {
    BOTTOM: 'BOTTOM',
    CENTER: 'CENTER',
    TOP: 'TOP',
  };

  class PinBuilder {
    fromColor(color, size) {
      return { toDataURL: () => PIN_BLUE };
    }
    fromText(text, color, size) {
      return { toDataURL: () => PIN_QUESTION };
    }
    fromUrl(url, color, size) {
      return Promise.resolve({ toDataURL: () => PIN_GROCERY });
    }
    fromMakiIconId(id, color, size) {
      return Promise.resolve({ toDataURL: () => PIN_HOSPITAL });
    }
  }

  const addedEntities = [];
  let loadedCzml = null;
  const toolbarMenus = [];

  class EntityCollection {
    constructor() {
      this.items = [];
    }
    add(e) {
      addedEntities.push(e);
      this.items.push(e);
      return e;
    }
    removeAll() {
      addedEntities.length = 0;
      this.items.length = 0;
    }
  }

  class CzmlDataSource {
    constructor() {
      this.entities = new EntityCollection();
    }
    then(cb) {
      if (typeof cb === 'function') cb(this);
      return this;
    }
    static load(data) {
      loadedCzml = data;
      const ds = new CzmlDataSource();
      return ds;
    }
  }

  class Viewer {
    constructor(container, options = {}) {
      this.options = options;
      this.entities = new EntityCollection();
      this.clock = {
        shouldAnimate: options.shouldAnimate ?? false,
      };
      this.camera = {
        flyHome: () => {},
      };
      this.dataSources = {
        add: ds => ds,
      };
      this.trackedEntity = null;
    }
    zoomTo(entities) {}
  }

  const Sandcastle = {
    declare: () => {},
    highlight: () => {},
    addToolbarMenu: items => {
      toolbarMenus.push(items);
    },
    reset: () => {},
  };

  // Run the source script
  const executable = source
    .replace(/^import .*;\s*$/gm, '')
    .replace(/\bawait\s+/g, '');
  assert(!/\bimport\b/.test(executable));

  vm.runInNewContext(
    executable,
    {
      Cesium: {
        Viewer,
        Color,
        Cartesian2,
        Cartesian3,
        Cartographic,
        EllipsoidGeodesic,
        Ellipsoid: { WGS84: {} },
        Rectangle,
        HeadingPitchRoll,
        Transforms,
        DistanceDisplayCondition,
        JulianDate,
        CallbackProperty,
        VerticalOrigin,
        PinBuilder,
        buildModuleUrl: path => `https://cesium.com/downloads/cesiumjs/releases/1.146/Build/Cesium/${path}`,
        CzmlDataSource,
        Math: {
          PI: Math.PI,
          toRadians: n => (n * Math.PI) / 180,
        },
      },
      Sandcastle,
    },
    { timeout: 2000 },
  );

  if (id === 'czml-polygon-interpolating-references') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');

    // Expected 2 entities: dynamicPolygon and "Polygon with Dynamic Holes"
    const entities = [
      {
        id: 'dynamicPolygon',
        show: true,
        name: 'Dynamic Polygon with Reference Properties',
        positionDegrees: null,
        shape: {
          type: 'polygonSurface',
          data: {
            hierarchy: {
              positions: [
                [-60.0, 35.0, 30000.0],
                [-45.0, 20.0, 4000000.0],
                [-45.0, 60.0, 4000.0],
              ],
              holes: [],
            },
            textureCoordinates: null,
            height: 0.0,
            perPositionHeight: true,
            extrudedHeight: null,
            closeTop: true,
            closeBottom: true,
            coplanar: false,
            arcType: 'GEODESIC',
            stRotation: 0.0,
          },
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 0.0, 1.0, 1.0],
        availability: [1344096000.0, 1344099600.0],
        properties: null,
      },
      {
        id: 'Polygon with Dynamic Holes',
        show: true,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: [
            [-110.0, 43.0, 0.0],
            [-90.0, 43.0, 0.0],
            [-90.0, 30.0, 0.0],
            [-110.0, 30.0, 0.0],
          ],
          holes: [
            [
              [-100.0, 41.0, 0.0],
              [-92.0, 42.0, 0.0],
              [-95.0, 37.0, 0.0],
            ],
          ],
          extrudedHeight: null,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 150 / 255, 0.0, 1.0],
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      czml: JSON.parse(JSON.stringify(loadedCzml)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        requirements: [
          'CZML positions references array resolving vertex coordinates',
          'nested holes references array resolving dynamic inner boundary',
          'perPositionHeight true on dynamic polygon surface',
          'interval-based rgbaf material color transitions',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'distance-display-conditions') {
    assert.equal(toolbarMenus.length, 1);
    const menu = toolbarMenus[0];
    assert.equal(menu.length, 2);

    // Variant 0: Billboard and Rectangle
    const v0Entities = [
      {
        id: 'distance-billboard',
        show: true,
        name: '',
        positionDegrees: [-77.0, 40.5, 0.0],
        shape: {
          type: 'billboard',
          image: FACILITY_PNG,
          width: 64.0,
          height: 64.0,
          scale: 1.0,
          pixelOffset: [0.0, 0.0],
        },
        screen: {
          distanceDisplayCondition: [0.0, 5500000.0],
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        distanceDisplayCondition: [0.0, 5500000.0],
        properties: null,
      },
      {
        id: 'distance-rectangle',
        show: true,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-80.5, 39.7, -75.1, 42.0],
          height: 0.0,
          extrudedHeight: null,
          rotation: 0.0,
        },
        fill: true,
        outline: true,
        outlineColor: [1.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 0.0, 0.0, 0.5],
        distanceDisplayCondition: [0.0, 5500000.0],
        properties: null,
      },
    ];

    // Variant 1: Point and Model
    const v1Entities = [
      {
        id: 'distance-point',
        show: true,
        name: '',
        positionDegrees: [-75.59777, 40.03883, 0.0],
        shape: {
          type: 'point',
          pixelSize: 10.0,
          outlineWidth: 0.0,
          outlineColor: [0.0, 0.0, 0.0, 1.0],
        },
        screen: {
          distanceDisplayCondition: [0.0, 250.5],
          horizontalOrigin: 'CENTER',
          verticalOrigin: 'CENTER',
          rotation: 0.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 0.0, 1.0],
        distanceDisplayCondition: [0.0, 250.5],
        properties: null,
      },
      {
        id: 'distance-model',
        show: true,
        name: '',
        positionDegrees: [-75.59777, 40.03883, 0.0],
        orientation: [0.0, 0.0, Math.sin((135 * Math.PI) / 360), Math.cos((135 * Math.PI) / 360)],
        shape: {
          type: 'model',
          uri: '../../SampleData/models/GroundVehicle/GroundVehicle.glb',
          scale: 1.0,
          minimumPixelSize: 0.0,
          maximumScale: null,
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        distanceDisplayCondition: [0.0, 250.5],
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(v0Entities)),
      variants: {
        variants: [
          { label: menu[0].text, entities: JSON.parse(JSON.stringify(v0Entities)) },
          { label: menu[1].text, entities: JSON.parse(JSON.stringify(v1Entities)) },
        ],
        initial: 0,
      },
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'variants',
        variantCount: 2,
        path: `../../tests/sandcastle/full/${id}.variants.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: v0Entities.length,
        framing: 'zoomToEntities',
        requirements: [
          'Sandcastle toolbar menu with Billboard and Primitive and Point and Model variants',
          'distanceDisplayCondition near/far distance clipping range on billboard and rectangle',
          'distanceDisplayCondition near/far distance clipping range on point and 3D model',
          'Transforms.headingPitchRollQuaternion orientation on GroundVehicle entity',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'callback-property') {
    const entities = [
      {
        id: 'redLine',
        show: true,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'widePolyline',
          data: {
            positions: [
              [-120.0, 35.0, 0.0],
              [-119.9, 35.0, 0.0],
            ],
            width: 5.0,
            arcType: 'GEODESIC',
            clampToGround: false,
            material: {
              type: 'solid',
            },
          },
        },
        fill: true,
        outline: false,
        outlineColor: [0.0, 0.0, 0.0, 1.0],
        outlineWidth: 1.0,
        color: [1.0, 0.0, 0.0, 1.0],
        properties: null,
      },
      {
        id: 'lengthLabel',
        show: true,
        name: '',
        positionDegrees: [-119.95, 35.0, 0.0],
        shape: {
          type: 'label',
          appearance: {
            style: 'FILL',
            showBackground: false,
            backgroundColor: [0.165, 0.165, 0.165, 0.8],
            backgroundPadding: [7.0, 5.0],
            outlineColor: [0.0, 0.0, 0.0, 1.0],
            outlineWidth: 1.0,
          },
          text: '9.1 km',
          fontSize: 20.0,
          scale: 1.0,
          pixelOffset: [0.0, 20.0],
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
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'entities',
        path: `../../tests/sandcastle/full/${id}.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: true,
        trackedEntity: 'lengthLabel',
        requirements: [
          'CallbackProperty dynamic positions on wide polyline',
          'EllipsoidGeodesic surfaceDistance length calculation in km',
          'CallbackProperty midpoint position and dynamic text on label',
          'viewer.trackedEntity tracking the moving length label',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'map-pins') {
    const entities = [
      {
        id: 'bluePin',
        show: true,
        name: 'Blank blue pin',
        positionDegrees: [-75.170726, 39.9208667, 0.0],
        shape: {
          type: 'billboard',
          image: PIN_BLUE,
          width: 48.0,
          height: 48.0,
          scale: 1.0,
          pixelOffset: [0.0, 0.0],
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
        properties: null,
      },
      {
        id: 'questionPin',
        show: true,
        name: 'Question mark',
        positionDegrees: [-75.1698529, 39.9220071, 0.0],
        shape: {
          type: 'billboard',
          image: PIN_QUESTION,
          width: 48.0,
          height: 48.0,
          scale: 1.0,
          pixelOffset: [0.0, 0.0],
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
        properties: null,
      },
      {
        id: 'groceryPin',
        show: true,
        name: 'Grocery store',
        positionDegrees: [-75.1705217, 39.921786, 0.0],
        shape: {
          type: 'billboard',
          image: PIN_GROCERY,
          width: 48.0,
          height: 48.0,
          scale: 1.0,
          pixelOffset: [0.0, 0.0],
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
        properties: null,
      },
      {
        id: 'hospitalPin',
        show: true,
        name: 'Hospital',
        positionDegrees: [-75.1698606, 39.9211275, 0.0],
        shape: {
          type: 'billboard',
          image: PIN_HOSPITAL,
          width: 48.0,
          height: 48.0,
          scale: 1.0,
          pixelOffset: [0.0, 0.0],
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
        properties: null,
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'entities',
        path: `../../tests/sandcastle/full/${id}.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        requirements: [
          'PinBuilder.fromColor royal blue 48px pin',
          'PinBuilder.fromText question mark 48px pin',
          'PinBuilder.fromUrl grocery icon 48px pin',
          'PinBuilder.fromMakiIconId hospital icon 48px pin',
          'VerticalOrigin.BOTTOM bottom-anchored pin tips',
          'Promise.all asynchronous pin loading and viewer.zoomTo framing',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`Unhandled Batch 34 recipe id: ${id}`);
}
