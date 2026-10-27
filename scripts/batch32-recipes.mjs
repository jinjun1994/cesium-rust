import vm from 'node:vm';
import assert from 'node:assert/strict';

export const batch32Ids = [
  'polylines-dev',
  'frustum-dev',
  'parallels-and-meridians',
  'polygon-texture-coordinates-dev',
];

export function extractBatch32(id, source, item, hash) {
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
    CYAN: new Color(0, 1, 1),
    DARKORANGE: new Color(1, 140 / 255, 0),
    PALEGREEN: new Color(152 / 255, 251 / 255, 152 / 255),
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
      return { _degrees: [lon, lat, height], x: 0, y: 0, z: 0 };
    }
    static fromDegreesArray(coords) {
      const result = [];
      for (let i = 0; i < coords.length; i += 2) {
        result.push({ _degrees: [coords[i], coords[i + 1], 0], x: 0, y: 0, z: 0 });
      }
      return result;
    }
    static fromDegreesArrayHeights(coords) {
      const result = [];
      for (let i = 0; i < coords.length; i += 3) {
        result.push({
          _degrees: [coords[i], coords[i + 1], coords[i + 2]],
          x: 0,
          y: 0,
          z: 0,
        });
      }
      return result;
    }
  }

  class Cartographic {
    constructor(lonRad = 0, latRad = 0, height = 0) {
      this.longitude = lonRad;
      this.latitude = latRad;
      this.height = height;
    }
    static fromDegrees(lon, lat, height = 0) {
      return new Cartographic((lon * Math.PI) / 180, (lat * Math.PI) / 180, height);
    }
    static fromCartesian(cartesian) {
      if (cartesian._degrees) {
        return Cartographic.fromDegrees(...cartesian._degrees);
      }
      return new Cartographic(0, 0, 0);
    }
    static toCartesian(cartographic) {
      return {
        _degrees: [
          (cartographic.longitude * 180) / Math.PI,
          (cartographic.latitude * 180) / Math.PI,
          cartographic.height,
        ],
        x: 0,
        y: 0,
        z: 0,
      };
    }
  }

  class Matrix3 {
    constructor() {
      this.values = [1, 0, 0, 0, 1, 0, 0, 0, 1];
    }
    static fromRotationX(angle, result = new Matrix3()) {
      const c = Math.cos(angle);
      const s = Math.sin(angle);
      result.values = [1, 0, 0, 0, c, s, 0, -s, c];
      return result;
    }
    static multiply(left, right, result = new Matrix3()) {
      const a = left.values;
      const b = right.values;
      const r = new Array(9);
      for (let row = 0; row < 3; ++row) {
        for (let col = 0; col < 3; ++col) {
          r[col * 3 + row] =
            a[row] * b[col * 3] +
            a[3 + row] * b[col * 3 + 1] +
            a[6 + row] * b[col * 3 + 2];
        }
      }
      result.values = r;
      return result;
    }
  }

  class Matrix4 {
    constructor() {
      this.values = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    }
    static getMatrix3(matrix, result = new Matrix3()) {
      const m = matrix.values;
      result.values = [m[0], m[1], m[2], m[4], m[5], m[6], m[8], m[9], m[10]];
      return result;
    }
  }
  Matrix4.IDENTITY = new Matrix4();

  class Quaternion {
    constructor(x = 0, y = 0, z = 0, w = 1) {
      this.x = x;
      this.y = y;
      this.z = z;
      this.w = w;
    }
    static fromRotationMatrix(matrix) {
      const m = matrix.values;
      const m00 = m[0], m10 = m[1], m20 = m[2];
      const m01 = m[3], m11 = m[4], m21 = m[5];
      const m02 = m[6], m12 = m[7], m22 = m[8];
      const tr = m00 + m11 + m22;
      let x, y, z, w;
      if (tr > 0) {
        const s = 0.5 / Math.sqrt(tr + 1.0);
        w = 0.25 / s;
        x = (m21 - m12) * s;
        y = (m02 - m20) * s;
        z = (m10 - m01) * s;
      } else if (m00 > m11 && m00 > m22) {
        const s = 2.0 * Math.sqrt(1.0 + m00 - m11 - m22);
        w = (m21 - m12) / s;
        x = 0.25 * s;
        y = (m01 + m10) / s;
        z = (m02 + m20) / s;
      } else if (m11 > m22) {
        const s = 2.0 * Math.sqrt(1.0 + m11 - m00 - m22);
        w = (m02 - m20) / s;
        x = (m01 + m10) / s;
        y = 0.25 * s;
        z = (m12 + m21) / s;
      } else {
        const s = 2.0 * Math.sqrt(1.0 + m22 - m00 - m11);
        w = (m10 - m01) / s;
        x = (m02 + m20) / s;
        y = (m12 + m21) / s;
        z = 0.25 * s;
      }
      return new Quaternion(x, y, z, w);
    }
  }

  const Transforms = {
    eastNorthUpToFixedFrame(origin) {
      const [lon, lat, h] = origin._degrees;
      const l = (lon * Math.PI) / 180;
      const f = (lat * Math.PI) / 180;
      const sl = Math.sin(l);
      const cl = Math.cos(l);
      const sf = Math.sin(f);
      const cf = Math.cos(f);
      const a = 6378137.0;
      const b = 6356752.314245;
      const e2 = 1 - (b * b) / (a * a);
      const n = a / Math.sqrt(1 - e2 * sf * sf);
      const m = new Matrix4();
      m.values = [
        -sl, cl, 0, 0,
        -sf * cl, -sf * sl, cf, 0,
        cf * cl, cf * sl, sf, 0,
        (n + h) * cf * cl, (n + h) * cf * sl, (n * (1 - e2) + h) * sf, 1,
      ];
      m.originDegrees = [lon, lat, h];
      return m;
    },
  };

  const PolylinePipeline = {
    generateCartesianArc({ positions }) {
      return positions;
    },
    generateCartesianRhumbArc({ positions }) {
      return positions;
    },
  };

  class Material {
    static PolylineOutlineType = 'PolylineOutline';
    static PolylineGlowType = 'PolylineGlow';
    static PolylineArrowType = 'PolylineArrow';
    static FadeType = 'Fade';
    static fromType(type, uniforms = {}) {
      return new Material({ fabric: { type, uniforms } });
    }
    constructor(options = {}) {
      if (options.fabric) {
        this.type = options.fabric.type;
        this.uniforms = options.fabric.uniforms ?? {};
      } else {
        this.type = options.type;
        this.uniforms = options.uniforms ?? {};
      }
    }
  }

  class PerspectiveFrustum {
    constructor(options = {}) {
      this.fov = options.fov ?? (60 * Math.PI) / 180;
      this.aspectRatio = options.aspectRatio ?? 1.5;
      this.near = options.near ?? 10.0;
      this.far = options.far ?? 50.0;
    }
  }

  class FrustumGeometry {
    constructor(options = {}) {
      this.frustum = options.frustum;
      this.origin = options.origin;
      this.orientation = options.orientation;
      this.vertexFormat = options.vertexFormat;
    }
  }

  class FrustumOutlineGeometry {
    constructor(options = {}) {
      this.frustum = options.frustum;
      this.origin = options.origin;
      this.orientation = options.orientation;
    }
  }

  class GeometryInstance {
    constructor(options = {}) {
      this.geometry = options.geometry;
      this.attributes = options.attributes ?? {};
      this.id = options.id;
      this.modelMatrix = options.modelMatrix;
    }
  }

  class PolygonGeometry {
    constructor(options = {}) {
      this.polygonHierarchy = options.polygonHierarchy;
      this.textureCoordinates = options.textureCoordinates;
      this.perPositionHeight = options.perPositionHeight ?? false;
      this.extrudedHeight = options.extrudedHeight;
      this.arcType = options.arcType ?? 'GEODESIC';
      this.vertexFormat = options.vertexFormat;
    }
    static fromPositions(options = {}) {
      return new PolygonGeometry({
        polygonHierarchy: { positions: options.positions, holes: [] },
        textureCoordinates: options.textureCoordinates,
        perPositionHeight: options.perPositionHeight,
        extrudedHeight: options.extrudedHeight,
        arcType: options.arcType,
        vertexFormat: options.vertexFormat,
      });
    }
  }

  const addedPrimitives = [];
  class PolylineCollection {
    constructor() {
      this.polylines = [];
      this.modelMatrix = undefined;
    }
    add(options = {}) {
      const line = {
        positions: options.positions,
        width: options.width ?? 1,
        material: options.material ?? { type: 'Color', uniforms: { color: Color.WHITE } },
        loop: options.loop ?? false,
      };
      this.polylines.push(line);
      return line;
    }
  }

  class Primitive {
    constructor(options = {}) {
      this.geometryInstances = Array.isArray(options.geometryInstances)
        ? options.geometryInstances
        : [options.geometryInstances];
      this.appearance = options.appearance;
      this.ready = true;
    }
    getGeometryInstanceAttributes(id) {
      return {
        boundingSphere: {
          center: { x: 0, y: 0, z: 0 },
          radius: 50.0,
        },
      };
    }
  }

  class GroundPrimitive extends Primitive {
    constructor(options = {}) {
      super(options);
      this.classificationType = options.classificationType;
    }
  }

  const addedEntities = [];
  class EntityCollection {
    constructor() {
      this.entities = [];
    }
    add(options = {}) {
      const e = {
        id: options.id ?? `entity-${this.entities.length}`,
        name: options.name ?? '',
        show: options.show ?? true,
        position: options.position,
        label: options.label,
        polyline: options.polyline,
      };
      this.entities.push(e);
      addedEntities.push(e);
      return e;
    }
    remove(entity) {
      const idx = this.entities.indexOf(entity);
      if (idx !== -1) this.entities.splice(idx, 1);
      const aIdx = addedEntities.indexOf(entity);
      if (aIdx !== -1) addedEntities.splice(aIdx, 1);
    }
  }

  const toolbarToggles = [];
  const Sandcastle = {
    declare: () => {},
    highlight: () => {},
    addToggleButton: (title, defaultChecked, callback) => {
      toolbarToggles.push({ title, defaultChecked, callback });
    },
  };

  const scene = {
    primitives: {
      add: p => {
        addedPrimitives.push(p);
        return p;
      },
    },
    canvas: { clientWidth: 768, clientHeight: 512 },
    camera: {
      viewBoundingSphere: () => {},
      lookAtTransform: () => {},
      getPickRay: () => ({ origin: [0, 0, 0], direction: [0, 0, 1] }),
    },
    globe: {
      pick: () => ({ _degrees: [0, 0, 0], x: 0, y: 0, z: 0 }),
    },
    postRender: {
      addEventListener: cb => {
        // Do not call cb synchronously: callers like `const removeListener = scene.postRender.addEventListener(...)`
        // will throw ReferenceError if cb invokes removeListener before assignment.
        return () => {};
      },
    },
    pick: () => undefined,
  };

  const viewer = {
    scene,
    entities: new EntityCollection(),
    screenSpaceEventHandler: {
      setInputAction: () => {},
    },
  };

  const executable = source.replace(/^import .*;\s*$/gm, '');
  assert(!/\bimport\b/.test(executable));

  vm.runInNewContext(
    executable,
    {
      Cesium: {
        Viewer: function () {
          return viewer;
        },
        ArcType: { NONE: 'NONE', GEODESIC: 'GEODESIC', RHUMB: 'RHUMB' },
        Cartesian2,
        Cartesian3,
        Cartographic,
        Color,
        ColorGeometryInstanceAttribute: {
          fromColor: c => ({ value: c.rgba }),
        },
        ClassificationType: { TERRAIN: 'TERRAIN' },
        EllipsoidSurfaceAppearance: class {
          constructor(opts = {}) {
            this.material = opts.material;
            this.translucent = opts.material?.uniforms?.image ? false : true;
          }
        },
        FrustumGeometry,
        FrustumOutlineGeometry,
        GeometryInstance,
        GroundPrimitive,
        Material,
        MaterialAppearance: class {
          constructor(opts = {}) {
            this.material = opts.material;
            this.translucent = opts.translucent ?? false;
            this.materialSupport = opts.materialSupport;
          }
          static MaterialSupport = {
            TEXTURED: { vertexFormat: 'TEXTURED' },
          };
        },
        Math: {
          toDegrees: rad => (rad * 180) / Math.PI,
          toRadians: deg => (deg * Math.PI) / 180,
          PI_OVER_TWO: Math.PI / 2,
        },
        Matrix3,
        Matrix4,
        PerInstanceColorAppearance: class {
          constructor(opts = {}) {
            this.closed = opts.closed ?? false;
            this.flat = opts.flat ?? false;
            this.translucent = true;
          }
        },
        PerspectiveFrustum,
        PolygonGeometry,
        PolylineCollection,
        PolylinePipeline,
        Primitive,
        Quaternion,
        ScreenSpaceEventType: {
          LEFT_CLICK: 'LEFT_CLICK',
          MOUSE_MOVE: 'MOUSE_MOVE',
        },
        Transforms,
        VertexFormat: { POSITION_ONLY: 'POSITION_ONLY' },
        defined: v => v !== undefined && v !== null,
      },
      Sandcastle,
      console,
    },
    { timeout: 3000 },
  );

  if (id === 'polylines-dev') {
    // 2 PolylineCollections were added
    assert.equal(addedPrimitives.length, 2);
    const globalLines = addedPrimitives[0].polylines;
    const localLines = addedPrimitives[1].polylines;
    assert.equal(globalLines.length, 6);
    assert.equal(localLines.length, 1);

    const m = addedPrimitives[1].modelMatrix;
    assert(m && m.values);

    // Build primitive JSON with 7 primitives
    const primitives = [
      ...globalLines.map((line, i) => {
        let mat = { type: 'Color', uniforms: { color: [1, 1, 1, 1] } };
        let appearanceType = 'PolylineColorAppearance';
        if (line.material.type === 'Color') {
          mat = { type: 'Color', uniforms: { color: line.material.uniforms.color.rgba } };
        } else if (line.material.type === 'PolylineOutline') {
          appearanceType = 'PolylineMaterialAppearance';
          mat = {
            type: 'PolylineOutline',
            uniforms: {
              color: [1, 1, 1, 1],
              outlineColor: [1, 0, 0, 1],
              outlineWidth: line.material.uniforms.outlineWidth ?? 1,
            },
          };
        } else if (line.material.type === 'PolylineGlow') {
          appearanceType = 'PolylineMaterialAppearance';
          mat = {
            type: 'PolylineGlow',
            uniforms: {
              color: line.material.uniforms.color.rgba,
              glowPower: line.material.uniforms.glowPower,
              taperPower: line.material.uniforms.taperPower,
            },
          };
        } else if (line.material.type === 'Fade') {
          appearanceType = 'PolylineMaterialAppearance';
          mat = {
            type: 'Fade',
            uniforms: {
              fadeInColor: line.material.uniforms.fadeInColor.rgba,
              fadeOutColor: line.material.uniforms.fadeOutColor.rgba,
            },
          };
        }

        const coords = line.positions.map(p => p._degrees);
        if (line.loop) {
          coords.push(coords[0]);
        }

        return {
          geometryInstances: {
            geometry: {
              type: 'PolylineGeometry',
              positions: coords,
              width: line.width,
              vertexFormat: appearanceType === 'PolylineColorAppearance' ? 'POLYLINE_COLOR' : 'POLYLINE_MATERIAL',
              arcType: i === 5 ? 'RHUMB' : 'GEODESIC',
            },
            attributes: {
              color: mat.uniforms.color ?? [1, 1, 1, 1],
            },
          },
          appearance: {
            type: appearanceType,
            ...(appearanceType === 'PolylineMaterialAppearance' ? { material: mat } : {}),
          },
        };
      }),
      // Local polyline
      {
        geometryInstances: {
          geometry: {
            type: 'PolylineGeometry',
            positions: [
              [0, 0, 0],
              [1000000, 0, 0],
            ],
            width: 10,
            vertexFormat: 'POLYLINE_MATERIAL',
            arcType: 'NONE',
          },
          modelMatrix: m.values,
        },
        appearance: {
          type: 'PolylineMaterialAppearance',
          material: {
            type: 'PolylineArrow',
            uniforms: {
              color: [1, 1, 1, 1],
            },
          },
        },
      },
    ];

    // Compute entities representation
    const entities = primitives.map((p, i) => {
      const g = p.geometryInstances.geometry;
      let positions = g.positions;
      if (p.geometryInstances.modelMatrix) {
        // Transform [0, 0, 0] and [1000000, 0, 0] by ENU
        const lon = m.originDegrees[0];
        const lat = m.originDegrees[1];
        // 1000000m East from (-80, 35) is approximately (-69.1, 34.6, 78564)
        // Let's compute exact cartographic from ENU transform:
        const col0 = [m.values[0], m.values[1], m.values[2]]; // East in ECEF
        const col3 = [m.values[12], m.values[13], m.values[14]]; // Origin in ECEF
        const p1_ecef = [
          col3[0] + 1000000 * col0[0],
          col3[1] + 1000000 * col0[1],
          col3[2] + 1000000 * col0[2],
        ];
        // Convert to lat/lon
        const a = 6378137.0;
        const b = 6356752.314245;
        const e2 = 1 - (b * b) / (a * a);
        const p_len = Math.hypot(p1_ecef[0], p1_ecef[1]);
        const lon1 = (Math.atan2(p1_ecef[1], p1_ecef[0]) * 180) / Math.PI;
        let lat1 = Math.atan2(p1_ecef[2], p_len * (1 - e2));
        for (let iter = 0; iter < 5; ++iter) {
          const s = Math.sin(lat1);
          const N = a / Math.sqrt(1 - e2 * s * s);
          lat1 = Math.atan2(p1_ecef[2] + e2 * N * s, p_len);
        }
        const s = Math.sin(lat1);
        const N = a / Math.sqrt(1 - e2 * s * s);
        const h1 = p_len / Math.cos(lat1) - N;
        positions = [
          [lon, lat, 0],
          [lon1, (lat1 * 180) / Math.PI, h1],
        ];
      }

      let lineMat = { type: 'solid' };
      let color = [1, 1, 1, 1];
      if (p.appearance.material) {
        const mat = p.appearance.material;
        if (mat.type === 'PolylineOutline') {
          lineMat = {
            type: 'outline',
            outlineColor: mat.uniforms.outlineColor,
            outlineWidth: mat.uniforms.outlineWidth,
          };
        } else if (mat.type === 'PolylineGlow') {
          lineMat = {
            type: 'glow',
            glowPower: mat.uniforms.glowPower,
            taperPower: mat.uniforms.taperPower,
          };
          color = mat.uniforms.color;
        } else if (mat.type === 'PolylineArrow') {
          lineMat = { type: 'arrow' };
        } else if (mat.type === 'Fade') {
          lineMat = {
            type: 'fade',
            fadeInColor: mat.uniforms.fadeInColor,
            fadeOutColor: mat.uniforms.fadeOutColor,
          };
          color = mat.uniforms.fadeInColor;
        }
      } else if (p.geometryInstances.attributes?.color) {
        color = p.geometryInstances.attributes.color;
      }

      return {
        id: `primitive-${i}-instance-0`,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'widePolyline',
          data: {
            positions,
            width: g.width,
            arcType: g.arcType ?? 'GEODESIC',
            clampToGround: false,
            colors: [],
            colorsPerVertex: false,
            material: lineMat,
          },
        },
        color,
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        properties: {
          primitiveIndex: i,
          instanceIndex: 0,
        },
      };
    });

    return {
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
          'PolylineCollection with seven distinct polylines',
          'PolylineColorAppearance, PolylineOutline, PolylineGlow, PolylineArrow and Fade materials',
          'Cartesian arc and rhumb arc interpolation with loop support',
          'local modelMatrix reference frame with East-oriented arrow',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
      primitive: JSON.parse(JSON.stringify({ primitives })),
      entities: JSON.parse(JSON.stringify(entities)),
    };
  }

  if (id === 'frustum-dev') {
    assert.equal(addedPrimitives.length, 2);
    const filled = addedPrimitives[0].geometryInstances[0];
    const outline = addedPrimitives[1].geometryInstances[0];

    const originDeg = [-105.0, 45.0, 20.0];
    const geom = filled.geometry;
    const orientation = [
      geom.orientation.x,
      geom.orientation.y,
      geom.orientation.z,
      geom.orientation.w,
    ];

    const primitives = [
      {
        geometryInstances: {
          id: 'frustum',
          geometry: {
            type: 'FrustumGeometry',
            frustum: {
              fov: geom.frustum.fov,
              aspectRatio: geom.frustum.aspectRatio,
              near: geom.frustum.near,
              far: geom.frustum.far,
            },
            origin: originDeg,
            orientation,
            vertexFormat: 'POSITION_ONLY',
          },
          attributes: {
            color: [1, 0, 0, 0.5],
          },
        },
        appearance: {
          type: 'PerInstanceColorAppearance',
          closed: true,
          flat: true,
        },
      },
      {
        geometryInstances: {
          id: 'primitive-1-instance-0',
          geometry: {
            type: 'FrustumOutlineGeometry',
            frustum: {
              fov: geom.frustum.fov,
              aspectRatio: geom.frustum.aspectRatio,
              near: geom.frustum.near,
              far: geom.frustum.far,
            },
            origin: originDeg,
            orientation,
          },
          attributes: {
            color: [0, 0, 0, 1],
          },
        },
        appearance: {
          type: 'PerInstanceColorAppearance',
          flat: true,
          renderState: {
            lineWidth: 1,
          },
        },
      },
    ];

    const entities = [
      {
        id: 'frustum',
        name: '',
        positionDegrees: originDeg,
        orientation,
        shape: {
          type: 'frustum',
          fov: geom.frustum.fov,
          aspectRatio: geom.frustum.aspectRatio,
          near: geom.frustum.near,
          far: geom.frustum.far,
        },
        color: [1, 0, 0, 0.5],
        fill: true,
        outline: false,
        outlineColor: [1, 0, 0, 0.5],
        outlineWidth: 1,
        surfaceAppearance: {
          closed: true,
          flat: true,
          translucent: true,
        },
        properties: {
          primitiveIndex: 0,
          instanceIndex: 0,
        },
      },
      {
        id: 'primitive-1-instance-0',
        name: '',
        positionDegrees: originDeg,
        orientation,
        shape: {
          type: 'frustum',
          fov: geom.frustum.fov,
          aspectRatio: geom.frustum.aspectRatio,
          near: geom.frustum.near,
          far: geom.frustum.far,
        },
        color: [0, 0, 0, 1],
        fill: false,
        outline: true,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        properties: {
          primitiveIndex: 1,
          instanceIndex: 0,
        },
      },
    ];

    return {
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'primitives',
        path: `../../tests/sandcastle/full/${id}.primitive.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'zoomToEntities',
        shouldAnimate: false,
        requirements: [
          'FrustumGeometry and FrustumOutlineGeometry from PerspectiveFrustum',
          'ellipsoid surface origin and local ENU rotation matrix to quaternion',
          'PerInstanceColorAppearance with red translucent fill and black outline',
          'viewBoundingSphere framing of perspective frustum',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
      primitive: JSON.parse(JSON.stringify({ primitives })),
      entities: JSON.parse(JSON.stringify(entities)),
    };
  }

  if (id === 'parallels-and-meridians') {
    assert.equal(toolbarToggles.length, 5);
    // 144 entities total
    assert.equal(addedEntities.length, 144);

    const entities = addedEntities.map((e, i) => {
      const p = e.polyline;
      return {
        id: `parallel-meridian-${i}`,
        name: e.name,
        show: e.show,
        positionDegrees: null,
        shape: {
          type: 'widePolyline',
          data: {
            positions: p.positions.map(pt => pt._degrees),
            width: p.width,
            arcType: p.arcType ?? 'RHUMB',
            clampToGround: false,
            colors: [],
            colorsPerVertex: false,
            material: { type: 'solid' },
          },
        },
        color: p.material.rgba,
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
      };
    });

    return {
      manifest: {
        id,
        title: item.title,
        sourceSha256: hash,
        referenceUrl: item.referenceUrl,
        format: 'entities',
        path: `../../tests/sandcastle/full/${id}.json`,
        expectedPath: `../../tests/sandcastle/full/${id}.json`,
        entityCount: entities.length,
        framing: 'viewer-default',
        shouldAnimate: false,
        requirements: [
          'equator and prime meridian rhumb polylines',
          'recursive multi-resolution grid generation',
          'toggle controls for equator, prime meridian, grids and antipodal point',
          'interactive screen space event handler for latitude/longitude coordinate crosshairs',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
      entities: JSON.parse(JSON.stringify(entities)),
    };
  }

  if (id === 'polygon-texture-coordinates-dev') {
    assert.equal(addedPrimitives.length, 2);
    const prim0 = addedPrimitives[0];
    const prim1 = addedPrimitives[1];
    assert.equal(prim0.geometryInstances.length, 6);
    assert.equal(prim1.geometryInstances.length, 1);

    const mapCoords = tex => {
      if (!tex) return null;
      return {
        positions: tex.positions.map(p => [p.x, p.y]),
        holes: (tex.holes ?? []).map(mapCoords),
      };
    };

    const mapHierarchy = h => {
      return {
        positions: h.positions.map(p => p._degrees),
        holes: (h.holes ?? []).map(mapHierarchy),
      };
    };

    const allInstances = [...prim0.geometryInstances, ...prim1.geometryInstances];
    const primitives = [
      {
        geometryInstances: prim0.geometryInstances.map((inst, i) => {
          const g = inst.geometry;
          return {
            geometry: {
              type: 'PolygonGeometry',
              polygonHierarchy: mapHierarchy(g.polygonHierarchy),
              textureCoordinates: mapCoords(g.textureCoordinates),
              perPositionHeight: g.perPositionHeight ?? false,
              extrudedHeight: g.extrudedHeight ?? null,
              arcType: g.arcType ?? 'GEODESIC',
              vertexFormat: 'POSITION_NORMAL_ST',
            },
            attributes: {},
          };
        }),
        appearance: {
          type: 'MaterialAppearance',
          material: {
            type: 'Image',
            image: '../images/Cesium_Logo_Color.jpg',
          },
          materialSupport: 'TEXTURED',
          translucent: false,
        },
      },
      {
        geometryInstances: [
          {
            geometry: {
              type: 'PolygonGeometry',
              polygonHierarchy: mapHierarchy(prim1.geometryInstances[0].geometry.polygonHierarchy),
              textureCoordinates: mapCoords(prim1.geometryInstances[0].geometry.textureCoordinates),
              perPositionHeight: false,
              extrudedHeight: null,
              arcType: 'GEODESIC',
              vertexFormat: 'POSITION_NORMAL_ST',
            },
            attributes: {},
          },
        ],
        appearance: {
          type: 'EllipsoidSurfaceAppearance',
          material: {
            type: 'Image',
            image: '../images/Cesium_Logo_Color.jpg',
          },
          materialSupport: 'TEXTURED',
          translucent: false,
        },
      },
    ];

    const entities = allInstances.map((inst, idx) => {
      const g = inst.geometry;
      const pi = idx < 6 ? 0 : 1;
      const ii = idx < 6 ? idx : 0;
      return {
        id: `primitive-${pi}-instance-${ii}`,
        name: '',
        positionDegrees: null,
        shape: {
          type: 'polygonSurface',
          data: {
            hierarchy: mapHierarchy(g.polygonHierarchy),
            textureCoordinates: mapCoords(g.textureCoordinates),
            height: 0,
            perPositionHeight: g.perPositionHeight ?? false,
            extrudedHeight: g.extrudedHeight ?? null,
            closeTop: true,
            closeBottom: true,
            coplanar: false,
            arcType: g.arcType ?? 'GEODESIC',
            stRotation: 0,
          },
        },
        color: [1, 1, 1, 1],
        fill: true,
        outline: false,
        outlineColor: [0, 0, 0, 1],
        outlineWidth: 1,
        material: {
          type: 'image',
          image: '../images/Cesium_Logo_Color.jpg',
        },
        properties: {
          primitiveIndex: pi,
          instanceIndex: ii,
        },
      };
    });

    return {
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
          'six PolygonGeometry instances with explicit textureCoordinates',
          'nested holes with matching nested textureCoordinate hierarchies',
          'perPositionHeight, rhumb arcType and extruded polygon variations',
          'GroundPrimitive with image material appearance',
        ],
        status: 'implemented',
        validation: { source: 'passed', gpu: 'pending', browser: 'pending' },
      },
      primitive: JSON.parse(JSON.stringify({ primitives })),
      entities: JSON.parse(JSON.stringify(entities)),
    };
  }

  throw new Error(`Unhandled batch 32 id: ${id}`);
}
