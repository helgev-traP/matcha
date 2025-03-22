#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum F32Hash {
    NaN,
    F32(u32),
}

impl From<f32> for F32Hash {
    fn from(x: f32) -> Self {
        if x.is_nan() {
            F32Hash::NaN
        } else {
            F32Hash::F32(x.to_bits())
        }
    }
}

impl F32Hash {
    pub fn into_f32(self) -> f32 {
        match self {
            F32Hash::NaN => f32::NAN,
            F32Hash::F32(x) => f32::from_bits(x),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum F64Hash {
    NaN,
    F64(u64),
}

impl From<f64> for F64Hash {
    fn from(x: f64) -> Self {
        if x.is_nan() {
            F64Hash::NaN
        } else {
            F64Hash::F64(x.to_bits())
        }
    }
}

impl F64Hash {
    pub fn into_f64(self) -> f64 {
        match self {
            F64Hash::NaN => f64::NAN,
            F64Hash::F64(x) => f64::from_bits(x),
        }
    }
}
