use nalgebra::Matrix4;

/// Quantization factor for layout size keys.
const SUB_PIXEL_QUANTIZE: f32 = 256.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutSizeKey([u32; 2]);

impl From<[f32; 2]> for LayoutSizeKey {
    fn from(size: [f32; 2]) -> Self {
        debug_assert!(size[0] >= 0.0);
        debug_assert!(size[1] >= 0.0);

        LayoutSizeKey([
            (size[0] * SUB_PIXEL_QUANTIZE).max(0.0) as u32,
            (size[1] * SUB_PIXEL_QUANTIZE).max(0.0) as u32,
        ])
    }
}

impl From<LayoutSizeKey> for [f32; 2] {
    fn from(key: LayoutSizeKey) -> Self {
        [
            key.0[0] as f32 / SUB_PIXEL_QUANTIZE,
            key.0[1] as f32 / SUB_PIXEL_QUANTIZE,
        ]
    }
}

/// A struct that represents the constraints for a widget's size.
/// This is passed from parent to child to define the available space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraints {
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

impl Constraints {
    /// `[{min}, {max}]`
    pub fn new(width: [f32; 2], height: [f32; 2]) -> Self {
        if width[0] < 0.0 || width[0] > width[1] || height[0] < 0.0 || height[0] > height[1] {
            panic!("Invalid constraints: {:?}", (width, height));
        }

        Self {
            min_width: (width[0] * SUB_PIXEL_QUANTIZE) as u32,
            max_width: (width[1] * SUB_PIXEL_QUANTIZE) as u32,
            min_height: (height[0] * SUB_PIXEL_QUANTIZE) as u32,
            max_height: (height[1] * SUB_PIXEL_QUANTIZE) as u32,
        }
    }

    pub fn from_max_size(size: [f32; 2]) -> Self {
        if size[0] < 0.0 || size[1] < 0.0 {
            panic!("Invalid constraints: {size:?}");
        }

        Self {
            min_width: 0,
            max_width: (size[0] * SUB_PIXEL_QUANTIZE) as u32,
            min_height: 0,
            max_height: (size[1] * SUB_PIXEL_QUANTIZE) as u32,
        }
    }

    pub fn min_width(&self) -> f32 {
        self.min_width as f32 / SUB_PIXEL_QUANTIZE
    }

    pub fn max_width(&self) -> f32 {
        self.max_width as f32 / SUB_PIXEL_QUANTIZE
    }

    pub fn width(&self) -> [f32; 2] {
        [self.min_width(), self.max_width()]
    }

    pub fn min_height(&self) -> f32 {
        self.min_height as f32 / SUB_PIXEL_QUANTIZE
    }

    pub fn max_height(&self) -> f32 {
        self.max_height as f32 / SUB_PIXEL_QUANTIZE
    }

    pub fn height(&self) -> [f32; 2] {
        [self.min_height(), self.max_height()]
    }
}

/// Arrangement for a child after layout pass.
/// Holds the allocated size and transform matrices for rendering/hit-testing.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrangement {
    /// size allocated to the child (width, height)
    pub size: [f32; 2],
    /// affine transform that maps child-local coordinates (origin at child's top-left)
    /// into global/window coordinates.
    pub affine: Matrix4<f32>,
    /// inverse of `affine` when invertible. If `None`, the affine collapses at least
    /// one axis and the child is effectively invisible / non-hit-testable in that axis.
    pub affine_inv: Option<Matrix4<f32>>,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            size: [0.0, 0.0],
            affine: Matrix4::identity(),
            affine_inv: Some(Matrix4::identity()),
        }
    }
}

impl Arrangement {
    /// Create a new Arrangement from size and affine transform.
    /// Attempts to compute inverse; if inversion fails, `affine_inv` is None.
    pub fn new(size: [f32; 2], affine: Matrix4<f32>) -> Self {
        let affine_inv = affine.try_inverse();
        Self {
            size,
            affine,
            affine_inv,
        }
    }

    /// Transforms a global `position` (window coordinates, origin top-left) into
    /// this child's local coordinates (origin = child's top-left).
    ///
    /// If `affine_inv` is unavailable, returns [f32::INFINITY, f32::INFINITY].
    pub fn to_local(&self, position: [f32; 2]) -> [f32; 2] {
        use nalgebra::Vector4;
        match &self.affine_inv {
            Some(inv) => {
                let v = Vector4::new(position[0], position[1], 0.0, 1.0);
                let r = inv * v;
                [r.x, r.y]
            }
            None => [f32::INFINITY, f32::INFINITY],
        }
    }

    /// Transforms a local position (relative to child's top-left) into global/window coordinates.
    pub fn to_global(&self, local: [f32; 2]) -> [f32; 2] {
        use nalgebra::Vector4;
        let v = Vector4::new(local[0], local[1], 0.0, 1.0);
        let r = self.affine * v;
        [r.x, r.y]
    }

    /// Returns true if the given global `position` lies inside the child's arranged rectangle.
    ///
    /// Returns false if `affine_inv` is None (collapsed axis). Otherwise transforms to local
    /// coordinates and checks against `size`. NaN/Infinite local coordinates are treated as outside.
    pub fn contains(&self, position: [f32; 2]) -> bool {
        let local = self.to_local(position);

        if !local[0].is_finite() || !local[1].is_finite() {
            return false;
        }

        // allow inclusive bounds
        (0.0..=self.size[0]).contains(&local[0]) && (0.0..=self.size[1]).contains(&local[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Matrix4;

    const EPS: f32 = 1e-5;

    fn approx_eq(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS
    }

    #[test]
    fn arrangement_identity_roundtrip_contains() {
        // identity affine, no translation
        let affine = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        );
        let size = [100.0, 50.0];
        let arr = Arrangement::new(size, affine);

        // to_local of a global point should equal itself
        let g = [10.0, 20.0];
        assert!(approx_eq(arr.to_local(g), g));
        // to_global of a local point should equal itself
        let l = [5.0, 5.0];
        assert!(approx_eq(arr.to_global(l), l));

        // contains true for inside, false for outside
        assert!(arr.contains([10.0, 20.0]));
        assert!(!arr.contains([200.0, 200.0]));
    }

    #[test]
    fn arrangement_translation_and_roundtrip() {
        // translation by (tx, ty)
        let tx = 30.0f32;
        let ty = 40.0f32;
        let affine = Matrix4::new(
            1.0, 0.0, 0.0, tx, //
            0.0, 1.0, 0.0, ty, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        );
        let size = [10.0, 10.0];
        let arr = Arrangement::new(size, affine);

        // a global point at (tx + 2, ty + 3) maps to local (2,3)
        let g = [tx + 2.0, ty + 3.0];
        let local = arr.to_local(g);
        assert!(approx_eq(local, [2.0, 3.0]));

        // roundtrip
        let back = arr.to_global(local);
        assert!(approx_eq(back, g));

        // contains
        assert!(arr.contains(g));
        assert!(!arr.contains([tx + 20.0, ty + 20.0]));
    }

    #[test]
    fn arrangement_singular_affine() {
        // collapse Y axis (scale y = 0)
        let affine = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        );
        let size = [10.0, 10.0];
        let arr = Arrangement::new(size, affine);

        // inverse should be None
        assert!(arr.affine_inv.is_none());

        // to_local returns infinite coords when inverse absent
        let local = arr.to_local([1.0, 1.0]);
        assert!(!local[0].is_finite() || !local[1].is_finite());

        // contains is false
        assert!(!arr.contains([1.0, 1.0]));
    }
}
