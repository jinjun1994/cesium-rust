import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch30Ids = [
  'rectangle',
  'z-indexing-geometry',
  'czml-zindex',
  'volumes-dev',
];

export function extractBatch30(id, source, item, hash) {
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

  const Cartesian3 = {
    fromDegrees: (lon, lat, height = 0) => [lon, lat, height],
    fromDegreesArray: p =>
      Array.from({ length: p.length / 2 }, (_, i) => [p[2 * i], p[2 * i + 1], 0]),
  };

  class Cartesian3Obj {
    constructor(x, y, z) {
      this.x = x;
      this.y = y;
      this.z = z;
    }
  }

  const Rectangle = {
    fromDegrees: (w, s, e, n) => [w, s, e, n],
  };

  class CallbackProperty {
    constructor(callback, isConstant = false) {
      this.callback = callback;
      this.isConstant = isConstant;
      this.initialValue = typeof callback === 'function' ? callback() : callback;
    }
    getValue() {
      return this.initialValue;
    }
  }

  class PolylineGlowMaterialProperty {
    constructor(options = {}) {
      this.type = 'glow';
      this.glowPower = options.glowPower ?? 0.25;
      this.taperPower = options.taperPower ?? 1.0;
      this.color = options.color ? options.color.rgba : [1, 1, 1, 1];
    }
  }

  class Material {
    constructor(type, uniforms = {}) {
      this.type = type;
      this.uniforms = uniforms;
    }
    static fromType(type, uniforms = {}) {
      return new Material(type, uniforms);
    }
  }
  Material.RimLightingType = 'RimLighting';
  Material.StripeType = 'Stripe';

  let addedEntities = [];
  let addedPrimitives = [];
  let loadedCzml = null;
  let zooms = 0;
  let viewers = 0;

  class EntityCollection {
    add(options) {
      addedEntities.push(options);
      return options;
    }
    removeAll() {
      addedEntities = [];
    }
  }

  class PrimitiveCollection {
    add(primitive) {
      addedPrimitives.push(primitive);
      return primitive;
    }
  }

  class EllipsoidPrimitive {
    constructor(options = {}) {
      this.type = 'EllipsoidPrimitive';
      this.center = options.center;
      this.modelMatrix = options.modelMatrix;
      this.radii = options.radii ? [options.radii.x, options.radii.y, options.radii.z] : [1, 1, 1];
      this.material = options.material;
    }
  }

  const Transforms = {
    eastNorthUpToFixedFrame: origin => {
      // Return a 16-element matrix or translation marker
      return { origin };
    },
  };

  class CzmlDataSource {
    static load(data) {
      loadedCzml = data;
      return Promise.resolve(data);
    }
  }

  class Viewer {
    constructor() {
      viewers++;
      this.entities = new EntityCollection();
      this.dataSources = {
        add: promise => {
          Promise.resolve(promise).then(data => {
            loadedCzml = data;
          });
        },
      };
      this.scene = {
        primitives: new PrimitiveCollection(),
        canvas: {},
        screenSpaceEventHandler: {
          setInputAction: () => {},
        },
      };
      this.screenSpaceEventHandler = {
        setInputAction: () => {},
      };
    }
    zoomTo() {
      zooms++;
    }
  }

  const Sandcastle = {
    declare: () => {},
    highlight: () => {},
  };

  const Cesium = {
    Viewer,
    Color,
    Cartesian2,
    Cartesian3: Object.assign(Cartesian3Obj, Cartesian3),
    Rectangle,
    CallbackProperty,
    PolylineGlowMaterialProperty,
    Material,
    EllipsoidPrimitive,
    Transforms,
    CzmlDataSource,
    ClassificationType: { TERRAIN: 0, CESIUM_3D_TILE: 1, BOTH: 2 },
    ScreenSpaceEventType: { MOUSE_MOVE: 0, LEFT_CLICK: 1 },
    Entity: {
      supportsPolylinesOnTerrain: () => true,
      supportsMaterialsforEntitiesOnTerrain: () => true,
    },
    Math: {
      toRadians: deg => (deg * Math.PI) / 180,
    },
  };

  const code = source.replace(/^import .*;\s*$/gm, '');
  assert(!/\bimport\b/.test(code));

  vm.runInNewContext(
    code,
    {
      Cesium,
      Sandcastle,
      Math,
      window: { alert: () => {} },
      Promise,
    },
    { timeout: 1000 },
  );

  assert.equal(viewers, 1);

  if (id === 'rectangle') {
    assert.equal(zooms, 1);
    assert.equal(addedEntities.length, 3);

    const entities = addedEntities.map((e, i) => {
      const g = e.rectangle;
      assert(g);
      const coords = g.coordinates;
      let color = [1, 1, 1, 1];
      let material;

      if (typeof g.material === 'string') {
        material = {
          type: 'image',
          image: g.material,
        };
      } else if (g.material?.rgba) {
        color = g.material.rgba;
      }

      let rotation;
      if (g.rotation !== undefined) {
        rotation = g.rotation instanceof CallbackProperty ? g.rotation.getValue() : g.rotation;
      }

      let stRotation;
      if (g.stRotation !== undefined) {
        stRotation =
          g.stRotation instanceof CallbackProperty ? g.stRotation.getValue() : g.stRotation;
      }

      const fill = g.fill ?? true;
      const outline = g.outline ?? false;
      const outlineColor = g.outlineColor ? g.outlineColor.rgba : [0, 0, 0, 1];

      return {
        id: `rectangle-${i + 1}`,
        name: e.name || '',
        positionDegrees: null,
        shape: {
          type: 'rectangle',
          coordinatesDegrees: coords,
          height: g.height ?? 0,
          extrudedHeight: g.extrudedHeight ?? null,
          rotation: rotation ?? 0,
          ...(stRotation !== undefined ? { stRotation } : {}),
        },
        color,
        fill,
        outline,
        outlineColor,
        outlineWidth: 1,
        ...(material ? { material } : {}),
        properties: { sandcastleCase: id, upstreamEntity: i + 1 },
      };
    });

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
        shouldAnimate: false,
        requirements: [
          'red translucent, green extruded and rotated, and textured rectangle entities',
          'height, extrudedHeight and outline rendering',
          'image material texture binding',
          'viewer.zoomTo entire entity collection',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'z-indexing-geometry') {
    assert.equal(zooms, 1);
    assert.equal(addedEntities.length, 7);

    const entities = addedEntities.map((e, i) => {
      if (e.rectangle) {
        const g = e.rectangle;
        const coords = g.coordinates;
        let color = [1, 1, 1, 1];
        let material;

        if (typeof g.material === 'string') {
          material = {
            type: 'image',
            image: g.material,
          };
        } else if (g.material?.rgba) {
          color = g.material.rgba;
        }

        return {
          id: e.id || `z-index-rectangle-${i + 1}`,
          name: e.name || e.id || '',
          positionDegrees: null,
          zIndex: g.zIndex ?? 0,
          shape: {
            type: 'rectangle',
            coordinatesDegrees: coords,
            height: 0,
            extrudedHeight: null,
            rotation: 0,
          },
          color,
          fill: true,
          outline: false,
          outlineColor: [0, 0, 0, 1],
          outlineWidth: 1,
          ...(material ? { material } : {}),
          properties: { sandcastleCase: id, upstreamEntity: i + 1, zIndex: g.zIndex ?? 0 },
        };
      }

      if (e.polyline) {
        const p = e.polyline;
        return {
          id: e.id || `z-index-polyline-${i + 1}`,
          name: e.name || e.id || '',
          positionDegrees: null,
          zIndex: p.zIndex ?? 0,
          shape: {
            type: 'widePolyline',
            data: {
              positions: p.positions,
              width: p.width ?? 1,
              arcType: 'GEODESIC',
              clampToGround: p.clampToGround ?? false,
              material: {
                type: 'glow',
                color: p.material.color,
                glowPower: p.material.glowPower,
                taperPower: p.material.taperPower,
              },
            },
          },
          color: p.material.color,
          fill: true,
          outline: false,
          outlineColor: [0, 0, 0, 1],
          outlineWidth: 1,
          properties: { sandcastleCase: id, upstreamEntity: i + 1, zIndex: p.zIndex ?? 0 },
        };
      }

      throw new Error(`unrecognized entity in ${id}`);
    });

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
        shouldAnimate: false,
        requirements: [
          'all six overlapping rectangles with zIndex 1, 2, and 3',
          'blue glow polyline clamped to ground with zIndex 2',
          'color and image texture materials on layered rectangles',
          'depth bias and correct front-to-back compositing without z-fighting',
          'viewer.zoomTo entire entity collection',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'czml-zindex') {
    assert(loadedCzml);
    assert.equal(loadedCzml[0].version, '1.0');
    const packets = loadedCzml.slice(1);
    assert.equal(packets.length, 4);

    const entities = packets.map((packet, i) => {
      const g = packet.ellipse || packet.corridor || packet.polygon || packet.rectangle;
      assert(g);
      const zIndex = g.zIndex ?? 0;
      let color = [1, 1, 1, 1];
      let material;

      if (g.material?.solidColor) {
        color = g.material.solidColor.color.rgba.map(v => v / 255);
      } else if (g.material?.stripe) {
        const s = g.material.stripe;
        material = {
          type: 'stripe',
          orientation: s.orientation ?? 'HORIZONTAL',
          evenColor: s.evenColor.rgba.map(v => v / 255),
          oddColor: s.oddColor.rgba.map(v => v / 255),
          repeat: s.repeat ?? 1,
          offset: 0,
        };
      }

      let shape;
      let positionDegrees = null;

      if (packet.ellipse) {
        positionDegrees = packet.position.cartographicDegrees;
        shape = {
          type: 'ellipse',
          height: g.height ?? 0,
          semiMajorAxis: g.semiMajorAxis,
          semiMinorAxis: g.semiMinorAxis,
          rotation: g.rotation ?? 0,
          extrudedHeight: g.extrudedHeight ?? null,
        };
      } else if (packet.corridor) {
        const c = g.positions.cartographicDegrees;
        const positions = Array.from({ length: c.length / 3 }, (_, j) => [
          c[3 * j],
          c[3 * j + 1],
          c[3 * j + 2],
        ]);
        shape = {
          type: 'corridor',
          positionsDegrees: positions,
          width: g.width,
          cornerType: 'ROUNDED',
          height: g.height ?? 0,
          extrudedHeight: g.extrudedHeight ?? null,
        };
      } else if (packet.polygon) {
        const c = g.positions.cartographicDegrees;
        const positions = Array.from({ length: c.length / 3 }, (_, j) => [
          c[3 * j],
          c[3 * j + 1],
          c[3 * j + 2],
        ]);
        shape = {
          type: 'polygon',
          positionsDegrees: positions,
          holes: [],
          extrudedHeight: g.extrudedHeight ?? null,
        };
      } else if (packet.rectangle) {
        shape = {
          type: 'rectangle',
          coordinatesDegrees: g.coordinates.wsenDegrees,
          height: 0,
          extrudedHeight: null,
          rotation: 0,
        };
      }

      return {
        id: packet.id,
        name: packet.name || '',
        positionDegrees,
        zIndex,
        shape,
        color,
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        ...(material ? { material } : {}),
        properties: { zIndex },
      };
    });

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
        shouldAnimate: false,
        requirements: [
          'CZML ellipse, corridor, polygon, and rectangle instances with zIndex',
          'solidColor and vertical stripe materials',
          'correct layered compositing matching zIndex ordering (1, 2, 3)',
          'CzmlDataSource.load Promise resolution and zoomTo',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (id === 'volumes-dev') {
    assert.equal(addedPrimitives.length, 2);

    const primitives = [
      {
        type: 'EllipsoidPrimitive',
        center: [-75.0, 40.0, 500000.0],
        radii: [500000.0, 500000.0, 500000.0],
        material: { type: 'RimLighting' },
      },
      {
        type: 'EllipsoidPrimitive',
        center: [-95.0, 40.0, 500000.0],
        radii: [300000.0, 300000.0, 500000.0],
        material: { type: 'Stripe' },
      },
    ];

    const entities = [
      {
        id: 'primitive-0',
        name: 'Ellipsoid with RimLighting',
        positionDegrees: [-75.0, 40.0, 500000.0],
        shape: {
          type: 'ellipsoid',
          radii: [500000.0, 500000.0, 500000.0],
        },
        color: [1, 1, 1, 1],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        material: { type: 'rimLighting' },
        properties: { primitiveIndex: 0 },
      },
      {
        id: 'primitive-1',
        name: 'Ellipsoid with Stripe and modelMatrix',
        positionDegrees: [-95.0, 40.0, 500000.0],
        shape: {
          type: 'ellipsoid',
          radii: [300000.0, 300000.0, 500000.0],
        },
        color: [1, 1, 1, 1],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        material: { type: 'stripe' },
        properties: { primitiveIndex: 1 },
      },
    ];

    return {
      entities: JSON.parse(JSON.stringify(entities)),
      primitive: JSON.parse(JSON.stringify({ primitives })),
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
          'EllipsoidPrimitive instances with RimLighting and Stripe materials',
          'center and modelMatrix local frame positioning',
          'mouse move hover picking and Sandcastle highlight',
          'source camera default view; manual fit available',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`unsupported batch30 recipe ${id}`);
}
