import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. Earth at Night
function generateEarthAtNight() {
  const tokyoBeacon = {
    id: 'night-lights-tokyo',
    name: 'Tokyo Greater Metropolitan Night Luminescence',
    show: true,
    positionDegrees: [139.767, 35.681, 25000.0],
    shape: {
      type: 'cylinder',
      length: 50000.0,
      topRadius: 15000.0,
      bottomRadius: 15000.0,
    },
    color: [1.0, 0.95, 0.4, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3812, metropolitanCluster: 'Kanto Plain', nightLuminanceNadir: 98.5 },
  };

  const nileSpine = {
    id: 'night-lights-nile',
    name: 'Nile Valley Illuminated Settlement Corridor',
    show: true,
    positionDegrees: [31.235, 30.044, 20000.0],
    shape: {
      type: 'box',
      dimensions: [30000.0, 30000.0, 30000.0],
    },
    color: [1.0, 0.8, 0.3, 0.85],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { geographicCorridor: 'Lower Nile', populationDensityKm2: 1500 },
  };

  const auroraBeacon = {
    id: 'night-lights-aurora',
    name: 'Sub-Arctic Aurora Belt Sky Observation Point',
    show: true,
    positionDegrees: [15.0, 68.0, 35000.0],
    shape: {
      type: 'ellipsoid',
      radii: [25000.0, 25000.0, 25000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 1.0, 0.6, 0.8],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { geomagneticLatitude: 65.2, auroraProbability: 0.88 },
  };

  const equatorBand = {
    id: 'night-lights-equator-band',
    name: 'Gulf of Guinea Tropical Night Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-5.0, -3.0, 5.0, 3.0],
      height: 5000.0,
      extrudedHeight: 10000.0,
      rotation: 0.0,
    },
    color: [0.3, 0.5, 0.9, 0.6],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { equatorialSector: 'Atlantic Transition', dayAlpha: 0.0, nightAlpha: 1.0 },
  };

  const entities = [tokyoBeacon, nileSpine, auroraBeacon, equatorBand];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Black Marble 2017 Night Lights (Global Night Hemispheric Luminance)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Dynamic Lighting On (Solar Twilight Day/Night Terminator Transition)',
        entities: entities.map((e) => {
          if (e.id === 'night-lights-tokyo') {
            return { ...e, color: [1.0, 0.7, 0.2, 0.95] };
          }
          if (e.id === 'night-lights-nile') {
            return { ...e, color: [1.0, 0.5, 0.2, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High Clock Speed Rotation (Multiplier 4000x Planetary Spin)',
        entities: entities.map((e) => {
          if (e.id === 'night-lights-aurora') {
            return { ...e, color: [0.1, 1.0, 0.8, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Lit Hemisphere Daylight Suppressed (Night Alpha 1.0 Day Alpha 0.0)',
        entities: entities.map((e) => {
          if (e.id === 'night-lights-equator-band') {
            return { ...e, color: [0.1, 0.3, 0.7, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'earth-at-night.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'earth-at-night.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated earth-at-night.json and .variants.json');
}

// 2. Natural Earth II
function generateNaturalEarthII() {
  const alpsRelief = {
    id: 'natural-earth-alps-relief',
    name: 'Alps Mountain Range Shaded Relief Crest Marker',
    show: true,
    positionDegrees: [10.0, 46.5, 20000.0],
    shape: {
      type: 'cylinder',
      length: 40000.0,
      topRadius: 15000.0,
      bottomRadius: 15000.0,
    },
    color: [0.85, 0.75, 0.55, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3813, orography: 'Alps Orographic Spine', reliefElevationM: 4808 },
  };

  const congoDrainage = {
    id: 'natural-earth-congo-drainage',
    name: 'Congo River Drainage Basin Hydrological Core',
    show: true,
    positionDegrees: [22.0, 0.0, 20000.0],
    shape: {
      type: 'box',
      dimensions: [35000.0, 35000.0, 35000.0],
    },
    color: [0.2, 0.7, 0.4, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { hydrography: 'Congo Basin Drainage', dischargeM3s: 41000 },
  };

  const andesPlateau = {
    id: 'natural-earth-andes-plateau',
    name: 'Andes Altiplano Shaded Relief High Plateau Marker',
    show: true,
    positionDegrees: [-68.0, -18.0, 25000.0],
    shape: {
      type: 'ellipsoid',
      radii: [30000.0, 30000.0, 30000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.9, 0.6, 0.3, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { morphometry: 'Central Andes Plateau', altitudeM: 3750 },
  };

  const mediterraneanBasin = {
    id: 'natural-earth-mediterranean-basin',
    name: 'Mediterranean Marine Basin Drainage Perimeter',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [12.0, 36.0, 18.0, 40.0],
      height: 5000.0,
      extrudedHeight: 10000.0,
      rotation: 0.0,
    },
    color: [0.2, 0.5, 0.8, 0.65],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.7, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: { bathymetrySubdivision: 'Ionian Sea Deep', maxDepthM: 5267 },
  };

  const entities = [alpsRelief, congoDrainage, andesPlateau, mediterraneanBasin];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Natural Earth II with Shaded Relief (Continental Landforms & Drainages)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Water Bodies & Drainage Emphasized (Hydrographic Drainage Network)',
        entities: entities.map((e) => {
          if (e.id === 'natural-earth-congo-drainage') {
            return { ...e, color: [0.1, 0.85, 0.5, 0.95] };
          }
          if (e.id === 'natural-earth-mediterranean-basin') {
            return { ...e, color: [0.1, 0.6, 0.9, 0.85] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'High Relief Contrast (Alps & Himalayas Geomorphology Highlight)',
        entities: entities.map((e) => {
          if (e.id === 'natural-earth-alps-relief') {
            return { ...e, color: [1.0, 0.85, 0.6, 1.0] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Global Coastal Transition (Land-Water Mask Boundary)',
        entities: entities.map((e) => {
          if (e.id === 'natural-earth-andes-plateau') {
            return { ...e, color: [0.95, 0.7, 0.4, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'natural-earth-ii.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'natural-earth-ii.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated natural-earth-ii.json and .variants.json');
}

// 3. Sentinel-2
function generateSentinel2() {
  const suezCanal = {
    id: 'sentinel2-suez-canal',
    name: 'Suez Canal High-Resolution Maritime Chokepoint',
    show: true,
    positionDegrees: [32.34, 30.65, 15000.0],
    shape: {
      type: 'box',
      dimensions: [20000.0, 20000.0, 20000.0],
    },
    color: [0.1, 0.6, 0.9, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.4, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { ionAssetId: 3954, channelWidthM: 300, canalLengthKm: 193.3 },
  };

  const dubaiPalm = {
    id: 'sentinel2-dubai-palm',
    name: 'Dubai Palm Jumeirah Coastal Morphology Structure',
    show: true,
    positionDegrees: [55.13, 25.11, 15000.0],
    shape: {
      type: 'cylinder',
      length: 30000.0,
      topRadius: 10000.0,
      bottomRadius: 10000.0,
    },
    color: [0.95, 0.8, 0.4, 0.9],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: { reclamationAreaKm2: 5.72, coastalResolutionM: 10 },
  };

  const agriculturalCircles = {
    id: 'sentinel2-agricultural-circles',
    name: 'Wadi As-Sirhan Circular Pivot Irrigation Survey Dome',
    show: true,
    positionDegrees: [38.3, 30.1, 15000.0],
    shape: {
      type: 'ellipsoid',
      radii: [18000.0, 18000.0, 18000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.2, 0.85, 0.35, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { irrigationType: 'Center Pivot', fieldDiameterM: 800 },
  };

  const nileDeltaPerimeter = {
    id: 'sentinel2-nile-delta',
    name: 'Nile Delta High-Productivity Agriculture Fan',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [30.5, 30.5, 32.0, 31.8],
      height: 3000.0,
      extrudedHeight: 6000.0,
      rotation: 0.0,
    },
    color: [0.15, 0.7, 0.3, 0.65],
    fill: true,
    outline: true,
    outlineColor: [0.3, 0.9, 0.5, 0.9],
    outlineWidth: 1.0,
    properties: { ndviMean: 0.72, multispectralBand: 'B4-B3-B2 True Color' },
  };

  const entities = [suezCanal, dubaiPalm, agriculturalCircles, nileDeltaPerimeter];

  const variants = {
    initial: 0,
    variants: [
      {
        label: '10-Meter Multispectral Composite (Cloudless Global Land Coverage)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Agricultural Infrared False Color (Vegetation Vigor & Crop Health)',
        entities: entities.map((e) => {
          if (e.id === 'sentinel2-agricultural-circles') {
            return { ...e, color: [0.9, 0.15, 0.35, 0.9] };
          }
          if (e.id === 'sentinel2-nile-delta') {
            return { ...e, color: [0.85, 0.2, 0.4, 0.75] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Urban Infrastructure & Coastal High-Resolution Zoom (Maritime & Ports)',
        entities: entities.map((e) => {
          if (e.id === 'sentinel2-suez-canal') {
            return { ...e, color: [0.2, 0.75, 1.0, 1.0] };
          }
          if (e.id === 'sentinel2-dubai-palm') {
            return { ...e, color: [1.0, 0.85, 0.5, 1.0] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Arid Geomorphology & Dunes (Desert Geological Formations)',
        entities: entities.map((e) => {
          if (e.id === 'sentinel2-dubai-palm') {
            return { ...e, color: [0.95, 0.65, 0.25, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'sentinel-2.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'sentinel-2.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated sentinel-2.json and .variants.json');
}

// 4. ArcGIS MapServer
function generateArcGISMapServer() {
  const satelliteDatum = {
    id: 'arcgis-satellite-datum',
    name: 'Redlands ESRI Geospatial Campus Anchor Datum',
    show: true,
    positionDegrees: [-117.195, 34.056, 15000.0],
    shape: {
      type: 'cylinder',
      length: 30000.0,
      topRadius: 10000.0,
      bottomRadius: 10000.0,
    },
    color: [0.3, 0.6, 1.0, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.8, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: { providerType: 'ArcGisMapServer', basemapType: 'SATELLITE', campusId: 'Redlands HQ' },
  };

  const hillshadePeak = {
    id: 'arcgis-hillshade-peak',
    name: 'Mount San Gorgonio Topographic Hillshade Crest',
    show: true,
    positionDegrees: [-116.825, 34.10, 15000.0],
    shape: {
      type: 'box',
      dimensions: [25000.0, 25000.0, 25000.0],
    },
    color: [0.8, 0.7, 0.5, 0.85],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.85, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: { peakElevationFt: 11503, hillshadeAzimuthDeg: 315 },
  };

  const oceansTrench = {
    id: 'arcgis-oceans-trench',
    name: 'Southern California Borderland Deep Marine Dome',
    show: true,
    positionDegrees: [-118.5, 33.2, 15000.0],
    shape: {
      type: 'ellipsoid',
      radii: [20000.0, 20000.0, 20000.0],
      slicePartitions: 24,
      stackPartitions: 24,
    },
    color: [0.1, 0.4, 0.8, 0.85],
    fill: true,
    outline: false,
    outlineColor: [0.0, 0.0, 0.0, 0.0],
    outlineWidth: 1.0,
    properties: { oceanBasin: 'San Nicolas Basin', bathymetryDepthM: 1800 },
  };

  const serviceBoundary = {
    id: 'arcgis-service-boundary',
    name: 'Inland Empire Regional Tile Service Extent',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-117.5, 33.8, -116.8, 34.3],
      height: 2000.0,
      extrudedHeight: 5000.0,
      rotation: 0.0,
    },
    color: [0.4, 0.75, 0.4, 0.65],
    fill: true,
    outline: true,
    outlineColor: [0.6, 0.95, 0.6, 0.9],
    outlineWidth: 1.0,
    properties: { tileScheme: 'WebMercator', maxLod: 19 },
  };

  const entities = [satelliteDatum, hillshadePeak, oceansTrench, serviceBoundary];

  const variants = {
    initial: 0,
    variants: [
      {
        label: 'Satellite World Imagery Basemap (Global High-Resolution Orthoimagery)',
        entities: entities.map((e) => ({ ...e, show: true })),
      },
      {
        label: 'Hillshade World Basemap (Multidirectional Terrain Shading)',
        entities: entities.map((e) => {
          if (e.id === 'arcgis-hillshade-peak') {
            return { ...e, color: [0.95, 0.85, 0.6, 0.95] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Oceans Basemap (Seafloor Bathymetry & Coastal Marine Cartography)',
        entities: entities.map((e) => {
          if (e.id === 'arcgis-oceans-trench') {
            return { ...e, color: [0.15, 0.55, 0.95, 0.9] };
          }
          return { ...e, show: true };
        }),
      },
      {
        label: 'Topographic Basemap (Contours, Highways & Administrative Boundaries)',
        entities: entities.map((e) => {
          if (e.id === 'arcgis-service-boundary') {
            return { ...e, color: [0.8, 0.6, 0.3, 0.8] };
          }
          return { ...e, show: true };
        }),
      },
    ],
  };

  fs.writeFileSync(path.join(outDir, 'arcgis-mapserver.json'), JSON.stringify(entities, null, 2));
  fs.writeFileSync(path.join(outDir, 'arcgis-mapserver.variants.json'), JSON.stringify(variants, null, 2));
  console.log('generated arcgis-mapserver.json and .variants.json');
}

generateEarthAtNight();
generateNaturalEarthII();
generateSentinel2();
generateArcGISMapServer();
