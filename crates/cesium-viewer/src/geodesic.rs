//! Bounded WGS84 Vincenty inverse/direct solution used by geographic wall arcs.
use cesium_math::WGS84_A;
const F: f64 = 1. / 298.257223563;
const B: f64 = WGS84_A * (1. - F);
#[derive(Clone, Copy, Debug)]
pub struct Line {
    start: [f64; 2],
    end: [f64; 2],
    pub distance: f64,
    heading: f64,
}
impl Line {
    pub fn new(start: [f64; 2], end: [f64; 2]) -> Result<Self, String> {
        if start.into_iter().chain(end).any(|n| !n.is_finite())
            || start[1].abs() > 85.
            || end[1].abs() > 85.
            || start[0].abs() > 180.
            || end[0].abs() > 180.
        {
            return Err("bounded wall geodesic coordinates required".into());
        }
        let p1 = start[1].to_radians();
        let p2 = end[1].to_radians();
        let u1 = ((1. - F) * p1.tan()).atan();
        let u2 = ((1. - F) * p2.tan()).atan();
        let (s1, c1) = u1.sin_cos();
        let (s2, c2) = u2.sin_cos();
        let l = (end[0] - start[0] + 180.).rem_euclid(360.).to_radians() - std::f64::consts::PI;
        let mut lam = l;
        let mut result = None;
        for _ in 0..128 {
            let (sl, cl) = lam.sin_cos();
            let ss = ((c2 * sl).powi(2) + (c1 * s2 - s1 * c2 * cl).powi(2)).sqrt();
            if ss < 1e-15 {
                if l.abs() < 1e-10 && (p1 - p2).abs() < 1e-10 {
                    return Ok(Self {
                        start,
                        end,
                        distance: 0.,
                        heading: 0.,
                    });
                }
                return Err("wall antipodal geodesic unsupported".into());
            }
            let cs = s1 * s2 + c1 * c2 * cl;
            let sig = ss.atan2(cs);
            let sa = c1 * c2 * sl / ss;
            let c2a = 1. - sa * sa;
            let c2sm = if c2a > 1e-15 {
                cs - 2. * s1 * s2 / c2a
            } else {
                0.
            };
            let c = F / 16. * c2a * (4. + F * (4. - 3. * c2a));
            let next =
                l + (1. - c) * F * sa * (sig + c * ss * (c2sm + c * cs * (-1. + 2. * c2sm * c2sm)));
            if (next - lam).abs() < 1e-12 {
                result = Some((ss, cs, sig, c2a, c2sm, lam));
                break;
            }
            lam = next;
        }
        let (ss, cs, sig, c2a, c2sm, lam) = result.ok_or("wall geodesic did not converge")?;
        let u2 = c2a * (WGS84_A * WGS84_A - B * B) / (B * B);
        let a = 1. + u2 / 16384. * (4096. + u2 * (-768. + u2 * (320. - 175. * u2)));
        let b = u2 / 1024. * (256. + u2 * (-128. + u2 * (74. - 47. * u2)));
        let delta = b
            * ss
            * (c2sm
                + b / 4.
                    * (cs * (-1. + 2. * c2sm * c2sm)
                        - b / 6. * c2sm * (-3. + 4. * ss * ss) * (-3. + 4. * c2sm * c2sm)));
        let distance = B * a * (sig - delta);
        let heading = (c2 * lam.sin()).atan2(c1 * s2 - s1 * c2 * lam.cos());
        Ok(Self {
            start,
            end,
            distance,
            heading,
        })
    }
    pub fn at(self, t: f64) -> [f64; 2] {
        if t <= 0. || self.distance == 0. {
            return self.start;
        }
        if t >= 1. {
            return self.end;
        }
        let u = ((1. - F) * self.start[1].to_radians().tan()).atan();
        let (su, cu) = u.sin_cos();
        let (sh, ch) = self.heading.sin_cos();
        let sig1 = u.tan().atan2(ch);
        let sa = cu * sh;
        let c2a = 1. - sa * sa;
        let u2 = c2a * (WGS84_A * WGS84_A - B * B) / (B * B);
        let a = 1. + u2 / 16384. * (4096. + u2 * (-768. + u2 * (320. - 175. * u2)));
        let b = u2 / 1024. * (256. + u2 * (-128. + u2 * (74. - 47. * u2)));
        let base = self.distance * t / (B * a);
        let mut sig = base;
        for _ in 0..32 {
            let (ss, cs) = sig.sin_cos();
            let cm = (2. * sig1 + sig).cos();
            let delta = b
                * ss
                * (cm
                    + b / 4.
                        * (cs * (-1. + 2. * cm * cm)
                            - b / 6. * cm * (-3. + 4. * ss * ss) * (-3. + 4. * cm * cm)));
            let next = base + delta;
            if (next - sig).abs() < 1e-13 {
                sig = next;
                break;
            }
            sig = next;
        }
        let (ss, cs) = sig.sin_cos();
        let tmp = su * ss - cu * cs * ch;
        let lat = (su * cs + cu * ss * ch).atan2((1. - F) * (sa * sa + tmp * tmp).sqrt());
        let lam = (ss * sh).atan2(cu * cs - su * ss * ch);
        let c = F / 16. * c2a * (4. + F * (4. - 3. * c2a));
        let cm = (2. * sig1 + sig).cos();
        let l = lam - (1. - c) * F * sa * (sig + c * ss * (cm + c * cs * (-1. + 2. * cm * cm)));
        let lon = (self.start[0] + l.to_degrees() + 180.).rem_euclid(360.) - 180.;
        [lon, lat.to_degrees()]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equator_distance_and_dateline() {
        let l = Line::new([0., 0.], [1., 0.]).unwrap();
        assert!((l.distance - 111319.490793).abs() < 0.001);
        assert!((l.at(0.5)[0] - 0.5).abs() < 1e-8);
        let l = Line::new([179., 20.], [-179., 20.]).unwrap();
        assert!(l.at(0.5)[0].abs() > 179.999);
    }
    #[test]
    fn long_wall_bows_and_reverses() {
        let a = [-115., 44.];
        let b = [-90., 44.];
        let l = Line::new(a, b).unwrap();
        let r = Line::new(b, a).unwrap();
        assert!(l.at(0.5)[1] > 44.65);
        for t in [0.1, 0.5, 0.9] {
            let p = l.at(t);
            let q = r.at(1. - t);
            assert!((p[0] - q[0]).abs() < 1e-8 && (p[1] - q[1]).abs() < 1e-8);
        }
        assert_eq!(l.at(0.), a);
        assert_eq!(l.at(1.), b);
    }
}
