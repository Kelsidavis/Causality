// Rhai API bindings - exposes game engine to scripts

use glam::{Quat, Vec3};
use rhai::Engine;

/// Register all game engine types and functions with Rhai
pub fn register_api(engine: &mut Engine) {
    register_vec3(engine);
    register_quat(engine);
    register_math(engine);
}

/// Register Vec3 type and methods
fn register_vec3(engine: &mut Engine) {
    engine
        .register_type::<Vec3>()
        .register_fn("Vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z))
        .register_fn("vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z))
        .register_get("x", |v: &mut Vec3| v.x)
        .register_set("x", |v: &mut Vec3, val: f32| v.x = val)
        .register_get("y", |v: &mut Vec3| v.y)
        .register_set("y", |v: &mut Vec3, val: f32| v.y = val)
        .register_get("z", |v: &mut Vec3| v.z)
        .register_set("z", |v: &mut Vec3, val: f32| v.z = val)
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, b: f32| a * b)
        .register_fn("*", |a: f32, b: Vec3| a * b)
        .register_fn("/", |a: Vec3, b: f32| a / b)
        .register_fn("length", |v: Vec3| v.length())
        .register_fn("normalize", |v: Vec3| v.normalize())
        .register_fn("dot", |a: Vec3, b: Vec3| a.dot(b))
        .register_fn("cross", |a: Vec3, b: Vec3| a.cross(b))
        .register_fn("to_string", |v: &mut Vec3| format!("Vec3({}, {}, {})", v.x, v.y, v.z));
}

/// Register Quat type and methods
fn register_quat(engine: &mut Engine) {
    engine
        .register_type::<Quat>()
        .register_fn("Quat", |x: f32, y: f32, z: f32, w: f32| Quat::from_xyzw(x, y, z, w))
        .register_fn("quat_identity", || Quat::IDENTITY)
        .register_fn("quat_from_rotation_x", |angle: f32| Quat::from_rotation_x(angle))
        .register_fn("quat_from_rotation_y", |angle: f32| Quat::from_rotation_y(angle))
        .register_fn("quat_from_rotation_z", |angle: f32| Quat::from_rotation_z(angle))
        .register_fn("*", |a: Quat, b: Quat| a * b)
        .register_fn("*", |q: Quat, v: Vec3| q * v)
        .register_fn("to_string", |q: &mut Quat| format!("Quat({}, {}, {}, {})", q.x, q.y, q.z, q.w));
}

/// Register common math functions
fn register_math(engine: &mut Engine) {
    engine
        .register_fn("sin", |x: f32| x.sin())
        .register_fn("cos", |x: f32| x.cos())
        .register_fn("tan", |x: f32| x.tan())
        .register_fn("sqrt", |x: f32| x.sqrt())
        .register_fn("abs", |x: f32| x.abs())
        .register_fn("min", |a: f32, b: f32| a.min(b))
        .register_fn("max", |a: f32, b: f32| a.max(b))
        .register_fn("clamp", |x: f32, min: f32, max: f32| x.clamp(min, max))
        .register_fn("lerp", |a: f32, b: f32, t: f32| a + (b - a) * t)
        .register_fn("to_radians", |degrees: f32| degrees.to_radians())
        .register_fn("to_degrees", |radians: f32| radians.to_degrees());
}
