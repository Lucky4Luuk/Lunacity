use std::path::Path;

/// Shader preprocessor. Handles things like `#include`
pub fn preprocessor(src_path: &Path) -> String {
    let src_path_dir = src_path.parent().expect("File must be in a directory of some kind. How did you manage this??");

    let src = std::fs::read_to_string(src_path).expect("Failed to open file!");

    let mut result = String::new();

    for line in src.lines() {
        if line.starts_with("#include") {
            let path = src_path_dir.join(&line.replace("#include ", "").replace('"', ""));
            debug!("#include {:?}", path);
            let include = std::fs::read_to_string(path).expect("Failed to open include file!");
            result.push_str(&include);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}
