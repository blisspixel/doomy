//! The shared movement step: the one function that both the server and the
//! predicting client run, written here in Rust and mirrored line for line in
//! `client/scripts/movement.gd`. Pure data in, pure data out, no engine calls,
//! no randomness, so the same inputs give the same states on both sides.
//!
//! Agreement is proven by golden vectors in `client/golden/move_vectors.json`:
//! this module generates them, the test below asserts this module reproduces
//! them exactly, and the headless Godot harness asserts the GDScript mirror
//! reproduces them within a tolerance. Change the model here, regenerate the
//! vectors, review the diff like code.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Fighter collision radius in units.
pub const RADIUS: f32 = 0.5;
/// Ground top speed in units per second.
pub const TOP_SPEED: f32 = 5.0;
/// Time constant for speeding up, seconds.
pub const TAU_ACCEL: f32 = 0.06;
/// Time constant for slowing down, seconds.
pub const TAU_DECEL: f32 = 0.04;
/// The movement step length at the 60 Hz rate the tick migration adopts.
pub const DT_60HZ: f32 = 1.0 / 60.0;
/// Floor height. The arena is flat, so this is the only ground there is until
/// the map tiers bring geometry with height in it.
pub const GROUND_Y: f32 = 0.0;
/// Downward acceleration in units per second squared. Chosen with the jump
/// below so a hop clears about 1.1 units and lasts a little under half a
/// second, which is the Quake-ish arc this game's speed wants rather than the
/// floatier one a slower game can afford.
pub const GRAVITY: f32 = 22.0;
/// Upward speed applied on a jump, in units per second.
pub const JUMP_SPEED: f32 = 7.0;

/// Where a fighter is and how it moves. Yaw is radians in `[0, 2 pi)`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoveState {
    pub x: f32,
    pub z: f32,
    /// Height above the floor. Zero is standing on it.
    #[serde(default)]
    pub y: f32,
    pub vx: f32,
    pub vz: f32,
    /// Vertical speed. Positive is upward.
    #[serde(default)]
    pub vy: f32,
    pub yaw: f32,
}

impl MoveState {
    /// Whether this fighter is standing on the floor, which is the only thing
    /// a jump is allowed to push off.
    pub fn grounded(&self) -> bool {
        self.y <= GROUND_Y && self.vy <= 0.0
    }
}

/// One step's worth of intent. `speed_scale` is 1.0 normally and 0.5 under
/// the compliance slow; `yaw` is the client-owned facing for this step.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoveInput {
    #[serde(default)]
    pub forward: bool,
    #[serde(default)]
    pub back: bool,
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    /// Held, not edge-triggered. A fighter leaves the ground on the first step
    /// where this is set and it is standing, and holding it does not keep it
    /// climbing, so a client that drops an input does not lose a jump it has
    /// already started.
    #[serde(default)]
    pub jump: bool,
    pub yaw: f32,
    #[serde(default = "one")]
    pub speed_scale: f32,
}

fn one() -> f32 {
    1.0
}

impl Default for MoveInput {
    fn default() -> Self {
        MoveInput {
            forward: false,
            back: false,
            left: false,
            right: false,
            jump: false,
            yaw: 0.0,
            speed_scale: 1.0,
        }
    }
}

/// An axis-aligned solid in the XZ plane, not yet inflated by the radius.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Solid {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl Solid {
    pub fn from_center(cx: f32, cz: f32, half_x: f32, half_z: f32) -> Self {
        Solid {
            min_x: cx - half_x,
            max_x: cx + half_x,
            min_z: cz - half_z,
            max_z: cz + half_z,
        }
    }

    /// True when a circle of `radius` at `(x, z)` overlaps this solid.
    pub fn blocks(&self, x: f32, z: f32, radius: f32) -> bool {
        x >= self.min_x - radius
            && x <= self.max_x + radius
            && z >= self.min_z - radius
            && z <= self.max_z + radius
    }
}

/// The playable square of half-width `half` (centre at the origin) and its solids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arena {
    pub half: f32,
    pub solids: Vec<Solid>,
}

impl Arena {
    pub fn blocked(&self, x: f32, z: f32) -> bool {
        self.solids.iter().any(|s| s.blocks(x, z, RADIUS))
    }

    pub fn clamp(&self, x: f32, z: f32) -> (f32, f32) {
        let limit = self.half - RADIUS;
        (x.clamp(-limit, limit), z.clamp(-limit, limit))
    }
}

/// Wrap any finite yaw into `[0, 2 pi)`. Non-finite yaw becomes 0.
pub fn normalize_yaw(yaw: f32) -> f32 {
    if !yaw.is_finite() {
        return 0.0;
    }
    let two_pi = 2.0 * PI;
    let mut y = yaw % two_pi;
    if y < 0.0 {
        y += two_pi;
    }
    if y >= two_pi {
        y -= two_pi;
    }
    y
}

/// Unit direction of the movement keys in the yaw frame, or zero.
pub fn wish_dir(input: &MoveInput, yaw: f32) -> (f32, f32) {
    let mut dx = 0.0f32;
    let mut dz = 0.0f32;
    if input.forward {
        dx += yaw.cos();
        dz += yaw.sin();
    }
    if input.back {
        dx -= yaw.cos();
        dz -= yaw.sin();
    }
    if input.left {
        dx += (yaw - PI / 2.0).cos();
        dz += (yaw - PI / 2.0).sin();
    }
    if input.right {
        dx += (yaw + PI / 2.0).cos();
        dz += (yaw + PI / 2.0).sin();
    }
    let len = (dx * dx + dz * dz).sqrt();
    if len > 0.0 {
        (dx / len, dz / len)
    } else {
        (0.0, 0.0)
    }
}

/// Advance one fighter by `dt` seconds. The order of operations is the
/// contract the GDScript mirror keeps: yaw, wish, velocity approach, integrate,
/// clamp, axis-separated slide with velocity zeroed on the blocked axis.
pub fn step(state: MoveState, input: &MoveInput, dt: f32, arena: &Arena) -> MoveState {
    let yaw = normalize_yaw(input.yaw);
    let (wx, wz) = wish_dir(input, yaw);
    let scale = if input.speed_scale.is_finite() {
        input.speed_scale.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let target_x = wx * TOP_SPEED * scale;
    let target_z = wz * TOP_SPEED * scale;

    let current_speed = (state.vx * state.vx + state.vz * state.vz).sqrt();
    let target_speed = (target_x * target_x + target_z * target_z).sqrt();
    let tau = if target_speed > current_speed {
        TAU_ACCEL
    } else {
        TAU_DECEL
    };
    let blend = (dt / tau).min(1.0);
    let mut vx = state.vx + (target_x - state.vx) * blend;
    let mut vz = state.vz + (target_z - state.vz) * blend;

    let old_x = state.x;
    let old_z = state.z;
    let (nx, nz) = arena.clamp(old_x + vx * dt, old_z + vz * dt);

    // Vertical is independent of the walls: the arena is flat, so nothing can
    // be blocked by standing on it. This changes when the map tiers land.
    let mut vy = state.vy;
    let mut y = state.y;
    let on_ground = state.y <= GROUND_Y && state.vy <= 0.0;
    if on_ground {
        y = GROUND_Y;
        vy = 0.0;
        if input.jump {
            vy = JUMP_SPEED;
        }
    } else {
        vy -= GRAVITY * dt;
    }
    y += vy * dt;
    if y <= GROUND_Y {
        y = GROUND_Y;
        if vy < 0.0 {
            vy = 0.0;
        }
    }

    let (x, z) = if !arena.blocked(nx, nz) {
        (nx, nz)
    } else if !arena.blocked(nx, old_z) {
        vz = 0.0;
        (nx, old_z)
    } else if !arena.blocked(old_x, nz) {
        vx = 0.0;
        (old_x, nz)
    } else {
        vx = 0.0;
        vz = 0.0;
        arena.clamp(old_x, old_z)
    };

    MoveState {
        x,
        z,
        y,
        vx,
        vz,
        vy,
        yaw,
    }
}

/// One golden case: an arena, a start, inputs, and the state expected after
/// every `stride` steps (so a long case stays small on disk).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldenCase {
    pub name: String,
    pub arena: Arena,
    pub start: MoveState,
    pub inputs: Vec<MoveInput>,
    pub stride: usize,
    pub expected: Vec<MoveState>,
}

/// The golden file: the constants the vectors were made with, and the cases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldenFile {
    pub version: u32,
    pub dt: f32,
    pub radius: f32,
    pub top_speed: f32,
    pub tau_accel: f32,
    pub tau_decel: f32,
    pub cases: Vec<GoldenCase>,
}

/// Where the vectors live, shared with the Godot harness.
pub const GOLDEN_PATH: &str = "client/golden/move_vectors.json";

fn run_case(
    arena: &Arena,
    start: MoveState,
    inputs: &[MoveInput],
    dt: f32,
    stride: usize,
) -> Vec<MoveState> {
    let mut state = start;
    let mut out = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        state = step(state, input, dt, arena);
        if (i + 1) % stride == 0 {
            out.push(state);
        }
    }
    out
}

fn at(x: f32, z: f32, yaw: f32) -> MoveState {
    MoveState {
        x,
        z,
        y: GROUND_Y,
        vx: 0.0,
        vz: 0.0,
        vy: 0.0,
        yaw,
    }
}

fn hold(input: MoveInput, n: usize) -> Vec<MoveInput> {
    vec![input; n]
}

fn keys(forward: bool, back: bool, left: bool, right: bool, yaw: f32) -> MoveInput {
    MoveInput {
        forward,
        back,
        left,
        right,
        jump: false,
        yaw,
        speed_scale: 1.0,
    }
}

/// The reference arena for the vectors: a 50 by 50 square with a crate, a wall,
/// and a corner pillar, in positions that keep every case off exact boundaries.
pub fn golden_arena() -> Arena {
    Arena {
        half: 25.0,
        solids: vec![
            Solid::from_center(6.0, 0.0, 1.0, 1.0),
            Solid::from_center(0.0, 10.0, 6.0, 0.5),
            Solid::from_center(-12.0, -12.0, 1.5, 1.5),
        ],
    }
}

/// Build the golden file from the current model.
pub fn golden_cases(dt: f32) -> GoldenFile {
    let arena = golden_arena();
    let mut cases = Vec::new();
    let mut push = |name: &str, start: MoveState, inputs: Vec<MoveInput>, stride: usize| {
        let expected = run_case(&arena, start, &inputs, dt, stride);
        cases.push(GoldenCase {
            name: name.to_string(),
            arena: arena.clone(),
            start,
            inputs,
            stride,
            expected,
        });
    };

    push(
        "straight_run",
        at(0.0, -5.0, 0.0),
        hold(keys(true, false, false, false, 0.0), 40),
        1,
    );
    push(
        "diagonal_run_normalised",
        at(-5.0, -5.0, 0.7),
        hold(keys(true, false, false, true, 0.7), 40),
        1,
    );
    let mut start_stop = hold(keys(true, false, false, false, 1.2), 30);
    start_stop.extend(hold(keys(false, false, false, false, 1.2), 30));
    push("start_and_stop", at(-8.0, 2.0, 1.2), start_stop, 1);
    push(
        "slide_along_wall_x",
        at(-3.0, 8.0, 0.3),
        hold(keys(true, false, false, false, 0.3), 60),
        1,
    );
    push(
        "slide_along_wall_z",
        at(4.2, -4.0, PI / 2.0 + 0.2),
        hold(keys(true, false, false, false, PI / 2.0 + 0.2), 60),
        1,
    );
    push(
        "corner_stop",
        at(-14.0, -14.0, PI / 4.0),
        hold(keys(true, false, false, false, PI / 4.0), 60),
        1,
    );
    push(
        "arena_edge_clamp",
        at(20.0, 20.0, PI / 4.0),
        hold(keys(true, false, false, false, PI / 4.0), 80),
        1,
    );
    let mut wrap = hold(keys(true, false, false, false, -0.05), 10);
    wrap.extend(hold(keys(true, false, false, false, 2.0 * PI + 0.05), 10));
    wrap.extend(hold(keys(true, false, false, false, 3.0 * PI), 10));
    push("yaw_wrap", at(0.0, 0.0, 0.0), wrap, 1);
    let mut slow = hold(keys(true, false, false, false, 2.5), 20);
    for input in slow.iter_mut().skip(10) {
        input.speed_scale = 0.5;
    }
    push("compliance_slow", at(5.0, -15.0, 2.5), slow, 1);
    let mut long = Vec::with_capacity(1000);
    for i in 0..1000usize {
        let phase = (i / 50) % 4;
        let yaw = 0.017 * i as f32;
        long.push(match phase {
            0 => keys(true, false, false, false, yaw),
            1 => keys(true, false, true, false, yaw),
            2 => keys(false, false, false, true, yaw),
            _ => keys(false, true, false, false, yaw),
        });
    }
    push("long_wander_1000", at(2.0, 2.0, 0.0), long, 100);
    // One tick of jump held, then nothing, so the arc is gravity's and not the
    // key's. Every checkpoint pins a height, which is what holds the GDScript
    // mirror to the same curve.
    let mut hop = Vec::with_capacity(40);
    let mut first = keys(true, false, false, false, 0.0);
    first.jump = true;
    hop.push(first);
    hop.extend(hold(keys(true, false, false, false, 0.0), 39));
    push("jump_arc", at(0.0, 0.0, 0.0), hop, 4);

    GoldenFile {
        version: 1,
        dt,
        radius: RADIUS,
        top_speed: TOP_SPEED,
        tau_accel: TAU_ACCEL,
        tau_decel: TAU_DECEL,
        cases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn workspace_path(rel: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
    }

    #[test]
    fn yaw_normalises_and_survives_nonsense() {
        assert_eq!(normalize_yaw(0.0), 0.0);
        assert!((normalize_yaw(-0.5) - (2.0 * PI - 0.5)).abs() < 1e-6);
        assert!((normalize_yaw(2.0 * PI + 0.25) - 0.25).abs() < 1e-6);
        assert!(normalize_yaw(2.0 * PI) < 1e-6);
        assert_eq!(normalize_yaw(f32::NAN), 0.0);
        assert_eq!(normalize_yaw(f32::INFINITY), 0.0);
    }

    #[test]
    fn wish_direction_is_unit_or_zero() {
        let none = keys(false, false, false, false, 0.0);
        assert_eq!(wish_dir(&none, 0.0), (0.0, 0.0));
        let opposed = keys(true, true, false, false, 0.0);
        assert_eq!(wish_dir(&opposed, 0.0), (0.0, 0.0));
        let diag = keys(true, false, false, true, 0.0);
        let (dx, dz) = wish_dir(&diag, 0.0);
        assert!(((dx * dx + dz * dz).sqrt() - 1.0).abs() < 1e-6);
        let fwd = keys(true, false, false, false, PI / 2.0);
        let (dx, dz) = wish_dir(&fwd, PI / 2.0);
        assert!(dx.abs() < 1e-6 && (dz - 1.0).abs() < 1e-6);
    }

    #[test]
    fn velocity_approaches_top_speed_and_stops() {
        let arena = Arena {
            half: 25.0,
            solids: vec![],
        };
        let mut s = at(0.0, 0.0, 0.0);
        let go = keys(true, false, false, false, 0.0);
        for _ in 0..30 {
            s = step(s, &go, DT_60HZ, &arena);
        }
        assert!((s.vx - TOP_SPEED).abs() < 1e-3, "{s:?}");
        assert!(
            s.x > 2.0 && s.x < 2.5,
            "about half a second of running: {s:?}"
        );
        let stop = keys(false, false, false, false, 0.0);
        let mut steps = 0;
        while s.vx > 0.01 {
            s = step(s, &stop, DT_60HZ, &arena);
            steps += 1;
        }
        assert!(steps <= 20, "stops inside a third of a second: {steps}");
        let slow = MoveInput {
            speed_scale: 0.5,
            ..go
        };
        for _ in 0..60 {
            s = step(s, &slow, DT_60HZ, &arena);
        }
        assert!((s.vx - TOP_SPEED * 0.5).abs() < 1e-3, "{s:?}");
        let bad = MoveInput {
            speed_scale: f32::NAN,
            ..go
        };
        let s2 = step(s, &bad, DT_60HZ, &arena);
        assert!(s2.vx.is_finite());
    }

    #[test]
    fn slides_along_walls_and_clamps_to_the_arena() {
        let arena = golden_arena();
        // Into the long wall at z=10 from below, heading mostly +z with a little +x.
        let mut s = at(-3.0, 8.0, PI / 2.0 - 0.3);
        let go = keys(true, false, false, false, PI / 2.0 - 0.3);
        for _ in 0..90 {
            s = step(s, &go, DT_60HZ, &arena);
        }
        assert!(s.z <= 9.5 + 1e-4, "never inside the wall: {s:?}");
        assert!(s.x > -3.0, "slid along it: {s:?}");
        assert_eq!(s.vz, 0.0, "blocked axis velocity is zeroed");
        // Into the arena edge.
        let mut e = at(24.0, 0.0, 0.0);
        for _ in 0..30 {
            e = step(e, &keys(true, false, false, false, 0.0), DT_60HZ, &arena);
        }
        assert!((e.x - (25.0 - RADIUS)).abs() < 1e-5, "{e:?}");
        // Fully wedged: inside a corner where both axes are blocked.
        let boxed = Arena {
            half: 25.0,
            solids: vec![
                Solid::from_center(1.0, 0.0, 0.2, 5.0),
                Solid::from_center(0.0, 1.0, 5.0, 0.2),
            ],
        };
        let mut w = at(0.0, 0.0, PI / 4.0);
        for _ in 0..30 {
            w = step(
                w,
                &keys(true, false, false, false, PI / 4.0),
                DT_60HZ,
                &boxed,
            );
        }
        assert!(w.x < 0.35 && w.z < 0.35, "{w:?}");
        assert_eq!((w.vx, w.vz), (0.0, 0.0));
    }

    #[test]
    fn solid_blocks_with_radius() {
        let s = Solid::from_center(0.0, 0.0, 1.0, 1.0);
        assert!(s.blocks(1.4, 0.0, RADIUS));
        assert!(!s.blocks(1.6, 0.0, RADIUS));
        assert!(s.blocks(0.0, -1.49, RADIUS));
    }

    #[test]
    fn golden_vectors_match_this_model() {
        let path = workspace_path(GOLDEN_PATH);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "read {}: {e}; regenerate with the ignored test",
                path.display()
            )
        });
        let file: GoldenFile = serde_json::from_str(&text).expect("golden json");
        assert_eq!(file.version, 1);
        assert_eq!(file.dt, DT_60HZ);
        assert_eq!(
            (file.radius, file.top_speed, file.tau_accel, file.tau_decel),
            (RADIUS, TOP_SPEED, TAU_ACCEL, TAU_DECEL),
            "constants changed; regenerate the vectors"
        );
        let fresh = golden_cases(DT_60HZ);
        assert_eq!(fresh.cases.len(), file.cases.len());
        for (want, have) in file.cases.iter().zip(fresh.cases.iter()) {
            assert_eq!(want.name, have.name);
            assert_eq!(want.expected.len(), have.expected.len(), "{}", want.name);
            for (i, (w, h)) in want.expected.iter().zip(have.expected.iter()).enumerate() {
                for (label, a, b) in [
                    ("x", w.x, h.x),
                    ("z", w.z, h.z),
                    ("vx", w.vx, h.vx),
                    ("vz", w.vz, h.vz),
                    ("yaw", w.yaw, h.yaw),
                ] {
                    assert!(
                        (a - b).abs() <= 1e-6,
                        "{} step {} {label}: file {a} vs model {b}",
                        want.name,
                        i
                    );
                }
            }
        }
    }

    /// Rewrites the golden file from the current model. Run it on purpose:
    /// `cargo test -p fragr-server regenerate_golden_vectors -- --ignored`.
    #[test]
    #[ignore]
    fn regenerate_golden_vectors() {
        let path = workspace_path(GOLDEN_PATH);
        let file = golden_cases(DT_60HZ);
        let text = serde_json::to_string(&file).unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text + "\n").unwrap();
    }

    #[test]
    fn golden_cases_cover_the_model() {
        let file = golden_cases(DT_60HZ);
        let names: Vec<&str> = file.cases.iter().map(|c| c.name.as_str()).collect();
        for needed in [
            "straight_run",
            "diagonal_run_normalised",
            "start_and_stop",
            "slide_along_wall_x",
            "slide_along_wall_z",
            "corner_stop",
            "arena_edge_clamp",
            "yaw_wrap",
            "compliance_slow",
            "long_wander_1000",
            "jump_arc",
        ] {
            assert!(names.contains(&needed), "missing golden case {needed}");
        }
        let long = file
            .cases
            .iter()
            .find(|c| c.name == "long_wander_1000")
            .unwrap();
        assert_eq!(long.expected.len(), 10, "one checkpoint per hundred steps");
        let hop = file.cases.iter().find(|c| c.name == "jump_arc").unwrap();
        let peak = hop.expected.iter().fold(f32::MIN, |a, s| a.max(s.y));
        assert!(
            peak > 0.8,
            "the jump case has to leave the ground, peaked {peak}"
        );
        assert!(
            hop.expected.last().unwrap().y.abs() < 1e-3,
            "and it has to come back down"
        );
        let wrap = file.cases.iter().find(|c| c.name == "yaw_wrap").unwrap();
        for s in &wrap.expected {
            assert!((0.0..2.0 * PI).contains(&s.yaw));
        }
        let json = serde_json::to_string(&file).unwrap();
        let back: GoldenFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, file);
    }
}
