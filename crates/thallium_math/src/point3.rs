use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

use thallium_derive::Component;

/// The type for storing PGA points
#[derive(Debug, Clone, Copy, Component)]
pub struct Point3 {
    /// The "x" component
    pub e023: f32,
    /// The "-y" component
    pub e013: f32,
    /// The "z" component
    pub e012: f32,
    /// The magnitude of the point, a magnitude of 0 is a point at infinity
    pub e0123: f32,
}

impl Point3 {
    /// The point representing the origin
    pub const IDENTITY: Point3 = Point3 {
        e023: 0.0,
        e013: 0.0,
        e012: 0.0,
        e0123: 1.0,
    };

    /// Converts a cartesian point to a [`Point3`]
    #[inline]
    #[must_use]
    pub fn from_cartesian(x: f32, y: f32, z: f32) -> Point3 {
        Point3 {
            e023: x,
            e013: -y,
            e012: z,
            e0123: 1.0,
        }
    }

    /// Converts a cartesian normal to a [`Point3`], will return a point at infinity
    #[inline]
    #[must_use]
    pub fn from_cartesian_normal(x: f32, y: f32, z: f32) -> Point3 {
        Point3 {
            e023: x,
            e013: -y,
            e012: z,
            e0123: 0.0,
        }
    }

    /// Converts a [`Point3`] to a cartesian point, will divide by 0 if it is a point at infinity
    #[inline]
    #[must_use]
    pub fn into_cartesian(self) -> [f32; 3] {
        [
            self.e023 / self.e0123,
            -self.e013 / self.e0123,
            self.e012 / self.e0123,
        ]
    }

    /// Converts a [`Point3`] to a cartesian normal, assuming the magnitude is 0
    #[inline]
    #[must_use]
    pub fn into_cartesian_normal(self) -> [f32; 3] {
        [self.e023, -self.e013, self.e012]
    }

    /// Gets the magnitude of the [`Point3`], this just returns [`Point3::e0123`]
    #[inline]
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.e0123
    }

    /// Gets the squared magnitude of the [`Point3`], this is just for convenience and isnt more efficient than squaring the result of [`Point3::magnitude`]
    #[inline]
    #[must_use]
    pub fn magnitude_sq(self) -> f32 {
        self.e0123 * self.e0123
    }

    /// Returns a normalized version of this [`Point3`] by dividing all the members by [`Self::magnitude`]
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Point3 {
        self / self.magnitude()
    }
}

impl Add for Point3 {
    type Output = Point3;

    /// Takes the average of 2 [`Point3`]s
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let Point3 {
            e023: a_e023,
            e013: a_e013,
            e012: a_e012,
            e0123: a_e0123,
        } = self;
        let Point3 {
            e023: b_e023,
            e013: b_e013,
            e012: b_e012,
            e0123: b_e0123,
        } = rhs;
        Point3 {
            e023: a_e023 + b_e023,
            e013: a_e013 + b_e013,
            e012: a_e012 + b_e012,
            e0123: a_e0123 + b_e0123,
        }
    }
}

impl AddAssign for Point3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul<f32> for Point3 {
    type Output = Point3;

    /// Scales the magnitude of a [`Point3`], the result will represent the same [`Point3`] unless 0 or infinity is passed for `rhs`
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        let Point3 {
            e023,
            e013,
            e012,
            e0123,
        } = self;
        Point3 {
            e023: e023 * rhs,
            e013: e013 * rhs,
            e012: e012 * rhs,
            e0123: e0123 * rhs,
        }
    }
}

impl MulAssign<f32> for Point3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Point3 {
    type Output = Point3;

    /// Divides the magnitude of a [`Point3`], the result will represent the same [`Point3`] unless 0 or infinity is passed for `rhs`
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        let Point3 {
            e023,
            e013,
            e012,
            e0123,
        } = self;
        Point3 {
            e023: e023 / rhs,
            e013: e013 / rhs,
            e012: e012 / rhs,
            e0123: e0123 / rhs,
        }
    }
}

impl DivAssign<f32> for Point3 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
