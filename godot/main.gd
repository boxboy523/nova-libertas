extends Node2D

@onready var unit_manager = $UnitManager
@onready var multimesh_unit = $MultiMeshUnit
@onready var multimesh_wall = $MultiMeshWall

func _physics_process(_delta):
    if unit_manager and multimesh_unit:
        unit_manager.update_multimesh_buffer(0, multimesh_unit.multimesh)
    if unit_manager and multimesh_wall:
        unit_manager.update_multimesh_buffer(1, multimesh_wall.multimesh)

func _unhandled_input(event):
    if event is InputEventMouseButton and event.pressed:
        var mouse_pos = get_global_mouse_position()
        unit_manager.order_move(mouse_pos)
