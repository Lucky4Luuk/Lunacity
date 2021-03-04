use std::path::Path;

/// Shader preprocessor. Handles things like `#include`
//TODO: Handle comments at the end of the `#include` line
pub fn preprocessor(src_path: &Path, dispatch_size: (u32, u32)) -> String {
    let src_path_dir = src_path.parent().expect("File must be in a directory of some kind. How did you manage this??");

    let src = std::fs::read_to_string(src_path).expect("Failed to open file!");

    let mut result = String::new();

    for line in src.lines() {
        if line.starts_with("#include") {
            let path = src_path_dir.join(&line.replace("#include ", "").replace('"', ""));
            let include = preprocessor(&path, dispatch_size);
            result.push_str(&include);
            result.push('\n');
        } else {
            let mut fline = line.replace("DISPATCH_SIZE_X", &format!("{}", dispatch_size.0));
            fline = fline.replace("DISPATCH_SIZE_Y", &format!("{}", dispatch_size.1));
            result.push_str(&fline);
            result.push('\n');
        }
    }

    result
}
