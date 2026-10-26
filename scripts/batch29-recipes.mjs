import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch29Ids = [
  'polyline-volume',
  'polyline-volume-dev',
  'polyline-volume-outline-dev',
  'czml-polyline-volume',
];

export function extractBatch29(id, source, item, hash) {
  const keys = (o, a) =>
    assert(
      Object.keys(o).every(k => a.includes(k)),
      `unmapped option ${Object.keys(o)}`,
    );

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
    fromDegreesArrayHeights: p =>
      Array.from({ length: p.length / 3 }, (_, i) => [
        p[3 * i],
        p[3 * i + 1],
        p[3 * i + 2],
      ]),
  };

  const CesiumMath = {
    toRadians: deg => (deg * Math.PI) / 180,
  };

  const geometry = (type, allowed) =>
    class {
      constructor(o) {
        keys(o, allowed);
        Object.assign(this, { type }, o);
      }
    };

  const PolylineVolumeGeometry = geometry('PolylineVolumeGeometry', [
    'polylinePositions',
    'shapePositions',
    'vertexFormat',
    'cornerType',
  ]);
  const PolylineVolumeOutlineGeometry = geometry(
    'PolylineVolumeOutlineGeometry',
    ['polylinePositions', 'shapePositions', 'cornerType'],
  );

  class GeometryInstance {
    constructor(o) {
      keys(o, ['geometry', 'attributes']);
      keys(o.attributes, ['color']);
      Object.assign(this, o);
    }
  }

  class PerInstanceColorAppearance {
    constructor(o = {}) {
      keys(o, ['flat', 'closed', 'translucent', 'renderState']);
      Object.assign(this, { type: 'PerInstanceColorAppearance' }, o);
    }
  }
  PerInstanceColorAppearance.VERTEX_FORMAT = 'POSITION_NORMAL';

  class Primitive {
    constructor(o) {
      keys(o, ['geometryInstances', 'appearance']);
      Object.assign(this, o);
    }
  }

  const records = [],
    primitives = [];
  let viewers = 0,
    zooms = 0,
    loadedCzml = null;

  class CzmlDataSource {
    static load(data) {
      loadedCzml = data;
      return Promise.resolve(data);
    }
  }

  class Viewer {
    constructor(container) {
      assert.equal(container, 'cesiumContainer');
      viewers++;
      this.entities = {
        add: o => {
          records.push(o);
          return o;
        },
      };
      this.scene = {
        maximumAliasedLineWidth: 1,
        primitives: {
          add: p => {
            assert(p instanceof Primitive);
            primitives.push(p);
            return p;
          },
        },
      };
      this.dataSources = {
        add: p => {
          assert(p instanceof Promise);
        },
      };
    }
    zoomTo(v) {
      zooms++;
    }
  }

  const CornerType = {
    ROUNDED: 'ROUNDED',
    MITERED: 'MITERED',
    BEVELED: 'BEVELED',
  };

  const Cesium = {
    Viewer,
    Cartesian2,
    Cartesian3,
    Color,
    Math: CesiumMath,
    PolylineVolumeGeometry,
    PolylineVolumeOutlineGeometry,
    GeometryInstance,
    PerInstanceColorAppearance,
    Primitive,
    CornerType,
    CzmlDataSource,
    ColorGeometryInstanceAttribute: {
      fromColor: c => c.rgba.map(v => (v === 1 ? 255 : Math.floor(v * 256)) / 255),
    },
  };

  const code = source.replace(/^import .*;\s*$/gm, '');
  assert(!/\bimport\b/.test(code));
  vm.runInNewContext(code, { Cesium, Math }, { timeout: 1000 });
  assert.equal(viewers, 1);

  const isEntity = id === 'polyline-volume';
  const isCzml = id === 'czml-polyline-volume';
  const isPrimitive = id.endsWith('-dev');

  if (isEntity) {
    assert.equal(zooms, 1);
    assert.equal(records.length, 3);
    const entities = records.map((e, i) => {
      keys(e, ['name', 'polylineVolume']);
      const g = e.polylineVolume;
      keys(g, ['positions', 'shape', 'cornerType', 'material', 'outline', 'outlineColor']);
      const shapePositions = g.shape.map(p => [p.x, p.y]);
      return {
        id: `${id}-${i + 1}`,
        name: e.name,
        positionDegrees: null,
        shape: {
          type: 'polylineVolume',
          positionsDegrees: g.positions,
          shapePositions,
          cornerType: g.cornerType ?? 'ROUNDED',
        },
        color: g.material.rgba,
        fill: true,
        outline: g.outline ?? false,
        outlineColor: g.outlineColor?.rgba || [0, 0, 0, 1],
        outlineWidth: 1,
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
          'all three original polyline volume instances (circle tube, square box, star)',
          'rounded, beveled, and mitered corner topology',
          'materials with alpha, fill and outline controls',
          'viewer.zoomTo entire entity collection',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (isPrimitive) {
    assert.equal(primitives.length, 1);
    const p = primitives[0];
    const isOutline = id === 'polyline-volume-outline-dev';
    const instances = [p.geometryInstances].flat();
    assert.equal(instances.length, isOutline ? 2 : 3);
    const entities = instances.map((ins, ii) => {
      const g = ins.geometry;
      const outline = g.type === 'PolylineVolumeOutlineGeometry';
      const a = p.appearance;
      const shapePositions = g.shapePositions.map(pt => [pt.x, pt.y]);
      return {
        id: `primitive-0-instance-${ii}`,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'polylineVolume',
          positionsDegrees: g.polylinePositions,
          shapePositions,
          cornerType: g.cornerType ?? 'ROUNDED',
        },
        color: outline ? [1, 1, 1, 1] : ins.attributes.color,
        fill: !outline,
        outline,
        outlineColor: outline ? ins.attributes.color : [0, 0, 0, 1],
        outlineWidth: 1,
        ...(!outline
          ? {
              surfaceAppearance: {
                flat: a.flat ?? false,
                closed: a.closed ?? false,
                translucent: a.translucent ?? false,
              },
            }
          : {}),
        properties: { primitiveIndex: 0, instanceIndex: ii },
      };
    });

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
          'all original geometry instances and appearance options',
          isOutline
            ? 'PolylineVolumeOutlineGeometry with mitered and rounded corners'
            : 'PolylineVolumeGeometry with tube, box and star shapes',
          'per-instance colors and appearance states',
          'source camera default view; manual fit available',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  if (isCzml) {
    assert(loadedCzml);
    assert.equal(zooms, 1);
    const packets = loadedCzml.slice(1);
    assert.equal(packets.length, 2);
    const entities = packets.map((packet, i) => {
      const pv = packet.polylineVolume;
      assert(pv);
      const coords = pv.positions.cartographicDegrees;
      const positionsDegrees = Array.from({ length: coords.length / 3 }, (_, j) => [
        coords[3 * j],
        coords[3 * j + 1],
        coords[3 * j + 2],
      ]);
      const shapeFlat = pv.shape.cartesian2;
      const shapePositions = Array.from({ length: shapeFlat.length / 2 }, (_, j) => [
        shapeFlat[2 * j],
        shapeFlat[2 * j + 1],
      ]);
      const color = pv.material.solidColor.color.rgba.map(v => v / 255);
      const outlineColor = pv.outlineColor
        ? pv.outlineColor.rgba.map(v => v / 255)
        : [0, 0, 0, 1];
      return {
        id: packet.id,
        name: packet.name || '',
        positionDegrees: null,
        shape: {
          type: 'polylineVolume',
          positionsDegrees,
          shapePositions,
          cornerType: pv.cornerType || 'ROUNDED',
        },
        color,
        fill: true,
        outline: pv.outline ?? false,
        outlineColor,
        outlineWidth: 1,
        properties: null,
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
          'CZML polylineVolume packet decoding and validation',
          'beveled box and mitered star volume instances',
          'solidColor material with alpha and outline controls',
          'CzmlDataSource.load Promise resolution and zoomTo',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
    };
  }

  throw new Error(`unsupported batch29 recipe ${id}`);
}
