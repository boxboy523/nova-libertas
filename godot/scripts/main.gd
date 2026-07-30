extends Node2D

@onready var unit_manager = $UnitManager

func _physics_process(_delta):
    queue_redraw()

func _unhandled_input(event):
    if event is InputEventMouseButton and event.pressed:
        if event.button_index == MOUSE_BUTTON_RIGHT:
            var mouse_pos = get_global_mouse_position()
            unit_manager.order_move(mouse_pos)
    if Input.is_action_just_pressed("attack"):
        var mouse_pos = get_global_mouse_position()
        unit_manager.order_attack(mouse_pos)
    if Input.is_action_just_pressed("attack-move"):
        var mouse_pos = get_global_mouse_position()
        unit_manager.order_move_with_auto_attack(mouse_pos)

func _draw():
    var vectors = unit_manager.get_flow_vectors()
    var grid_size = 40
    var map_size = Vector2(20 * grid_size, 20 * grid_size)
    for x in range(0, map_size.x, grid_size):
        draw_line(Vector2(x, 0), Vector2(x, map_size.y), Color(1, 1, 1, 0.2), 1)
    for y in range(0, map_size.y, grid_size):
        draw_line(Vector2(0, y), Vector2(map_size.x, y), Color(1, 1, 1, 0.2), 1)
    for i in range(0, vectors.size(), 4):
        var pos = Vector2(vectors[i], vectors[i+1])
        var dir = Vector2(vectors[i+2], vectors[i+3])
        draw_line(pos, pos + dir * 10, Color(0, 1, 0), 1)
