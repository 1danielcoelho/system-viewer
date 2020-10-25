use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{entity::Entity, events::EventReceiver, materials::SimpleMaterial, mesh::Mesh, world::World};

pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Camera = 3,
    UI = 4,
}
pub const NUM_COMPONENTS: usize = 4;

pub trait Component: Default {
    type ComponentType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_component_index() -> ComponentIndex;
    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType>;
}

pub struct ComponentManager {
    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub camera: Vec<CameraComponent>,
    pub interface: Vec<UIComponent>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            camera: vec![],
            interface: vec![],
        };
    }

    pub fn get_component<T>(&mut self, entity: &Entity) -> Option<&T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let comp_vec = T::get_components_vector(self);
        return comp_vec.get(entity.id as usize);
    }

    pub fn add_component<'a, T>(&'a mut self, entity: &mut Entity) -> Option<&'a mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let comp_vec = T::get_components_vector(self);
        comp_vec.push(T::default());

        if comp_vec.len() < entity.id as usize {
            comp_vec.resize_with((entity.id + 1) as usize, Default::default);
        }

        comp_vec[entity.id as usize].set_enabled(true);
        return Some(&mut comp_vec[entity.id as usize]);
    }
}
impl EventReceiver for ComponentManager {
    fn receive_event(&mut self, event: crate::events::Event) {
        //
    }
}

//=============================================================================

pub struct PhysicsComponent {
    enabled: bool,

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
            enabled: false,
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

//=============================================================================

pub struct MeshComponent {
    enabled: bool,

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
            enabled: false,
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

//=============================================================================

pub struct TransformComponent {
    enabled: bool,

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
            enabled: false,
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

//=============================================================================

pub struct CameraComponent {
    enabled: bool,

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
            enabled: false,
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

pub struct UIComponent {
    enabled: bool,
}
impl UIComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for UIComponent {
    fn default() -> Self {
        return Self {
            enabled: true,
        }
    }
}
impl Component for UIComponent {
    type ComponentType = UIComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::UI;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType> {
        return &mut w.interface;
    }
}