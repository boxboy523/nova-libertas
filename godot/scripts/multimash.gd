extends MultiMeshInstance2D

@export var thing_type_num: int
@export var unit_manager: UnitManager
@export var debug_draw: bool = false
@export var debug_color: Color = Color(1, 0, 0)
@export var debug_shape: String = "circle"
@export var debug_size: float = 5.0

var selected: Array = []

func _physics_process(_delta):
    queue_redraw()
    unit_manager.update_multimesh_buffer(thing_type_num, multimesh)

func _draw():
    for i in selected:
        var t = multimesh.get_instance_transform_2d(i)
        draw_circle(t.origin, 20, Color(1, 0, 0), false, 1.0)
    for i in range(multimesh.instance_count):
        var t = multimesh.get_instance_transform_2d(i)
        if debug_draw:
            match debug_shape:
                "circle":
                    draw_circle(t.origin, debug_size, debug_color, false, 1.0)
                "square":
                    var size = Vector2(debug_size, debug_size)
                    draw_rect(Rect2(t.origin, size), debug_color, false, 1.0)
                _:
                    draw_circle(t.origin, debug_size, debug_color, false, 1.0)

func _on_drag_node_selection_changed(t_type: int, new_selection: Array) -> void:
    if thing_type_num == t_type:
        selected = new_selection
        queue_redraw()
