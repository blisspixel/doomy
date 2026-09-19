extends RefCounted
class_name ServerYaw

## The one place that converts between the server's facing and Godot's.
##
## The server's yaw is `atan2(dz, dx)`, so a fighter at yaw zero moves along
## world +X. A Godot node with `rotation.y = t` has its local +X at
## `(cos t, 0, -sin t)` and looks down -Z at `(-sin t, 0, -cos t)`.
##
## Those are ninety degrees apart, and they agree at exactly two angles. The
## client used to assign the server's yaw straight to `rotation.y`, so a player
## moved where the server thought they were facing and saw somewhere else
## entirely, which reads as being able to move only sideways. Nothing rendered
## it until a first-person camera existed, which is why it survived this long.

## Where the server sends a fighter at this yaw.
static func forward(yaw: float) -> Vector3:
	return Vector3(cos(yaw), 0.0, sin(yaw))

## Rotation for a node whose local +X is its forward, which is where the pawn's
## muzzle and weapon sprites are parented.
static func pawn_rotation_y(yaw: float) -> float:
	return -yaw

## Rotation for a camera, which in Godot looks down its own -Z.
static func camera_rotation_y(yaw: float) -> float:
	return -(yaw + PI / 2.0)
