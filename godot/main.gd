extends Node2D

@onready var unit_manager = $UnitManager
@onready var multimesh_instance = $MultiMeshInstance2D

func _process(_delta):
    if unit_manager and multimesh_instance:
        # Rust ECS의 데이터를 MultiMesh 버퍼로 직접 꽂아넣기
        unit_manager.update_multimesh_buffer(multimesh_instance.multimesh)

func _unhandled_input(event):
    if event is InputEventMouseButton and event.pressed:
        var mouse_pos = get_global_mouse_position()
        unit_manager.order_move(mouse_pos)
