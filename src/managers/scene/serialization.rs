use crate::components::{
    LightComponent, MeshComponent, OrbitalComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::scene::component_storage::{HashStorage, PackedStorage, SparseStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct EntityIndex(pub u32);
impl PartialEq for EntityIndex {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerEntity<'a> {
    pub name: Option<&'a str>,
    pub index: EntityIndex,
    pub parent: Option<EntityIndex>,
    pub children: Vec<EntityIndex>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerScene<'a> {
    pub identifier: &'a str,
    pub entities: Vec<SerEntity<'a>>,

    pub physics: PackedStorage<PhysicsComponent>,
    pub mesh: SparseStorage<MeshComponent>,
    pub transform: SparseStorage<TransformComponent>,
    pub light: HashStorage<LightComponent>,
    pub orbital: HashStorage<OrbitalComponent>,
}
