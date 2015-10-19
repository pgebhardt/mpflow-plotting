extern crate rustc_serialize;

use rustc_serialize::json::Json;

pub fn extract_mesh_path(config: &Json) -> Option<String> {
    let default = Some("mesh".to_string());

    // try to extract mesh path from config
    if let Some(config) = config.as_object() {
        // try to get model config
        match config.get("model") {
            Some(&Json::Object(ref model_config)) => {
                // try to extract mesh config
                match model_config.get("mesh") {
                    // mesh config is already the path
                    Some(&Json::String(ref path)) => Some(path.clone()),

                    // mesh config contains not only mesh path
                    Some(&Json::Object(ref mesh_config)) => {
                        match mesh_config.get("path") {
                            Some(&Json::String(ref path)) => Some(path.clone()),
                            _ => default,
                        }
                    },
                    _ => default,
                }
            },
            _ => None,
        }
    }
    else {
        None
    }
}

pub fn extract_ports_path(config: &Json) -> Option<String> {
    // try to extract mesh path from config
    if let Some(config) = config.as_object() {
        // try to get model config
        match config.get("model") {
            Some(&Json::Object(ref model_config)) => {
                // try to extract ports config
                match model_config.get("ports") {
                    // ports config is already the path
                    Some(&Json::String(ref path)) => Some(path.clone()),

                    // ports config contains not only mesh path
                    Some(&Json::Object(ref ports_config)) => {
                        match ports_config.get("edges") {
                            Some(&Json::String(ref path)) => Some(path.clone()),
                            _ => None,
                        }
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }
    else {
        None
    }

}
