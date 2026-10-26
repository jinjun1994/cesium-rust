# Earth base color attribution

Credit: **NASA Earth Observatory**. Source: Blue Marble: Next Generation with Topography and Bathymetry, December 2004.

- [NASA dataset page](https://science.nasa.gov/earth/earth-observatory/blue-marble-next-generation/base-topography-bathymetry/)
- [Original image](https://eoimages.gsfc.nasa.gov/images/imagerecords/73000/73909/world.topo.bathy.200412.3x5400x2700.jpg)
- [NASA media usage guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/)

`earth-base-color.png` is a 1024×512 RGB equirectangular reduction of the 5400×2700 source, resized with Pillow Lanczos. Longitude is -180° to +180° from left to right, latitude +90° to -90° from top to bottom. The shader samples an sRGB texture; illumination is applied in linear space.

This is a static satellite composite, not live weather or a terrain height model. No NASA logos are bundled; the project is not endorsed by NASA. Preserve the NASA Earth Observatory credit when redistributing the texture or rendered views.

## Online imagery

Default without token: Cesium's Natural Earth II sample tiles (256×256 geographic TMS, levels 0–2), served from the official Cesium 1.146 release asset tree. Source: https://cesium.com/downloads/cesiumjs/releases/1.146/Build/Cesium/Assets/Textures/NaturalEarthII/tilemapresource.xml . Natural Earth public domain data: https://www.naturalearthdata.com/about/terms-of-use/ . `docs/imagery-preview.png` is an actual engine render of these tiles. Tile downloads are not bundled in the source package.

With a user-supplied ion token: Cesium ion default world imagery asset 2, currently Bing Maps Aerial. Endpoint/Bing attributions are retained by the provider, exposed by `imageryCredits`/`onCredits`, displayed in the Web example and printed by the native example. Preserve and display the returned credits and follow your provider account terms. No tokens or Bing imagery files are bundled.
