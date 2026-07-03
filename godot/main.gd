extends Node2D

@onready var unit_manager = $UnitManager
@onready var multimesh_unit = $MultiMeshUnit
@onready var multimesh_wall = $MultiMeshWall

func _physics_process(_delta):
   # print("field len: {}", unit_manager.get_flow_vectors())
    queue_redraw()
    if unit_manager and multimesh_unit:
        unit_manager.update_multimesh_buffer(0, multimesh_unit.multimesh)
    if unit_manager and multimesh_wall:
        unit_manager.update_multimesh_buffer(1, multimesh_wall.multimesh)

func _unhandled_input(event):
    if event is InputEventMouseButton and event.pressed:
        var mouse_pos = get_global_mouse_position()
        unit_manager.order_move(mouse_pos)
        print("event emited")

func _draw():
    var vectors = unit_manager.get_flow_vectors()
    var grid_size = 40
    var map_size = Vector2(20 * grid_size, 20 * grid_size)
    for x in range(0, map_size.x, grid_size):
        draw_line(Vector2(x, 0), Vector2(x, map_size.y), Color(1, 1, 1, 0.2), 1)
    for y in range(0, map_size.y, grid_size):
        draw_line(Vector2(0, y), Vector2(map_size.x, y), Color(1, 1, 1, 0.2), 1)
    for i in range(multimesh_wall.multimesh.instance_count):
        var transform = multimesh_wall.multimesh.get_instance_transform_2d(i)
        var pos = transform.origin
        var size = Vector2(grid_size, grid_size)
        draw_rect(Rect2(pos, size), Color.BLUE, false, 1.0)
    for i in range(0, vectors.size(), 4):
        var pos = Vector2(vectors[i], vectors[i+1])
        var dir = Vector2(vectors[i+2], vectors[i+3])
        draw_line(pos, pos + dir * 10, Color.GREEN, 1)
    for i in range(multimesh_unit.multimesh.instance_count):
        var transform = multimesh_unit.multimesh.get_instance_transform_2d(i)
        var pos = transform.origin
        draw_circle(pos, 10, Color.GREEN)
