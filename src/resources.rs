use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{materials::SimpleMaterial, mesh::Mesh, texture::Texture};

pub static ResourceManagerInstance: ResourceManager = ResourceManager::new();

pub struct ResourceManager {
    meshes: Vec<Rc<Mesh>>,
    textures: Vec<Rc<Texture>>,
}
impl ResourceManager {
    pub fn new() -> Self {
        return Self {
            meshes: vec![],
            textures: vec![],
        };
    }

    pub fn register(&self, new_mesh: Mesh) -> Rc<Mesh> {
        new_mesh.id = self.meshes.len() as u32;
        let new_rc = Rc::new(new_mesh);
        self.meshes.push(new_rc);
        return new_rc;
    }

    pub fn get_mesh(&self, id: u32) -> Option<&Rc<Mesh>> {
        return self.meshes.get(id as usize);
    }
}
