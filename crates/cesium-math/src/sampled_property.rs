//! SampledProperty and SampledPositionProperty matching CesiumJS `DataSources/SampledProperty.js` and `SampledPositionProperty.js` (Apache-2.0).

use crate::{
    Cartesian2, Cartesian3, Cartesian4, DynPositionProperty, DynProperty, Event,
    HermitePolynomialApproximation, JulianDate, LagrangePolynomialApproximation,
    LinearApproximation, PositionProperty, Quaternion, ReferenceFrame,
};

/// Constants to determine how an interpolated value is extrapolated when querying outside the bounds of available data.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ExtrapolationType {
    #[default]
    None = 0,
    Hold = 1,
    Extrapolate = 2,
}

/// The interpolation algorithm to use when interpolating sampled properties.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum InterpolationAlgorithm {
    #[default]
    Linear,
    Lagrange,
    Hermite,
}

/// A trait for types that can be packed and unpacked for time-series interpolation.
pub trait Sampleable: Clone + PartialEq + Default + 'static {
    const PACKED_LENGTH: usize;
    fn pack(&self, dest: &mut [f64]);
    fn unpack(src: &[f64]) -> Self;

    fn convert_packed_array_for_interpolation(
        packed_array: &[f64],
        starting_index: usize,
        last_index: usize,
        result: &mut [f64],
    ) {
        let stride = Self::PACKED_LENGTH;
        let start = starting_index * stride;
        let count = (last_index - starting_index + 1) * stride;
        result[..count].copy_from_slice(&packed_array[start..start + count]);
    }

    fn unpack_interpolation_result(
        interpolated: &[f64],
        _source_array: &[f64],
        _last_index: usize,
    ) -> Self {
        Self::unpack(interpolated)
    }
}

impl Sampleable for f64 {
    const PACKED_LENGTH: usize = 1;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = *self;
    }
    fn unpack(src: &[f64]) -> Self {
        src[0]
    }
}

impl Sampleable for Cartesian2 {
    const PACKED_LENGTH: usize = 2;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = self.x;
        dest[1] = self.y;
    }
    fn unpack(src: &[f64]) -> Self {
        Cartesian2::new(src[0], src[1])
    }
}

impl Sampleable for Cartesian3 {
    const PACKED_LENGTH: usize = 3;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = self.x;
        dest[1] = self.y;
        dest[2] = self.z;
    }
    fn unpack(src: &[f64]) -> Self {
        Cartesian3::new(src[0], src[1], src[2])
    }
}

impl Sampleable for Cartesian4 {
    const PACKED_LENGTH: usize = 4;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = self.x;
        dest[1] = self.y;
        dest[2] = self.z;
        dest[3] = self.w;
    }
    fn unpack(src: &[f64]) -> Self {
        Cartesian4::new(src[0], src[1], src[2], src[3])
    }
}

/// Represents an angle in radians that always interpolates towards the shortest angular distance.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rotation(pub f64);

impl Sampleable for Rotation {
    const PACKED_LENGTH: usize = 1;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = self.0;
    }
    fn unpack(src: &[f64]) -> Self {
        Rotation(src[0])
    }

    fn convert_packed_array_for_interpolation(
        packed_array: &[f64],
        starting_index: usize,
        last_index: usize,
        result: &mut [f64],
    ) {
        let mut previous_value = 0.0;
        for i in 0..(last_index - starting_index + 1) {
            let value = packed_array[starting_index + i];
            if i == 0 || (previous_value - value).abs() < std::f64::consts::PI {
                result[i] = value;
            } else {
                result[i] = value - crate::core_math::TWO_PI;
            }
            previous_value = value;
        }
    }

    fn unpack_interpolation_result(
        interpolated: &[f64],
        _source_array: &[f64],
        _last_index: usize,
    ) -> Self {
        let mut res = interpolated[0];
        if res < 0.0 {
            res += crate::core_math::TWO_PI;
        }
        Rotation(res)
    }
}

impl Sampleable for Quaternion {
    const PACKED_LENGTH: usize = 4;
    fn pack(&self, dest: &mut [f64]) {
        dest[0] = self.x;
        dest[1] = self.y;
        dest[2] = self.z;
        dest[3] = self.w;
    }
    fn unpack(src: &[f64]) -> Self {
        Quaternion::new(src[0], src[1], src[2], src[3])
    }
}

/// A Property whose value is interpolated for a given time from the provided set of samples
/// and specified interpolation algorithm and degree.
pub struct SampledProperty<T: Sampleable> {
    times: Vec<JulianDate>,
    values: Vec<f64>,
    interpolation_degree: usize,
    interpolation_algorithm: InterpolationAlgorithm,
    forward_extrapolation_type: ExtrapolationType,
    forward_extrapolation_duration: f64,
    backward_extrapolation_type: ExtrapolationType,
    backward_extrapolation_duration: f64,
    definition_changed: Event<()>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Sampleable> Clone for SampledProperty<T> {
    fn clone(&self) -> Self {
        Self {
            times: self.times.clone(),
            values: self.values.clone(),
            interpolation_degree: self.interpolation_degree,
            interpolation_algorithm: self.interpolation_algorithm,
            forward_extrapolation_type: self.forward_extrapolation_type,
            forward_extrapolation_duration: self.forward_extrapolation_duration,
            backward_extrapolation_type: self.backward_extrapolation_type,
            backward_extrapolation_duration: self.backward_extrapolation_duration,
            definition_changed: self.definition_changed.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Sampleable> Default for SampledProperty<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sampleable> SampledProperty<T> {
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            values: Vec::new(),
            interpolation_degree: 1,
            interpolation_algorithm: InterpolationAlgorithm::Linear,
            forward_extrapolation_type: ExtrapolationType::None,
            forward_extrapolation_duration: 0.0,
            backward_extrapolation_type: ExtrapolationType::None,
            backward_extrapolation_duration: 0.0,
            definition_changed: Event::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn is_constant(&self) -> bool {
        self.times.is_empty()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn interpolation_degree(&self) -> usize {
        self.interpolation_degree
    }

    pub fn interpolation_algorithm(&self) -> InterpolationAlgorithm {
        self.interpolation_algorithm
    }

    pub fn forward_extrapolation_type(&self) -> ExtrapolationType {
        self.forward_extrapolation_type
    }

    pub fn set_forward_extrapolation_type(&mut self, ext: ExtrapolationType) {
        if self.forward_extrapolation_type != ext {
            self.forward_extrapolation_type = ext;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn forward_extrapolation_duration(&self) -> f64 {
        self.forward_extrapolation_duration
    }

    pub fn set_forward_extrapolation_duration(&mut self, dur: f64) {
        if (self.forward_extrapolation_duration - dur).abs() > f64::EPSILON {
            self.forward_extrapolation_duration = dur;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn backward_extrapolation_type(&self) -> ExtrapolationType {
        self.backward_extrapolation_type
    }

    pub fn set_backward_extrapolation_type(&mut self, ext: ExtrapolationType) {
        if self.backward_extrapolation_type != ext {
            self.backward_extrapolation_type = ext;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn backward_extrapolation_duration(&self) -> f64 {
        self.backward_extrapolation_duration
    }

    pub fn times(&self) -> &[JulianDate] {
        &self.times
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn get_sample(&self, index: isize) -> Option<JulianDate> {
        let len = self.times.len() as isize;
        if index >= 0 {
            if index < len {
                Some(self.times[index as usize])
            } else {
                None
            }
        } else {
            let actual = len + index;
            if actual >= 0 && actual < len {
                Some(self.times[actual as usize])
            } else {
                None
            }
        }
    }

    pub fn remove_sample(&mut self, time: &JulianDate) -> bool {
        if let Ok(idx) = self.times.binary_search_by(|t| JulianDate::compare(t, time)) {
            self.times.remove(idx);
            let start = idx * T::PACKED_LENGTH;
            self.values.drain(start..start + T::PACKED_LENGTH);
            self.definition_changed.raise_event(&());
            true
        } else {
            false
        }
    }

    pub fn remove_samples(&mut self, interval: &crate::TimeInterval) -> bool {
        let mut i = 0;
        let mut removed = false;
        while i < self.times.len() {
            if interval.contains(&self.times[i]) {
                self.times.remove(i);
                let start = i * T::PACKED_LENGTH;
                self.values.drain(start..start + T::PACKED_LENGTH);
                removed = true;
            } else {
                i += 1;
            }
        }
        if removed {
            self.definition_changed.raise_event(&());
        }
        removed
    }

    pub fn set_backward_extrapolation_duration(&mut self, dur: f64) {
        if (self.backward_extrapolation_duration - dur).abs() > f64::EPSILON {
            self.backward_extrapolation_duration = dur;
            self.definition_changed.raise_event(&());
        }
    }

    pub fn set_interpolation_options(
        &mut self,
        degree: Option<usize>,
        algorithm: Option<InterpolationAlgorithm>,
    ) {
        let mut changed = false;
        if let Some(d) = degree {
            if self.interpolation_degree != d {
                self.interpolation_degree = d;
                changed = true;
            }
        }
        if let Some(a) = algorithm {
            if self.interpolation_algorithm != a {
                self.interpolation_algorithm = a;
                changed = true;
            }
        }
        if changed {
            self.definition_changed.raise_event(&());
        }
    }

    pub fn add_sample(&mut self, time: JulianDate, value: T) {
        let mut packed = vec![0.0; T::PACKED_LENGTH];
        value.pack(&mut packed);

        match self.times.binary_search_by(|t| JulianDate::compare(t, &time)) {
            Ok(idx) => {
                let start = idx * T::PACKED_LENGTH;
                self.values[start..start + T::PACKED_LENGTH].copy_from_slice(&packed);
            }
            Err(idx) => {
                self.times.insert(idx, time);
                let start = idx * T::PACKED_LENGTH;
                self.values.splice(start..start, packed);
            }
        }
        self.definition_changed.raise_event(&());
    }

    pub fn add_samples(&mut self, times: &[JulianDate], values: &[T]) {
        assert_eq!(
            times.len(),
            values.len(),
            "times and values must have the same length"
        );
        for (t, v) in times.iter().zip(values.iter()) {
            let mut packed = vec![0.0; T::PACKED_LENGTH];
            v.pack(&mut packed);
            match self.times.binary_search_by(|cur| JulianDate::compare(cur, t)) {
                Ok(idx) => {
                    let start = idx * T::PACKED_LENGTH;
                    self.values[start..start + T::PACKED_LENGTH].copy_from_slice(&packed);
                }
                Err(idx) => {
                    self.times.insert(idx, *t);
                    let start = idx * T::PACKED_LENGTH;
                    self.values.splice(start..start, packed);
                }
            }
        }
        self.definition_changed.raise_event(&());
    }

    pub fn add_samples_packed_array(&mut self, data: &[f64], epoch: &JulianDate) {
        let stride = 1 + T::PACKED_LENGTH;
        assert_eq!(
            data.len() % stride,
            0,
            "packed array length must be multiple of 1 + PACKED_LENGTH"
        );
        for chunk in data.chunks_exact(stride) {
            let seconds_offset = chunk[0];
            let time = JulianDate::add_seconds(epoch, seconds_offset);
            let packed = &chunk[1..];
            match self.times.binary_search_by(|cur| JulianDate::compare(cur, &time)) {
                Ok(idx) => {
                    let start = idx * T::PACKED_LENGTH;
                    self.values[start..start + T::PACKED_LENGTH].copy_from_slice(packed);
                }
                Err(idx) => {
                    self.times.insert(idx, time);
                    let start = idx * T::PACKED_LENGTH;
                    self.values.splice(start..start, packed.to_vec());
                }
            }
        }
        self.definition_changed.raise_event(&());
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        let default_time;
        let t = match time {
            Some(time) => time,
            None => {
                default_time = JulianDate::now();
                &default_time
            }
        };

        let times_len = self.times.len();
        if times_len == 0 {
            return None;
        }

        let packed_len = T::PACKED_LENGTH;

        if times_len == 1 {
            let cmp = JulianDate::compare(t, &self.times[0]);
            if cmp == std::cmp::Ordering::Equal {
                return Some(T::unpack(&self.values[..packed_len]));
            }
            if cmp == std::cmp::Ordering::Less {
                if self.backward_extrapolation_type == ExtrapolationType::Hold {
                    let diff = JulianDate::seconds_difference(&self.times[0], t);
                    if self.backward_extrapolation_duration == 0.0
                        || diff <= self.backward_extrapolation_duration
                    {
                        return Some(T::unpack(&self.values[..packed_len]));
                    }
                }
                return None;
            }
            if self.forward_extrapolation_type == ExtrapolationType::Hold {
                let diff = JulianDate::seconds_difference(t, &self.times[0]);
                if self.forward_extrapolation_duration == 0.0
                    || diff <= self.forward_extrapolation_duration
                {
                    return Some(T::unpack(&self.values[..packed_len]));
                }
            }
            return None;
        }

        match self.times.binary_search_by(|cur| JulianDate::compare(cur, t)) {
            Ok(idx) => {
                let start = idx * packed_len;
                Some(T::unpack(&self.values[start..start + packed_len]))
            }
            Err(index) => {
                if index == 0 {
                    if self.backward_extrapolation_type == ExtrapolationType::None {
                        return None;
                    }
                    let diff = JulianDate::seconds_difference(&self.times[0], t);
                    if self.backward_extrapolation_duration != 0.0
                        && diff > self.backward_extrapolation_duration
                    {
                        return None;
                    }
                    if self.backward_extrapolation_type == ExtrapolationType::Hold {
                        return Some(T::unpack(&self.values[..packed_len]));
                    }
                }

                if index >= times_len {
                    if self.forward_extrapolation_type == ExtrapolationType::None {
                        return None;
                    }
                    let diff = JulianDate::seconds_difference(t, &self.times[times_len - 1]);
                    if self.forward_extrapolation_duration != 0.0
                        && diff > self.forward_extrapolation_duration
                    {
                        return None;
                    }
                    if self.forward_extrapolation_type == ExtrapolationType::Hold {
                        let start = (times_len - 1) * packed_len;
                        return Some(T::unpack(&self.values[start..start + packed_len]));
                    }
                }

                let num_points = match self.interpolation_algorithm {
                    InterpolationAlgorithm::Linear => 2.min(times_len),
                    InterpolationAlgorithm::Lagrange => {
                        (self.interpolation_degree + 1).min(times_len)
                    }
                    InterpolationAlgorithm::Hermite => {
                        HermitePolynomialApproximation::get_required_data_points(
                            self.interpolation_degree,
                            0,
                        )
                        .min(times_len)
                    }
                };

                let degree = num_points.saturating_sub(1);
                if degree < 1 {
                    return None;
                }

                let mut first_index = 0;
                let mut last_index = times_len - 1;
                let points_in_collection = last_index - first_index + 1;

                if points_in_collection > degree {
                    let mut computed_first_index = (index as isize) - ((degree as isize) / 2) - 1;
                    if computed_first_index < first_index as isize {
                        computed_first_index = first_index as isize;
                    }
                    let mut computed_last_index = computed_first_index + (degree as isize);
                    if computed_last_index > last_index as isize {
                        computed_last_index = last_index as isize;
                        computed_first_index = computed_last_index - (degree as isize);
                        if computed_first_index < first_index as isize {
                            computed_first_index = first_index as isize;
                        }
                    }
                    first_index = computed_first_index as usize;
                    last_index = computed_last_index as usize;
                }

                let window_len = last_index - first_index + 1;
                let mut x_table = vec![0.0; window_len];
                for (i, entry) in x_table.iter_mut().enumerate() {
                    *entry = JulianDate::seconds_difference(
                        &self.times[first_index + i],
                        &self.times[last_index],
                    );
                }

                let mut y_table = vec![0.0; window_len * packed_len];
                T::convert_packed_array_for_interpolation(
                    &self.values,
                    first_index,
                    last_index,
                    &mut y_table,
                );

                let x = JulianDate::seconds_difference(t, &self.times[last_index]);
                let interpolation_result = match self.interpolation_algorithm {
                    InterpolationAlgorithm::Linear => {
                        LinearApproximation::interpolate_order_zero(
                            x,
                            &x_table,
                            &y_table,
                            packed_len,
                        )
                        .ok()?
                    }
                    InterpolationAlgorithm::Lagrange => {
                        LagrangePolynomialApproximation::interpolate_order_zero(
                            x,
                            &x_table,
                            &y_table,
                            packed_len,
                        )
                        .ok()?
                    }
                    InterpolationAlgorithm::Hermite => {
                        HermitePolynomialApproximation::interpolate_order_zero(
                            x,
                            &x_table,
                            &y_table,
                            packed_len,
                        )
                        .ok()?
                    }
                };

                Some(T::unpack_interpolation_result(
                    &interpolation_result,
                    &self.values,
                    last_index,
                ))
            }
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };

        self.interpolation_degree == other.interpolation_degree
            && self.interpolation_algorithm == other.interpolation_algorithm
            && self.forward_extrapolation_type == other.forward_extrapolation_type
            && (self.forward_extrapolation_duration - other.forward_extrapolation_duration).abs()
                < f64::EPSILON
            && self.backward_extrapolation_type == other.backward_extrapolation_type
            && (self.backward_extrapolation_duration - other.backward_extrapolation_duration).abs()
                < f64::EPSILON
            && self.times == other.times
            && self.values == other.values
    }
}

impl<T: Sampleable> DynProperty<T> for SampledProperty<T> {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<T> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<T>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

/// A SampledProperty which is also a PositionProperty.
#[derive(Clone)]
pub struct SampledPositionProperty {
    property: SampledProperty<Cartesian3>,
    reference_frame: ReferenceFrame,
    definition_changed: Event<()>,
}

impl Default for SampledPositionProperty {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SampledPositionProperty {
    pub fn new(reference_frame: Option<ReferenceFrame>) -> Self {
        Self {
            property: SampledProperty::<Cartesian3>::new(),
            reference_frame: reference_frame.unwrap_or(ReferenceFrame::Fixed),
            definition_changed: Event::new(),
        }
    }

    pub fn reference_frame(&self) -> ReferenceFrame {
        self.reference_frame
    }

    pub fn is_constant(&self) -> bool {
        self.property.is_constant()
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn interpolation_degree(&self) -> usize {
        self.property.interpolation_degree()
    }

    pub fn interpolation_algorithm(&self) -> InterpolationAlgorithm {
        self.property.interpolation_algorithm()
    }

    pub fn forward_extrapolation_type(&self) -> ExtrapolationType {
        self.property.forward_extrapolation_type()
    }

    pub fn set_forward_extrapolation_type(&mut self, ext: ExtrapolationType) {
        self.property.set_forward_extrapolation_type(ext);
        self.definition_changed.raise_event(&());
    }

    pub fn forward_extrapolation_duration(&self) -> f64 {
        self.property.forward_extrapolation_duration()
    }

    pub fn set_forward_extrapolation_duration(&mut self, dur: f64) {
        self.property.set_forward_extrapolation_duration(dur);
        self.definition_changed.raise_event(&());
    }

    pub fn backward_extrapolation_type(&self) -> ExtrapolationType {
        self.property.backward_extrapolation_type()
    }

    pub fn set_backward_extrapolation_type(&mut self, ext: ExtrapolationType) {
        self.property.set_backward_extrapolation_type(ext);
        self.definition_changed.raise_event(&());
    }

    pub fn backward_extrapolation_duration(&self) -> f64 {
        self.property.backward_extrapolation_duration()
    }

    pub fn times(&self) -> &[JulianDate] {
        self.property.times()
    }

    pub fn values(&self) -> &[f64] {
        self.property.values()
    }

    pub fn get_sample(&self, index: isize) -> Option<JulianDate> {
        self.property.get_sample(index)
    }

    pub fn remove_sample(&mut self, time: &JulianDate) -> bool {
        let removed = self.property.remove_sample(time);
        if removed {
            self.definition_changed.raise_event(&());
        }
        removed
    }

    pub fn remove_samples(&mut self, interval: &crate::TimeInterval) -> bool {
        let removed = self.property.remove_samples(interval);
        if removed {
            self.definition_changed.raise_event(&());
        }
        removed
    }

    pub fn set_backward_extrapolation_duration(&mut self, dur: f64) {
        self.property.set_backward_extrapolation_duration(dur);
        self.definition_changed.raise_event(&());
    }

    pub fn set_interpolation_options(
        &mut self,
        degree: Option<usize>,
        algorithm: Option<InterpolationAlgorithm>,
    ) {
        self.property.set_interpolation_options(degree, algorithm);
        self.definition_changed.raise_event(&());
    }

    pub fn add_sample(&mut self, time: JulianDate, position: Cartesian3) {
        self.property.add_sample(time, position);
        self.definition_changed.raise_event(&());
    }

    pub fn add_samples(&mut self, times: &[JulianDate], positions: &[Cartesian3]) {
        self.property.add_samples(times, positions);
        self.definition_changed.raise_event(&());
    }

    pub fn add_samples_packed_array(&mut self, data: &[f64], epoch: &JulianDate) {
        self.property.add_samples_packed_array(data, epoch);
        self.definition_changed.raise_event(&());
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, ReferenceFrame::Fixed)
    }

    pub fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        let raw = self.property.get_value(time)?;
        PositionProperty::convert_to_reference_frame(
            time,
            Some(raw),
            self.reference_frame,
            target_frame,
        )
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };
        self.reference_frame == other.reference_frame && self.property.equals(Some(&other.property))
    }
}

impl DynPositionProperty for SampledPositionProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<Cartesian3> {
        self.get_value(time)
    }
    fn get_value_in_reference_frame(
        &self,
        time: Option<&JulianDate>,
        target_frame: ReferenceFrame,
    ) -> Option<Cartesian3> {
        self.get_value_in_reference_frame(time, target_frame)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynPositionProperty) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}
