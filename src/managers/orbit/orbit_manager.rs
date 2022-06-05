use crate::managers::orbit::{BodyDescription, OrbitalElements, StateVector};
use crate::utils::log::*;
use std::collections::HashMap;

pub struct OrbitManager {
    bodies: HashMap<String, HashMap<String, BodyDescription>>,
    state_vectors: HashMap<String, Vec<StateVector>>,
    osc_elements: HashMap<String, Vec<OrbitalElements>>,
}
impl OrbitManager {
    pub fn new() -> Self {
        let new_man = Self {
            bodies: HashMap::new(),
            state_vectors: HashMap::new(),
            osc_elements: HashMap::new(),
        };

        return new_man;
    }

    pub fn load_database_file(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "body_database" => {
                let mut parsed_data: HashMap<String, BodyDescription> =
                    serde_json::de::from_str(text)
                        .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                        .unwrap();

                // TODO: Do I need the ids in the bodies as well?
                for (key, val) in parsed_data.iter_mut() {
                    val.id = Some(key.clone());
                }

                let database_name: String = std::path::Path::new(url)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();

                let num_parsed = parsed_data.len();
                self.bodies.insert(database_name, parsed_data);

                info!(
                    LogCat::Orbit,
                    "Parsed {} bodies from database '{}'", num_parsed, url
                );
            }
            "vectors_database" => {
                let parsed_data: HashMap<String, Vec<StateVector>> = serde_json::de::from_str(text)
                    .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                    .unwrap();

                let num_parsed = parsed_data.len();
                self.state_vectors = parsed_data;

                info!(
                    LogCat::Orbit,
                    "Parsed {} state vectors from database '{}'", num_parsed, url
                );
            }
            "elements_database" => {
                let parsed_data: HashMap<String, Vec<OrbitalElements>> =
                    serde_json::de::from_str(text)
                        .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                        .unwrap();

                let num_parsed = parsed_data.len();
                self.osc_elements = parsed_data;

                info!(
                    LogCat::Orbit,
                    "Parsed {} orbital elements from database '{}'", num_parsed, url
                );
            }
            _ => {
                error!(
                    LogCat::Orbit,
                    "Unexpected database content type '{}' with url '{}'", content_type, url
                );
                return;
            }
        }
    }

    pub fn get_state_vectors(&self) -> &HashMap<String, Vec<StateVector>> {
        return &self.state_vectors;
    }

    pub fn get_osc_elements(&self) -> &HashMap<String, Vec<OrbitalElements>> {
        return &self.osc_elements;
    }

    pub fn get_body(&self, db_name: &str, body_id: &str) -> Result<&BodyDescription, String> {
        let db = self.bodies.get(db_name).ok_or(String::from(format!(
            "Resource manager has no database with name '{}'",
            db_name
        )))?;

        let body = db.get(body_id).ok_or(String::from(format!(
            "Resource manager's database '{}' has no body with id '{}'",
            db_name, body_id
        )))?;

        return Ok(body);
    }

    pub fn get_n_bodies(&self, db_name: &str, limit: Option<usize>) -> Vec<&BodyDescription> {
        let limit_num = limit.unwrap_or(std::usize::MAX);

        let db = self.bodies.get(db_name).unwrap();

        let mut result: Vec<&BodyDescription> = Vec::new();
        for (_, body) in db.iter() {
            result.push(&body);
            if result.len() >= limit_num {
                break;
            }
        }

        return result;
    }
}
