//! Bounded perspective camera control. Angles in radians, geodetic destinations in degrees/metres.
use crate::{Orbit, RenderError, Scene};
use cesium_math::{Camera, Cartographic, WGS84_A};
use glam::{DMat3, DQuat, DVec3};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ViewOptions {
    pub destination_degrees: [f64; 3],
    #[serde(default)]
    pub heading: f64,
    #[serde(default = "down")]
    pub pitch: f64,
    #[serde(default)]
    pub roll: f64,
}
fn down() -> f64 {
    -std::f64::consts::FRAC_PI_2
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct LookOffset {
    pub heading: f64,
    pub pitch: f64,
    pub range: f64,
}
impl Default for LookOffset {
    fn default() -> Self {
        Self {
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_4,
            range: 100000.,
        }
    }
}
impl LookOffset {
    pub fn validate(self) -> Result<(), RenderError> {
        if !self.heading.is_finite()
            || self.heading.abs() > 10000.
            || !self.pitch.is_finite()
            || !(-std::f64::consts::FRAC_PI_2..=0.).contains(&self.pitch)
            || !self.range.is_finite()
            || !(10.0..=30000000.).contains(&self.range)
        {
            return Err(RenderError(
                "bounded heading/pitch(-pi/2..0)/range required".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn orbit(self, target: DVec3) -> Result<Orbit, RenderError> {
        self.validate()?;
        let c = Cartographic::from_ecef(target).map_err(|e| RenderError(e.to_string()))?;
        let mut orbit = Orbit::new(target, self.range / 4.);
        orbit.distance = self.range;
        orbit.minimum_distance = 10.;
        orbit.basis = DMat3::from_cols(
            c.enu_to_ecef().x_axis.truncate(),
            c.enu_to_ecef().y_axis.truncate(),
            c.enu_to_ecef().z_axis.truncate(),
        );
        orbit.yaw = -std::f64::consts::FRAC_PI_2 - self.heading;
        orbit.pitch = (-self.pitch).clamp(1e-6, std::f64::consts::FRAC_PI_2 - 1e-6);
        let height = Cartographic::from_ecef(orbit.camera(1, 1).eye)
            .map_err(|e| RenderError(e.to_string()))?
            .height;
        if height < 9.999 {
            return Err(RenderError(
                "lookAt camera requires 10m ellipsoid clearance".into(),
            ));
        }
        Ok(orbit)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub eye: DVec3,
    pub rotation: DQuat,
}
impl ViewOptions {
    pub fn validate(self) -> Result<(), RenderError> {
        let [lon, lat, height] = self.destination_degrees;
        if !lon.is_finite()
            || !lat.is_finite()
            || !(-180.0..=180.).contains(&lon)
            || !(-90.0..=90.).contains(&lat)
            || !height.is_finite()
            || !(10.0..=30000000.).contains(&height)
            || [self.heading, self.pitch, self.roll]
                .iter()
                .any(|x| !x.is_finite() || x.abs() > 10000.)
            || self.pitch.abs() > std::f64::consts::FRAC_PI_2
        {
            return Err(RenderError(
                "bounded camera destination/HPR required".into(),
            ));
        }
        Ok(())
    }
    pub fn pose(self) -> Result<Pose, RenderError> {
        self.validate()?;
        let c = Cartographic::degrees(
            self.destination_degrees[0],
            self.destination_degrees[1],
            self.destination_degrees[2],
        )
        .map_err(|e| RenderError(e.to_string()))?;
        let enu = c.enu_to_ecef();
        let basis = DMat3::from_cols(
            enu.x_axis.truncate(),
            enu.y_axis.truncate(),
            enu.z_axis.truncate(),
        );
        let forward = DVec3::new(
            self.heading.sin() * self.pitch.cos(),
            self.heading.cos() * self.pitch.cos(),
            self.pitch.sin(),
        );
        let right = DVec3::new(self.heading.cos(), -self.heading.sin(), 0.);
        let up = right.cross(forward);
        let right = right * self.roll.cos() + up * self.roll.sin();
        let up = right.cross(forward);
        Ok(Pose {
            eye: c.to_ecef(),
            rotation: DQuat::from_mat3(&(basis * DMat3::from_cols(right, up, -forward)))
                .normalize(),
        })
    }
}
impl Pose {
    pub fn from_camera(camera: Camera) -> Self {
        Self {
            eye: camera.eye,
            rotation: DQuat::from_mat3(&camera.camera_to_ecef).normalize(),
        }
    }
    pub fn camera(self, w: u32, h: u32) -> Camera {
        let height = Cartographic::from_ecef(self.eye).map_or(1000., |c| c.height);
        Camera {
            eye: self.eye,
            camera_to_ecef: DMat3::from_quat(self.rotation),
            fovy: Camera::cesium_fovy(w, h),
            near: (height * 0.01).clamp(0.1, 1000.),
            width: w,
            height: h,
        }
    }
    pub fn options(self) -> ViewOptions {
        let c = Cartographic::from_ecef(self.eye).expect("validated camera eye");
        let enu = c.enu_to_ecef();
        let basis = DMat3::from_cols(
            enu.x_axis.truncate(),
            enu.y_axis.truncate(),
            enu.z_axis.truncate(),
        );
        let local = basis.transpose() * DMat3::from_quat(self.rotation);
        let forward = -local.z_axis;
        let heading = forward.x.atan2(forward.y);
        let pitch = forward.z.clamp(-1., 1.).asin();
        let right = DVec3::new(heading.cos(), -heading.sin(), 0.);
        let up = right.cross(forward).normalize();
        let roll = local.x_axis.dot(up).atan2(local.x_axis.dot(right));
        ViewOptions {
            destination_degrees: [c.longitude.to_degrees(), c.latitude.to_degrees(), c.height],
            heading,
            pitch,
            roll,
        }
    }
    pub(crate) fn drag(&mut self, dx: f64, dy: f64) {
        if !dx.is_finite() || !dy.is_finite() || (dx == 0. && dy == 0.) {
            return;
        }
        let mut v = self.options();
        v.heading = (v.heading - dx * 0.005).rem_euclid(std::f64::consts::TAU);
        v.pitch =
            (v.pitch + dy * 0.005).clamp(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
        if let Ok(p) = v.pose() {
            *self = p;
        }
    }
    pub(crate) fn zoom(&mut self, delta: f64) {
        if !delta.is_finite() || delta == 0. {
            return;
        }
        let v = self.options();
        let scale = (-delta.clamp(-100., 100.) * 0.1).exp();
        let forward = -(self.rotation * DVec3::Z);
        let next = self.eye + forward * v.destination_degrees[2] * (1. - scale);
        if let Ok(mut c) = Cartographic::from_ecef(next) {
            c.height = c.height.clamp(10., 30000000.);
            self.eye = c.to_ecef();
        }
    }
}
#[derive(Clone, Debug)]
pub struct Flight {
    start: Pose,
    end: Pose,
    start_geo: Cartographic,
    end_geo: Cartographic,
    duration: f64,
    elapsed: f64,
    lift: f64,
}
fn normal(c: Cartographic) -> DVec3 {
    DVec3::new(
        c.latitude.cos() * c.longitude.cos(),
        c.latitude.cos() * c.longitude.sin(),
        c.latitude.sin(),
    )
}
fn arc(a: DVec3, b: DVec3, t: f64) -> DVec3 {
    let angle = a.dot(b).clamp(-1., 1.).acos();
    if angle < 1e-8 {
        return a.lerp(b, t).normalize();
    }
    let axis = a.cross(b).try_normalize().unwrap_or_else(|| {
        a.cross(if a.z.abs() < 0.9 { DVec3::Z } else { DVec3::X })
            .normalize()
    });
    DQuat::from_axis_angle(axis, angle * t) * a
}
impl Flight {
    pub fn new(start: Pose, end: Pose, duration: f64) -> Result<Self, RenderError> {
        if !duration.is_finite() || !(0.0..=120.).contains(&duration) {
            return Err(RenderError(
                "flight duration 0..120 seconds required".into(),
            ));
        }
        let start_geo =
            Cartographic::from_ecef(start.eye).map_err(|e| RenderError(e.to_string()))?;
        let end_geo = Cartographic::from_ecef(end.eye).map_err(|e| RenderError(e.to_string()))?;
        let lift = (normal(start_geo).dot(normal(end_geo)).clamp(-1., 1.).acos() * WGS84_A * 0.25)
            .min(10000000.);
        Ok(Self {
            start,
            end,
            start_geo,
            end_geo,
            duration,
            elapsed: 0.,
            lift,
        })
    }
    pub fn progress(&self) -> f64 {
        if self.duration == 0. {
            1.
        } else {
            (self.elapsed / self.duration).clamp(0., 1.)
        }
    }
    pub fn advance(&mut self, elapsed: f64) -> Pose {
        if elapsed.is_finite() && (0.0..=60.).contains(&elapsed) {
            self.elapsed += elapsed;
        }
        let t = self.progress();
        if t >= 1. {
            return self.end;
        }
        if t == 0. {
            return self.start;
        }
        let t = t * t * (3. - 2. * t);
        let n = arc(normal(self.start_geo), normal(self.end_geo), t);
        let height = (self.start_geo.height * (1. - t)
            + self.end_geo.height * t
            + self.lift * (std::f64::consts::PI * t).sin().powi(2))
        .clamp(10., 30000000.);
        let c = Cartographic::radians(n.y.atan2(n.x), n.z.clamp(-1., 1.).asin(), height).unwrap();
        Pose {
            eye: c.to_ecef(),
            rotation: self.start.rotation.slerp(self.end.rotation, t).normalize(),
        }
    }
}
#[derive(Default)]
pub(crate) struct Control {
    pub flight: Option<Flight>,
    pub tracked: Option<(String, LookOffset)>,
    pub flight_id: u64,
    pub status: &'static str,
    pub progress: f64,
    pub mode: &'static str,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    SetView {
        view: ViewOptions,
    },
    FlyTo {
        view: ViewOptions,
        duration: f64,
    },
    CancelFlight,
    LookAt {
        #[serde(rename = "targetDegrees")]
        target: [f64; 3],
        #[serde(default)]
        offset: LookOffset,
    },
    TrackEntity {
        id: String,
        #[serde(default)]
        offset: LookOffset,
    },
    StopTracking,
}
impl Scene {
    pub fn set_view(&mut self, view: ViewOptions) -> Result<(), RenderError> {
        let pose = view.pose()?;
        self.release_camera();
        self.orbit.view = Some(pose);
        self.camera_control.mode = "view";
        Ok(())
    }
    pub fn fly_to(&mut self, view: ViewOptions, duration: f64) -> Result<u64, RenderError> {
        let end = view.pose()?;
        let start = Pose::from_camera(self.orbit.camera(1, 1));
        let flight = Flight::new(start, end, duration)?;
        self.release_camera();
        self.camera_control.flight_id = self.camera_control.flight_id.wrapping_add(1);
        self.camera_control.progress = 0.;
        self.orbit.view = Some(start);
        if duration == 0. {
            self.orbit.view = Some(end);
            self.camera_control.progress = 1.;
            self.camera_control.status = "completed";
            self.camera_control.mode = "view";
        } else {
            self.camera_control.flight = Some(flight);
            self.camera_control.status = "flying";
            self.camera_control.mode = "flying";
        }
        Ok(self.camera_control.flight_id)
    }
    pub fn cancel_flight(&mut self) -> bool {
        if self.camera_control.flight.take().is_some() {
            self.camera_control.status = "cancelled";
            self.camera_control.mode = "view";
            true
        } else {
            false
        }
    }
    pub fn stop_tracking(&mut self) -> bool {
        if self.camera_control.tracked.take().is_some() {
            self.camera_control.mode = "orbit";
            true
        } else {
            false
        }
    }
    pub(crate) fn release_camera(&mut self) {
        self.cancel_flight();
        self.stop_tracking();
        self.terrain_focus = None;
        self.camera_control.mode = "orbit";
    }
    pub fn camera_drag(&mut self, dx: f64, dy: f64) {
        if !dx.is_finite() || !dy.is_finite() || (dx == 0. && dy == 0.) {
            return;
        }
        self.release_camera();
        self.orbit.drag(dx, dy);
        if self.orbit.view.is_some() {
            self.camera_control.mode = "view";
        }
    }
    pub fn camera_zoom(&mut self, delta: f64) {
        if !delta.is_finite() || delta == 0. {
            return;
        }
        self.release_camera();
        self.orbit.zoom(delta);
        if self.orbit.view.is_some() {
            self.camera_control.mode = "view";
        }
    }
    pub fn look_at(&mut self, target: [f64; 3], offset: LookOffset) -> Result<(), RenderError> {
        let c = Cartographic::degrees(target[0], target[1], target[2])
            .map_err(|e| RenderError(e.to_string()))?;
        if target[0].abs() > 180. || !(-10000.0..=1e8).contains(&target[2]) {
            return Err(RenderError("lookAt target bounds".into()));
        }
        let orbit = offset.orbit(c.to_ecef())?;
        self.release_camera();
        self.orbit = orbit;
        self.camera_control.mode = "lookAt";
        Ok(())
    }
    pub fn track_entity(&mut self, id: &str, offset: LookOffset) -> Result<(), RenderError> {
        offset.validate()?;
        let target = self
            .entities
            .get(id)
            .and_then(|e| e.spec.position(self.clock.current_time))
            .ok_or_else(|| RenderError("tracked entity unavailable".into()))?;
        let orbit = offset.orbit(target)?;
        self.release_camera();
        self.orbit = orbit;
        self.camera_control.tracked = Some((id.into(), offset));
        self.camera_control.mode = "tracking";
        Ok(())
    }
    pub(crate) fn advance_camera(&mut self, elapsed: f64) {
        if let Some(flight) = &mut self.camera_control.flight {
            self.orbit.view = Some(flight.advance(elapsed));
            self.camera_control.progress = flight.progress();
            if self.camera_control.progress >= 1. {
                self.camera_control.flight = None;
                self.camera_control.status = "completed";
                self.camera_control.mode = "view";
            }
        }
        if let Some((id, offset)) = &self.camera_control.tracked {
            if let Some(position) = self
                .entities
                .get(id)
                .and_then(|e| e.spec.position(self.clock.current_time))
            {
                if let Ok(orbit) = offset.orbit(position) {
                    self.orbit = orbit;
                } else {
                    self.stop_tracking();
                }
            } else {
                self.stop_tracking();
            }
        }
    }
    pub fn camera_state(&self) -> serde_json::Value {
        let view = Pose::from_camera(self.orbit.camera(1, 1)).options();
        serde_json::json!({"mode":if self.camera_control.mode.is_empty(){"orbit"}else{self.camera_control.mode},"view":view,"flightId":self.camera_control.flight_id,"flightStatus":if self.camera_control.status.is_empty(){"idle"}else{self.camera_control.status},"flightProgress":self.camera_control.progress,"trackedEntity":self.camera_control.tracked.as_ref().map(|(id,_)|id)})
    }
    pub fn camera_command(&mut self, command: Command) -> Result<(), RenderError> {
        match command {
            Command::SetView { view } => self.set_view(view),
            Command::FlyTo { view, duration } => self.fly_to(view, duration).map(|_| ()),
            Command::CancelFlight => {
                self.cancel_flight();
                Ok(())
            }
            Command::StopTracking => {
                self.stop_tracking();
                Ok(())
            }
            Command::LookAt { target, offset } => self.look_at(target, offset),
            Command::TrackEntity { id, offset } => self.track_entity(&id, offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn view(lon: f64, lat: f64) -> ViewOptions {
        ViewOptions {
            destination_degrees: [lon, lat, 90000.],
            heading: 0.7,
            pitch: -0.8,
            roll: 0.2,
        }
    }
    #[test]
    fn set_view_hpr_roundtrip_and_poles() {
        let v = view(110., 25.);
        let pose = v.pose().unwrap();
        let actual = pose.options();
        assert!((actual.heading - v.heading).abs() < 1e-10);
        assert!((actual.pitch - v.pitch).abs() < 1e-10);
        assert!((actual.roll - v.roll).abs() < 1e-10);
        for lat in [-90., 0., 90.] {
            let p = view(179., lat).pose().unwrap();
            let m = p.camera(768, 512).camera_to_ecef;
            assert!(m.is_finite());
            assert!((m.determinant() - 1.).abs() < 1e-12);
            assert!((m.transpose() * m - DMat3::IDENTITY).abs_diff_eq(DMat3::ZERO, 1e-12));
        }
    }
    #[test]
    fn flight_dateline_arc_and_exact_endpoint() {
        let a = view(179., 20.).pose().unwrap();
        let b = view(-179., 20.).pose().unwrap();
        let mut f = Flight::new(a, b, 4.).unwrap();
        assert_eq!(f.advance(0.).eye, a.eye);
        let mid = f.advance(2.);
        let c = Cartographic::from_ecef(mid.eye).unwrap();
        assert!(c.longitude.to_degrees().abs() > 179.9);
        assert!(c.height > 90000.);
        assert_eq!(f.advance(2.).eye, b.eye);
        assert_eq!(f.progress(), 1.);
    }
    #[test]
    fn flight_antipodes_pause_and_zero_duration() {
        let a = view(0., 0.).pose().unwrap();
        let b = view(180., 0.).pose().unwrap();
        let mut f = Flight::new(a, b, 10.).unwrap();
        for _ in 0..10 {
            let p = f.advance(1.);
            assert!(p.eye.is_finite());
            assert!(Cartographic::from_ecef(p.eye).unwrap().height >= 89999.);
            assert!((DMat3::from_quat(p.rotation).determinant() - 1.).abs() < 1e-10);
        }
        let progress = f.progress();
        f.advance(f64::NAN);
        f.advance(-1.);
        assert_eq!(progress, f.progress());
        assert_eq!(Flight::new(a, b, 0.).unwrap().advance(0.).eye, b.eye);
    }
    #[test]
    fn bounds_and_strict_commands() {
        let mut v = view(110., 25.);
        v.destination_degrees[2] = 0.;
        assert!(v.pose().is_err());
        v = view(110., 25.);
        v.pitch = 2.;
        assert!(v.pose().is_err());
        assert!(LookOffset {
            range: 0.,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(serde_json::from_str::<Command>(r#"{"type":"setView","view":{"destinationDegrees":[110,25,1000],"orientation":"unsupported"}}"#).is_err());
        assert!(Flight::new(
            view(0., 0.).pose().unwrap(),
            view(1., 0.).pose().unwrap(),
            121.
        )
        .is_err());
    }
    #[test]
    fn look_at_heading_range_and_manual_pose_clearance() {
        let c = Cartographic::degrees(110., 25., 16000.).unwrap();
        let close = LookOffset {
            range: 10.,
            ..Default::default()
        }
        .orbit(c.to_ecef())
        .unwrap();
        assert!((close.camera(1, 1).eye.distance(c.to_ecef()) - 10.).abs() < 1e-6);
        let offset = LookOffset {
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_4,
            range: 40000.,
        };
        let orbit = offset.orbit(c.to_ecef()).unwrap();
        let camera = orbit.camera(768, 512);
        assert!((camera.eye.distance(c.to_ecef()) - 40000.).abs() < 1e-6);
        assert!(
            (-camera.camera_to_ecef.z_axis - (c.to_ecef() - camera.eye).normalize()).length()
                < 1e-10
        );
        let mut pose = ViewOptions {
            destination_degrees: [110., 25., 100.],
            heading: 0.,
            pitch: down(),
            roll: 0.,
        }
        .pose()
        .unwrap();
        for _ in 0..20 {
            pose.zoom(10.);
        }
        assert!((pose.options().destination_degrees[2] - 10.).abs() < 1e-5);
        pose.drag(10., 10.);
        assert!(pose.camera(1, 1).camera_to_ecef.is_finite());
    }
}
