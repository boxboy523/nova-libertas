extends Node2D

@onready var unit_manager = $"../UnitManager"

var drag_start = Vector2.ZERO
var drag_end = Vector2.ZERO
var is_dragging = false

signal selection_changed(t_type: int, new_selection: Array)

func _unhandled_input(event):
    if event is InputEventMouseButton:
        if event.button_index == MOUSE_BUTTON_LEFT:
            if event.pressed:
                drag_start = get_global_mouse_position()
                is_dragging = true
            else:
                is_dragging = false
                # 선택 처리
                var rect_min = Vector2(min(drag_start.x, drag_end.x), min(drag_start.y, drag_end.y))
                var rect_max = Vector2(max(drag_start.x, drag_end.x), max(drag_start.y, drag_end.y))
                _select(rect_min, rect_max)
                drag_start = Vector2.ZERO
                drag_end = Vector2.ZERO
                queue_redraw()
    if event is InputEventMouseMotion and is_dragging:
        drag_end = get_global_mouse_position()
        queue_redraw()

func _draw():
    if is_dragging:
        var rect = Rect2(drag_start, drag_end - drag_start)
        draw_rect(rect, Color(0, 1, 0, 0.2), true)
        draw_rect(rect, Color(0, 1, 0, 0.8), false, 1.0)

func _select(rect_min: Vector2, rect_max: Vector2):
    unit_manager.remove_selection()
    unit_manager.select_unit_in_area(rect_min, rect_max)
    var new_selection = unit_manager.get_selected_units()
    print("New selection: ", new_selection)
    var new_selection_array = []
    for i in range(Config.UNIT_TYPES_LEN):
        new_selection_array.append([])
    for i in range(0, new_selection.size(), 2):
        new_selection_array[new_selection[i]].append(new_selection[i + 1])
    print("New selection array: ", new_selection_array)
    for t_type in range(new_selection_array.size()):
        emit_signal("selection_changed", t_type, new_selection_array[t_type])
