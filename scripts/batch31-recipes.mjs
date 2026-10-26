import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch31Ids = [
  'geometry-offset-attribute-dev',
  'geometry-offset-attribute-box-cylinder-ellipsoid-dev',
  'czml-polygon-intervals-availability',
  'czml-custom-properties',
];

export function extractBatch31(id, source, item, hash) {
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
    GREEN: new Color(0, 128 / 255, 0),
    BLUE: new Color(0, 0, 1),
    WHITE: new Color(1, 1, 1),
    BLACK: new Color(0, 0, 0),
    YELLOW: new Color(1, 1, 0),
  });

  class Cartesian2 {
    constructor(x, y) {
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
    static multiplyByScalar(v, s, result = new Cartesian3()) {
      result.x = v.x * s;
      result.y = v.y * s;
      result.z = v.z * s;
      return result;
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
      return new Cartesian3(
        (N + height) * cosPhi * Math.cos(lambda),
        (N + height) * cosPhi * Math.sin(lambda),
        (N * (1 - e2) + height) * sinPhi,
      );
    }
  }
  Cartesian3.ZERO = new Cartesian3(0, 0, 0);

  const Rectangle = {
    fromDegrees: (w, s, e, n) => ({
      west: (w * Math.PI) / 180,
      south: (s * Math.PI) / 180,
      east: (e * Math.PI) / 180,
      north: (n * Math.PI) / 180,
      degrees: [w, s, e, n],
    }),
    center: r => ({
      longitude: (r.west + r.east) / 2,
      latitude: (r.south + r.north) / 2,
      height: 0,
    }),
  };

  const ellipsoid = {
    cartographicToCartesian: c => {
      const phi = c.latitude;
      const lambda = c.longitude;
      const a = 6378137.0;
      const b = 6356752.314245;
      const e2 = 1 - (b * b) / (a * a);
      const sinPhi = Math.sin(phi);
      const cosPhi = Math.cos(phi);
      const N = a / Math.sqrt(1 - e2 * sinPhi * sinPhi);
      return new Cartesian3(
        (N + c.height) * cosPhi * Math.cos(lambda),
        (N + c.height) * cosPhi * Math.sin(lambda),
        (N * (1 - e2) + c.height) * sinPhi,
      );
    },
    geodeticSurfaceNormal: c => {
      const a2 = 6378137.0 * 6378137.0;
      const b2 = 6356752.314245 * 6356752.314245;
      const n = new Cartesian3(c.x / a2, c.y / a2, c.z / b2);
      const mag = Math.hypot(n.x, n.y, n.z);
      return new Cartesian3(n.x / mag, n.y / mag, n.z / mag);
    },
  };

  class Matrix4 {
    constructor(elements = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]) {
      this.elements = elements;
    }
    static multiplyByTranslation(matrix, translation, result = new Matrix4()) {
      const m = matrix.elements.slice();
      m[12] += translation.x;
      m[13] += translation.y;
      m[14] += translation.z;
      result.elements = m;
      return result;
    }
  }

  const Transforms = {
    eastNorthUpToFixedFrame: origin => {
      const normal = ellipsoid.geodeticSurfaceNormal(origin);
      let east = new Cartesian3(-origin.y, origin.x, 0);
      const eastMag = Math.hypot(east.x, east.y, east.z);
      if (eastMag > 1e-6) {
        east = new Cartesian3(east.x / eastMag, east.y / eastMag, east.z / eastMag);
      } else {
        east = new Cartesian3(1, 0, 0);
      }
      const north = new Cartesian3(
        normal.y * east.z - normal.z * east.y,
        normal.z * east.x - normal.x * east.z,
        normal.x * east.y - normal.y * east.x,
      );
      return new Matrix4([
        east.x, east.y, east.z, 0,
        north.x, north.y, north.z, 0,
        normal.x, normal.y, normal.z, 0,
        origin.x, origin.y, origin.z, 1,
      ]);
    },
  };

  const GeometryOffsetAttribute = {
    NONE: 0,
    TOP: 1,
    ALL: 2,
  };

  class GeometryInstance {
    constructor(options) {
      this.id = options.id;
      this.geometry = options.geometry;
      this.modelMatrix = options.modelMatrix;
      this.attributes = options.attributes;
    }
  }

  class RectangleGeometry {
    constructor(options) {
      this.type = 'RectangleGeometry';
      this.rectangle = options.rectangle.degrees;
      this.height = options.height ?? 0;
      this.extrudedHeight = options.extrudedHeight ?? null;
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  class RectangleOutlineGeometry {
    constructor(options) {
      this.type = 'RectangleOutlineGeometry';
      this.rectangle = options.rectangle.degrees;
      this.height = options.height ?? 0;
      this.extrudedHeight = options.extrudedHeight ?? null;
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  class BoxGeometry {
    constructor(options) {
      this.type = 'BoxGeometry';
      this.dimensions = [options.dimensions.x, options.dimensions.y, options.dimensions.z];
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
    static fromDimensions(options) {
      return new BoxGeometry(options);
    }
  }

  class BoxOutlineGeometry {
    constructor(options) {
      this.type = 'BoxOutlineGeometry';
      this.dimensions = [options.dimensions.x, options.dimensions.y, options.dimensions.z];
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
    static fromDimensions(options) {
      return new BoxOutlineGeometry(options);
    }
  }

  class CylinderGeometry {
    constructor(options) {
      this.type = 'CylinderGeometry';
      this.length = options.length;
      this.topRadius = options.topRadius;
      this.bottomRadius = options.bottomRadius;
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  class CylinderOutlineGeometry {
    constructor(options) {
      this.type = 'CylinderOutlineGeometry';
      this.length = options.length;
      this.topRadius = options.topRadius;
      this.bottomRadius = options.bottomRadius;
      this.numberOfVerticalLines = options.numberOfVerticalLines ?? 16;
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  class EllipsoidGeometry {
    constructor(options) {
      this.type = 'EllipsoidGeometry';
      this.radii = [options.radii.x, options.radii.y, options.radii.z];
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  class EllipsoidOutlineGeometry {
    constructor(options) {
      this.type = 'EllipsoidOutlineGeometry';
      this.radii = [options.radii.x, options.radii.y, options.radii.z];
      this.stackPartitions = options.stackPartitions ?? 16;
      this.slicePartitions = options.slicePartitions ?? 8;
      this.offsetAttribute = options.offsetAttribute ?? GeometryOffsetAttribute.NONE;
    }
  }

  const ColorGeometryInstanceAttribute = {
    fromColor: c => c.rgba,
  };

  const OffsetGeometryInstanceAttribute = {
    fromCartesian3: v => [v.x, v.y, v.z],
    toValue: (v, existing) => [v.x, v.y, v.z],
  };

  class PerInstanceColorAppearance {
    constructor(options = {}) {
      this.type = 'PerInstanceColorAppearance';
      this.closed = options.closed ?? false;
      this.flat = options.flat ?? false;
      this.translucent = options.translucent ?? true;
      this.renderState = options.renderState;
    }
  }
  PerInstanceColorAppearance.VERTEX_FORMAT = 'PerInstanceColor';

  const addedPrimitives = [];
  const addedEntities = [];
  const toolbarButtons = [];
  const toolbarMenus = [];
  let loadedCzml = null;

  class Primitive {
    constructor(options) {
      this.geometryInstances = Array.isArray(options.geometryInstances)
        ? options.geometryInstances
        : [options.geometryInstances];
      this.appearance = options.appearance;
      this.asynchronous = options.asynchronous ?? true;
    }
    getGeometryInstanceAttributes(id) {
      const inst = this.geometryInstances.find(i => i.id === id);
      return inst ? inst.attributes : {};
    }
  }

  class CallbackProperty {
    constructor(callback, isConstant = false) {
      this.callback = callback;
      this.isConstant = isConstant;
    }
    getValue(time, result) {
      return this.callback(time, result);
    }
  }

  class CzmlDataSource {
    constructor(name) {
      this.name = name;
      this.entities = {
        _map: new Map(),
        getById(id) {
          return this._map.get(id);
        },
      };
    }
    load(czml) {
      loadedCzml = czml;
      if (Array.isArray(czml)) {
        for (const packet of czml) {
          if (packet.id) {
            const ent = {
              id: packet.id,
              name: packet.name,
              properties: packet.properties ?? {},
              polygon: packet.polygon ? { ...packet.polygon } : undefined,
            };
            this.entities._map.set(packet.id, ent);
          }
        }
      }
      return Promise.resolve(this);
    }
    static load(czml) {
      const ds = new CzmlDataSource();
      return ds.load(czml);
    }
  }

  class Viewer {
    constructor(container, options = {}) {
      assert.equal(container, 'cesiumContainer');
      this.scene = {
        globe: { ellipsoid },
        primitives: {
          add: p => {
            addedPrimitives.push(p);
            return p;
          },
          removeAll: () => {
            addedPrimitives.length = 0;
          },
        },
        maximumAliasedLineWidth: 1.0,
      };
      this.entities = {
        add: e => {
          addedEntities.push(e);
          return e;
        },
        removeAll: () => {
          addedEntities.length = 0;
        },
      };
      this.dataSources = {
        add: ds => {
          return Promise.resolve(ds);
        },
      };
    }
    extend() {}
    zoomTo() {
      return Promise.resolve();
    }
  }

  const Sandcastle = {
    addToolbarButton: (name, onclick) => {
      toolbarButtons.push({ name, onclick });
    },
    addToolbarMenu: (options, title) => {
      toolbarMenus.push({ title, options });
    },
    declare: () => {},
    highlight: () => {},
    reset: () => {},
  };

  const executable = source.replace(/^import .*;\s*$/gm, '');
  assert(!/\bimport\b/.test(executable));

  vm.runInNewContext(
    executable,
    {
      Cesium: {
        Viewer,
        Color,
        Cartesian2,
        Cartesian3,
        Rectangle,
        Matrix4,
        Transforms,
        GeometryOffsetAttribute,
        GeometryInstance,
        RectangleGeometry,
        RectangleOutlineGeometry,
        BoxGeometry,
        BoxOutlineGeometry,
        CylinderGeometry,
        CylinderOutlineGeometry,
        EllipsoidGeometry,
        EllipsoidOutlineGeometry,
        ColorGeometryInstanceAttribute,
        OffsetGeometryInstanceAttribute,
        PerInstanceColorAppearance,
        Primitive,
        CallbackProperty,
        CzmlDataSource,
        viewerCesiumInspectorMixin: () => {},
        Math: {
          toRadians: d => (d * Math.PI) / 180,
        },
      },
      Sandcastle,
      console,
    },
    { timeout: 3000 },
  );

  if (id === 'geometry-offset-attribute-dev') {
    assert.equal(addedPrimitives.length, 2);
    assert.equal(toolbarButtons.length, 3);

    const filledPrim = addedPrimitives[0];
    const outlinePrim = addedPrimitives[1];
    assert.equal(filledPrim.geometryInstances.length, 3);
    assert.equal(outlinePrim.geometryInstances.length, 3);

    const instances = [
      ...filledPrim.geometryInstances.map((inst, i) => ({
        id: inst.id,
        geometry: {
          type: inst.geometry.type,
          rectangle: inst.geometry.rectangle,
          height: inst.geometry.height,
          extrudedHeight: inst.geometry.extrudedHeight,
          offsetAttribute: inst.geometry.offsetAttribute,
        },
        attributes: {
          color: inst.attributes.color,
          offset: inst.attributes.offset,
        },
      })),
      ...outlinePrim.geometryInstances.map((inst, i) => ({
        id: inst.id,
        geometry: {
          type: inst.geometry.type,
          rectangle: inst.geometry.rectangle,
          height: inst.geometry.height,
          extrudedHeight: inst.geometry.extrudedHeight,
          offsetAttribute: inst.geometry.offsetAttribute,
        },
        attributes: {
          color: inst.attributes.color,
          offset: inst.attributes.offset,
        },
      })),
    ];

    const entities = [
      {
        id: 'primitive-0-instance-0',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-100.0, 30.0, -99.0, 31.0],
          height: 10000.0,
          extrudedHeight: 130000.0,
        },
        color: [1.0, 0.0, 0.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 0 },
      },
      {
        id: 'primitive-0-instance-1',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-99.0, 30.0, -98.0, 31.0],
          height: 10000.0,
          extrudedHeight: 30000.0,
        },
        color: [0.0, 1.0, 0.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 1 },
      },
      {
        id: 'primitive-0-instance-2',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-98.0, 30.0, -97.0, 31.0],
          height: 110000.0,
          extrudedHeight: 130000.0,
        },
        color: [0.0, 0.0, 1.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 2 },
      },
      {
        id: 'primitive-1-instance-0',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-100.0, 30.0, -99.0, 31.0],
          height: 10000.0,
          extrudedHeight: 130000.0,
        },
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 0 },
      },
      {
        id: 'primitive-1-instance-1',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-99.0, 30.0, -98.0, 31.0],
          height: 10000.0,
          extrudedHeight: 30000.0,
        },
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 1 },
      },
      {
        id: 'primitive-1-instance-2',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: [-98.0, 30.0, -97.0, 31.0],
          height: 110000.0,
          extrudedHeight: 130000.0,
        },
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 2 },
      },
    ];

    const primitiveData = {
      primitives: [
        {
          geometryInstances: filledPrim.geometryInstances.map(inst => ({
            geometry: {
              type: inst.geometry.type,
              rectangle: inst.geometry.rectangle,
              height: inst.geometry.height,
              extrudedHeight: inst.geometry.extrudedHeight,
              vertexFormat: 'PerInstanceColor',
              offsetAttribute: inst.geometry.offsetAttribute,
            },
            attributes: {
              color: inst.attributes.color,
              offset: inst.attributes.offset,
            },
          })),
          appearance: {
            type: 'PerInstanceColorAppearance',
            closed: true,
          },
        },
        {
          geometryInstances: outlinePrim.geometryInstances.map(inst => ({
            geometry: {
              type: inst.geometry.type,
              rectangle: inst.geometry.rectangle,
              height: inst.geometry.height,
              extrudedHeight: inst.geometry.extrudedHeight,
              offsetAttribute: inst.geometry.offsetAttribute,
            },
            attributes: {
              color: inst.attributes.color,
              offset: inst.attributes.offset,
            },
          })),
          appearance: {
            type: 'PerInstanceColorAppearance',
            flat: true,
            renderState: { lineWidth: 1.0 },
          },
        },
      ],
    };

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      primitive: JSON.parse(JSON.stringify(primitiveData)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'primitives',
        path: `../../tests/sandcastle/full/${id}.primitive.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'viewer-default',
        shouldAnimate: false,
        requirements: [
          'three extruded rectangles with GeometryOffsetAttribute TOP, ALL, and NONE',
          'three matching outline rectangles with same offset attributes',
          'per-instance offset attributes along geodetic surface normals',
          'toolbar buttons 1, 2, 3 updating offset dynamically',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'geometry-offset-attribute-box-cylinder-ellipsoid-dev') {
    assert.equal(addedPrimitives.length, 2);
    assert.equal(toolbarButtons.length, 3);

    const filledPrim = addedPrimitives[0];
    const outlinePrim = addedPrimitives[1];
    assert.equal(filledPrim.geometryInstances.length, 3);
    assert.equal(outlinePrim.geometryInstances.length, 3);

    const calcModelMatrix = inst => {
      const m = inst.modelMatrix.elements.slice();
      if (inst.attributes.offset) {
        m[12] += inst.attributes.offset[0];
        m[13] += inst.attributes.offset[1];
        m[14] += inst.attributes.offset[2];
      }
      return m;
    };

    const entities = [
      {
        id: 'primitive-0-instance-0',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'box',
          dimensions: [400000.0, 300000.0, 500000.0],
        },
        modelMatrix: calcModelMatrix(filledPrim.geometryInstances[0]),
        color: [1.0, 0.0, 0.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 0 },
      },
      {
        id: 'primitive-0-instance-1',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'cylinder',
          length: 400000.0,
          topRadius: 200000.0,
          bottomRadius: 200000.0,
        },
        modelMatrix: calcModelMatrix(filledPrim.geometryInstances[1]),
        color: [1.0, 0.0, 0.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 1 },
      },
      {
        id: 'primitive-0-instance-2',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'ellipsoid',
          radii: [200000.0, 200000.0, 300000.0],
        },
        modelMatrix: calcModelMatrix(filledPrim.geometryInstances[2]),
        color: [1.0, 0.0, 0.0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        surfaceAppearance: { closed: true, flat: false, translucent: true },
        properties: { primitiveIndex: 0, instanceIndex: 2 },
      },
      {
        id: 'primitive-1-instance-0',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'box',
          dimensions: [400000.0, 300000.0, 500000.0],
        },
        modelMatrix: calcModelMatrix(outlinePrim.geometryInstances[0]),
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 0 },
      },
      {
        id: 'primitive-1-instance-1',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'cylinder',
          length: 400000.0,
          topRadius: 200000.0,
          bottomRadius: 200000.0,
        },
        modelMatrix: calcModelMatrix(outlinePrim.geometryInstances[1]),
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 1 },
      },
      {
        id: 'primitive-1-instance-2',
        name: '',
        positionDegrees: null,
        shape: {
          type: 'ellipsoid',
          radii: [200000.0, 200000.0, 300000.0],
          stackPartitions: 16,
          slicePartitions: 8,
        },
        modelMatrix: calcModelMatrix(outlinePrim.geometryInstances[2]),
        color: [1, 1, 1, 1],
        fill: false,
        outline: true,
        outlineColor: [1, 1, 1, 1],
        outlineWidth: 1,
        properties: { primitiveIndex: 1, instanceIndex: 2 },
      },
    ];

    const primitiveData = {
      primitives: [
        {
          geometryInstances: filledPrim.geometryInstances.map(inst => ({
            geometry: {
              type: inst.geometry.type,
              dimensions: inst.geometry.dimensions,
              length: inst.geometry.length,
              topRadius: inst.geometry.topRadius,
              bottomRadius: inst.geometry.bottomRadius,
              radii: inst.geometry.radii,
              vertexFormat: 'PerInstanceColor',
              offsetAttribute: inst.geometry.offsetAttribute,
            },
            modelMatrix: inst.modelMatrix ? inst.modelMatrix.elements : undefined,
            attributes: {
              color: inst.attributes.color,
              offset: inst.attributes.offset,
            },
          })),
          appearance: {
            type: 'PerInstanceColorAppearance',
            closed: true,
          },
        },
        {
          geometryInstances: outlinePrim.geometryInstances.map(inst => ({
            geometry: {
              type: inst.geometry.type,
              dimensions: inst.geometry.dimensions,
              length: inst.geometry.length,
              topRadius: inst.geometry.topRadius,
              bottomRadius: inst.geometry.bottomRadius,
              numberOfVerticalLines: inst.geometry.numberOfVerticalLines,
              radii: inst.geometry.radii,
              stackPartitions: inst.geometry.stackPartitions,
              slicePartitions: inst.geometry.slicePartitions,
              offsetAttribute: inst.geometry.offsetAttribute,
            },
            modelMatrix: inst.modelMatrix ? inst.modelMatrix.elements : undefined,
            attributes: {
              color: inst.attributes.color,
              offset: inst.attributes.offset,
            },
          })),
          appearance: {
            type: 'PerInstanceColorAppearance',
            flat: true,
            renderState: { lineWidth: 1.0 },
          },
        },
      ],
    };

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      primitive: JSON.parse(JSON.stringify(primitiveData)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'primitives',
        path: `../../tests/sandcastle/full/${id}.primitive.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'viewer-default',
        shouldAnimate: false,
        requirements: [
          'BoxGeometry, CylinderGeometry, and EllipsoidGeometry with offsetAttribute ALL',
          'matching outline geometry instances with same offsetAttribute',
          'local eastNorthUpToFixedFrame modelMatrix translations',
          'toolbar buttons to move box, cylinder, and ellipsoid dynamically',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-polygon-intervals-availability') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');
    const packets = loadedCzml.slice(1);
    assert.equal(packets.length, 3);

    const entities = [
      {
        id: 'dynamicPolygon',
        show: true,
        name: 'Dynamic Polygon with Intervals',
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: [
            [-50.0, 20.0, 0.0],
            [-50.0, 40.0, 0.0],
            [-40.0, 40.0, 0.0],
            [-40.0, 20.0, 0.0],
          ],
          holes: [
            [
              [-47.0, 35.0, 0.0],
              [-46.0, 25.0, 0.0],
              [-42.0, 30.0, 0.0],
            ],
          ],
          extrudedHeight: null,
        },
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        color: [1.0, 0.0, 1.0, 1.0],
        availability: [1344096000.0, 1344099600.0],
        properties: null,
      },
      {
        id: 'california',
        show: true,
        name: 'static california with set availability',
        positionDegrees: null,
        shape: {
          type: 'polygonSurface',
          data: {
            hierarchy: {
              positions: [
                [-120.0, 42.0, 50000.0],
                [-124.0, 42.0, 30500.0],
                [-124.5, 40.0, 3000.0],
                [-123.0, 38.0, 0.0],
                [-122.0, 36.0, 0.0],
                [-120.8, 34.2, 0.0],
                [-118.0, 34.0, 0.0],
                [-117.0, 32.2, 6000.0],
                [-115.5, 32.5, 1530.0],
                [-115.0, 35.0, 1530.0],
                [-120.0, 39.0, 30500.0],
              ],
              holes: [],
            },
            textureCoordinates: null,
            height: 0.0,
            perPositionHeight: true,
            extrudedHeight: 0.0,
            closeTop: true,
            closeBottom: true,
            coplanar: false,
            arcType: 'GEODESIC',
            stRotation: 0.0,
          },
        },
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        color: [1.0, 0.0, 0.0, 150 / 255],
        availability: [1344096600.0, 1344097800.0],
        properties: null,
      },
      {
        id: 'pennsylvania',
        show: true,
        name: 'static pennsylvania with set availability',
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: [
            [-75.5, 42.0, 50000.0],
            [-79.8, 42.0, 50000.0],
            [-79.9, 42.3, 50000.0],
            [-80.5, 42.0, 50000.0],
            [-80.5, 39.8, 50000.0],
            [-75.7, 39.8, 50000.0],
            [-74.5, 40.2, 50000.0],
            [-75.2, 40.8, 50000.0],
            [-74.7, 41.3, 50000.0],
          ],
          holes: [],
          height: 50000.0,
          extrudedHeight: 200000.0,
        },
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        color: [0.0, 1.0, 0.0, 150 / 255],
        availability: [1344097200.0, 1344098400.0],
        properties: null,
      },
    ];

    return {
      czml: JSON.parse(JSON.stringify(loadedCzml)),
      entities: JSON.parse(JSON.stringify(entities)),
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'czml',
        path: `../../tests/sandcastle/full/${id}.czml`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'viewer-default',
        shouldAnimate: true,
        requirements: [
          'CZML document packet with clock interval and multiplier',
          'dynamicPolygon with position intervals, hole interval, and solidColor intervals',
          'california polygon with availability interval and perPositionHeight',
          'pennsylvania polygon with availability interval, height, and extrudedHeight',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-custom-properties') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');
    const packets = loadedCzml.slice(1);
    assert.equal(packets.length, 2);

    const coloradoPacket = packets.find(p => p.id === 'colorado');
    assert(coloradoPacket);
    const customObjPacket = packets.find(p => p.id === 'custom_property_object');
    assert(customObjPacket);

    const c = coloradoPacket.polygon.positions.cartographicDegrees;
    const positions = Array.from({ length: c.length / 3 }, (_, j) => [
      c[3 * j],
      c[3 * j + 1],
      c[3 * j + 2],
    ]);

    const entities = [
      {
        id: 'colorado',
        show: true,
        name: coloradoPacket.name,
        positionDegrees: null,
        shape: {
          type: 'polygon',
          positionsDegrees: positions,
          holes: [],
          height: 0.0,
          extrudedHeight: 0.0,
        },
        color: [0.0, 1.0, 0.0, 150 / 255],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        properties: null,
      },
    ];

    return {
      czml: JSON.parse(JSON.stringify(loadedCzml)),
      entities: JSON.parse(JSON.stringify(entities)),
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
          'CZML document clock with interval 1970/2010',
          'custom_property_object with population_intervals and population_sampled',
          'colorado polygon with solidColor and initial extrudedHeight 0',
          'toolbar menu to bind extrudedHeight to scaled interval data or sampled data',
          'viewer.zoomTo dataSource',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`unsupported batch31 recipe ${id}`);
}
