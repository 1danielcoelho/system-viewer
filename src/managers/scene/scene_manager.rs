use crate::app_state::{AppState, ReferenceChange};
use crate::components::{MeshComponent, TransformComponent};
use crate::managers::orbit::{BodyDescription, BodyInstanceDescription};
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::description::SceneDescription;
use crate::managers::scene::orbits::{add_body_instance_entities, fetch_default_motion_if_needed};
use crate::managers::scene::{Entity, Scene};
use crate::managers::OrbitManager;
use crate::managers::ResourceManager;
use crate::utils::log::*;
use crate::utils::orbits::OBLIQUITY_OF_ECLIPTIC;
use crate::utils::string::get_unique_name;
use crate::utils::units::J2000_JDN;
use na::*;
use std::collections::HashMap;

pub struct SceneManager {
    main: Option<String>,
    loaded_scenes: HashMap<String, Scene>,
    pub descriptions: HashMap<String, SceneDescription>,
}
impl SceneManager {
    pub fn new() -> Self {
        let mut new_man = Self {
            main: None,
            loaded_scenes: HashMap::new(),
            descriptions: HashMap::new(),
        };

        // Don't actually call set_scene here as we don't want to trigger all that stuff
        new_man.new_scene("empty");
        new_man.main = Some(String::from("empty"));

        return new_man;
    }

    /// For SceneManager usage. Consumers should just `set_scene` instead, and the scenes
    /// will be created automatically when needed
    fn new_scene(&mut self, scene_name: &str) -> Option<&mut Scene> {
        let unique_scene_name = get_unique_name(scene_name, &self.loaded_scenes);

        let scene = Scene::new(&unique_scene_name);
        debug!(LogCat::Scene, "Created scene '{}'", unique_scene_name);

        assert!(!self.loaded_scenes.contains_key(&unique_scene_name));

        self.loaded_scenes
            .insert(unique_scene_name.to_string(), scene);
        let scene_in_storage = self.loaded_scenes.get_mut(&unique_scene_name);

        return scene_in_storage;
    }

    /// Sets the current scene to one with `identifier`. Will load/create the scene
    /// if we can/know how to. This includes constructing new `Scene`s from `SceneDescription`s
    pub fn set_scene(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) {
        if let Some(main) = &self.main {
            if main.as_str() == identifier {
                return;
            }
        };

        if !self.loaded_scenes.contains_key(identifier) {
            self.load_scene(identifier, res_man, orbit_man, state);
        }

        // Don't start resetting/unloading stuff if we have nowhere else to go
        // If this failed to load I'm sure we'll have another error message about it too
        if self.get_scene(identifier).is_none() {
            return;
        }

        // Discard previous scene and reset state
        if let Some(main) = self.main.clone() {
            self.delete_scene(&main);
        }
        state.selection = None;
        state.hovered = None;

        // Set new scene
        self.main = Some(identifier.to_string());
        let main_scene = self.get_main_scene().unwrap();

        // Check if we have a description for that scene (they should have same name)
        if let Some(desc) = self.descriptions.get(identifier) {
            state.simulation_speed = desc.simulation_scale;

            let mut need_go_to: bool = false;

            info!(LogCat::Scene, "Loading new scene from its defaults");

            if desc.camera_pos.is_some() && desc.camera_target.is_some() && desc.camera_up.is_some()
            {
                state.camera.pos = desc.camera_pos.unwrap();
                state.camera.up = desc.camera_up.unwrap();
                state.camera.target = desc.camera_target.unwrap();
            } else {
                need_go_to = true;
            }

            if let Some(focus) = &desc.focus {
                // Ugh.. this shouldn't be too often though
                for (entity, component) in main_scene.metadata.iter() {
                    if let Some(id) = component.get_metadata("body_id") {
                        if id == focus {
                            state.next_reference_entity =
                                Some(ReferenceChange::FocusKeepCoords(*entity));

                            if need_go_to {
                                state.entity_going_to = Some(*entity);
                            }

                            debug!(
                                LogCat::Scene,
                                "Setting initial focused entity to '{:?}'",
                                main_scene.get_entity_name(*entity)
                            );
                            break;
                        }
                    }
                }
            } else {
                state.next_reference_entity = Some(ReferenceChange::Clear);
            }

            // TODO: Proper handling of time (would involve rolling simulation/orbits forward/back
            // to match target time, for now let's just all sit at J2000)
            assert!(desc.time == String::from("J2000"));
            state.sim_time_s = 0.0;
        }

        // Do this last because we'll check whether we should goto something or keep our state
        // by comparing against this
        state.last_scene_identifier = identifier.to_string();
    }

    fn load_scene(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) -> &mut Scene {
        self.load_scene_from_desc(identifier, res_man, orbit_man, state)
    }

    pub fn load_last_scene(
        &mut self,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) {
        let identifier = state.last_scene_identifier.clone();

        if !identifier.is_empty() {
            if self.descriptions.contains_key(&identifier) {
                info!(LogCat::Scene, "Trying to load last scene '{}'", identifier);
                self.set_scene(&identifier, res_man, orbit_man, state);
            } else {
                warning!(
                    LogCat::Scene,
                    "Failed to find a description for last loaded scene '{}'. Current descriptions: '{:#?}'",
                    identifier,
                    self.descriptions.keys()
                );
            }
        }
    }

    /// Completely unloads the scene with `identifier`. The next time `set_scene` is called with
    /// `identifier`, it may need to be re-constructed
    pub fn delete_scene(&mut self, identifier: &str) {
        // Never delete the empty scene as that is our "fallback" scene
        if identifier == "empty" {
            return;
        }

        if let Some(main_name) = self.main.as_ref() {
            if main_name == identifier {
                self.main = Some(String::from("empty"));
            }
        }

        info!(
            LogCat::Scene,
            "Unloading scene '{}'. Scene is now '{:?}'", identifier, self.main
        );

        self.loaded_scenes.remove(identifier);
    }

    pub fn get_main_scene(&self) -> Option<&Scene> {
        match self.main.as_ref() {
            Some(ident_ref) => self.get_scene(&ident_ref),
            None => None,
        }
    }

    pub fn get_main_scene_mut(&mut self) -> Option<&mut Scene> {
        match self.main.as_ref() {
            Some(ident_ref) => self.loaded_scenes.get_mut(ident_ref),
            None => None,
        }
    }

    pub fn get_scene(&self, identifier: &str) -> Option<&Scene> {
        return self.loaded_scenes.get(identifier);
    }

    pub fn get_scene_mut(&mut self, identifier: &str) -> Option<&mut Scene> {
        return self.loaded_scenes.get_mut(identifier);
    }

    pub fn receive_serialized_scene(&mut self, serialized: &str) {
        let new_desc: Result<SceneDescription, String> = ron::de::from_str(serialized)
            .map_err(|e| format!("RON deserialization error:\n{}", e).to_owned());
        if let Err(e) = new_desc {
            error!(LogCat::Io, "{}", e);
            return;
        }
        let new_desc = new_desc.unwrap();

        info!(
            LogCat::Scene,
            "Loaded new scene description '{}'", new_desc.name
        );

        let name = new_desc.name.clone();
        self.descriptions.insert(name, new_desc);
    }

    fn load_scene_from_desc(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) -> &mut Scene {
        let desc = self.descriptions.get(identifier).cloned().unwrap();

        // Make Rust happy
        // TODO: Remove this cloning
        let name = desc.name.clone();
        let mut instance_descs: Vec<BodyInstanceDescription> = desc.bodies.clone();
        let time = J2000_JDN;

        let scene = self.new_scene(&name).unwrap();

        let mut parsed_body_name_to_main_ent: HashMap<String, Entity> = HashMap::new();
        let mut bodies_to_parse: Vec<(Option<&BodyDescription>, &BodyInstanceDescription)> =
            Vec::new();

        // Collect all bodies to parse
        // TODO: This type of stuff shouldn't be here...
        for instance_desc in instance_descs.iter_mut() {
            if let Some(source) = instance_desc.source.clone() {
                let slash_pos = source.rfind("/").unwrap();
                let db_name = &source[..slash_pos]; // e.g. "major_bodies"
                let body_id = &source[slash_pos + 1..]; // e.g. "399"

                // Wildcard
                if let Some(num_str) = body_id.strip_prefix("*") {
                    let mut limit: Option<usize> = None;

                    if num_str.len() > 0 {
                        let parsed = num_str.parse::<usize>();
                        if parsed.is_err() {
                            warning!(
                                LogCat::Scene,
                                "Failed to parse the wildcard number in instance desc source '{}'",
                                source
                            );
                            continue;
                        }
                        limit = Some(parsed.unwrap());
                    }

                    let bodies = orbit_man.get_n_bodies(db_name, limit);

                    info!(
                        LogCat::Scene,
                        "Found {} wildcard bodies from db '{}'",
                        bodies.len(),
                        db_name
                    );

                    bodies_to_parse.reserve(bodies.len());
                    for body in bodies {
                        bodies_to_parse.push((Some(body), instance_desc));
                    }
                // Just a single body with a source
                } else {
                    bodies_to_parse
                        .push((orbit_man.get_body(db_name, body_id).ok(), instance_desc));
                }
            }
        }

        // Parse bodies
        let mut last_num_left = 0;
        let num_left = loop {
            bodies_to_parse.retain(|(body, instance)| {
                // Check if we have/need a parent
                let mut parent_entity: Option<Entity> = None;
                if let Some(parent_name) = &instance.parent {
                    if parent_name.len() > 0 {
                        if let Some(parsed_parent) = parsed_body_name_to_main_ent.get(parent_name) {
                            parent_entity = Some(*parsed_parent);
                        } else {
                            // We haven't parsed the parent yet, skip this body for now
                            return true;
                        }
                    }
                }

                let default_state_vector = body.and_then(|b| fetch_default_motion_if_needed(b.id.as_ref().unwrap(), orbit_man, time));

                let name_ent = add_body_instance_entities(
                    scene,
                    time,
                    *body,
                    &instance,
                    default_state_vector,
                    res_man,
                );

                // Sometimes we abort due to missing mass/radius, etc.
                if name_ent.is_none() {
                    return false;
                }
                let name_ent = name_ent.unwrap();

                // Add our entity as a child if we have a designated parent that we found
                if let Some(parent_entity) = parent_entity {
                    scene.set_entity_parent(parent_entity, name_ent.1);
                }

                let old = parsed_body_name_to_main_ent.insert(name_ent.0.clone(), name_ent.1);
                if let Some(_) = old {
                    error!(LogCat::Scene, "Name collision for body instance name '{}'. Entity '{:?}' will be used from now on", name_ent.0, name_ent.1);
                }

                // We successfully parsed this one, so remove it from the vec
                return false;
            });

            // Break if we haven't parsed anything this whole pass
            let new_num_left = bodies_to_parse.len();
            if new_num_left == last_num_left {
                break new_num_left;
            }
            last_num_left = new_num_left;
        };
        if num_left > 0 {
            error!(
                LogCat::Scene,
                "Failed to parse {} bodies due to missing parents! Bodies left:\n{:#?}",
                num_left,
                bodies_to_parse
            );
        }

        // Grid
        if state.show_grid {
            let grid = scene.new_entity(Some("grid"));
            let trans_comp = scene.add_component::<TransformComponent>(grid);
            trans_comp.get_local_transform_mut().scale =
                Vector3::new(1000000.0, 1000000.0, 1000000.0);
            let mesh_comp = scene.add_component::<MeshComponent>(grid);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));
        }

        // Axes
        if state.show_axes {
            let axes = scene.new_entity(Some("axes"));
            let trans_comp = scene.add_component::<TransformComponent>(axes);
            trans_comp.get_local_transform_mut().scale = Vector3::new(10000.0, 10000.0, 10000.0);
            let mesh_comp = scene.add_component::<MeshComponent>(axes);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));
        }

        // Skybox
        if state.use_skyboxes {
            scene.skybox_mesh = res_man.get_or_create_mesh("quad");
            scene.skybox_trans = Some(
                Rotation3::<f64>::from_axis_angle(
                    &Vector3::x_axis(),
                    OBLIQUITY_OF_ECLIPTIC.to_rad().0,
                )
                .to_homogeneous(),
            );
            scene.skybox_mat = res_man.instantiate_material("skybox", "test_skybox");
            scene.skybox_mat.as_ref().unwrap().borrow_mut().set_texture(
                TextureUnit::BaseColor,
                res_man
                    .get_or_request_texture(&("public/textures/".to_owned() + "starmap_16k"), true),
            );
        }

        // Points
        if state.show_points {
            scene.points_mesh = res_man.get_or_create_mesh("points");
            scene.points_mat = res_man.get_or_create_material("default_points");
        }

        return scene;
    }
}
