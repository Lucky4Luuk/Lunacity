use glam::*;

pub trait IsBRDF {
    /// The signature needs to be the same as the function name in the code
    fn signature(&self) -> String;
    fn code(&self) -> String;
}

pub struct Lambert;
impl IsBRDF for Lambert {
    fn signature(&self) -> String {
        "lambert".to_string()
    }
    fn code(&self) -> String {
        return
"vec3 lambert(Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {
    float n_dot_l = dot(light, normal);
    return mat.albedo * clamp(n_dot_l, 0.0, 1.0);
}".to_string();
    }
}
