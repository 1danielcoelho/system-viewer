pub trait GltfResource {
    fn get_identifier(&self, scene_identifier: &str) -> String;
}

impl GltfResource for gltf::Mesh<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Material<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().unwrap().to_string();
    }
}

impl GltfResource for gltf::Texture<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Node<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_node_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Scene<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_scene_" + &self.index().to_string();
    }
}
