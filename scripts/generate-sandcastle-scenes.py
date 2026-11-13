#!/usr/bin/env python3
"""Generate tests/e2e/sandcastle-scenes.json from upstream Cesium Sandcastle gallery main.js."""
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RECIPES_PATH = os.path.join(ROOT, "tests", "sandcastle", "full-recipes.json")
GALLERY_DIR = os.path.join(os.path.dirname(ROOT), "cesium", "packages", "sandcastle", "gallery")
OUT_PATH = os.path.join(ROOT, "tests", "e2e", "sandcastle-scenes.json")


def eval_num(expr):
    expr = expr.strip()
    m = re.match(r"Cesium\.Math\.toRadians\(\s*([+-]?[0-9.]+)\s*\)", expr)
    if m:
        return float(m.group(1)) * 3.141592653589793 / 180.0
    expr = expr.replace("Cesium.Math.PI_OVER_TWO", "1.5707963267948966")
    expr = expr.replace("Cesium.Math.PI_OVER_FOUR", "0.7853981633974483")
    expr = expr.replace("Cesium.Math.PI", "3.141592653589793")
    expr = expr.replace("Math.PI", "3.141592653589793")
    if re.match(r"^[+-]?[0-9.*/+\s()-]+$", expr):
        try:
            return float(eval(expr, {"__builtins__": {}}, {}))
        except Exception:
            return None
    return None


def extract_camera(src):
    # First collect any named Cartesian3.fromDegrees(...) variables
    pos_vars = {}
    for vm in re.finditer(
        r"(?:const|let|var)\s+(\w+)\s*=\s*(?:new\s+)?Cesium\.Cartesian3\.fromDegrees\(\s*([^,]+),\s*([^,]+)(?:,\s*([^)]+))?\)",
        src,
    ):
        lon = eval_num(vm.group(2))
        lat = eval_num(vm.group(3))
        h = eval_num(vm.group(4)) if vm.group(4) else 1000.0
        if lon is not None and lat is not None and h is not None:
            pos_vars[vm.group(1)] = [lon, lat, h]

    # Match inline setView({ destination: Cartesian3.fromDegrees(...) })
    m = re.search(
        r"setView\(\s*\{[\s\S]*?destination\s*:\s*(?:new\s+)?Cesium\.Cartesian3\.fromDegrees\(\s*([^,]+),\s*([^,]+)(?:,\s*([^)]+))?\)",
        src,
    )
    dest = None
    search_start = 0
    if m:
        lon = eval_num(m.group(1))
        lat = eval_num(m.group(2))
        h = eval_num(m.group(3)) if m.group(3) else 1000.0
        if lon is not None and lat is not None and h is not None:
            dest = [lon, lat, h]
            search_start = m.start()
    else:
        # Match setView({ destination: varName })
        vm = re.search(r"setView\(\s*\{[\s\S]*?destination\s*:\s*(\w+)", src)
        if vm and vm.group(1) in pos_vars:
            dest = pos_vars[vm.group(1)]
            search_start = vm.start()

    if dest is not None:
        dest[2] = min(max(dest[2], 15.0), 29000000.0)
        cam = {"setView": {"destinationDegrees": dest, "heading": 0.0, "pitch": -1.5707963, "roll": 0.0}}
        om = re.search(r"orientation\s*:\s*\{([^}]+)\}", src[search_start : search_start + 500])
        if om:
            body = om.group(1)
            for key in ("heading", "pitch", "roll"):
                km = re.search(rf"{key}\s*:\s*([^,\n}}]+)", body)
                if km:
                    val = eval_num(km.group(1))
                    if val is not None:
                        cam["setView"][key] = val
        else:
            # Check orientation: initialOrientation where initialOrientation = HeadingPitchRoll.fromDegrees(h, p, r)
            ovm = re.search(r"orientation\s*:\s*(\w+)", src[search_start : search_start + 500])
            if ovm:
                hpr_m = re.search(
                    rf"{ovm.group(1)}\s*=\s*(?:new\s+)?Cesium\.HeadingPitchRoll\.fromDegrees\(\s*([^,]+),\s*([^,]+),\s*([^)]+)\)",
                    src,
                )
                if hpr_m:
                    for key, idx in (("heading", 1), ("pitch", 2), ("roll", 3)):
                        val = eval_num(hpr_m.group(idx))
                        if val is not None:
                            cam["setView"][key] = val * 3.141592653589793 / 180.0
        return cam
    return None


def extract_position_degrees(src):
    m = re.search(
        r"Cesium\.Cartesian3\.fromDegrees\(\s*([+-]?[0-9.]+)\s*,\s*([+-]?[0-9.]+)(?:\s*,\s*([+-]?[0-9.]+|height))?",
        src,
    )
    if m:
        lon = float(m.group(1))
        lat = float(m.group(2))
        h = 0.0
        if m.group(3) and m.group(3) != "height":
            h = float(m.group(3))
        else:
            # Check createModel("...", height) first call
            cm = re.search(r"createModel\([^,]+,\s*([+-]?[0-9.]+)\s*\)", src)
            if cm:
                h = float(cm.group(1))
        return [lon, lat, h]
    return [-123.0744619, 44.0503706, 0.0]


def main():
    recipes = json.load(open(RECIPES_PATH))
    items = recipes["cases"] if isinstance(recipes, dict) and "cases" in recipes else recipes
    variants = [c["id"] for c in items if isinstance(c, dict) and c.get("format") == "variants"]

    scenes = {}
    counts = {"tileset_ion": 0, "tileset_url": 0, "model": 0, "geojson": 0, "terrain": 0}

    for vid in variants:
        main_js = os.path.join(GALLERY_DIR, vid, "main.js")
        if not os.path.exists(main_js):
            continue
        src = open(main_js).read()
        scene = {}

        if (
            "createWorldTerrainAsync" in src
            or "Terrain.fromWorldTerrain" in src
            or "CesiumTerrainProvider.fromIonAssetId" in src
        ):
            scene["terrain"] = True
            counts["terrain"] += 1

        cam = extract_camera(src)

        # 1. 3D Tilesets (Ion or SampleData URL)
        ion_m = re.search(r"Cesium3DTileset\.fromIonAssetId\(\s*(\d+)", src)
        url_m = re.search(r"Cesium3DTileset\.fromUrl\(\s*[\"']([^\"']*SampleData/[^\"']+)[\"']", src)
        sample_tileset_m = re.search(r"[\"'](\.\./\.\./SampleData/Cesium3DTiles/[^\"']+tileset\.json)[\"']", src)

        if "createOsmBuildingsAsync" in src:
            scene["tileset"] = {"ionAssetId": 96188}
            counts["tileset_ion"] += 1
        elif "createGooglePhotorealistic3DTileset" in src:
            scene["tileset"] = {"ionAssetId": 2275207}
            counts["tileset_ion"] += 1
        elif ion_m:
            scene["tileset"] = {"ionAssetId": int(ion_m.group(1))}
            counts["tileset_ion"] += 1
        elif url_m or sample_tileset_m:
            raw = (url_m or sample_tileset_m).group(1)
            rel = raw.replace("../../SampleData/", "/tests/SampleData/")
            scene["tileset"] = {"url": rel}
            counts["tileset_url"] += 1

        # 2. glTF Models (when not primarily a 3D Tileset)
        if "tileset" not in scene:
            model_m = re.search(r"(\.\./\.\./SampleData/models/[^\"'`]+\.(?:glb|gltf))", src)
            if model_m:
                uri = model_m.group(1).replace("../../SampleData/", "/tests/SampleData/")
                pos = extract_position_degrees(src)
                min_px = 128.0
                mm = re.search(r"minimumPixelSize\s*:\s*([0-9.]+)", src)
                if mm:
                    min_px = float(mm.group(1))
                scale = 1.0
                sm = re.search(r"\bscale\s*:\s*([0-9.]+)", src)
                if sm:
                    scale = float(sm.group(1))
                max_scale = None
                mxm = re.search(r"maximumScale\s*:\s*([0-9.]+)", src)
                if mxm:
                    max_scale = float(mxm.group(1))
                shape = {
                    "type": "model",
                    "uri": uri,
                    "scale": scale,
                    "minimumPixelSize": min_px,
                }
                if max_scale is not None:
                    shape["maximumScale"] = max_scale
                scene["entities"] = [
                    {
                        "id": f"{vid}-model",
                        "name": vid,
                        "positionDegrees": pos,
                        "shape": shape,
                        "color": [1.0, 1.0, 1.0, 1.0],
                        "fill": True,
                        "outline": False,
                        "outlineColor": [0.0, 0.0, 0.0, 1.0],
                    }
                ]
                counts["model"] += 1

        # 3. GeoJSON (skip TopoJSON)
        geo_m = re.search(r"GeoJsonDataSource\.load\(\s*[\"'](\.\./\.\./SampleData/[^\"']+\.(?:geo)?json)[\"']", src)
        if geo_m and "tileset" not in scene and "entities" not in scene:
            scene["geojson"] = {"url": geo_m.group(1).replace("../../SampleData/", "/tests/SampleData/")}
            counts["geojson"] += 1

        if vid == "voxels-in-3d-tiles":
            scene.pop("tileset", None)

        if cam and "tileset" not in scene:
            # For tilesets without explicit camera override, keep Scene's auto-orbit on root bounds unless setView was given
            scene["camera"] = cam
        elif cam and "tileset" in scene:
            scene["camera"] = cam

        if scene:
            scenes[vid] = scene

    with open(OUT_PATH, "w") as f:
        json.dump(scenes, f, indent=2)
    print(f"Wrote {len(scenes)} scene overrides to {OUT_PATH}: {counts}")


if __name__ == "__main__":
    main()
