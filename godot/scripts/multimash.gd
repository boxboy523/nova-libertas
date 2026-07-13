extends MultiMeshInstance2D

enum THING_TYPE {
    UNIT = 0,
    WALL = 1,
}

enum TEAM {
    PLAYER = 0,
    ENEMY = 1,
}

@export var thing_type: THING_TYPE = THING_TYPE.UNIT
@export var unit_manager: UnitManager
@export var debug_draw: bool = false
@export var debug_color: Color = Color(1, 0, 0)
@export var debug_shape: String = "circle"
@export var debug_size: float = 5.0

var selected: Array = []
var units: Array = []

func _physics_process(_delta):
    queue_redraw()
    unit_manager.update_multimesh_buffer(thing_type, multimesh)

func _draw():
    for u in units:
        var t = multimesh.get_instance_transform_2d(u["index"])
        var team_color = Color(0, 0, 1) if u["team"] == TEAM.PLAYER else Color(1, 0, 0)
        if debug_draw:
            draw_circle(t.origin, 5.0, team_color, true)
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

func _on_unit_manager_selection_changed(t_type: THING_TYPE, indices: PackedInt32Array) -> void:
    if t_type == thing_type:
        selected = indices
        queue_redraw()


func _on_unit_manager_t_type_changed(t_type: THING_TYPE, indices: PackedInt32Array, team: PackedInt32Array) -> void:
    var new_units = []
    if t_type == thing_type:
        for i in range(indices.size()):
            new_units.append({"index": indices[i], "team": team[i]})
        units = new_units
        queue_redraw()
