use crate::components::{
    LightComponent, MeshComponent, OrbitalComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::scene::component_storage::{HashStorage, PackedStorage, SparseStorage};
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerEntity<'a> {
    pub name: Option<&'a str>,
    pub entity: Entity,
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
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
