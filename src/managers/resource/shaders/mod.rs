use regex::Regex;
use std::collections::{HashMap, HashSet};

macro_rules! include_str_map {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_map = HashMap::new();
            $(
                temp_map.insert(String::from($x), include_str!($x));
            )*
            temp_map
        }
    };
}

lazy_static! {
    pub static ref SHADER_STORAGE: HashMap<String, String> = {
        // TODO: Find how to scan all files in a directory at compile time
        let shader_paths: HashMap<String, &str> = include_str_map![
            "basecolor.frag",
            "color.frag",
            "white.frag",
            "skybox.frag",
            "screenspace.frag",
            "gltf_metal_rough.frag",
            "phong.frag",
            "uv0.frag",
            "uv1.frag",
            "normals.frag",
            "tangents.frag",

            "brdf.glsl",
            "constants.glsl",
            "functions.glsl",

            "relay_all.vert",
            "relay_locals.vert",
            "relay_color.vert",
            "screenspace.vert"
        ];

        let re = Regex::new("#include [\"<](.*)[\">]").unwrap();

        // Resolve includes (also handles nested includes)
        let mut storage = HashMap::new();
        for (path, code) in shader_paths.iter() {
            let mut modified_code: String = (*code).to_owned();
            let mut included_files: HashSet<String> = HashSet::new();

            loop {
                // We intentionally go one at a time, because an include should always
                // show up as early as possible
                let cap = re.captures(&modified_code);
                if cap.is_none() {
                    break;
                }
                let cap = cap.unwrap();

                let outer_match = cap.get(0).unwrap();
                let start = outer_match.start();
                let end = outer_match.end();

                let file = cap.get(1).unwrap().as_str().to_owned();

                if included_files.contains(&file) {
                    modified_code.replace_range(start..end, "");
                } else {
                    modified_code.replace_range(start..end, shader_paths.get(&file).expect(&format!("Failed to find shader '{}'!", file)));
                    included_files.insert(file.clone());
                }
            }

            storage.insert(path.clone(), modified_code);
        }

        storage
    };
}
