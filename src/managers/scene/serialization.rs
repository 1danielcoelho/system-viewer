use crate::components::{
    LightComponent, MeshComponent, OrbitalComponent, PhysicsComponent, TransformComponent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub light: HashMap<EntityIndex, LightComponent>,
    pub orbital: HashMap<EntityIndex, OrbitalComponent>,
}
