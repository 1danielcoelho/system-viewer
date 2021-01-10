use na::{Matrix4, Point3, RealField, SimdRealField, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

// Heavily based off of how Amethyst has a custom wrapper over nalgebra stuff: https://docs.amethyst.rs/stable/src/amethyst_core/transform/components/transform.rs.html#500-508

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform<T>
where
    T: RealField + SimdRealField,
{
    pub trans: Vector3<T>,
    pub rot: UnitQuaternion<T>,
    pub scale: Vector3<T>,
}
impl<T> Transform<T>
where
    T: RealField + SimdRealField,
{
    pub fn concat(&mut self, other: &Self) -> &mut Self {
        self.trans += self.rot * other.trans.component_mul(&self.scale);
        self.scale.component_mul_assign(&other.scale);
        self.rot *= other.rot;
        return self;
    }

    pub fn concat_clone(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.concat(other);
        return result;
    }

    pub fn to_matrix4(&self) -> Matrix4<T> {
        let mut mat = self.rot.to_homogeneous();
        mat[(0, 0)] *= self.scale.x;
        mat[(1, 0)] *= self.scale.x;
        mat[(2, 0)] *= self.scale.x;

        mat[(0, 1)] *= self.scale.y;
        mat[(1, 1)] *= self.scale.y;
        mat[(2, 1)] *= self.scale.y;

        mat[(0, 2)] *= self.scale.z;
        mat[(1, 2)] *= self.scale.z;
        mat[(2, 2)] *= self.scale.z;

        mat[(0, 3)] = self.trans.x;
        mat[(1, 3)] = self.trans.y;
        mat[(2, 3)] = self.trans.z;
        return mat;
    }

    pub fn transform_point(&self, point: &Point3<T>) -> Point3<T> {
        return Point3::from(
            self.trans
                + self
                    .rot
                    .transform_vector(&self.scale.component_mul(&point.coords)),
        );
    }

    pub fn transform_vector(&self, vector: &Vector3<T>) -> Vector3<T> {
        return self
            .rot
            .transform_vector(&self.scale.component_mul(&vector));
    }

    pub fn identity() -> Self {
        Self {
            trans: Vector3::zeros(),
            rot: UnitQuaternion::identity(),
            scale: Vector3::new(T::one(), T::one(), T::one()),
        }
    }
}

impl<T> Default for Transform<T>
where
    T: RealField + SimdRealField,
{
    fn default() -> Self {
        return Transform::identity();
    }
}
