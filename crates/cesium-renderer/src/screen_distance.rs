//! Static Cesium NearFarScalar curves evaluated from f64 camera distance.
use crate::RenderError;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScreenDistance {
    pub display: Option<[f64; 2]>,
    pub scale: Option<[f64; 4]>,
    pub translucency: Option<[f64; 4]>,
    pub offset: Option<[f64; 4]>,
    pub disable_depth_test_distance: f64,
}
impl ScreenDistance {
    pub fn validate(&self) -> Result<(), RenderError> {
        if !self.disable_depth_test_distance.is_finite()
            || !(0.0..=1e9).contains(&self.disable_depth_test_distance)
        {
            return Err(RenderError(
                "finite disableDepthTestDistance 0..1e9 required".into(),
            ));
        }
        if self.display.is_some_and(|[near, far]| {
            !near.is_finite() || !far.is_finite() || near < 0. || far - near < 0.001 || far > 1e9
        }) {
            return Err(RenderError(
                "distanceDisplayCondition requires 0<=near<far<=1e9".into(),
            ));
        }
        for (curve, max) in [(self.scale, 8.), (self.translucency, 1.), (self.offset, 8.)] {
            if curve.is_some_and(|[near, a, far, b]| {
                [near, a, far, b].iter().any(|x| !x.is_finite())
                    || near < 0.
                    || far - near < 0.001
                    || far > 1e9
                    || !(0.0..=max).contains(&a)
                    || !(0.0..=max).contains(&b)
            }) {
                return Err(RenderError(
                    "bounded NearFarScalar [near,nearValue,far,farValue] required".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn overlay(&self, distance: f64) -> bool {
        self.disable_depth_test_distance > 0. && distance > self.disable_depth_test_distance
    }
    pub fn offset_scale(&self, distance: f64) -> f32 {
        if !distance.is_finite() {
            return 0.;
        }
        self.offset.map_or(1., |[near, a, far, b]| {
            let t = ((distance * distance - near * near) / (far * far - near * near))
                .clamp(0., 1.)
                .powf(0.2);
            (a + (b - a) * t) as f32
        })
    }
    pub fn evaluate(&self, distance: f64) -> [f32; 3] {
        if !distance.is_finite() {
            return [1., 0., 0.];
        }
        let visible = self
            .display
            .is_none_or(|[a, b]| distance >= a && distance <= b);
        let curve = |v: Option<[f64; 4]>| {
            v.map_or(1., |[near, a, far, b]| {
                // Same squared-distance/pow(0.2) interpolation as pinned Cesium 1.146.
                let t = ((distance * distance - near * near) / (far * far - near * near))
                    .clamp(0., 1.)
                    .powf(0.2);
                a + (b - a) * t
            })
        };
        [
            curve(self.scale) as f32,
            curve(self.translucency) as f32,
            if visible { 1. } else { 0. },
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn near_far_clamps_and_reference_interpolation() {
        let options = ScreenDistance {
            display: Some([10., 100.]),
            scale: Some([10., 2., 100., 0.5]),
            translucency: Some([10., 1., 100., 0.]),
            ..Default::default()
        };
        options.validate().unwrap();
        assert_eq!(options.evaluate(10.), [2., 1., 1.]);
        assert_eq!(options.evaluate(100.), [0.5, 0., 1.]);
        assert_eq!(options.evaluate(101.)[2], 0.);
        let t = ((2500_f64 - 100.) / (10000. - 100.)).powf(0.2);
        assert!((options.evaluate(50.)[0] as f64 - (2. - 1.5 * t)).abs() < 1e-6);
        assert_eq!(options.evaluate(0.)[0], 2.);
        for bad in [
            ScreenDistance {
                display: Some([1., 1.]),
                ..Default::default()
            },
            ScreenDistance {
                scale: Some([0., 9., 100., 1.]),
                ..Default::default()
            },
            ScreenDistance {
                translucency: Some([0., 1., 100., -1.]),
                ..Default::default()
            },
        ] {
            assert!(bad.validate().is_err());
        }
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    #[test]
    fn offset_curve_and_depth_threshold_are_independent() {
        let x = ScreenDistance {
            offset: Some([40000., 1., 160000., 3.]),
            disable_depth_test_distance: 80000.,
            ..Default::default()
        };
        x.validate().unwrap();
        assert_eq!(x.offset_scale(20000.), 1.);
        assert_eq!(x.offset_scale(220000.), 3.);
        assert!(!x.overlay(80000.));
        assert!(x.overlay(80001.));
        assert!(!ScreenDistance::default().overlay(1e8));
        assert!(ScreenDistance {
            disable_depth_test_distance: f64::INFINITY,
            ..Default::default()
        }
        .validate()
        .is_err());
    }
}
