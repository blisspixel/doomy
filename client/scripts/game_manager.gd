extends Node

@onready var net_client = $NetClient
@onready var hud = $HUD
@onready var arena = $Arena
@onready var camera = $SpectatorCamera
@onready var frag_sound = $AudioPlayers/FragSound
@onready var round_start_sound = $AudioPlayers/RoundStartSound
@onready var round_end_sound = $AudioPlayers/RoundEndSound

var players = {}
var pickups = {}
var pickup_scene = preload("res://scenes/weapon_pickup.tscn")
var player_scene = preload("res://scenes/player.tscn")

var is_human_player = false
var local_fp_pawn_id = ""
var local_hp_seen = -1
var fp_spawn_flashed = false
var action_state = {
	"forward": false,
	"back": false,
	"left": false,
	"right": false,
	"turn_left": false,
	"turn_right": false,
	"fire": false
}

func _ready():
	net_client.snapshot_received.connect(_on_snapshot_received)
	net_client.event_received.connect(_on_event_received)
	net_client.connected_to_server.connect(_on_connected)
	net_client.disconnected_from_server.connect(_on_disconnected)
	
	_load_audio_streams()
	
	var boot = _resolve_boot()
	var role = str(boot.get("role", "spectator"))
	var player_name = str(boot.get("name", "Spectator"))
	is_human_player = role == "human"
	
	if boot.has("host") and str(boot["host"]) != "":
		net_client.set_server_host(str(boot["host"]))
	
	net_client.connect_to_server(role, player_name)
	hud.set_mode(str(boot.get("hud_mode", "SPECTATING")))

func _resolve_boot() -> Dictionary:
	# Boot menu meta wins; then --solo / FRAGR_SOLO; then --human; else spectator.
	if get_tree().has_meta("fragr_boot"):
		var meta = get_tree().get_meta("fragr_boot")
		if typeof(meta) == TYPE_DICTIONARY:
			var mode = str(meta.get("mode", "spectate"))
			var host = str(meta.get("host", "127.0.0.1:6767"))
			if mode == "solo":
				return {"role": "human", "name": "Human Player", "host": host, "hud_mode": "SOLO SCRAP"}
			if mode == "join":
				return {"role": "human", "name": "Human Player", "host": host, "hud_mode": "PLAYING"}
			return {"role": "spectator", "name": "Spectator", "host": host, "hud_mode": "SPECTATING"}
	
	var args = OS.get_cmdline_args()
	var user_args = OS.get_cmdline_user_args()
	var wants_solo = OS.get_environment("FRAGR_SOLO") == "1" or "--solo" in args or "--solo" in user_args
	if wants_solo:
		return {"role": "human", "name": "Human Player", "host": "127.0.0.1:6767", "hud_mode": "SOLO SCRAP"}
	if "--human" in args or "--human" in user_args:
		return {"role": "human", "name": "Human Player", "host": "", "hud_mode": "PLAYING"}
	return {"role": "spectator", "name": "Spectator", "host": "", "hud_mode": "SPECTATING"}

func _load_audio_streams():
	var audio_dir = "res://assets/audio/"
	
	if frag_sound and ResourceLoader.exists(audio_dir + "frag.wav"):
		frag_sound.stream = load(audio_dir + "frag.wav")
	
	if round_start_sound and ResourceLoader.exists(audio_dir + "round_start.wav"):
		round_start_sound.stream = load(audio_dir + "round_start.wav")
	
	if round_end_sound and ResourceLoader.exists(audio_dir + "round_end.wav"):
		round_end_sound.stream = load(audio_dir + "round_end.wav")

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_J and not is_human_player:
			print("Joining as human player...")
			net_client.disconnect_from_server()
			await get_tree().create_timer(0.5).timeout
			is_human_player = true
			fp_spawn_flashed = false
			net_client.connect_to_server("human", "Human Player")
			hud.set_mode("PLAYING")
			_pick_ghost_rival_from_alive()
		elif event.keycode == KEY_L and is_human_player:
			print("Leaving match, returning to spectator...")
			net_client.disconnect_from_server()
			await get_tree().create_timer(0.5).timeout
			is_human_player = false
			_clear_fp_state()
			net_client.connect_to_server("spectator", "Spectator")
			hud.set_ghost_rival("")
			hud.set_mode("SPECTATING")

func _process(_delta):
	if is_human_player and net_client.connection_state == WebSocketPeer.STATE_OPEN:
		action_state.forward = Input.is_action_pressed("move_forward")
		action_state.back = Input.is_action_pressed("move_back")
		action_state.left = Input.is_action_pressed("move_left")
		action_state.right = Input.is_action_pressed("move_right")
		action_state.fire = Input.is_action_pressed("fire")
		var turns = {"turn_left": false, "turn_right": false}
		if camera and camera.has_method("consume_turn_bits"):
			turns = camera.consume_turn_bits()
		action_state.turn_left = turns.get("turn_left", false)
		action_state.turn_right = turns.get("turn_right", false)
		net_client.send_action(action_state)

func _on_connected():
	hud.set_status("Connected to server")

func _on_disconnected():
	hud.set_status("Disconnected")
	hud.reset_host_chrome()
	_clear_world()

func _clear_world() -> void:
	_clear_fp_state()
	# Drop presentation nodes so rejoin does not keep stale pawns/pads.
	for id in players.keys():
		if is_instance_valid(players[id]):
			players[id].queue_free()
	players.clear()
	for pid in pickups.keys():
		if is_instance_valid(pickups[pid]):
			pickups[pid].queue_free()
	pickups.clear()
	if camera:
		camera.set_available_targets([])

func _on_snapshot_received(data):
	var tick = data.get("tick", 0)
	var player_list = data.get("players", [])
	var round_state = data.get("round_state", "")
	var round_time_left = data.get("round_time_left", 0)
	var frag_limit = data.get("frag_limit", 0)
	var mode_name = str(data.get("mode_name", "Contested Frequency"))
	var playlist = str(data.get("playlist", "Arena Duel"))
	var pressure = data.get("pressure", null)
	var host_line = str(data.get("host_line", ""))
	
	hud.set_league_identity(mode_name, playlist)
	if pressure == null:
		hud.set_pressure("")
	else:
		hud.set_pressure(str(pressure))
	# Sticky Host chrome always. Flash once only on mid-round join (Active/Ended),
	# so Warmup still waits for RoundStart Host bumper instead of double-flashing.
	if host_line != "":
		var flash = round_state == "Active" or round_state == "Ended"
		if hud.set_host_line(host_line, flash):
			if round_start_sound and round_start_sound.stream:
				round_start_sound.play()
	hud.set_tick(tick)
	hud.set_player_count(len(player_list))
	hud.sync_scores_from_players(player_list)
	hud.set_round_info(round_state, round_time_left, frag_limit)
	_maybe_assign_ghost_rival(player_list)
	
	var current_ids = {}
	
	for player_data in player_list:
		var id = player_data.id
		current_ids[id] = true
		
		if not players.has(id):
			if player_scene == null or not is_instance_valid(arena):
				continue
			var pawn = player_scene.instantiate()
			if pawn == null:
				push_warning("game_manager: player instantiate returned null for " + str(id))
				continue
			arena.add_child(pawn)
			pawn.position = Vector3(player_data.x, player_data.y, player_data.z)
			pawn.rotation.y = player_data.yaw
			pawn.set_player_data(id, player_data.name)
			players[id] = pawn
		
		if players.has(id):
			players[id].update_state(player_data)
	
	for id in players.keys():
		if not current_ids.has(id):
			if is_instance_valid(players[id]):
				players[id].queue_free()
			players.erase(id)
	
	var targets = []
	for pawn in players.values():
		if is_instance_valid(pawn):
			targets.append(pawn)
	if camera:
		camera.set_available_targets(targets)
		
		var followed = camera.get_followed_target()
		for pawn in players.values():
			if is_instance_valid(pawn):
				pawn.set_highlighted(pawn == followed)
	
	_update_followed_weapon()
	_sync_pickups(data.get("pickups", []))
	if is_human_player:
		_refresh_fp_target()
		_update_local_fp_hud(data.get("players", []))

func _on_event_received(data):
	var event_type = data.get("event", "")
	if event_type == "frag":
		var killer_name = data.get("killer", "?")
		var victim_name = data.get("victim", "?")
		
		var killer_id = ""
		for pawn in players.values():
			if is_instance_valid(pawn) and pawn.player_name == killer_name:
				killer_id = pawn.player_id
				break
		
		var killer_color = Color.WHITE
		var victim_color = Color.WHITE
		if killer_id != "" and players.has(killer_id):
			killer_color = players[killer_id].player_color
		for pawn in players.values():
			if is_instance_valid(pawn) and pawn.player_name == victim_name:
				victim_color = pawn.player_color
				break
		
		hud.show_frag(killer_name, victim_name, killer_color, victim_color)
		
		if camera:
			camera.camera_punch()
		
		for pawn in players.values():
			if is_instance_valid(pawn) and pawn.player_name == killer_name:
				pawn.show_winner_glow()
				break
		
		if frag_sound and frag_sound.stream:
			frag_sound.play()
		
		if not is_human_player and killer_id != "" and camera:
			camera.lock_on_frag(killer_id, 2.0)
	elif event_type == "round_start":
		var mode_name = str(data.get("mode_name", "Contested Frequency"))
		var playlist = str(data.get("playlist", "Arena Duel"))
		hud.set_league_identity(mode_name, playlist)
		hud.show_round_start(data.get("round_number", 0), str(data.get("host_line", "")))
		if round_start_sound and round_start_sound.stream:
			round_start_sound.play()
	elif event_type == "compliance_ping":
		hud.set_pressure("compliance")
		var duration_ticks = int(data.get("duration_ticks", 120))
		var duration_sec = float(duration_ticks) / 20.0
		hud.show_compliance_ping(str(data.get("message", "")), duration_sec)
	elif event_type == "boss_spawn":
		hud.set_pressure("compliance_drone")
		hud.show_boss_spawn(str(data.get("message", "")), str(data.get("name", "COMPLIANCE-DRONE")))
	elif event_type == "boss_down":
		hud.set_pressure("")
		hud.show_boss_down(str(data.get("message", "")), str(data.get("killer", "")))
	elif event_type == "hit":
		var target_id = str(data.get("target_id", ""))
		var my_id = str(net_client.player_id) if net_client.player_id != null else ""
		if is_human_player and my_id != "" and target_id == my_id:
			if hud and hud.has_method("show_damage_flash"):
				hud.show_damage_flash()
			if camera:
				camera.camera_punch()
	elif event_type == "respawn":
		var who = str(data.get("player", ""))
		var my_name = str(net_client.player_name) if net_client else ""
		if is_human_player and who != "" and who == my_name:
			if hud and hud.has_method("show_spawn_flash"):
				hud.show_spawn_flash()
			fp_spawn_flashed = true
	elif event_type == "pickup":
		var who = str(data.get("player", "?"))
		var kind = str(data.get("kind", "weapon"))
		var weapon = str(data.get("weapon", ""))
		var amount = int(data.get("amount", 0))
		hud.show_pickup_toast(who, weapon, kind, amount)
	elif event_type == "speak":
		var speaker = str(data.get("player", "?"))
		var line = str(data.get("text", ""))
		hud.show_speak(speaker, line)
	elif event_type == "round_end":
		hud.show_round_end(data.get("winner", ""), data.get("reason", ""))
		if round_end_sound and round_end_sound.stream:
			round_end_sound.play()


func _sync_pickups(pickup_list):
	var seen = {}
	for pad in pickup_list:
		var pid = str(pad.get("id", ""))
		if pid == "":
			continue
		seen[pid] = true
		var kind = str(pad.get("kind", "weapon"))
		var weapon = str(pad.get("weapon", ""))
		var amount = int(pad.get("amount", 0))
		var pos = Vector3(float(pad.get("x", 0.0)), float(pad.get("y", 0.4)), float(pad.get("z", 0.0)))
		var is_up = bool(pad.get("available", true))
		if not pickups.has(pid):
			if pickup_scene == null or not is_instance_valid(arena):
				continue
			var node = pickup_scene.instantiate()
			if node == null:
				push_warning("game_manager: pickup instantiate returned null for " + pid)
				continue
			arena.add_child(node)
			node.setup(pid, weapon, pos, kind, amount)
			pickups[pid] = node
		if pickups.has(pid):
			pickups[pid].position = pos
			var dirty = false
			if pickups[pid].weapon_name != weapon:
				pickups[pid].weapon_name = weapon
				dirty = true
			if pickups[pid].pickup_kind != kind:
				pickups[pid].pickup_kind = kind
				dirty = true
			if pickups[pid].amount != amount:
				pickups[pid].amount = amount
				dirty = true
			if dirty:
				pickups[pid]._apply_look()
			pickups[pid].set_available(is_up)
	for pid in pickups.keys():
		if not seen.has(pid):
			if is_instance_valid(pickups[pid]):
				pickups[pid].queue_free()
			pickups.erase(pid)

func _update_followed_weapon():
	if not camera or not hud:
		return
	
	if is_human_player:
		# FP path owns weapon chrome via _update_local_fp_hud.
		return
	
	if not camera.follow_mode or len(camera.available_targets) == 0:
		hud.set_followed_weapon("", "")
		return
	
	var target_index = camera.follow_target_index % len(camera.available_targets)
	var target = camera.available_targets[target_index]
	
	if not is_instance_valid(target):
		hud.set_followed_weapon("", "")
		return
	
	var weapon_name = ""
	var player_name = ""
	var behavior = ""
	if players.has(target.player_id):
		var pawn = players[target.player_id]
		if pawn.has_method("get_weapon_name"):
			weapon_name = pawn.get_weapon_name()
		player_name = pawn.player_name
		if "behavior" in pawn:
			behavior = pawn.behavior
	
	hud.set_followed_weapon(weapon_name, player_name, behavior)

func _pick_ghost_rival_from_alive():
	var names = []
	for pawn in players.values():
		if is_instance_valid(pawn) and pawn.player_name != "" and pawn.player_name != "Human Player":
			names.append(pawn.player_name)
	if names.is_empty():
		hud.set_ghost_rival("")
		return
	names.shuffle()
	hud.set_ghost_rival(names[0])

func _maybe_assign_ghost_rival(player_list: Array):
	if is_human_player:
		return
	# Spectators: keep a live rival chip so the fight has a face.
	if hud.ghost_rival != "":
		for p in player_list:
			if str(p.get("name", "")) == hud.ghost_rival:
				return
	var names = []
	for p in player_list:
		var n = str(p.get("name", ""))
		if n != "" and n != "Spectator" and n != "Human Player":
			names.append(n)
	if names.is_empty():
		hud.set_ghost_rival("")
		return
	names.shuffle()
	hud.set_ghost_rival(names[0])

func _set_human_fp(enabled: bool) -> void:
	if not enabled:
		_clear_fp_state()
		return
	_refresh_fp_target()

func _clear_fp_state() -> void:
	if local_fp_pawn_id != "" and players.has(local_fp_pawn_id):
		var old = players[local_fp_pawn_id]
		if is_instance_valid(old) and old.has_method("set_local_fp"):
			old.set_local_fp(false)
	local_fp_pawn_id = ""
	local_hp_seen = -1
	fp_spawn_flashed = false
	if camera and camera.has_method("set_fp_mode"):
		camera.set_fp_mode(false)
	if hud and hud.has_method("set_fp_juice"):
		hud.set_fp_juice(false)

func _refresh_fp_target() -> void:
	if not is_human_player:
		return
	var pid = str(net_client.player_id) if net_client.player_id != null else ""
	if pid == "" or not players.has(pid):
		return
	var pawn = players[pid]
	if not is_instance_valid(pawn):
		return
	if local_fp_pawn_id != "" and local_fp_pawn_id != pid and players.has(local_fp_pawn_id):
		var prev = players[local_fp_pawn_id]
		if is_instance_valid(prev) and prev.has_method("set_local_fp"):
			prev.set_local_fp(false)
	local_fp_pawn_id = pid
	if pawn.has_method("set_local_fp"):
		pawn.set_local_fp(true)
	if camera and camera.has_method("set_fp_mode"):
		camera.set_fp_mode(true, pawn)
	if hud and hud.has_method("set_fp_juice"):
		hud.set_fp_juice(true)
		if pawn.has_method("get_weapon_name"):
			hud.set_fp_weapon(pawn.get_weapon_name())
	if not fp_spawn_flashed and hud and hud.has_method("show_spawn_flash"):
		hud.show_spawn_flash()
		fp_spawn_flashed = true

func _update_local_fp_hud(player_list: Array) -> void:
	var pid = str(net_client.player_id) if net_client.player_id != null else ""
	if pid == "":
		return
	for pdata in player_list:
		if str(pdata.get("id", "")) != pid:
			continue
		var hp = int(pdata.get("hp", 100))
		if local_hp_seen >= 0 and hp < local_hp_seen and hp > 0:
			if hud and hud.has_method("show_damage_flash"):
				hud.show_damage_flash()
			if camera:
				camera.camera_punch()
		# Respawn: hp jumped back up while we were playing.
		if local_hp_seen >= 0 and local_hp_seen <= 0 and hp > 0:
			if hud and hud.has_method("show_spawn_flash"):
				hud.show_spawn_flash()
		local_hp_seen = hp
		var weapon = str(pdata.get("weapon", ""))
		if hud and hud.has_method("set_fp_weapon"):
			hud.set_fp_weapon(weapon)
		if hud and hud.has_method("set_followed_weapon"):
			hud.set_followed_weapon(weapon, str(pdata.get("name", "YOU")), "")
		return
