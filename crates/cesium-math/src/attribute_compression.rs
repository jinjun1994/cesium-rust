use crate::{
    clamp, from_snorm, sign_not_zero, to_snorm, AttributeType, Cartesian2, Cartesian3, Cartesian4,
    Color, ComponentDatatype, DeveloperError, EPSILON6,
};

/// Attribute compression and decompression functions.
pub struct AttributeCompression;

impl AttributeCompression {
    /// Encodes a normalized vector into 2 SNORM values in the range of [0-rangeMax] following the 'oct' encoding.
    pub fn oct_encode_in_range(
        vector: Cartesian3,
        range_max: f64,
    ) -> Result<Cartesian2, DeveloperError> {
        let mag_squared = vector.magnitude_squared();
        if (mag_squared - 1.0).abs() > EPSILON6 {
            return Err(DeveloperError::new("vector must be normalized."));
        }

        let l1 = vector.x.abs() + vector.y.abs() + vector.z.abs();
        let mut x = vector.x / l1;
        let mut y = vector.y / l1;
        if vector.z < 0.0 {
            let old_x = x;
            let old_y = y;
            x = (1.0 - old_y.abs()) * sign_not_zero(old_x);
            y = (1.0 - old_x.abs()) * sign_not_zero(old_y);
        }

        let snorm_x = to_snorm(x, Some(range_max));
        let snorm_y = to_snorm(y, Some(range_max));

        Ok(Cartesian2::new(snorm_x, snorm_y))
    }

    /// Encodes a normalized vector into 2 SNORM values in the range of [0-255] following the 'oct' encoding.
    #[inline]
    pub fn oct_encode(vector: Cartesian3) -> Result<Cartesian2, DeveloperError> {
        Self::oct_encode_in_range(vector, 255.0)
    }

    /// Encodes a normalized vector into 4 byte 'oct' encoding.
    pub fn oct_encode_to_cartesian4(vector: Cartesian3) -> Result<Cartesian4, DeveloperError> {
        let oct16 = Self::oct_encode_in_range(vector, 65535.0)?;
        let x_val = oct16.x.round() as u32;
        let y_val = oct16.y.round() as u32;
        let x = ((x_val >> 8) & 0xFF) as f64;
        let y = (x_val & 0xFF) as f64;
        let z = ((y_val >> 8) & 0xFF) as f64;
        let w = (y_val & 0xFF) as f64;
        Ok(Cartesian4::new(x, y, z, w))
    }

    /// Decodes a unit-length vector in 'oct' encoding to a normalized 3-component vector.
    pub fn oct_decode_in_range(
        x: f64,
        y: f64,
        range_max: f64,
    ) -> Result<Cartesian3, DeveloperError> {
        if x < 0.0 || x > range_max || y < 0.0 || y > range_max {
            return Err(DeveloperError::new(format!(
                "x and y must be unsigned normalized integers between 0 and {}",
                range_max
            )));
        }

        let mut rx = from_snorm(x, Some(range_max));
        let mut ry = from_snorm(y, Some(range_max));
        let rz = 1.0 - (rx.abs() + ry.abs());

        let mut rz_final = rz;
        if rz < 0.0 {
            let old_x = rx;
            rx = (1.0 - ry.abs()) * sign_not_zero(old_x);
            ry = (1.0 - old_x.abs()) * sign_not_zero(ry);
            rz_final = rz;
        }

        let res = Cartesian3::new(rx, ry, rz_final);
        Ok(res.normalize())
    }

    /// Decodes a unit-length vector in 2 byte 'oct' encoding to a normalized 3-component vector.
    #[inline]
    pub fn oct_decode(x: f64, y: f64) -> Result<Cartesian3, DeveloperError> {
        Self::oct_decode_in_range(x, y, 255.0)
    }

    /// Decodes a unit-length vector in 4 byte 'oct' encoding to a normalized 3-component vector.
    pub fn oct_decode_from_cartesian4(encoded: Cartesian4) -> Result<Cartesian3, DeveloperError> {
        let x = encoded.x;
        let y = encoded.y;
        let z = encoded.z;
        let w = encoded.w;

        if !(0.0..=255.0).contains(&x)
            || !(0.0..=255.0).contains(&y)
            || !(0.0..=255.0).contains(&z)
            || !(0.0..=255.0).contains(&w)
        {
            return Err(DeveloperError::new(
                "x, y, z, and w must be unsigned normalized integers between 0 and 255",
            ));
        }

        let x_oct16 = x * 256.0 + y;
        let y_oct16 = z * 256.0 + w;
        Self::oct_decode_in_range(x_oct16, y_oct16, 65535.0)
    }

    /// Packs an oct encoded vector into a single floating-point number.
    #[inline]
    pub fn oct_pack_float(encoded: Cartesian2) -> f64 {
        256.0 * encoded.x + encoded.y
    }

    /// Encodes a normalized vector into 2 byte 'oct' encoding stored in a single float.
    pub fn oct_encode_float(vector: Cartesian3) -> Result<f64, DeveloperError> {
        let enc = Self::oct_encode(vector)?;
        Ok(Self::oct_pack_float(enc))
    }

    /// Decodes an oct-encoded unit-length vector stored as a single float.
    pub fn oct_decode_float(value: f64) -> Result<Cartesian3, DeveloperError> {
        let temp = value / 256.0;
        let x = temp.floor();
        let y = (temp - x) * 256.0;
        Self::oct_decode(x, y)
    }

    /// Encodes three normalized vectors and packs them into two floating-point numbers.
    pub fn oct_pack(
        v1: Cartesian3,
        v2: Cartesian3,
        v3: Cartesian3,
    ) -> Result<Cartesian2, DeveloperError> {
        let encoded1 = Self::oct_encode_float(v1)?;
        let encoded2 = Self::oct_encode_float(v2)?;
        let encoded3 = Self::oct_encode(v3)?;
        Ok(Cartesian2::new(
            65536.0 * encoded3.x + encoded1,
            65536.0 * encoded3.y + encoded2,
        ))
    }

    /// Decodes three unit-length vectors packed into two floating-point numbers.
    pub fn oct_unpack(
        packed: Cartesian2,
    ) -> Result<(Cartesian3, Cartesian3, Cartesian3), DeveloperError> {
        let temp_x = packed.x / 65536.0;
        let x = temp_x.floor();
        let encoded_float1 = (temp_x - x) * 65536.0;

        let temp_y = packed.y / 65536.0;
        let y = temp_y.floor();
        let encoded_float2 = (temp_y - y) * 65536.0;

        let v1 = Self::oct_decode_float(encoded_float1)?;
        let v2 = Self::oct_decode_float(encoded_float2)?;
        let v3 = Self::oct_decode(x, y)?;
        Ok((v1, v2, v3))
    }

    /// Pack texture coordinates into a single float with 12 bits of precision.
    #[inline]
    pub fn compress_texture_coordinates(texture_coordinates: Cartesian2) -> f64 {
        let x = (texture_coordinates.x * 4095.0) as i64;
        let y = (texture_coordinates.y * 4095.0) as i64;
        (4096.0 * x as f64) + (y as f64)
    }

    /// Decompresses texture coordinates from a single float.
    pub fn decompress_texture_coordinates(compressed: f64) -> Cartesian2 {
        let temp = compressed / 4096.0;
        let x_zero_to_4095 = temp.floor();
        let x = x_zero_to_4095 / 4095.0;
        let y = (compressed - x_zero_to_4095 * 4096.0) / 4095.0;
        Cartesian2::new(x, y)
    }

    /// Decodes delta and ZigZag encoded vertices in place.
    pub fn zig_zag_delta_decode(
        u_buffer: &mut [u16],
        v_buffer: &mut [u16],
        height_buffer: Option<&mut [u16]>,
    ) -> Result<(), DeveloperError> {
        if u_buffer.len() != v_buffer.len() {
            return Err(DeveloperError::new(
                "uBuffer.length must equal vBuffer.length",
            ));
        }
        if let Some(ref h) = height_buffer {
            if u_buffer.len() != h.len() {
                return Err(DeveloperError::new(
                    "uBuffer.length must equal heightBuffer.length",
                ));
            }
        }

        #[inline]
        fn zig_zag_decode(value: u16) -> i32 {
            let v = value as i32;
            (v >> 1) ^ -(v & 1)
        }

        let mut u: i32 = 0;
        let mut v: i32 = 0;
        let mut height: i32 = 0;

        let has_height = height_buffer.is_some();
        let count = u_buffer.len();

        if let Some(h_buf) = height_buffer {
            for i in 0..count {
                u += zig_zag_decode(u_buffer[i]);
                v += zig_zag_decode(v_buffer[i]);
                height += zig_zag_decode(h_buf[i]);
                u_buffer[i] = u as u16;
                v_buffer[i] = v as u16;
                h_buf[i] = height as u16;
            }
        } else if !has_height {
            for i in 0..count {
                u += zig_zag_decode(u_buffer[i]);
                v += zig_zag_decode(v_buffer[i]);
                u_buffer[i] = u as u16;
                v_buffer[i] = v as u16;
            }
        }

        Ok(())
    }

    /// Encodes RGB values into a single float.
    pub fn encode_rgb8(color: &Color) -> f64 {
        let r = clamp((color.red as f64 * 255.0).round(), 0.0, 255.0);
        let g = clamp((color.green as f64 * 255.0).round(), 0.0, 255.0);
        let b = clamp((color.blue as f64 * 255.0).round(), 0.0, 255.0);
        r * 65536.0 + g * 256.0 + b
    }

    /// Decodes RGB values from a single float.
    pub fn decode_rgb8(encoded: f64) -> Color {
        let enc = encoded.floor() as u32;
        let r = (((enc >> 16) & 255) as f32) / 255.0;
        let g = (((enc >> 8) & 255) as f32) / 255.0;
        let b = ((enc & 255) as f32) / 255.0;
        Color::new(r, g, b, 1.0)
    }

    /// Decodes RGB565 values into normalized RGB floats.
    pub fn decode_rgb565(typed_array: &[u16]) -> Vec<f32> {
        let count = typed_array.len();
        let mut result = Vec::with_capacity(count * 3);
        let mask5 = (1 << 5) - 1;
        let mask6 = (1 << 6) - 1;
        let normalize5 = 1.0f32 / 31.0f32;
        let normalize6 = 1.0f32 / 63.0f32;

        for &value in typed_array {
            let red = ((value >> 11) & mask5) as f32;
            let green = ((value >> 5) & mask6) as f32;
            let blue = (value & mask5) as f32;
            result.push(red * normalize5);
            result.push(green * normalize6);
            result.push(blue * normalize5);
        }

        result
    }

    /// Dequantizes a slice of numbers according to component datatype and attribute type.
    pub fn dequantize_f64(
        values: &[f64],
        component_datatype: ComponentDatatype,
        attribute_type: AttributeType,
        count: usize,
    ) -> Result<Vec<f32>, DeveloperError> {
        let components_per_attr = attribute_type.get_number_of_components();
        let divisor: f64 = match component_datatype {
            ComponentDatatype::Byte => 127.0,
            ComponentDatatype::UnsignedByte => 255.0,
            ComponentDatatype::Short => 32767.0,
            ComponentDatatype::UnsignedShort => 65535.0,
            ComponentDatatype::Int => 2147483647.0,
            ComponentDatatype::UnsignedInt => 4294967295.0,
            _ => {
                return Err(DeveloperError::new(format!(
                    "Cannot dequantize component datatype: {:?}",
                    component_datatype
                )))
            }
        };

        let total = count * components_per_attr;
        let mut result = Vec::with_capacity(total);
        for &val in values.iter().take(total) {
            let deq = (val / divisor).max(-1.0) as f32;
            result.push(deq);
        }
        Ok(result)
    }
}
