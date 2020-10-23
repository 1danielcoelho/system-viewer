use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
    sync::Mutex,
};

use crate::{materials::SimpleMaterial, mesh::Mesh};

pub static EntityManagerInstance: EntityManager = EntityManager::new();
pub static ComponentManagerInstance: ComponentManager = ComponentManager::new();

pub trait Drawable3D {
    fn draw(&self);
}

pub trait DrawableUI {
    fn draw(&self, ui: &egui::Ui);
}

pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Camera = 3,
}
const NUM_COMPONENTS: usize = 4;

pub struct Entity {
    id: u32,
    component_ids: [u32; NUM_COMPONENTS],
    name: String,
}
impl Entity {
    pub fn new(name: &str) -> &Self {
        let new_entity = Self {
            id: 0,
            component_ids: [0; NUM_COMPONENTS],
            name: String::from(name),
        };

        let ent_man = EntityManagerInstance;
        return ent_man.register(new_entity).unwrap();
    }

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        let comp_id = self.component_ids[T::get_component_index() as usize];
        if comp_id == 0 {
            return None;
        };

        return T::get_components_vector().get(comp_id as usize);
    }

    // TODO: Some generic way of doing this that also lets us pass arbitrary arguments to the new components?
    pub fn get_transform_component(&self) -> Option<&TransformComponent> {
        let comp_id = self.component_ids[ComponentIndex::Transform as usize];
        if comp_id == 0 {
            return None;
        };
        return ComponentManagerInstance.transform.get(comp_id as usize);
    }

    pub fn add_transform_component(&self) -> Option<&TransformComponent> {
        // We already have one of those
        if self.component_ids[ComponentIndex::Transform as usize] != 0 {
            log::info!("Tried to add a repeated component");
        };

        ComponentManagerInstance
            .transform
            .push(TransformComponent::new());
        self.component_ids[ComponentIndex::Transform as usize] =
            (ComponentManagerInstance.transform.len() - 1) as u32;

        return ComponentManagerInstance.transform.last();
    }

    pub fn get_mesh_component(&self) -> Option<&MeshComponent> {
        let comp_id = self.component_ids[ComponentIndex::Transform as usize];
        if comp_id == 0 {
            return None;
        };
        return ComponentManagerInstance.mesh.get(comp_id as usize);
    }

    pub fn add_mesh_component(&self) -> Option<&MeshComponent> {
        // We already have one of those
        if self.component_ids[ComponentIndex::Mesh as usize] != 0 {
            log::info!("Tried to add a repeated component");
        };

        ComponentManagerInstance.mesh.push(MeshComponent::new());
        self.component_ids[ComponentIndex::Mesh as usize] =
            (ComponentManagerInstance.mesh.len() - 1) as u32;

        return ComponentManagerInstance.mesh.last();
    }
}

pub struct EntityManager {
    last_id: u32,
    entities: HashMap<u32, Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            last_id: 1,
            entities: HashMap::new(),
        }
    }

    pub fn register(&self, entity: Entity) -> Option<&Entity> {
        entity.id = self.last_id;
        self.entities.insert(self.last_id, entity);

        self.last_id += 1;
        return self.entities.get(&(self.last_id - 1));
    }
    pub fn get_entity(&self, id: &u32) -> Option<&Entity> {
        return self.entities.get(id);
    }
}

pub trait Component {
    type ComponentType;
    fn get_component_index() -> ComponentIndex;
    fn get_components_vector() -> &'static Vec<Self::ComponentType>;
}

pub struct ComponentManager {
    physics: Vec<PhysicsComponent>,
    mesh: Vec<MeshComponent>,
    transform: Vec<TransformComponent>,
    camera: Vec<CameraComponent>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            camera: vec![],
        };
    }
}

//----------------

pub struct PhysicsComponent {
    collision_enabled: bool,
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,
    mass: f32,
}
impl PhysicsComponent {
    fn new() -> Self {
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

    fn get_components_vector() -> &'static Vec<PhysicsComponent> {
        return &ComponentManagerInstance.physics;
    }
}

//-------------------

pub struct MeshComponent {
    aabb_min: cgmath::Vector3<f32>,
    aabb_max: cgmath::Vector3<f32>,
    raycasting_visible: bool,
    visible: bool,
    mesh: Option<Arc<Mesh>>,
    material: Option<Arc<SimpleMaterial>>,
}
impl MeshComponent {
    fn new() -> Self {
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

    fn get_components_vector() -> &'static Vec<MeshComponent> {
        return &ComponentManagerInstance.mesh;
    }
}

pub struct TransformComponent {
    transform: cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>,
    parent: u32,
    children: Vec<u32>,
}
impl TransformComponent {
    pub fn new() -> Self {
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

    pub fn set_parent(&mut self, new_parent: u32) {
        self.parent = new_parent;
    }
}
impl Component for TransformComponent {
    type ComponentType = TransformComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Transform;
    }

    fn get_components_vector() -> &'static Vec<TransformComponent> {
        return &ComponentManagerInstance.transform;
    }
}

pub struct CameraComponent {
    fov_vert: cgmath::Deg<f32>,
    near: f32,
    far: f32,
}
impl CameraComponent {
    fn new() -> Self {
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

    fn get_components_vector() -> &'static Vec<CameraComponent> {
        return &ComponentManagerInstance.camera;
    }
}
