shader_type canvas_item;
render_mode unshaded;

void vertex() {
    float hp_ratio = clamp(INSTANCE_CUSTOM.x, 0.0, 1.0);

    // 중앙 정렬된 QuadMesh의 왼쪽 끝 좌표
    float left_edge = -abs(VERTEX.x);

    // 왼쪽은 고정하고 오른쪽 끝만 HP 비율에 맞게 이동
    VERTEX.x = mix(left_edge, VERTEX.x, hp_ratio);

    // texture도 같은 비율만큼 잘라서 표시
    UV.x *= hp_ratio;
}

void fragment() {
    COLOR = texture(TEXTURE, UV);
}