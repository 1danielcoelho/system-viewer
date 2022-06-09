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
use crate::utils::units::J2000_JDN;
use na::*;
use std::collections::HashMap;

pub struct SceneManager {
    current_scene: Option<Scene>,
    pub descriptions: HashMap<String, SceneDescription>,
}
impl SceneManager {
    pub fn new() -> Self {
        Self {
            current_scene: Some(Scene::new("empty")),
            descriptions: HashMap::new(),
        }
    }

    pub fn load_scene(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) {
        state.selection = None;
        state.hovered = None;

        self.current_scene = Some(Scene::new(&identifier));

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
                for (entity, component) in self.current_scene.unwrap().metadata.iter() {
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
                                self.current_scene.unwrap().get_entity_name(*entity)
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

        self.load_scene_from_desc(identifier, res_man, orbit_man, state);
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
                self.load_scene(&identifier, res_man, orbit_man, state);
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

    pub fn get_current_scene(&self) -> Option<&Scene> {
        self.current_scene.as_ref()
    }

    pub fn get_current_scene_mut(&mut self) -> Option<&mut Scene> {
        self.current_scene.as_mut()
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
    ) {
        let desc = self.descriptions.get(identifier).cloned().unwrap();

        // Make Rust happy
        // TODO: Remove this cloning
        let name = desc.name.clone();
        let mut instance_descs: Vec<BodyInstanceDescription> = desc.bodies.clone();
        let time = J2000_JDN;

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
                    self.current_scene.as_mut().unwrap(),
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
                    self.current_scene.as_ref().unwrap().set_entity_parent(parent_entity, name_ent.1);
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

        let mut_scene = self.current_scene.as_mut().unwrap();

        // Grid
        if state.show_grid {
            let grid = mut_scene.new_entity(Some("grid"));
            let trans_comp = mut_scene.add_component::<TransformComponent>(grid);
            trans_comp.get_local_transform_mut().scale =
                Vector3::new(1000000.0, 1000000.0, 1000000.0);
            let mesh_comp = mut_scene.add_component::<MeshComponent>(grid);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));
        }

        // Axes
        if state.show_axes {
            let axes = mut_scene.new_entity(Some("axes"));
            let trans_comp = mut_scene.add_component::<TransformComponent>(axes);
            trans_comp.get_local_transform_mut().scale = Vector3::new(10000.0, 10000.0, 10000.0);
            let mesh_comp = mut_scene.add_component::<MeshComponent>(axes);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));
        }

        // Skybox
        if state.use_skyboxes {
            mut_scene.skybox_mesh = res_man.get_or_create_mesh("quad");
            mut_scene.skybox_trans = Some(
                Rotation3::<f64>::from_axis_angle(
                    &Vector3::x_axis(),
                    OBLIQUITY_OF_ECLIPTIC.to_rad().0,
                )
                .to_homogeneous(),
            );
            mut_scene.skybox_mat = res_man.instantiate_material("skybox", "test_skybox");
            mut_scene
                .skybox_mat
                .as_ref()
                .unwrap()
                .borrow_mut()
                .set_texture(
                    TextureUnit::BaseColor,
                    res_man.get_or_request_texture(
                        &("public/textures/".to_owned() + "starmap_16k"),
                        true,
                    ),
                );
        }

        // Points
        if state.show_points {
            mut_scene.points_mesh = res_man.get_or_create_mesh("points");
            mut_scene.points_mat = res_man.get_or_create_material("default_points");
        }
    }
}
