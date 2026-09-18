extends Node

signal connected_to_server
signal disconnected_from_server
signal snapshot_received(data)
signal event_received(data)

var socket = WebSocketPeer.new()
var connection_state = WebSocketPeer.STATE_CLOSED
var server_url = "ws://127.0.0.1:6767"

func _init():
	# Allow server URL override via environment variable for LAN/Tailscale
	var env_server = OS.get_environment("FRAGR_SERVER")
	if env_server != "":
		server_url = "ws://" + env_server if not env_server.begins_with("ws://") else env_server
		print("Using server from FRAGR_SERVER: ", server_url)

var role = "spectator"
var player_name = "Spectator"
var player_id = null

func _ready():
	set_process(false)

func connect_to_server(p_role: String = "spectator", p_name: String = "Player"):
	role = p_role
	player_name = p_name
	
	var err = socket.connect_to_url(server_url)
	if err != OK:
		push_error("Failed to connect to server: " + str(err))
		return false
	
	connection_state = socket.get_ready_state()
	set_process(true)
	print("Connecting to ", server_url, " as ", role)
	return true

func disconnect_from_server():
	socket.close()
	connection_state = WebSocketPeer.STATE_CLOSED
	set_process(false)
	disconnected_from_server.emit()

func send_hello():
	var hello = {
		"type": "hello",
		"role": role,
		"name": player_name
	}
	send_json(hello)

func send_action(action: Dictionary):
	var msg = {
		"type": "action",
		"forward": action.get("forward", false),
		"back": action.get("back", false),
		"left": action.get("left", false),
		"right": action.get("right", false),
		"turn_left": action.get("turn_left", false),
		"turn_right": action.get("turn_right", false),
		"fire": action.get("fire", false)
	}
	send_json(msg)

func send_json(data: Dictionary):
	var json = JSON.stringify(data)
	socket.send_text(json)

func _process(_delta):
	socket.poll()
	var state = socket.get_ready_state()
	
	if state != connection_state:
		connection_state = state
		
		if state == WebSocketPeer.STATE_OPEN:
			print("Connected to server!")
			send_hello()
			connected_to_server.emit()
		elif state == WebSocketPeer.STATE_CLOSED:
			print("Disconnected from server")
			set_process(false)
			disconnected_from_server.emit()
	
	while socket.get_ready_state() == WebSocketPeer.STATE_OPEN and socket.get_available_packet_count() > 0:
		var packet = socket.get_packet()
		var text = packet.get_string_from_utf8()
		_handle_message(text)

func _handle_message(text: String):
	var json = JSON.new()
	var error = json.parse(text)
	if error != OK:
		push_error("Failed to parse JSON: " + text)
		return
	
	var data = json.data
	if not data is Dictionary:
		return
	
	var msg_type = data.get("type", "")
	
	match msg_type:
		"welcome":
			player_id = data.get("player_id")
			print("Welcome received! Role: ", data.get("role"), " Player ID: ", player_id, " Mode: ", data.get("mode_name", "Contested Frequency"), "/", data.get("playlist", "Arena Duel"))
		
		"snapshot":
			snapshot_received.emit(data)
		
		"event":
			event_received.emit(data)
			var event_type = data.get("event", "")
			if event_type == "frag":
				print("FRAG: ", data.get("killer"), " → ", data.get("victim"))
			elif event_type == "respawn":
				print("Respawn: ", data.get("player"))
