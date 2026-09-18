extends Node
## Contested Frequency radio: station list from stations.json, tracks from the
## audiogen manifest, shuffle without repeats, ducking under Host lines and combat.
## Presentation only. Missing assets are fine: the radio stays silent.

signal track_started(station_name: String, title: String)
signal station_changed(station_name: String, tagline: String, has_tracks: bool)
## Full card for the HUD: name, badge, color, tagline, enabled, has_tracks.
signal station_card(card: Dictionary)

const STATIONS_PATH := "res://assets/audio/radio/stations.json"
const MANIFEST_PATH := "res://assets/audio/audiogen-manifest.json"
const AUDIO_ROOT := "res://assets/audio/"
const RADIO_PREFIX := "radio/"
const NO_REPEAT_WINDOW := 12
const SPECTATE_DB := -6.0
const PLAYING_DB := -12.0
const DUCK_DB := -9.0
const MUTE_DB := -80.0
const DUCK_SECONDS := 3.0
const DUCK_RECOVER_SECONDS := 1.5
const DEFAULT_BADGE_COLOR := "#5A554F"

var stations: Array = []
var station_index := 0
var enabled := true
var human_mode := false
var duck_timer := 0.0
var history: Dictionary = {}
var player: AudioStreamPlayer
var rng := RandomNumberGenerator.new()
var current_title := ""


func _ready() -> void:
	rng.randomize()
	player = AudioStreamPlayer.new()
	player.name = "Player"
	player.bus = "Master"
	add_child(player)
	player.finished.connect(_on_track_finished)
	load_catalog()
	if stations.is_empty():
		return
	_apply_volume(0.0, true)
	play_random()
	# Signals connected by the parent after add_child would miss an emit here.
	call_deferred("announce_station")


## Build the station list. Texts can be injected for headless tests.
func load_catalog(stations_text: String = "", manifest_text: String = "") -> void:
	var stations_data: Variant = _parse_json(stations_text if stations_text != "" else _read_text(STATIONS_PATH))
	var manifest_data: Variant = _parse_json(manifest_text if manifest_text != "" else _read_text(MANIFEST_PATH))
	var station_list: Array = stations_data.get("stations", []) if stations_data is Dictionary else []
	var entries: Dictionary = manifest_data.get("entries", {}) if manifest_data is Dictionary else {}
	stations = build_stations(station_list, entries)
	station_index = clampi(station_index, 0, maxi(stations.size() - 1, 0))


## Group manifest entries by station folder. Pure so tests can call it directly.
static func build_stations(station_list: Array, entries: Dictionary) -> Array:
	var result: Array = []
	for raw in station_list:
		if not (raw is Dictionary) or not raw.has("id"):
			continue
		var id := str(raw["id"])
		var prefix := RADIO_PREFIX + id + "/"
		var tracks: Array = []
		var names := entries.keys()
		names.sort()
		for name in names:
			var name_str := str(name)
			if not name_str.begins_with(prefix):
				continue
			var entry: Dictionary = entries[name]
			var file := str(entry.get("file", ""))
			if file == "":
				continue
			var title := str(entry.get("title", ""))
			if title == "":
				title = name_str.substr(prefix.length())
			tracks.append({
				"name": name_str,
				"path": AUDIO_ROOT + file,
				"title": title,
				"length_ms": int(entry.get("length_ms", 0)),
			})
		var badge := str(raw.get("badge", ""))
		if badge == "":
			badge = id.substr(0, 2).to_upper()
		result.append({
			"id": id,
			"name": str(raw.get("name", id)),
			"dj": str(raw.get("dj", "")),
			"badge": badge,
			"color": str(raw.get("color", DEFAULT_BADGE_COLOR)),
			"tagline": str(raw.get("tagline", "")),
			"ducks_in_combat": bool(raw.get("ducks_in_combat", true)),
			"tracks": tracks,
		})
	return result


func current_station() -> Dictionary:
	if stations.is_empty():
		return {}
	return stations[station_index]


## What the HUD shows on a station switch or toggle.
func station_card_for(station: Dictionary) -> Dictionary:
	return {
		"id": str(station.get("id", "")),
		"name": str(station.get("name", "")),
		"badge": str(station.get("badge", "")),
		"color": str(station.get("color", DEFAULT_BADGE_COLOR)),
		"tagline": str(station.get("tagline", "")),
		"enabled": enabled,
		"has_tracks": not station.get("tracks", []).is_empty(),
	}


func announce_station() -> void:
	var station := current_station()
	if station.is_empty():
		return
	station_card.emit(station_card_for(station))


## Pick a track not heard in the last NO_REPEAT_WINDOW plays for this station.
func pick_next(station: Dictionary) -> Dictionary:
	var tracks: Array = station.get("tracks", [])
	if tracks.is_empty():
		return {}
	var id := str(station.get("id", ""))
	var recent: Array = history.get(id, [])
	var window := mini(NO_REPEAT_WINDOW, tracks.size() - 1)
	var candidates: Array = []
	for track in tracks:
		if not recent.has(track["name"]):
			candidates.append(track)
	if candidates.is_empty():
		candidates = tracks
	var chosen: Dictionary = candidates[rng.randi_range(0, candidates.size() - 1)]
	recent.append(chosen["name"])
	while recent.size() > window:
		recent.pop_front()
	history[id] = recent
	return chosen


func play_random() -> void:
	var station := current_station()
	var track := pick_next(station)
	if track.is_empty():
		# Off the air: the station card already says there are no tracks.
		current_title = ""
		if player:
			player.stop()
		return
	var stream = load(str(track["path"])) if ResourceLoader.exists(str(track["path"])) else null
	if stream == null:
		current_title = ""
		return
	if stream is AudioStreamMP3:
		stream.loop = false
	player.stream = stream
	player.play()
	current_title = str(track["title"])
	track_started.emit(str(station.get("name", "")), current_title)


func next_station() -> void:
	_step_station(1)


func prev_station() -> void:
	_step_station(-1)


func _step_station(delta: int) -> void:
	if stations.is_empty():
		return
	station_index = posmod(station_index + delta, stations.size())
	var station := current_station()
	station_changed.emit(str(station.get("name", "")), str(station.get("tagline", "")), not station.get("tracks", []).is_empty())
	announce_station()
	if enabled:
		play_random()


func next_track() -> void:
	if enabled and not stations.is_empty():
		play_random()


func toggle() -> void:
	enabled = not enabled
	if enabled:
		play_random()
	elif player:
		player.stop()
		current_title = ""
	var station := current_station()
	station_changed.emit(str(station.get("name", "")), "on" if enabled else "off", enabled)
	announce_station()


func set_human_mode(on: bool) -> void:
	human_mode = on


## Lower the music for a moment while the Host talks.
func duck(seconds: float = DUCK_SECONDS) -> void:
	duck_timer = maxf(duck_timer, seconds)


## Volume for the current state. Pure so tests can check the table.
static func target_db(is_human: bool, is_ducked: bool, ducks_in_combat: bool) -> float:
	var db := PLAYING_DB if is_human else SPECTATE_DB
	if is_ducked and (ducks_in_combat or not is_human):
		db += DUCK_DB
	return db


func _process(delta: float) -> void:
	if duck_timer > 0.0:
		duck_timer = maxf(duck_timer - delta, 0.0)
	_apply_volume(delta, false)


func _apply_volume(delta: float, immediate: bool) -> void:
	if player == null:
		return
	if not enabled:
		player.volume_db = MUTE_DB
		return
	var station := current_station()
	var target := target_db(human_mode, duck_timer > 0.0, bool(station.get("ducks_in_combat", true)))
	if immediate:
		player.volume_db = target
		return
	var rate := 60.0 if target < player.volume_db else (DUCK_DB * -1.0) / DUCK_RECOVER_SECONDS
	player.volume_db = move_toward(player.volume_db, target, rate * delta)


func _unhandled_input(event: InputEvent) -> void:
	if InputMap.has_action("radio_next_station") and event.is_action_pressed("radio_next_station"):
		next_station()
	elif InputMap.has_action("radio_next_track") and event.is_action_pressed("radio_next_track"):
		next_track()
	elif InputMap.has_action("radio_toggle") and event.is_action_pressed("radio_toggle"):
		toggle()


func _on_track_finished() -> void:
	if enabled:
		play_random()


func _read_text(path: String) -> String:
	if not FileAccess.file_exists(path):
		return ""
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return ""
	return file.get_as_text()


func _parse_json(text: String) -> Variant:
	if text.strip_edges() == "":
		return {}
	var parsed = JSON.parse_string(text)
	return parsed if parsed != null else {}
