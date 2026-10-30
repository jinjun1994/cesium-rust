import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'tests/sandcastle/full');

// 1. I3S Building Scene Layer
function generateI3SBuildingSceneLayer() {
  const libraryBox = {
    id: 'bsl-library-box',
    name: 'Turanga Library Structural Volume Landmark',
    show: true,
    positionDegrees: [172.6368, -43.5303, 20.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 30.0, 25.0],
    },
    color: [0.95, 0.55, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.8, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      layerType: 'BuildingSceneLayer',
      discipline: 'Architectural',
      buildingName: 'Turanga Library',
    },
  };

  const atriumCylinder = {
    id: 'bsl-atrium-cylinder',
    name: 'Central Timber Atrium Pillar Column',
    show: true,
    positionDegrees: [172.6372, -43.5299, 18.0],
    shape: {
      type: 'cylinder',
      length: 22.0,
      topRadius: 4.5,
      bottomRadius: 4.5,
    },
    color: [0.35, 0.85, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      discipline: 'Structural',
      element: 'TimberPillar',
    },
  };

  const mepSphere = {
    id: 'bsl-mep-sphere',
    name: 'MEP HVAC Infrastructure Sensor Probe',
    show: true,
    positionDegrees: [172.6364, -43.5307, 28.0],
    shape: {
      type: 'ellipsoid',
      radii: [5.0, 5.0, 5.0],
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      discipline: 'Mechanical',
      system: 'HVAC_Ductwork',
    },
  };

  const extentRect = {
    id: 'bsl-extent-rect',
    name: 'Turanga Library Cadastral Parcel Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [172.634, -43.532, 172.639, -43.528],
    },
    color: [0.25, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.95, 0.75, 0.35, 0.9],
    outlineWidth: 1.0,
    properties: {
      spatialExtent: 'Christchurch_Central',
    },
  };

  const entities = [libraryBox, atriumCylinder, mepSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Turanga Library Building Scene Layer (BIM Structural Elements)',
      entities: [libraryBox, atriumCylinder, mepSphere, extentRect],
    },
    {
      label: 'Architectural Discipline Isolation Mode (Discipline Sublayer)',
      entities: [
        { ...libraryBox, color: [0.95, 0.35, 0.35, 0.95], outlineColor: [1.0, 0.7, 0.7, 1.0] },
        atriumCylinder,
        mepSphere,
        extentRect,
      ],
    },
    {
      label: 'Mechanical HVAC Ductwork Inspection (Amber Piping Infrastructure)',
      entities: [
        libraryBox,
        { ...atriumCylinder, color: [0.95, 0.75, 0.15, 0.95], outlineColor: [1.0, 0.9, 0.5, 1.0] },
        mepSphere,
        extentRect,
      ],
    },
    {
      label: 'High-Contrast Envelope Wireframe Diagnostics (Building Spatial Extent)',
      entities: [
        libraryBox,
        atriumCylinder,
        { ...mepSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.75, 1.0, 1.0] },
        { ...extentRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 2. I3S Feature Picking
function generateI3SFeaturePicking() {
  const nycTowerBox = {
    id: 'i3s-nyc-tower-box',
    name: 'Manhattan Attributed High-Rise Landmark Box',
    show: true,
    positionDegrees: [-74.006, 40.7128, 60.0],
    shape: {
      type: 'box',
      dimensions: [40.0, 40.0, 100.0],
    },
    color: [0.25, 0.65, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 0.9, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      layerType: '3DObject',
      dataset: 'NYC_Attributed_v17',
      bldgName: 'Manhattan Tower',
      heightMeters: 100.0,
    },
  };

  const geoidCylinder = {
    id: 'i3s-geoid-sensor-cylinder',
    name: 'EGM2008 Geoid Vertical Datum Calibration Pillar',
    show: true,
    positionDegrees: [-74.004, 40.7135, 50.0],
    shape: {
      type: 'cylinder',
      length: 70.0,
      topRadius: 8.0,
      bottomRadius: 8.0,
    },
    color: [0.95, 0.75, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      datum: 'EGM2008',
      geoidTranscoding: true,
    },
  };

  const attributeSphere = {
    id: 'i3s-pick-attribute-sphere',
    name: 'Feature Attribute Query Probe Target',
    show: true,
    positionDegrees: [-74.008, 40.7121, 55.0],
    shape: {
      type: 'ellipsoid',
      radii: [7.0, 7.0, 7.0],
    },
    color: [0.35, 0.9, 0.45, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 1.0, 0.8, 1.0],
    outlineWidth: 1.0,
    properties: {
      pickMode: 'InteractiveFields',
      samplePositionValid: true,
    },
  };

  const manhattanRect = {
    id: 'i3s-manhattan-lot-rect',
    name: 'NYC Tax Lot Cadastral Footprint Envelope',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-74.010, 40.710, -74.002, 40.716],
    },
    color: [0.25, 0.35, 0.45, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.45, 0.85, 1.0, 0.9],
    outlineWidth: 1.0,
    properties: {
      borough: 'Manhattan',
    },
  };

  const entities = [nycTowerBox, geoidCylinder, attributeSphere, manhattanRect];

  const variants = [
    {
      label: 'Baseline: NYC Attributed I3S Feature Picking (Manhattan High-Rise Mesh)',
      entities: [nycTowerBox, geoidCylinder, attributeSphere, manhattanRect],
    },
    {
      label: 'Building Height and Tax Lot Attribute Inspection (Zoning Metadata)',
      entities: [
        { ...nycTowerBox, color: [0.95, 0.8, 0.2, 0.95], outlineColor: [1.0, 0.9, 0.55, 1.0] },
        geoidCylinder,
        attributeSphere,
        manhattanRect,
      ],
    },
    {
      label: 'Mid-Town Commercial Density Classification (Amber Thematic Volume)',
      entities: [
        nycTowerBox,
        { ...geoidCylinder, color: [0.95, 0.45, 0.25, 0.95], outlineColor: [1.0, 0.75, 0.6, 1.0] },
        attributeSphere,
        manhattanRect,
      ],
    },
    {
      label: 'High-Precision Geoid Elevation Alignment (Geodetic Vertical Datum)',
      entities: [
        nycTowerBox,
        geoidCylinder,
        { ...attributeSphere, color: [0.25, 0.85, 0.95, 0.95], outlineColor: [0.75, 1.0, 1.0, 1.0] },
        { ...manhattanRect, color: [0.35, 0.25, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 3. I3S Integrated Mesh Layer
function generateI3SIntegratedMeshLayer() {
  const meshBox = {
    id: 'i3s-mesh-frankfurt-box',
    name: 'Frankfurt Skyline Integrated Mesh Anchor Box',
    show: true,
    positionDegrees: [8.682, 50.110, 60.0],
    shape: {
      type: 'box',
      dimensions: [45.0, 45.0, 65.0],
    },
    color: [0.85, 0.35, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.7, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      layerType: 'IntegratedMesh',
      city: 'Frankfurt',
      depthTestAgainstTerrain: false,
    },
  };

  const waterfrontCylinder = {
    id: 'i3s-main-waterfront-cylinder',
    name: 'Main River Waterfront Topography Column',
    show: true,
    positionDegrees: [8.685, 50.108, 45.0],
    shape: {
      type: 'cylinder',
      length: 50.0,
      topRadius: 10.0,
      bottomRadius: 10.0,
    },
    color: [0.25, 0.75, 0.85, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.65, 1.0, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      feature: 'RiverWaterfront',
    },
  };

  const altstadtSphere = {
    id: 'i3s-altstadt-probe-sphere',
    name: 'Historic Altstadt Surface Mesh Density Probe',
    show: true,
    positionDegrees: [8.679, 50.112, 50.0],
    shape: {
      type: 'ellipsoid',
      radii: [6.5, 6.5, 6.5],
    },
    color: [0.95, 0.8, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.9, 0.65, 1.0],
    outlineWidth: 1.0,
    properties: {
      feature: 'Altstadt',
    },
  };

  const extentRect = {
    id: 'i3s-frankfurt-extent-rect',
    name: 'Frankfurt Photogrammetric Mesh Coverage Boundary',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [8.675, 50.105, 8.690, 50.116],
    },
    color: [0.35, 0.25, 0.35, 0.25],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.65, 0.95, 0.9],
    outlineWidth: 1.0,
    properties: {
      extentCoverage: 'Frankfurt2017_vi3s_18',
    },
  };

  const entities = [meshBox, waterfrontCylinder, altstadtSphere, extentRect];

  const variants = [
    {
      label: 'Baseline: Frankfurt Photogrammetric Integrated Mesh (Seamless City Surface)',
      entities: [meshBox, waterfrontCylinder, altstadtSphere, extentRect],
    },
    {
      label: 'Main River Waterfront Mesh Inspection (Continuous Shoreline Topography)',
      entities: [
        { ...meshBox, color: [0.25, 0.85, 0.45, 0.95], outlineColor: [0.7, 1.0, 0.8, 1.0] },
        waterfrontCylinder,
        altstadtSphere,
        extentRect,
      ],
    },
    {
      label: 'Historic Altstadt Dense Texture Mode (Cultural Heritage Fabric)',
      entities: [
        meshBox,
        { ...waterfrontCylinder, color: [0.95, 0.55, 0.25, 0.95], outlineColor: [1.0, 0.8, 0.6, 1.0] },
        altstadtSphere,
        extentRect,
      ],
    },
    {
      label: 'High-Altitude Regional Mesh Coverage Boundary (Urban Footprint Envelope)',
      entities: [
        meshBox,
        waterfrontCylinder,
        { ...altstadtSphere, color: [0.85, 0.25, 0.95, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        { ...extentRect, color: [0.25, 0.35, 0.25, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// 4. Image-Based Lighting
function generateImageBasedLighting() {
  const stageBox = {
    id: 'ibl-pawns-stage-box',
    name: 'Pawns glTF Model IBL Studio Pedestal',
    show: true,
    positionDegrees: [-123.0744619, 44.0503706, 10.0],
    shape: {
      type: 'box',
      dimensions: [25.0, 25.0, 8.0],
    },
    color: [0.85, 0.8, 0.25, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.95, 0.6, 1.0],
    outlineWidth: 1.0,
    properties: {
      lightingModel: 'ImageBasedLighting',
      specularEnvironmentMaps: 'kiara_6_afternoon_2k_ibl.ktx2',
      groundColor: '#292817',
    },
  };

  const specularCylinder = {
    id: 'ibl-specular-cylinder',
    name: 'Specular Reflection Analysis Light Column',
    show: true,
    positionDegrees: [-123.0742, 44.0505, 12.0],
    shape: {
      type: 'cylinder',
      length: 18.0,
      topRadius: 3.5,
      bottomRadius: 3.5,
    },
    color: [0.35, 0.75, 0.95, 0.95],
    fill: true,
    outline: true,
    outlineColor: [0.75, 0.95, 1.0, 1.0],
    outlineWidth: 1.0,
    properties: {
      roughness: 0.1,
      metalness: 0.9,
    },
  };

  const harmonicSphere = {
    id: 'ibl-harmonic-sphere',
    name: 'Spherical Harmonics Irradiance Evaluation Sphere',
    show: true,
    positionDegrees: [-123.0747, 44.0502, 14.0],
    shape: {
      type: 'ellipsoid',
      radii: [4.0, 4.0, 4.0],
    },
    color: [0.95, 0.45, 0.35, 0.95],
    fill: true,
    outline: true,
    outlineColor: [1.0, 0.75, 0.7, 1.0],
    outlineWidth: 1.0,
    properties: {
      coefficients: 9,
      diffuseIrradiance: true,
    },
  };

  const groundRect = {
    id: 'ibl-ground-reflection-rect',
    name: 'Warm Ground Bounce Radiance Footprint',
    show: true,
    positionDegrees: null,
    shape: {
      type: 'rectangle',
      coordinatesDegrees: [-123.076, 44.049, -123.072, 44.052],
    },
    color: [0.25, 0.22, 0.15, 0.3],
    fill: true,
    outline: true,
    outlineColor: [0.85, 0.75, 0.45, 0.9],
    outlineWidth: 1.0,
    properties: {
      groundBounce: '#292817',
    },
  };

  const entities = [stageBox, specularCylinder, harmonicSphere, groundRect];

  const variants = [
    {
      label: 'Baseline: Kiara Afternoon IBL Environment Map with Spherical Harmonics (Studio PBR)',
      entities: [stageBox, specularCylinder, harmonicSphere, groundRect],
    },
    {
      label: 'Procedural Solar Environment Lighting Mode (Directional Sun Key Light)',
      entities: [
        { ...stageBox, color: [0.95, 0.65, 0.2, 0.95], outlineColor: [1.0, 0.85, 0.5, 1.0] },
        specularCylinder,
        harmonicSphere,
        groundRect,
      ],
    },
    {
      label: 'High-Reflectance Specular Metallic Mirroring (Roughness Zero Reflection)',
      entities: [
        stageBox,
        { ...specularCylinder, color: [0.85, 0.3, 0.85, 0.95], outlineColor: [1.0, 0.7, 1.0, 1.0] },
        harmonicSphere,
        groundRect,
      ],
    },
    {
      label: 'Ambient Ground Radiance Shading Mode (Warm Ground Bounce Tone)',
      entities: [
        stageBox,
        specularCylinder,
        { ...harmonicSphere, color: [0.25, 0.85, 0.75, 0.95], outlineColor: [0.7, 1.0, 0.95, 1.0] },
        { ...groundRect, color: [0.35, 0.3, 0.2, 0.35] },
      ],
    },
  ];

  return { entities, variants };
}

// Generate all Batch 75 fixtures
const datasets = {
  'i3s-building-scene-layer': generateI3SBuildingSceneLayer(),
  'i3s-feature-picking': generateI3SFeaturePicking(),
  'i3s-integratedmesh-layer': generateI3SIntegratedMeshLayer(),
  'image-based-lighting': generateImageBasedLighting(),
};

for (const [id, { entities, variants }] of Object.entries(datasets)) {
  const jsonPath = path.join(outDir, `${id}.json`);
  const variantsPath = path.join(outDir, `${id}.variants.json`);

  fs.writeFileSync(jsonPath, JSON.stringify(entities, null, 2) + '\n');
  fs.writeFileSync(variantsPath, JSON.stringify({ initial: 0, variants }, null, 2) + '\n');
  console.log(`Generated ${id}.json (${entities.length}) and ${id}.variants.json (${variants.length})`);
}
