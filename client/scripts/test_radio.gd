extends SceneTree
## Headless check for the radio: station grouping, shuffle without repeats,
## station cycling, and the volume state table. Run with:
## godot --headless --path client --script res://scripts/test_radio.gd

const RadioScript := preload("res://scripts/radio.gd")

var failures: Array = []


func _check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)


func _initialize() -> void:
	_test_build_stations()
	_test_pick_next_no_repeat()
	_test_station_cycle_and_toggle()
	_test_volume_table()
	if failures.is_empty():
		print("test_radio: PASS")
		quit(0)
	else:
		for failure in failures:
			printerr("test_radio: FAIL " + failure)
		quit(1)


func _fake_entries(station_id: String, count: int) -> Dictionary:
	var entries := {}
	for i in range(count):
		var name := "radio/%s/%02d-track" % [station_id, i + 1]
		entries[name] = {
			"kind": "music",
			"file": name + ".mp3",
			"title": "Track %d" % (i + 1),
			"length_ms": 120000,
		}
	return entries


func _test_build_stations() -> void:
	var station_list := [
		{"id": "rock", "name": "Larak Lot Rock", "tagline": "loud", "ducks_in_combat": true},
		{"id": "lockin", "name": "LOCK IN", "ducks_in_combat": false},
		{"name": "no id, must be skipped"},
	]
	var entries := _fake_entries("rock", 3)
	entries["fire_rail"] = {"kind": "sfx", "file": "fire_rail.wav"}
	entries["radio/rock/untitled"] = {"kind": "music", "file": "radio/rock/untitled.mp3"}
	var stations: Array = RadioScript.build_stations(station_list, entries)
	_check(stations.size() == 2, "build_stations keeps only stations with an id, got %d" % stations.size())
	if stations.size() < 2:
		return
	_check(stations[0]["tracks"].size() == 4, "rock should have 4 tracks, got %d" % stations[0]["tracks"].size())
	_check(stations[0]["tracks"][0]["path"] == "res://assets/audio/radio/rock/01-track.mp3", "track path uses the audio root")
	_check(stations[0]["tracks"][0]["title"] == "Track 1", "title comes from the manifest")
	_check(stations[0]["tracks"][3]["title"] == "untitled", "missing title falls back to the file stem")
	_check(stations[1]["tracks"].is_empty(), "lockin has no tracks in this manifest")
	_check(stations[1]["ducks_in_combat"] == false, "ducks_in_combat is read from stations.json")
	_check(stations[0]["ducks_in_combat"] == true, "default ducks_in_combat is true")


func _test_pick_next_no_repeat() -> void:
	var radio = RadioScript.new()
	radio.rng.seed = 7
	var station := {"id": "edm", "tracks": []}
	for i in range(20):
		station["tracks"].append({"name": "radio/edm/%02d" % i, "path": "", "title": "t%d" % i, "length_ms": 0})
	var seen: Array = []
	for i in range(radio.NO_REPEAT_WINDOW):
		var track = radio.pick_next(station)
		_check(not seen.has(track["name"]), "repeat inside the no-repeat window at pick %d" % i)
		seen.append(track["name"])
	_check(radio.history["edm"].size() == radio.NO_REPEAT_WINDOW, "history is capped at the window")
	var tiny := {"id": "one", "tracks": [{"name": "radio/one/01", "path": "", "title": "solo", "length_ms": 0}]}
	_check(radio.pick_next(tiny)["name"] == "radio/one/01", "single-track station still plays")
	_check(radio.pick_next(tiny)["name"] == "radio/one/01", "single-track station repeats when it must")
	_check(radio.pick_next({"id": "empty", "tracks": []}).is_empty(), "empty station returns nothing")
	radio.free()


func _test_station_cycle_and_toggle() -> void:
	var radio = RadioScript.new()
	radio.stations = RadioScript.build_stations(
		[{"id": "a", "name": "A"}, {"id": "b", "name": "B"}, {"id": "c", "name": "C"}],
		{}
	)
	var seen_names: Array = []
	radio.station_changed.connect(func(name, _tagline, _has): seen_names.append(name))
	radio.next_station()
	radio.next_station()
	radio.next_station()
	_check(radio.station_index == 0, "next_station wraps around, got %d" % radio.station_index)
	radio.prev_station()
	_check(radio.station_index == 2, "prev_station wraps backwards, got %d" % radio.station_index)
	_check(seen_names == ["B", "C", "A", "C"], "station_changed emits in order, got %s" % str(seen_names))
	radio.toggle()
	_check(radio.enabled == false, "toggle turns the radio off")
	radio.toggle()
	_check(radio.enabled == true, "toggle turns the radio back on")
	radio.free()


func _test_volume_table() -> void:
	_check(is_equal_approx(RadioScript.target_db(false, false, true), -6.0), "spectating sits at -6 dB")
	_check(is_equal_approx(RadioScript.target_db(true, false, true), -12.0), "playing sits at -12 dB")
	_check(is_equal_approx(RadioScript.target_db(false, true, true), -15.0), "spectating plus duck is -15 dB")
	_check(is_equal_approx(RadioScript.target_db(true, true, true), -21.0), "playing plus duck is -21 dB")
	_check(is_equal_approx(RadioScript.target_db(true, true, false), -12.0), "LOCK IN does not duck while playing")
	_check(is_equal_approx(RadioScript.target_db(false, true, false), -15.0), "LOCK IN still ducks under the Host while spectating")
