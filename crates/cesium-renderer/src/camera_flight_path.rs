//! Camera flight path tween generation matching CesiumJS `Scene/CameraFlightPath.js`.
use cesium_math::{
    equals_epsilon, lerp, negative_pi_to_pi, zero_to_two_pi, Cartesian2, Cartesian3,
    Cartographic, DeveloperError, EasingFunction, Ellipsoid, GeographicProjection,
    OrthographicOffCenterFrustum, PerspectiveFrustum, PerspectiveOffCenterFrustum, SceneMode,
    EPSILON10, EPSILON11, EPSILON6, PI, PI_OVER_TWO, TWO_PI,
};
use glam::DVec3;

/// Frustum variants supported by [`FlightCamera`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CameraFrustum {
    Perspective(PerspectiveFrustum),
    PerspectiveOffCenter(PerspectiveOffCenterFrustum),
    OrthographicOffCenter(OrthographicOffCenterFrustum),
}

impl Default for CameraFrustum {
    fn default() -> Self {
        Self::Perspective(PerspectiveFrustum::default())
    }
}

/// Camera state manipulated by [`CameraFlightPath`] tweens.
#[derive(Clone, Copy, Debug)]
pub struct FlightCamera {
    pub position: Cartesian3,
    pub direction: Cartesian3,
    pub up: Cartesian3,
    pub right: Cartesian3,
    pub frustum: CameraFrustum,
    pub heading: f64,
    pub pitch: f64,
    pub roll: f64,
    pub ellipsoid: Ellipsoid,
}

impl Default for FlightCamera {
    fn default() -> Self {
        Self::new(Ellipsoid::WGS84)
    }
}

impl FlightCamera {
    /// Creates a default camera over `(-95°, 35°, 1e7 m)` on `ellipsoid`.
    pub fn new(ellipsoid: Ellipsoid) -> Self {
        let pos = ellipsoid
            .cartographic_to_cartesian(Cartographic::from_degrees(-95.0, 35.0, Some(1.0e7)))
            .map(Cartesian3::from)
            .unwrap_or(Cartesian3::new(0.0, -1.0e7, 0.0));
        let mut cam = Self {
            position: pos,
            direction: Cartesian3::new(0.0, 1.0, 0.0),
            up: Cartesian3::new(0.0, 0.0, 1.0),
            right: Cartesian3::new(1.0, 0.0, 0.0),
            frustum: CameraFrustum::default(),
            heading: 0.0,
            pitch: -PI_OVER_TWO,
            roll: 0.0,
            ellipsoid,
        };
        cam.update_axes_from_hpr();
        cam
    }

    /// Returns the camera position in cartographic coordinates on its ellipsoid.
    pub fn position_cartographic(&self) -> Cartographic {
        self.ellipsoid
            .cartesian_to_cartographic(DVec3::from(self.position))
            .unwrap_or(Cartographic::new(0.0, 0.0, 0.0))
    }

    /// Sets the camera destination and/or orientation (heading, pitch, roll).
    pub fn set_view(
        &mut self,
        destination: Option<Cartesian3>,
        heading: Option<f64>,
        pitch: Option<f64>,
        roll: Option<f64>,
    ) {
        if let Some(dest) = destination {
            self.position = dest;
        }
        if let Some(h) = heading {
            self.heading = h;
        }
        if let Some(p) = pitch {
            self.pitch = p;
        }
        if let Some(r) = roll {
            self.roll = r;
        }
        self.update_axes_from_hpr();
    }

    fn update_axes_from_hpr(&mut self) {
        let ch = self.heading.cos();
        let sh = self.heading.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        let cr = self.roll.cos();
        let sr = self.roll.sin();

        // Local ENU direction/up/right from heading (CW from North), pitch, roll.
        let local_dir = Cartesian3::new(sh * cp, ch * cp, sp).normalize();
        let local_right_base = Cartesian3::new(ch, -sh, 0.0).normalize();
        let local_up_base = local_right_base.cross(&local_dir).normalize();
        let local_right = (local_right_base.multiply_by_scalar(cr)
            + local_up_base.multiply_by_scalar(sr))
            .normalize();
        let local_up = local_right.cross(&local_dir).normalize();

        let enu = Cartographic::from_dvec3_ecef_or_fallback(self.ellipsoid, self.position);
        let east = enu.0;
        let north = enu.1;
        let up = enu.2;

        self.direction = (east.multiply_by_scalar(local_dir.x)
            + north.multiply_by_scalar(local_dir.y)
            + up.multiply_by_scalar(local_dir.z))
            .normalize();
        self.right = (east.multiply_by_scalar(local_right.x)
            + north.multiply_by_scalar(local_right.y)
            + up.multiply_by_scalar(local_right.z))
            .normalize();
        self.up = (east.multiply_by_scalar(local_up.x)
            + north.multiply_by_scalar(local_up.y)
            + up.multiply_by_scalar(local_up.z))
            .normalize();
    }
}

trait CartographicEnu {
    fn from_dvec3_ecef_or_fallback(
        ellipsoid: Ellipsoid,
        pos: Cartesian3,
    ) -> (Cartesian3, Cartesian3, Cartesian3);
}

impl CartographicEnu for Cartographic {
    fn from_dvec3_ecef_or_fallback(
        ellipsoid: Ellipsoid,
        pos: Cartesian3,
    ) -> (Cartesian3, Cartesian3, Cartesian3) {
        let p = DVec3::from(pos);
        if p.length_squared() < 1e-12 {
            return (Cartesian3::UNIT_X, Cartesian3::UNIT_Y, Cartesian3::UNIT_Z);
        }
        let up = ellipsoid
            .geodetic_surface_normal(p)
            .unwrap_or_else(|| p.normalize());
        let east_raw = DVec3::Z.cross(up);
        let east = if east_raw.length_squared() < 1e-12 {
            DVec3::X
        } else {
            east_raw.normalize()
        };
        let north = up.cross(east).normalize();
        (
            Cartesian3::from(east),
            Cartesian3::from(north),
            Cartesian3::from(up),
        )
    }
}

/// Scene context passed to [`CameraFlightPath::create_tween`].
#[derive(Clone, Copy, Debug)]
pub struct FlightScene {
    pub mode: SceneMode,
    pub map_projection: GeographicProjection,
    pub camera: FlightCamera,
    pub enable_inputs: bool,
}

impl Default for FlightScene {
    fn default() -> Self {
        let projection = GeographicProjection::default();
        Self {
            mode: SceneMode::Scene3D,
            map_projection: projection,
            camera: FlightCamera::new(projection.ellipsoid()),
            enable_inputs: true,
        }
    }
}

impl FlightScene {
    pub fn with_projection(map_projection: GeographicProjection) -> Self {
        Self {
            mode: SceneMode::Scene3D,
            map_projection,
            camera: FlightCamera::new(map_projection.ellipsoid()),
            enable_inputs: true,
        }
    }
}

/// Options for [`CameraFlightPath::create_tween`].
#[derive(Default)]
pub struct CameraFlightOptions {
    pub destination: Option<Cartesian3>,
    pub heading: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub duration: Option<f64>,
    pub maximum_height: Option<f64>,
    pub pitch_adjust_height: Option<f64>,
    pub fly_over_longitude: Option<f64>,
    pub fly_over_longitude_weight: Option<f64>,
    pub convert: Option<bool>,
    pub easing_function: Option<EasingFunction>,
    pub complete: Option<Box<dyn FnMut()>>,
    pub cancel: Option<Box<dyn FnMut()>>,
}

type UpdateFn = Box<dyn FnMut(&mut FlightScene, f64)>;

/// Tween produced by [`CameraFlightPath::create_tween`].
pub struct CameraFlightTween {
    pub duration: f64,
    pub easing_function: EasingFunction,
    pub start_time: f64,
    pub stop_time: f64,
    pub is_empty: bool,
    updater: Option<UpdateFn>,
    complete_cb: Option<Box<dyn FnMut()>>,
    cancel_cb: Option<Box<dyn FnMut()>>,
    apply_on_complete: bool,
}

impl CameraFlightTween {
    fn empty(
        complete_cb: Option<Box<dyn FnMut()>>,
        cancel_cb: Option<Box<dyn FnMut()>>,
    ) -> Self {
        Self {
            duration: 0.0,
            easing_function: EasingFunction::LinearNone,
            start_time: 0.0,
            stop_time: 0.0,
            is_empty: true,
            updater: None,
            complete_cb,
            cancel_cb,
            apply_on_complete: false,
        }
    }

    pub fn has_update(&self) -> bool {
        self.updater.is_some()
    }

    pub fn update(&mut self, scene: &mut FlightScene, time: f64) {
        if let Some(ref mut updater) = self.updater {
            updater(scene, time);
        }
    }

    pub fn complete(&mut self, scene: &mut FlightScene) {
        if self.apply_on_complete {
            if let Some(ref mut updater) = self.updater {
                updater(scene, 1.0);
            }
        }
        if let Some(ref mut cb) = self.complete_cb {
            cb();
        }
        scene.enable_inputs = true;
    }

    pub fn cancel(&mut self, scene: &mut FlightScene) {
        if let Some(ref mut cb) = self.cancel_cb {
            cb();
        }
        scene.enable_inputs = true;
    }
}

/// Creates tweens for camera flights matching CesiumJS `CameraFlightPath`.
pub struct CameraFlightPath;

fn get_altitude(frustum: &CameraFrustum, dx: f64, dy: f64) -> f64 {
    match frustum {
        CameraFrustum::Perspective(p) => {
            let fovy = p.fovy().unwrap_or(p.fov);
            let tan_theta = (0.5 * fovy).tan();
            let near = p.near;
            let top = near * tan_theta;
            let right = p.aspect_ratio * top;
            ((dx * near) / right).max((dy * near) / top)
        }
        CameraFrustum::PerspectiveOffCenter(p) => {
            ((dx * p.near) / p.right).max((dy * p.near) / p.top)
        }
        CameraFrustum::OrthographicOffCenter(_) => dx.max(dy),
    }
}

fn create_height_function(
    camera: &FlightCamera,
    destination: Cartesian3,
    start_height: f64,
    end_height: f64,
    option_altitude: Option<f64>,
) -> Box<dyn Fn(f64) -> f64> {
    let max_height = start_height.max(end_height);
    let altitude = match option_altitude {
        Some(alt) => alt,
        None => {
            let start = camera.position;
            let diff = start - destination;
            let vertical = camera
                .up
                .multiply_by_scalar(diff.dot(&camera.up))
                .magnitude();
            let horizontal = camera
                .right
                .multiply_by_scalar(diff.dot(&camera.right))
                .magnitude();
            (get_altitude(&camera.frustum, vertical, horizontal) * 0.2).min(1_000_000_000.0)
        }
    };

    if max_height < altitude {
        let power = 8.0;
        let factor = 1_000_000.0;
        let s = -((altitude - start_height) * factor).powf(1.0 / power);
        let e = ((altitude - end_height) * factor).powf(1.0 / power);
        Box::new(move |t| {
            let x = t * (e - s) + s;
            -(x.powf(power)) / factor + altitude
        })
    } else {
        Box::new(move |t| lerp(start_height, end_height, t))
    }
}

fn create_pitch_function(
    start_pitch: f64,
    end_pitch: f64,
    height_fn: Box<dyn Fn(f64) -> f64>,
    pitch_adjust_height: Option<f64>,
) -> Box<dyn Fn(f64) -> f64> {
    if let Some(adjust_height) = pitch_adjust_height {
        if height_fn(0.5) > adjust_height {
            let start_height = height_fn(0.0);
            let end_height = height_fn(1.0);
            let middle_height = height_fn(0.5);
            let d1 = middle_height - start_height;
            let d2 = middle_height - end_height;
            return Box::new(move |time| {
                let altitude = height_fn(time);
                if time <= 0.5 {
                    let t1 = if d1.abs() > 0.0 {
                        (altitude - start_height) / d1
                    } else {
                        0.0
                    };
                    lerp(start_pitch, -PI_OVER_TWO, t1)
                } else {
                    let t2 = if d2.abs() > 0.0 {
                        (altitude - end_height) / d2
                    } else {
                        0.0
                    };
                    lerp(-PI_OVER_TWO, end_pitch, 1.0 - t2)
                }
            });
        }
    }
    Box::new(move |time| lerp(start_pitch, end_pitch, time))
}

fn adjust_angle_for_lerp(mut start_angle: f64, end_angle: f64) -> f64 {
    if equals_epsilon(start_angle, TWO_PI, Some(EPSILON11), None) {
        start_angle = 0.0;
    }
    if end_angle > start_angle + PI {
        start_angle += TWO_PI;
    } else if end_angle < start_angle - PI {
        start_angle -= TWO_PI;
    }
    start_angle
}

#[allow(clippy::too_many_arguments)]
fn create_update_cv(
    scene: &FlightScene,
    duration: f64,
    destination: Cartesian3,
    heading: f64,
    pitch: f64,
    roll: f64,
    option_altitude: Option<f64>,
    option_pitch_adjust_height: Option<f64>,
) -> UpdateFn {
    let camera = &scene.camera;
    let start = camera.position;
    let start_pitch = camera.pitch;
    let start_heading = adjust_angle_for_lerp(camera.heading, heading);
    let start_roll = adjust_angle_for_lerp(camera.roll, roll);

    let height_fn = create_height_function(
        camera,
        destination,
        start.z,
        destination.z,
        option_altitude,
    );
    let height_fn_for_pitch = create_height_function(
        camera,
        destination,
        start.z,
        destination.z,
        option_altitude,
    );
    let pitch_fn = create_pitch_function(
        start_pitch,
        pitch,
        height_fn_for_pitch,
        option_pitch_adjust_height,
    );

    let start_xy = Cartesian2::new(start.x, start.y);
    let dest_xy = Cartesian2::new(destination.x, destination.y);

    Box::new(move |s, raw_time| {
        let time = raw_time / duration;
        s.camera.set_view(
            None,
            Some(lerp(start_heading, heading, time)),
            Some(pitch_fn(time)),
            Some(lerp(start_roll, roll, time)),
        );
        let xy = Cartesian2::lerp(start_xy, dest_xy, time);
        s.camera.position.x = xy.x;
        s.camera.position.y = xy.y;
        s.camera.position.z = height_fn(time);
    })
}

#[allow(clippy::too_many_arguments)]
fn create_update_3d(
    scene: &FlightScene,
    duration: f64,
    destination: Cartesian3,
    heading: f64,
    pitch: f64,
    roll: f64,
    option_altitude: Option<f64>,
    option_fly_over_longitude: Option<f64>,
    option_fly_over_longitude_weight: Option<f64>,
    option_pitch_adjust_height: Option<f64>,
) -> UpdateFn {
    let camera = &scene.camera;
    let ellipsoid = scene.map_projection.ellipsoid();

    let mut start_cart = ellipsoid
        .cartesian_to_cartographic(DVec3::from(camera.position))
        .unwrap_or(Cartographic::new(0.0, 0.0, 0.0));
    let mut dest_cart = ellipsoid
        .cartesian_to_cartographic(DVec3::from(destination))
        .unwrap_or(Cartographic::new(0.0, 0.0, 0.0));

    let start_pitch = camera.pitch;
    let start_heading = adjust_angle_for_lerp(camera.heading, heading);
    let start_roll = adjust_angle_for_lerp(camera.roll, roll);

    start_cart.longitude = zero_to_two_pi(start_cart.longitude);
    dest_cart.longitude = zero_to_two_pi(dest_cart.longitude);

    let mut use_long_flight = false;
    if let Some(fly_over_lon) = option_fly_over_longitude {
        let hit_lon = zero_to_two_pi(fly_over_lon);
        let lon_min = start_cart.longitude.min(dest_cart.longitude);
        let lon_max = start_cart.longitude.max(dest_cart.longitude);
        let hit_inside = hit_lon >= lon_min && hit_lon <= lon_max;

        if let Some(weight) = option_fly_over_longitude_weight {
            let din = (start_cart.longitude - dest_cart.longitude).abs();
            let dout = TWO_PI - din;
            let hit_distance = if hit_inside { din } else { dout };
            let off_distance = if hit_inside { dout } else { din };
            if hit_distance < off_distance * weight && !hit_inside {
                use_long_flight = true;
            }
        } else if !hit_inside {
            use_long_flight = true;
        }
    }

    if use_long_flight {
        if start_cart.longitude < dest_cart.longitude {
            start_cart.longitude += TWO_PI;
        } else {
            dest_cart.longitude += TWO_PI;
        }
    } else {
        let diff = start_cart.longitude - dest_cart.longitude;
        if diff < -PI {
            start_cart.longitude += TWO_PI;
        } else if diff > PI {
            dest_cart.longitude += TWO_PI;
        }
    }

    let height_fn = create_height_function(
        camera,
        destination,
        start_cart.height,
        dest_cart.height,
        option_altitude,
    );
    let height_fn_for_pitch = create_height_function(
        camera,
        destination,
        start_cart.height,
        dest_cart.height,
        option_altitude,
    );
    let pitch_fn = create_pitch_function(
        start_pitch,
        pitch,
        height_fn_for_pitch,
        option_pitch_adjust_height,
    );

    let start_lon = start_cart.longitude;
    let dest_lon = dest_cart.longitude;
    let start_lat = start_cart.latitude;
    let dest_lat = dest_cart.latitude;

    Box::new(move |s, raw_time| {
        let time = raw_time / duration;
        let cart = Cartographic::new(
            lerp(start_lon, dest_lon, time),
            lerp(start_lat, dest_lat, time),
            height_fn(time),
        );
        let pos = ellipsoid
            .cartographic_to_cartesian(cart)
            .map(Cartesian3::from)
            .unwrap_or(destination);
        s.camera.set_view(
            Some(pos),
            Some(lerp(start_heading, heading, time)),
            Some(pitch_fn(time)),
            Some(lerp(start_roll, roll, time)),
        );
    })
}

fn create_update_2d(
    scene: &FlightScene,
    duration: f64,
    destination: Cartesian3,
    heading: f64,
    option_altitude: Option<f64>,
) -> UpdateFn {
    let camera = &scene.camera;
    let start = camera.position;
    let start_heading = adjust_angle_for_lerp(camera.heading, heading);

    let start_height = match camera.frustum {
        CameraFrustum::OrthographicOffCenter(o) => o.right - o.left,
        _ => 1.0,
    };
    let height_fn = create_height_function(
        camera,
        destination,
        start_height,
        destination.z,
        option_altitude,
    );
    let start_xy = Cartesian2::new(start.x, start.y);
    let dest_xy = Cartesian2::new(destination.x, destination.y);

    Box::new(move |s, raw_time| {
        let time = raw_time / duration;
        s.camera
            .set_view(None, Some(lerp(start_heading, heading, time)), None, None);
        let xy = Cartesian2::lerp(start_xy, dest_xy, time);
        s.camera.position.x = xy.x;
        s.camera.position.y = xy.y;

        let zoom = height_fn(time);
        if let CameraFrustum::OrthographicOffCenter(ref mut frustum) = s.camera.frustum {
            let ratio = frustum.top / frustum.right;
            let increment = (zoom - (frustum.right - frustum.left)) * 0.5;
            frustum.right += increment;
            frustum.left -= increment;
            frustum.top = ratio * frustum.right;
            frustum.bottom = -frustum.top;
        }
    })
}

impl CameraFlightPath {
    /// Creates a flight tween for `scene` toward `options.destination`.
    pub fn create_tween(
        scene: &mut FlightScene,
        options: CameraFlightOptions,
    ) -> Result<CameraFlightTween, DeveloperError> {
        let mut destination = options
            .destination
            .ok_or_else(|| DeveloperError::new("destination is required."))?;

        let mode = scene.mode;
        if mode == SceneMode::Morphing {
            return Ok(CameraFlightTween::empty(options.complete, options.cancel));
        }

        let convert = options.convert.unwrap_or(true);
        let projection = scene.map_projection;
        let ellipsoid = projection.ellipsoid();

        if convert && mode != SceneMode::Scene3D {
            if let Some(carto) = ellipsoid.cartesian_to_cartographic(DVec3::from(destination)) {
                if let Some(projected) = projection.project(carto) {
                    destination = Cartesian3::from(projected);
                }
            }
        }

        let duration = options.duration.unwrap_or_else(|| {
            let d = (scene.camera.position.distance(&destination) / 1_000_000.0).ceil() + 2.0;
            d.min(3.0)
        });

        let heading = options.heading.unwrap_or(0.0);
        let pitch = options.pitch.unwrap_or(-PI_OVER_TWO);
        let roll = options.roll.unwrap_or(0.0);

        scene.enable_inputs = false;

        let mut empty = mode == SceneMode::Scene2D;
        if empty {
            empty = Cartesian2::equals_epsilon(
                &Cartesian2::new(scene.camera.position.x, scene.camera.position.y),
                &Cartesian2::new(destination.x, destination.y),
                Some(EPSILON6),
                Some(EPSILON6),
            );
            if let CameraFrustum::OrthographicOffCenter(f) = scene.camera.frustum {
                let extent = (f.right - f.left).max(f.top - f.bottom);
                empty = empty && equals_epsilon(extent, destination.z, Some(EPSILON6), None);
            } else {
                empty = false;
            }
        } else {
            empty = Cartesian3::equals_epsilon(
                &destination,
                &scene.camera.position,
                Some(EPSILON10),
                Some(EPSILON10),
            );
        }

        empty = empty
            && equals_epsilon(
                negative_pi_to_pi(heading),
                negative_pi_to_pi(scene.camera.heading),
                Some(EPSILON10),
                None,
            )
            && equals_epsilon(
                negative_pi_to_pi(pitch),
                negative_pi_to_pi(scene.camera.pitch),
                Some(EPSILON10),
                None,
            )
            && equals_epsilon(
                negative_pi_to_pi(roll),
                negative_pi_to_pi(scene.camera.roll),
                Some(EPSILON10),
                None,
            );

        if empty {
            return Ok(CameraFlightTween::empty(options.complete, options.cancel));
        }

        let make_updater = |s: &FlightScene, dur: f64| -> UpdateFn {
            match mode {
                SceneMode::Scene2D => create_update_2d(
                    s,
                    dur,
                    destination,
                    heading,
                    options.maximum_height,
                ),
                SceneMode::ColumbusView => create_update_cv(
                    s,
                    dur,
                    destination,
                    heading,
                    pitch,
                    roll,
                    options.maximum_height,
                    options.pitch_adjust_height,
                ),
                _ => create_update_3d(
                    s,
                    dur,
                    destination,
                    heading,
                    pitch,
                    roll,
                    options.maximum_height,
                    options.fly_over_longitude,
                    options.fly_over_longitude_weight,
                    options.pitch_adjust_height,
                ),
            }
        };

        if duration <= 0.0 {
            let updater = make_updater(scene, 1.0);
            return Ok(CameraFlightTween {
                duration: 0.0,
                easing_function: EasingFunction::LinearNone,
                start_time: 0.0,
                stop_time: 0.0,
                is_empty: true,
                updater: Some(updater),
                complete_cb: options.complete,
                cancel_cb: options.cancel,
                apply_on_complete: true,
            });
        }

        let updater = make_updater(scene, duration);
        let easing = options.easing_function.unwrap_or_else(|| {
            let start_height = scene.camera.position_cartographic().height;
            let end_height = if mode == SceneMode::Scene3D {
                ellipsoid
                    .cartesian_to_cartographic(DVec3::from(destination))
                    .map(|c| c.height)
                    .unwrap_or(0.0)
            } else {
                destination.z
            };
            if start_height > end_height && start_height > 11500.0 {
                EasingFunction::CubicOut
            } else {
                EasingFunction::QuinticInOut
            }
        });

        Ok(CameraFlightTween {
            duration,
            easing_function: easing,
            start_time: 0.0,
            stop_time: duration,
            is_empty: false,
            updater: Some(updater),
            complete_cb: options.complete,
            cancel_cb: options.cancel,
            apply_on_complete: false,
        })
    }
}
