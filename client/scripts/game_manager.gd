extends Node

@onready var net_client = $NetClient
@onready var hud = $HUD
@onready var arena = $Arena
@onready var camera = $SpectatorCamera

var players = {}
var player_scene = preload("res://scenes/player.tscn")

var is_human_player = false
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
	
	var role = "spectator"
	var player_name = "Spectator"
	
	if "--human" in OS.get_cmdline_args():
		role = "human"
		player_name = "Human Player"
		is_human_player = true
	
	net_client.connect_to_server(role, player_name)

func _input(event):
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_J and not is_human_player:
			print("Joining as human player...")
			net_client.disconnect_from_server()
			await get_tree().create_timer(0.5).timeout
			is_human_player = true
			net_client.connect_to_server("human", "Human Player")
			hud.set_mode("PLAYING")
		elif event.keycode == KEY_L and is_human_player:
			print("Leaving match, returning to spectator...")
			net_client.disconnect_from_server()
			await get_tree().create_timer(0.5).timeout
			is_human_player = false
			net_client.connect_to_server("spectator", "Spectator")
			hud.set_mode("SPECTATING")

func _process(_delta):
	if is_human_player and net_client.connection_state == WebSocketPeer.STATE_OPEN:
		action_state.forward = Input.is_action_pressed("move_forward")
		action_state.back = Input.is_action_pressed("move_back")
		action_state.left = Input.is_action_pressed("move_left")
		action_state.right = Input.is_action_pressed("move_right")
		action_state.fire = Input.is_action_pressed("fire")
		
		net_client.send_action(action_state)

func _on_connected():
	hud.set_status("Connected to server")

func _on_disconnected():
	hud.set_status("Disconnected")

func _on_snapshot_received(data):
	var tick = data.get("tick", 0)
	var player_list = data.get("players", [])
	
	hud.set_tick(tick)
	hud.set_player_count(len(player_list))
	
	var current_ids = {}
	
	for player_data in player_list:
		var id = player_data.id
		current_ids[id] = true
		
		if not players.has(id):
			var pawn = player_scene.instantiate()
			arena.add_child(pawn)
			# Set initial position before setting player data to avoid snap
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

func _on_event_received(data):
	var event_type = data.get("event", "")
	if event_type == "frag":
		hud.show_frag(data.get("killer", "?"), data.get("victim", "?"))
