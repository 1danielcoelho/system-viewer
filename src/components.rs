use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{entity::Entity, materials::SimpleMaterial, mesh::Mesh, world::World};

pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Camera = 3,
}
pub const NUM_COMPONENTS: usize = 4;

pub trait Component: Default {
    type ComponentType;
    fn get_component_index() -> ComponentIndex;
    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType>;
}

pub struct ComponentManager {
    world: Option<Weak<RefCell<World>>>,

    physics: Vec<PhysicsComponent>,
    mesh: Vec<MeshComponent>,
    transform: Vec<TransformComponent>,
    camera: Vec<CameraComponent>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            world: None,
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            camera: vec![],
        };
    }

    // A bit awkward but we need this when initializing
    pub fn set_world(&mut self, world: Weak<RefCell<World>>) {
        self.world = Some(world);
    }

    pub fn get_component<T>(&mut self, entity: &Entity) -> Option<&T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let entity_comp_ids: &[u32; NUM_COMPONENTS] = &entity.component_ids;
        let comp_index: usize = T::get_component_index() as usize;
        let comp_vec = T::get_components_vector(self);
        return comp_vec.get(entity_comp_ids[comp_index] as usize);
    }

    pub fn add_component<'a, T>(&'a mut self, entity: &mut Entity) -> Option<&'a T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let entity_comp_ids: &mut [u32; NUM_COMPONENTS] = &mut entity.component_ids;
        let comp_index: usize = T::get_component_index() as usize;

        let comp_id = entity_comp_ids[comp_index];
        if comp_id != 0 {
            log::info!("Tried to add a repeated component");
            return self.get_component::<T>(entity);
        };

        let comp_vec = T::get_components_vector(self);
        comp_vec.push(T::default());

        entity_comp_ids[comp_index] = (comp_vec.len() - 1) as u32;

        return comp_vec.last();
    }
}

//=============================================================================

pub struct PhysicsComponent {
    pub collision_enabled: bool,
    pub position: cgmath::Vector3<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub mass: f32,
}
impl PhysicsComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for PhysicsComponent {
    fn default() -> Self {
        return Self {
            collision_enabled: true,
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            mass: 1.0,
        };
    }
}
impl Component for PhysicsComponent {
    type ComponentType = PhysicsComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Physics;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<PhysicsComponent> {
        return &mut w.physics;
    }
}

//=============================================================================

pub struct MeshComponent {
    pub aabb_min: cgmath::Vector3<f32>,
    pub aabb_max: cgmath::Vector3<f32>,
    pub raycasting_visible: bool,
    pub visible: bool,
    pub mesh: Option<Rc<Mesh>>,
    pub material: Option<Rc<SimpleMaterial>>,
}
impl MeshComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for MeshComponent {
    fn default() -> Self {
        return Self {
            aabb_min: cgmath::Vector3::new(0.0, 0.0, 0.0),
            aabb_max: cgmath::Vector3::new(0.0, 0.0, 0.0),
            raycasting_visible: true,
            visible: true,
            mesh: None,
            material: None,
        };
    }
}
impl Component for MeshComponent {
    type ComponentType = MeshComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Mesh;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<MeshComponent> {
        return &mut w.mesh;
    }
}

//=============================================================================

pub struct TransformComponent {
    pub transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>,
    pub parent: u32,
    pub children: Vec<u32>,
}
impl TransformComponent {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn set_parent(&mut self, new_parent: u32) {
        self.parent = new_parent;
    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        return Self {
            transform: cgmath::Decomposed {
                scale: 1.0,
                disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rot: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            },
            parent: 0,
            children: vec![0],
        };
    }
}
impl Component for TransformComponent {
    type ComponentType = TransformComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Transform;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<TransformComponent> {
        return &mut w.transform;
    }
}

//=============================================================================

pub struct CameraComponent {
    pub fov_vert: cgmath::Deg<f32>,
    pub near: f32,
    pub far: f32,
}
impl CameraComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for CameraComponent {
    fn default() -> Self {
        return Self {
            fov_vert: cgmath::Deg(80.0),
            near: 10.0,
            far: 1000.0,
        };
    }
}
impl Component for CameraComponent {
    type ComponentType = CameraComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Camera;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<CameraComponent> {
        return &mut w.camera;
    }
}
