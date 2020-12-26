use na::{Isometry3, Matrix4, Translation3, UnitQuaternion, Vector3};

// Heavily based off of how Amethyst has a custom wrapper over nalgebra stuff: https://docs.amethyst.rs/stable/src/amethyst_core/transform/components/transform.rs.html#500-508

#[derive(Clone, Debug)]
pub struct Transform {
    pub trans: Vector3<f32>,
    pub rot: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}
impl Transform {
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

    pub fn to_matrix4(&self) -> Matrix4<f32> {
        return Isometry3::from_parts(Translation3::from(self.trans), self.rot)
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale);
    }

    pub fn identity() -> Self {
        Self {
            trans: Vector3::new(0.0, 0.0, 0.0),
            rot: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
