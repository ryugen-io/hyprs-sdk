//! wlr-gamma-control: adjust gamma tables for outputs.

/// A gamma lookup table with red, green, and blue ramp channels.
///
/// Each channel contains `size` entries of `u16` values representing
/// the gamma ramp. The table can be serialized to bytes for submission
/// to the compositor via the wlr-gamma-control protocol.
#[derive(Debug, Clone)]
pub struct GammaTable {
    /// Number of entries per channel.
    pub size: u32,
    /// Red channel ramp values.
    pub red: Vec<u16>,
    /// Green channel ramp values.
    pub green: Vec<u16>,
    /// Blue channel ramp values.
    pub blue: Vec<u16>,
}

impl GammaTable {
    /// Create an identity gamma table (linear ramp from 0 to `u16::MAX`).
    #[must_use]
    pub fn identity(size: u32) -> Self {
        let ramp: Vec<u16> = (0..size)
            .map(|i| {
                if size <= 1 {
                    u16::MAX
                } else {
                    ((i as u64 * u16::MAX as u64) / (size as u64 - 1)) as u16
                }
            })
            .collect();
        Self {
            size,
            red: ramp.clone(),
            green: ramp.clone(),
            blue: ramp,
        }
    }

    /// Serialize the gamma table to bytes in native-endian format.
    ///
    /// The layout is: all red values, then all green values, then all blue values,
    /// each as native-endian `u16`.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.size as usize * 6);
        for &v in &self.red {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.green {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.blue {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        buf
    }

    /// Create a gamma table with uniform brightness adjustment.
    ///
    /// `brightness` is clamped to `[0.0, 1.0]`, where `1.0` is identity
    /// and `0.0` is full black.
    #[must_use]
    pub fn with_brightness(size: u32, brightness: f64) -> Self {
        let mut table = Self::identity(size);
        let factor = brightness.clamp(0.0, 1.0);
        for v in table
            .red
            .iter_mut()
            .chain(table.green.iter_mut())
            .chain(table.blue.iter_mut())
        {
            *v = (*v as f64 * factor) as u16;
        }
        table
    }

    /// Create a gamma table with gamma correction applied.
    ///
    /// `gamma` values greater than 1.0 darken midtones, values less than
    /// 1.0 brighten midtones. A value of 1.0 produces an identity table.
    #[must_use]
    pub fn with_gamma(size: u32, gamma: f64) -> Self {
        let mut table = Self::identity(size);
        for v in table
            .red
            .iter_mut()
            .chain(table.green.iter_mut())
            .chain(table.blue.iter_mut())
        {
            let normalized = *v as f64 / u16::MAX as f64;
            *v = (normalized.powf(gamma) * u16::MAX as f64) as u16;
        }
        table
    }
}
