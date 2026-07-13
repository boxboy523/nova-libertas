extends Node2D

@onready var unit_manager = $"../UnitManager"

var drag_start = Vector2.ZERO
var drag_end = Vector2.ZERO
var is_dragging = false

func _unhandled_input(event):
    if event is InputEventMouseButton:
        if event.button_index == MOUSE_BUTTON_LEFT:
            if event.pressed:
                drag_start = get_global_mouse_position()
                is_dragging = true
            else:
                is_dragging = false
                # 선택 처리
                if drag_end == Vector2.ZERO:
                    drag_end = drag_start
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
