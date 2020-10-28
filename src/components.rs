use std::rc::Rc;

use crate::{entity::Entity, events::EventReceiver, materials::Material, mesh::Mesh};

pub type TransformType = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Ui = 3,
}

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
    pub interface: Vec<UIComponent>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
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
        // Ensure size. Very temp for now, never shrinks...
        self.resize_components((entity.id + 1) as usize);

        let comp_vec = T::get_components_vector(self);
        comp_vec[entity.id as usize].set_enabled(true);

        return Some(&mut comp_vec[entity.id as usize]);
    }

    fn resize_components(&mut self, min_length: usize) {
        if min_length <= self.physics.len() {
            return;
        }

        self.physics.resize_with(min_length, Default::default);
        self.mesh.resize_with(min_length, Default::default);
        self.transform.resize_with(min_length, Default::default);
        self.interface.resize_with(min_length, Default::default);
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
            collision_enabled: false,
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
    pub material: Option<Rc<Material>>,
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

    pub transform: TransformType,
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

pub enum WidgetType {
    None,
    TestWidget,
}

pub struct UIComponent {
    enabled: bool,
    pub widget_type: WidgetType,
}
impl UIComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for UIComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            widget_type: WidgetType::None,
        };
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
        return ComponentIndex::Ui;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType> {
        return &mut w.interface;
    }
}
