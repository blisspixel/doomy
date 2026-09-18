use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Named scrap-league identity (Contested Frequency denies it exists).
pub const MODE_NAME: &str = "Contested Frequency";
/// Playlist label under the league lie.
pub const PLAYLIST_NAME: &str = "Arena Duel";

pub fn default_mode_name() -> String {
    MODE_NAME.to_string()
}

pub fn default_playlist() -> String {
    PLAYLIST_NAME.to_string()
}

pub fn default_host_line() -> String {
    "HOST: CONTESTED FREQUENCY. PLAY VS COMPLIANCE. ARENA DUEL IS LIVE.".to_string()
}

pub const MAP_ID_ARENA_DUEL: u32 = 1;
pub const MAP_NAME_ARENA_DUEL: &str = "Arena Duel";

pub fn default_map_id() -> u32 {
    MAP_ID_ARENA_DUEL
}

pub fn default_map_name() -> String {
    MAP_NAME_ARENA_DUEL.to_string()
}

/// Host line while Continuance compliance pressure is live.
pub fn compliance_host_line() -> String {
    "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY.".to_string()
}

/// Host line while the Continuance Compliance Drone (NODS flavor) is on the floor.
pub fn boss_host_line() -> String {
    "HOST: CONTINUANCE COMPLIANCE DRONE ON DECK. ARTICLE 7 ENFORCEMENT.".to_string()
}

/// Host line after the Compliance Drone is fragged.
pub fn boss_down_host_line() -> String {
    "HOST: DRONE DOWN. CONTINUANCE DENIES THE INCIDENT. SCRAP ON.".to_string()
}

/// Host line when a live drone is wiped at round end (no frag credit).
pub fn boss_round_wipe_host_line() -> String {
    "HOST: FREQUENCY CLOSES. DRONE RECALLED. CONTINUANCE DENIES THE BEAT.".to_string()
}

/// Host line for a within-round killstreak tier (Contested Frequency voice).
pub fn killstreak_host_line(streak: u32, player: &str) -> Option<(String, String)> {
    match streak {
        2 => Some((
            "double".to_string(),
            format!("HOST: DOUBLE FREQUENCY. {player} DENIES THE DENIAL."),
        )),
        3 => Some((
            "triple".to_string(),
            "HOST: TRIPLE SCRAP. CONTINUANCE LOSES COUNT.".to_string(),
        )),
        5 => Some((
            "rampage".to_string(),
            format!("HOST: FREQUENCY RAMPAGE. {player} BREAKS EVERY APPROVED LANE."),
        )),
        _ => None,
    }
}

/// Host line for round-end MVP / podium (Contested Frequency voice).
pub fn mvp_host_line(mvp: &str, frags: u32) -> String {
    format!("HOST: ROUND MVP. {mvp} WITH {frags} FRAGS. CONTINUANCE DENIES THE PODIUM.")
}

/// Host line when a round ends with no scored MVP.
pub fn empty_mvp_host_line() -> String {
    "HOST: ROUND CLOSED. NO MVP. LEAGUE DENIES THE SCRAP.".to_string()
}

/// Host bumper when a named scrap bot dials in (Warmup / spawn flavor).
pub fn bot_intro_host_line(name: &str) -> String {
    format!("HOST: {} DIALS THE FREQUENCY.", name.to_uppercase())
}

/// Sticky Warmup / mid-join Host line naming the dialed-in rule-bot roster.
pub fn roster_host_line(names: &[String]) -> String {
    if names.is_empty() {
        return default_host_line();
    }
    let listed = format_roster_names(names);
    format!("HOST: {listed} ON THE SCRAP. FREQUENCY STAYS LIVE.")
}

fn format_roster_names(names: &[String]) -> String {
    let upper: Vec<String> = names.iter().map(|n| n.to_uppercase()).collect();
    if upper.len() <= 4 {
        upper.join(", ")
    } else {
        format!("{}, +{}", upper[..3].join(", "), upper.len() - 3)
    }
}

/// Warmup / pre-round Host drama: Contested Frequency bumper, map, roster, countdown.
pub fn warmup_host_line(map_name: &str, names: &[String], secs_left: u32) -> String {
    let map = map_name.to_uppercase();
    let secs = secs_left.max(1);
    if names.is_empty() {
        return format!(
            "HOST: CONTESTED FREQUENCY. {map} TUNES IN. FREQUENCY GOES LIVE IN {secs}."
        );
    }
    let listed = format_roster_names(names);
    format!(
        "HOST: CONTESTED FREQUENCY. {map} TUNES IN. {listed} ON THE SCRAP. GOES LIVE IN {secs}."
    )
}

/// RoundStart Host line once Warmup ends (map + roster, fight energy, no countdown).
pub fn round_open_host_line(map_name: &str, names: &[String]) -> String {
    let map = map_name.to_uppercase();
    if names.is_empty() {
        return format!("HOST: CONTESTED FREQUENCY. {map} IS LIVE. FIGHT!");
    }
    let listed = format_roster_names(names);
    format!("HOST: CONTESTED FREQUENCY. {map}. {listed} ON THE SCRAP. FIGHT!")
}

/// Beat that triggers a Contested Frequency rule-bot speak line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotTauntKind {
    Frag,
    Death,
    Killstreak,
    Warmup,
}

/// Strip overflow refill suffix (`Dead Air Dan-2` -> `Dead Air Dan`).
pub fn rule_bot_callsign_base(name: &str) -> &str {
    if let Some((base, suffix)) = name.rsplit_once('-') {
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return base;
        }
    }
    name
}

/// Callsign-flavored Contested Frequency taunt for a named scrap rule bot.
pub fn rule_bot_taunt_line(name: &str, kind: BotTauntKind, salt: u64) -> String {
    let base = rule_bot_callsign_base(name);
    let pool: &[&str] = match (base, kind) {
        ("Dead Air Dan", BotTauntKind::Frag) => &["dead air cleared", "booth owns that frag"],
        ("Dead Air Dan", BotTauntKind::Death) => &["booth cut out", "dead air for a sec"],
        ("Dead Air Dan", BotTauntKind::Killstreak) => {
            &["frequency owns this lane", "host keep the mic open"]
        }
        ("Dead Air Dan", BotTauntKind::Warmup) => &["booth is hot", "dead air dan dials in"],
        ("Nightfall", BotTauntKind::Frag) => &["grid marked", "night watch confirms"],
        ("Nightfall", BotTauntKind::Death) => &["night watch resets", "rail resets"],
        ("Nightfall", BotTauntKind::Killstreak) => {
            &["rail holds the scrap", "approved lanes denied"]
        }
        ("Nightfall", BotTauntKind::Warmup) => &["night watch on station", "rail tuned"],
        ("Static Kid", BotTauntKind::Frag) => &["static kids push", "glitch landed"],
        ("Static Kid", BotTauntKind::Death) => &["glitch respawn", "static wiped"],
        ("Static Kid", BotTauntKind::Killstreak) => &["scatter sings", "frequency glitches hard"],
        ("Static Kid", BotTauntKind::Warmup) => &["tuning the glitch", "static kid on air"],
        ("Aunt Linda", BotTauntKind::Frag) => &["value for value", "frag for frag"],
        ("Aunt Linda", BotTauntKind::Death) => &["call back later", "hold the line"],
        ("Aunt Linda", BotTauntKind::Killstreak) => {
            &["league says keep dialing", "aunt linda stays live"]
        }
        ("Aunt Linda", BotTauntKind::Warmup) => &["aunt linda dials in", "live laugh frag"],
        ("Scout Ant", BotTauntKind::Frag) => &["scout ant deleted you", "antenna got you"],
        ("Scout Ant", BotTauntKind::Death) => &["not a metaphor", "scout ant wiped"],
        ("Scout Ant", BotTauntKind::Killstreak) => &["antenna up", "scout owns the scrap"],
        ("Scout Ant", BotTauntKind::Warmup) => &["scouting the scrap", "antenna warming"],
        ("Crackpot", BotTauntKind::Frag) => &["they deny this frag", "conspiracy confirmed"],
        ("Crackpot", BotTauntKind::Death) => &["continuance redacted me", "denied on air"],
        ("Crackpot", BotTauntKind::Killstreak) => &["crackpot was right", "they lose count"],
        ("Crackpot", BotTauntKind::Warmup) => &["crackpot on air", "redacted warmup"],
        ("Buzzkill", BotTauntKind::Frag) => &["buzzkill closes", "close scrap done"],
        ("Buzzkill", BotTauntKind::Death) => &["close scrap lost", "buzzkill reset"],
        ("Buzzkill", BotTauntKind::Killstreak) => &["approved lanes? nah", "buzzkill rampage"],
        ("Buzzkill", BotTauntKind::Warmup) => &["buzzkill warming", "scrap opens loud"],
        ("Tin Foil Tina", BotTauntKind::Frag) => &["not for public release", "eyes only wipe"],
        ("Tin Foil Tina", BotTauntKind::Death) => &["access restricted", "foil wiped"],
        ("Tin Foil Tina", BotTauntKind::Killstreak) => &["hangar candy lane", "denied watermark"],
        ("Tin Foil Tina", BotTauntKind::Warmup) => &["foil tuned in", "hangar candy warm"],
        (_, BotTauntKind::Frag) => &["nice scrap", "frequency contested"],
        (_, BotTauntKind::Death) => &["respawn the booth", "cut for now"],
        (_, BotTauntKind::Killstreak) => &["host is watching", "league stays live"],
        (_, BotTauntKind::Warmup) => &["tuning in", "frequency warming"],
    };
    let idx = (salt as usize) % pool.len();
    pool[idx].to_string()
}

/// Display name for the mid-round Continuance boss NPC (NODS-flavored Compliance Drone).
pub const BOSS_NAME: &str = "COMPLIANCE-DRONE";

/// Solo Broadcast Episode 0 id.
pub const EPISODE_ID_EP0: &str = "ep0";
/// Solo Broadcast Episode 0 title face.
pub const EPISODE_TITLE_EP0: &str = "Solo Broadcast: Calibration";
/// Larak Lot map face for Episode 0 (geometry may reuse map 1).
pub const EPISODE_MAP_LARAK_LOT: &str = "Larak Lot";
/// Continuance Auditor elite display name (Compliance Drone retitled).
pub const AUDITOR_NAME: &str = "AUDITOR";

pub fn episode0_host_line_cold_open() -> String {
    "HOST: In the morning. Calibration night. Continuance brought NODS. You're on the air."
        .to_string()
}

pub fn episode0_host_line_nods() -> String {
    "HOST: Null-Objective Drones don't trash-talk. That's how you know they're approved."
        .to_string()
}

pub fn episode0_host_line_jammer() -> String {
    "HOST: Jammer's up. Seize the dish or I go text-only. Value for value.".to_string()
}

pub fn episode0_host_line_auditor() -> String {
    "HOST: Auditor on the lot. Clipboard shield. Smile for the audit.".to_string()
}

pub fn episode0_host_line_win() -> String {
    "HOST: Amen, fistbump. Frequency still unmetered. Don't touch that dial.".to_string()
}

pub fn episode0_host_line_fail() -> String {
    "HOST: Citizen Handle assigned. Reload.".to_string()
}

pub fn episode0_objective_chip() -> String {
    "Clear NODS. Seize jammer dish. Drop the Auditor.".to_string()
}

pub fn episode0_unlock_teaser() -> String {
    "Callsign stub unlocked. Next: Area Kitchen.".to_string()
}

pub fn default_pickup_kind() -> String {
    "weapon".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeaponType {
    #[default]
    Flechette,
    Rail,
    Scatter,
}

impl WeaponType {
    pub fn damage(self) -> i32 {
        match self {
            WeaponType::Flechette => 25,
            WeaponType::Rail => 75,
            WeaponType::Scatter => 15,
        }
    }

    pub fn cooldown_ticks(self) -> u32 {
        match self {
            WeaponType::Flechette => 10,
            WeaponType::Rail => 40,
            WeaponType::Scatter => 5,
        }
    }

    pub fn spread_radians(self) -> f32 {
        match self {
            // Mid workhorse: readable cone.
            WeaponType::Flechette => 0.10,
            // Long precision: tighter than legacy 0.05.
            WeaponType::Rail => 0.04,
            // Close shred: wide cone, dies beyond range_units.
            WeaponType::Scatter => 0.38,
        }
    }

    /// Max hitscan reach in world units. Caps roles so Scatter is close-only
    /// and Rail owns long lanes (arena is ~50 across).
    pub fn range_units(self) -> f32 {
        match self {
            WeaponType::Flechette => 42.0,
            WeaponType::Rail => 100.0,
            WeaponType::Scatter => 14.0,
        }
    }

    /// Preferred bot engagement band (min, max) for role play.
    pub fn preferred_range(self) -> (f32, f32) {
        match self {
            WeaponType::Flechette => (8.0, 28.0),
            WeaponType::Rail => (18.0, 45.0),
            WeaponType::Scatter => (2.0, 10.0),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponType::Flechette => "Flechette",
            WeaponType::Rail => "Rail",
            WeaponType::Scatter => "Scatter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello {
        role: Role,
        name: String,
    },
    Action(Action),
    Speak(Speak),
    /// Agent-only display label echoed into Snapshot PlayerState.behavior.
    /// Never trusted for combat. Rule-bot behaviors still come from BotController.
    SetDisplayBehavior(SetDisplayBehavior),
}

#[allow(clippy::large_enum_variant)] // Snapshot carries round chrome; boxing churns every tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        player_id: Option<Uuid>,
        role: Role,
        #[serde(default = "default_mode_name")]
        mode_name: String,
        #[serde(default = "default_playlist")]
        playlist: String,
    },
    Snapshot(Snapshot),
    Event(GameEvent),
    /// Unicast control-plane rejection (e.g. speak rate limit). Not broadcast.
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Spectator,
    Human,
    Agent,
}

/// World-point or player-id aim target. Server applies yaw toward the target.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LookAt {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_id: Option<Uuid>,
}

/// Off-tick callout / taunt (control plane, not sticky Action).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Speak {
    pub text: String,
}

/// Observe-only stance / tactics chip for Agent clients (control plane).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SetDisplayBehavior {
    pub behavior: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(default)]
    pub forward: bool,
    #[serde(default)]
    pub back: bool,
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub turn_left: bool,
    #[serde(default)]
    pub turn_right: bool,
    #[serde(default)]
    pub fire: bool,
    #[serde(default)]
    pub weapon_swap: Option<WeaponType>,
    /// Authoritative aim: yaw snaps toward player_id (preferred) or world x/z.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look_at: Option<LookAt>,
}

/// Per-tick fire outcome for observe (hit-confirm without vision).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShotResult {
    pub shooter_id: Uuid,
    pub shooter: String,
    pub hit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub damage: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_hp_after: Option<i32>,
}

/// Floor pickup pad state (weapon / health / armor; authoritative mid-map).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PickupState {
    pub id: String,
    #[serde(default = "default_pickup_kind")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub weapon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respawn_in: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub players: Vec<PlayerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_time_left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frag_limit: Option<u32>,
    /// Shots resolved on this tick (empty omitted on wire).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shot_results: Vec<ShotResult>,
    /// Contested Frequency (scrap league that denies it exists).
    #[serde(default = "default_mode_name")]
    pub mode_name: String,
    /// Arena Duel under the league lie.
    #[serde(default = "default_playlist")]
    pub playlist: String,
    /// Live pressure beat id when Continuance is squeezing the round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure: Option<String>,
    /// Sticky Contested Frequency Host chrome (mid-join / observe).
    #[serde(default = "default_host_line")]
    pub host_line: String,
    /// Round MVP while Ended (mid-join / observe rehydrate). Omitted otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp: Option<String>,
    /// MVP frag count while Ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp_frags: Option<u32>,
    /// Mid-map pads: weapons, health, armor (Quake chase energy). Empty omitted on wire.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pickups: Vec<PickupState>,
    /// Contested Frequency scrap layout id (1 = Arena Duel, 2 = Compliance Yard).
    #[serde(default = "default_map_id")]
    pub map_id: u32,
    /// Human-readable scrap layout name.
    #[serde(default = "default_map_name")]
    pub map_name: String,
    /// Solo Broadcast episode id (e.g. ep0). Omitted on MP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<String>,
    /// Episode title card face.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    /// Short objective chip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_objective: Option<String>,
    /// Progress chip (NODS n/N | JAMMER | AUDITOR).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_progress: Option<String>,
    /// Phase: nods / jammer / auditor / won / failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub hp: i32,
    #[serde(default)]
    pub armor: i32,
    pub just_fired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<String>,
    pub score: u32,
    pub weapon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    pub name: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    Frag {
        killer: String,
        victim: String,
        killer_score: u32,
    },
    /// Non-lethal or pre-frag damage. Structured hit-confirm for agents.
    Hit {
        shooter: String,
        shooter_id: Uuid,
        target: String,
        target_id: Uuid,
        damage: i32,
        target_hp_after: i32,
    },
    Respawn {
        player: String,
    },
    RoundStart {
        round_number: u32,
        frag_limit: Option<u32>,
        time_limit: Option<u32>,
        players: Vec<String>,
        previous_winner: Option<String>,
        #[serde(default = "default_mode_name")]
        mode_name: String,
        #[serde(default = "default_playlist")]
        playlist: String,
        #[serde(default = "default_host_line")]
        host_line: String,
    },
    RoundEnd {
        winner: Option<String>,
        reason: String,
        final_scores: Vec<PlayerScore>,
        winner_score: Option<u32>,
        /// Round MVP (top score / frags). Same player as winner when scores exist.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mvp: Option<String>,
        /// MVP frag count (mirrors winner_score for agents / podium chrome).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mvp_frags: Option<u32>,
        /// Contested Frequency Host bumper for round-end podium.
        #[serde(default = "default_host_line")]
        host_line: String,
    },
    PlayerJoined {
        player: String,
        role: String,
        round_number: u32,
        player_count: usize,
    },
    PlayerLeft {
        player: String,
        score: u32,
        round_number: u32,
        player_count: usize,
    },
    /// Mid-round Continuance compliance pressure (Host bumper + move slow).
    CompliancePing {
        message: String,
        duration_ticks: u32,
    },
    /// Mid-round Continuance Compliance Drone spawn (killable boss beat).
    BossSpawn {
        name: String,
        boss_id: Uuid,
        message: String,
        hp: i32,
    },
    /// Compliance Drone fragged (no respawn).
    BossDown {
        name: String,
        boss_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        killer: Option<String>,
        message: String,
    },
    /// Mid-map pad claimed (weapon swap, heal, or armor scrap).
    Pickup {
        player: String,
        player_id: Uuid,
        #[serde(default = "default_pickup_kind")]
        kind: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        weapon: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amount: Option<i32>,
        pickup_id: String,
    },
    /// Within-round multi-kill Host callout (tiers 2 / 3 / 5).
    Killstreak {
        player: String,
        player_id: Uuid,
        streak: u32,
        tier: String,
        message: String,
    },
    /// Off-tick agent/human callout (rate-limited, length-capped).
    Speak {
        player: String,
        player_id: Uuid,
        text: String,
    },
    /// Solo Broadcast episode cold open / title card.
    EpisodeStart {
        id: String,
        title: String,
        objective: String,
        host_line: String,
        map_name: String,
    },
    /// Solo Broadcast episode win.
    EpisodeComplete {
        id: String,
        reason: String,
        host_line: String,
        unlock_teaser: String,
    },
    /// Solo Broadcast episode fail (Citizen Handle assigned).
    EpisodeFail {
        id: String,
        reason: String,
        host_line: String,
    },
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn unknown_action_field_fails_deserialize() {
        let json = r#"{"type":"action","forward":true,"laser":true}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "unknown Action field must fail: {:?}",
            parsed
        );
    }

    #[test]
    fn valid_action_deserializes() {
        let json = r#"{"type":"action","forward":true,"fire":true}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(parsed.is_ok(), "{:?}", parsed);
        match parsed.unwrap() {
            ClientMessage::Action(a) => {
                assert!(a.forward);
                assert!(a.fire);
                assert!(!a.back);
                assert!(a.look_at.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn look_at_player_id_deserializes() {
        let id = Uuid::new_v4();
        let json = format!(r#"{{"type":"action","look_at":{{"player_id":"{}"}}}}"#, id);
        let parsed: ClientMessage = serde_json::from_str(&json).expect("look_at action");
        match parsed {
            ClientMessage::Action(a) => {
                let look = a.look_at.expect("look_at present");
                assert_eq!(look.player_id, Some(id));
                assert!(look.x.is_none());
                assert!(look.z.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn look_at_world_xz_deserializes() {
        let json = r#"{"type":"action","look_at":{"x":10.0,"z":-5.0}}"#;
        let parsed: ClientMessage = serde_json::from_str(json).expect("look_at xz");
        match parsed {
            ClientMessage::Action(a) => {
                let look = a.look_at.expect("look_at present");
                assert_eq!(look.x, Some(10.0));
                assert_eq!(look.z, Some(-5.0));
                assert!(look.player_id.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn unknown_look_at_field_fails_deserialize() {
        let json = r#"{"type":"action","look_at":{"x":1.0,"z":2.0,"laser":true}}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "unknown LookAt field must fail: {:?}",
            parsed
        );
    }

    #[test]
    fn shot_result_and_hit_event_round_trip() {
        let shooter_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let shot = ShotResult {
            shooter_id,
            shooter: "A".into(),
            hit: true,
            target_id: Some(target_id),
            target: Some("B".into()),
            damage: 25,
            target_hp_after: Some(75),
        };
        let snap = Snapshot {
            tick: 1,
            players: vec![],
            round_state: None,
            round_time_left: None,
            frag_limit: None,
            shot_results: vec![shot.clone()],
            mode_name: default_mode_name(),
            playlist: default_playlist(),
            pressure: None,
            host_line: default_host_line(),
            mvp: None,
            mvp_frags: None,
            pickups: vec![],
            map_id: default_map_id(),
            map_name: default_map_name(),
            episode_id: None,
            episode_title: None,
            episode_objective: None,
            episode_progress: None,
            episode_phase: None,
        };
        let v = serde_json::to_value(&snap).unwrap();
        assert_eq!(v["shot_results"][0]["hit"], true);
        assert_eq!(v["shot_results"][0]["damage"], 25);
        let back: Snapshot = serde_json::from_value(v).unwrap();
        assert_eq!(back.shot_results, vec![shot]);

        let hit = GameEvent::Hit {
            shooter: "A".into(),
            shooter_id,
            target: "B".into(),
            target_id,
            damage: 25,
            target_hp_after: 75,
        };
        let ev = serde_json::to_value(&hit).unwrap();
        assert_eq!(ev["event"], "hit");
        assert_eq!(ev["target_hp_after"], 75);
        let back: GameEvent = serde_json::from_value(ev).unwrap();
        match back {
            GameEvent::Hit {
                damage,
                target_hp_after,
                ..
            } => {
                assert_eq!(damage, 25);
                assert_eq!(target_hp_after, 75);
            }
            other => panic!("expected Hit, got {:?}", other),
        }
    }

    #[test]
    fn speak_deserializes_and_deny_unknown() {
        let ok: ClientMessage =
            serde_json::from_str(r#"{"type":"speak","text":"nice scrap"}"#).expect("speak");
        match ok {
            ClientMessage::Speak(s) => assert_eq!(s.text, "nice scrap"),
            other => panic!("expected Speak, got {:?}", other),
        }
        let bad: Result<ClientMessage, _> =
            serde_json::from_str(r#"{"type":"speak","text":"x","laser":true}"#);
        assert!(bad.is_err(), "unknown Speak field must fail: {:?}", bad);
    }

    #[test]
    fn set_display_behavior_deserializes_and_deny_unknown() {
        let ok: ClientMessage =
            serde_json::from_str(r#"{"type":"set_display_behavior","behavior":"push_enemy"}"#)
                .expect("set_display_behavior");
        match ok {
            ClientMessage::SetDisplayBehavior(s) => assert_eq!(s.behavior, "push_enemy"),
            other => panic!("expected SetDisplayBehavior, got {:?}", other),
        }
        let bad: Result<ClientMessage, _> =
            serde_json::from_str(r#"{"type":"set_display_behavior","behavior":"x","laser":true}"#);
        assert!(
            bad.is_err(),
            "unknown SetDisplayBehavior field must fail: {:?}",
            bad
        );
    }

    #[test]
    fn speak_event_round_trip() {
        let id = Uuid::new_v4();
        let ev = GameEvent::Speak {
            player: "ArenaFox".into(),
            player_id: id,
            text: "frequency contested".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["event"], "speak");
        assert_eq!(v["text"], "frequency contested");
        assert_eq!(v["player"], "ArenaFox");
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::Speak { text, player, .. } => {
                assert_eq!(text, "frequency contested");
                assert_eq!(player, "ArenaFox");
            }
            other => panic!("expected Speak, got {:?}", other),
        }
    }

    #[test]
    fn boss_spawn_and_down_wire_json_shape() {
        let id = Uuid::new_v4();
        let spawn = GameEvent::BossSpawn {
            name: BOSS_NAME.into(),
            boss_id: id,
            message: boss_host_line(),
            hp: 200,
        };
        let v = serde_json::to_value(&spawn).unwrap();
        assert_eq!(v["event"], "boss_spawn");
        assert_eq!(v["name"], BOSS_NAME);
        assert_eq!(v["hp"], 200);
        assert_eq!(v["message"], boss_host_line());
        let back: GameEvent = serde_json::from_value(v).unwrap();
        assert!(matches!(back, GameEvent::BossSpawn { hp: 200, .. }));

        let down = GameEvent::BossDown {
            name: BOSS_NAME.into(),
            boss_id: id,
            killer: Some("Rusher".into()),
            message: boss_down_host_line(),
        };
        let v = serde_json::to_value(&down).unwrap();
        assert_eq!(v["event"], "boss_down");
        assert_eq!(v["killer"], "Rusher");
        assert!(v.get("message").is_some());
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::BossDown { killer, .. } => assert_eq!(killer.as_deref(), Some("Rusher")),
            other => panic!("expected BossDown, got {:?}", other),
        }

        let wipe = GameEvent::BossDown {
            name: BOSS_NAME.into(),
            boss_id: id,
            killer: None,
            message: boss_round_wipe_host_line(),
        };
        let v = serde_json::to_value(&wipe).unwrap();
        assert_eq!(v["event"], "boss_down");
        assert!(v.get("killer").is_none(), "wipe must omit killer: {}", v);
        assert_eq!(v["message"], boss_round_wipe_host_line());
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::BossDown { killer: None, .. } => {}
            other => panic!("expected BossDown wipe, got {:?}", other),
        }
    }

    #[test]
    fn pickup_state_and_event_wire_json_shape() {
        let pad = PickupState {
            id: "pad_rail".into(),
            kind: "weapon".into(),
            weapon: "Rail".into(),
            amount: None,
            x: 12.0,
            y: 0.4,
            z: 12.0,
            available: false,
            respawn_in: Some(80),
        };
        let snap = Snapshot {
            tick: 2,
            players: vec![],
            round_state: None,
            round_time_left: None,
            frag_limit: None,
            shot_results: vec![],
            mode_name: default_mode_name(),
            playlist: default_playlist(),
            pressure: None,
            host_line: default_host_line(),
            mvp: None,
            mvp_frags: None,
            pickups: vec![pad.clone()],
            map_id: default_map_id(),
            map_name: default_map_name(),
            episode_id: None,
            episode_title: None,
            episode_objective: None,
            episode_progress: None,
            episode_phase: None,
        };
        let v = serde_json::to_value(&snap).unwrap();
        assert_eq!(v["pickups"][0]["id"], "pad_rail");
        assert_eq!(v["pickups"][0]["weapon"], "Rail");
        assert_eq!(v["pickups"][0]["available"], false);
        assert_eq!(v["pickups"][0]["respawn_in"], 80);
        let back: Snapshot = serde_json::from_value(v).unwrap();
        assert_eq!(back.pickups, vec![pad]);

        let id = Uuid::new_v4();
        let ev = GameEvent::Pickup {
            player: "Rusher".into(),
            player_id: id,
            kind: "weapon".into(),
            weapon: "Rail".into(),
            amount: None,
            pickup_id: "pad_rail".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["event"], "pickup");
        assert_eq!(v["kind"], "weapon");
        assert_eq!(v["weapon"], "Rail");
        assert_eq!(v["pickup_id"], "pad_rail");
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::Pickup {
                kind,
                weapon,
                pickup_id,
                ..
            } => {
                assert_eq!(kind, "weapon");
                assert_eq!(weapon, "Rail");
                assert_eq!(pickup_id, "pad_rail");
            }
            other => panic!("expected Pickup, got {:?}", other),
        }

        let heal = GameEvent::Pickup {
            player: "Rusher".into(),
            player_id: id,
            kind: "health".into(),
            weapon: String::new(),
            amount: Some(40),
            pickup_id: "pad_health_n".into(),
        };
        let v = serde_json::to_value(&heal).unwrap();
        assert_eq!(v["kind"], "health");
        assert_eq!(v["amount"], 40);
        assert!(v.get("weapon").is_none());
    }
    #[test]
    fn killstreak_event_wire_json_shape() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let ev = GameEvent::Killstreak {
            player: "Rusher".into(),
            player_id: id,
            streak: 2,
            tier: "double".into(),
            message: "HOST: DOUBLE FREQUENCY. Rusher DENIES THE DENIAL.".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["event"], "killstreak");
        assert_eq!(v["player"], "Rusher");
        assert_eq!(v["streak"], 2);
        assert_eq!(v["tier"], "double");
        assert!(v["message"].as_str().unwrap().contains("DOUBLE FREQUENCY"));
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::Killstreak { streak, tier, .. } => {
                assert_eq!(streak, 2);
                assert_eq!(tier, "double");
            }
            other => panic!("expected Killstreak, got {:?}", other),
        }
    }

    #[test]
    fn killstreak_host_line_tiers() {
        assert!(killstreak_host_line(1, "X").is_none());
        assert!(killstreak_host_line(4, "X").is_none());
        let (tier, msg) = killstreak_host_line(2, "Rusher").unwrap();
        assert_eq!(tier, "double");
        assert!(msg.contains("Rusher"));
        let (tier, msg) = killstreak_host_line(3, "Rusher").unwrap();
        assert_eq!(tier, "triple");
        assert!(msg.contains("TRIPLE"));
        let (tier, msg) = killstreak_host_line(5, "Rusher").unwrap();
        assert_eq!(tier, "rampage");
        assert!(msg.contains("RAMPAGE"));
    }

    #[test]
    fn mvp_host_line_voice() {
        let line = mvp_host_line("Rusher", 10);
        assert!(line.contains("ROUND MVP"));
        assert!(line.contains("Rusher"));
        assert!(line.contains("10 FRAGS"));
        assert!(line.contains("CONTINUANCE DENIES THE PODIUM"));
        assert!(empty_mvp_host_line().contains("NO MVP"));
    }

    #[test]
    fn scrap_bot_host_lines() {
        let intro = bot_intro_host_line("Dead Air Dan");
        assert!(intro.contains("DEAD AIR DAN"));
        assert!(intro.contains("DIALS THE FREQUENCY"));
        let roster = roster_host_line(&[
            "Dead Air Dan".into(),
            "Nightfall".into(),
            "Static Kid".into(),
            "Aunt Linda".into(),
        ]);
        assert!(roster.contains("DEAD AIR DAN"));
        assert!(roster.contains("ON THE SCRAP"));
        assert_eq!(roster_host_line(&[]), default_host_line());
        let many = roster_host_line(&["A".into(), "B".into(), "C".into(), "D".into(), "E".into()]);
        assert!(many.contains("+2"));
    }

    #[test]
    fn warmup_and_round_open_host_lines() {
        let names = vec!["Dead Air Dan".into(), "Nightfall".into()];
        let warm = warmup_host_line("Arena Duel", &names, 2);
        assert!(warm.contains("CONTESTED FREQUENCY"));
        assert!(warm.contains("ARENA DUEL"));
        assert!(warm.contains("DEAD AIR DAN"));
        assert!(warm.contains("ON THE SCRAP"));
        assert!(warm.contains("GOES LIVE IN 2"));
        let empty = warmup_host_line("Compliance Yard", &[], 3);
        assert!(empty.contains("COMPLIANCE YARD"));
        assert!(empty.contains("GOES LIVE IN 3"));
        let open = round_open_host_line("Arena Duel", &names);
        assert!(open.contains("FIGHT!"));
        assert!(open.contains("ARENA DUEL"));
        assert!(open.contains("ON THE SCRAP"));
        let open_empty = round_open_host_line("Compliance Yard", &[]);
        assert!(open_empty.contains("IS LIVE"));
        assert!(open_empty.contains("FIGHT!"));
    }

    #[test]
    fn rule_bot_taunt_lines_are_callsign_flavored() {
        let frag = rule_bot_taunt_line("Dead Air Dan", BotTauntKind::Frag, 0);
        assert!(
            frag.contains("dead air") || frag.contains("booth"),
            "{frag}"
        );
        let death = rule_bot_taunt_line("Nightfall-2", BotTauntKind::Death, 0);
        assert!(
            death.contains("night watch") || death.contains("rail"),
            "{death}"
        );
        let warm = rule_bot_taunt_line("Static Kid", BotTauntKind::Warmup, 0);
        assert!(warm.contains("glitch") || warm.contains("static"), "{warm}");
        let streak = rule_bot_taunt_line("Buzzkill", BotTauntKind::Killstreak, 0);
        assert!(!streak.is_empty());
        assert!(streak.chars().count() <= 80);
        assert_eq!(rule_bot_callsign_base("Tin Foil Tina-3"), "Tin Foil Tina");
        let generic = rule_bot_taunt_line("Scrap Fox", BotTauntKind::Frag, 1);
        assert!(
            generic == "nice scrap" || generic == "frequency contested",
            "{generic}"
        );
    }

    #[test]
    fn round_end_mvp_wire_round_trip() {
        let event = GameEvent::RoundEnd {
            winner: Some("Rusher".to_string()),
            reason: "Frag limit reached".to_string(),
            final_scores: vec![PlayerScore {
                name: "Rusher".to_string(),
                score: 10,
            }],
            winner_score: Some(10),
            mvp: Some("Rusher".to_string()),
            mvp_frags: Some(10),
            host_line: mvp_host_line("Rusher", 10),
        };
        let v = serde_json::to_value(&event).unwrap();
        assert_eq!(v["event"], "round_end");
        assert_eq!(v["mvp"], "Rusher");
        assert_eq!(v["mvp_frags"], 10);
        assert!(v["host_line"].as_str().unwrap().contains("ROUND MVP"));
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::RoundEnd {
                mvp,
                mvp_frags,
                host_line,
                ..
            } => {
                assert_eq!(mvp.as_deref(), Some("Rusher"));
                assert_eq!(mvp_frags, Some(10));
                assert!(host_line.contains("ROUND MVP"));
            }
            other => panic!("expected RoundEnd, got {:?}", other),
        }
        // Legacy wire without mvp fields still deserializes.
        let legacy = serde_json::json!({
            "event": "round_end",
            "winner": "Bot1",
            "reason": "Time limit reached",
            "final_scores": [],
            "winner_score": null
        });
        let legacy_ev: GameEvent = serde_json::from_value(legacy).unwrap();
        match legacy_ev {
            GameEvent::RoundEnd { mvp, host_line, .. } => {
                assert!(mvp.is_none());
                assert_eq!(host_line, default_host_line());
            }
            other => panic!("expected RoundEnd, got {:?}", other),
        }
    }
}
