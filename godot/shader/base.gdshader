shader_type canvas_item;

uniform vec2 sheet_size = vec2(1.0, 5.0); // (열 = 프레임, 행 = 방향)
uniform vec3 team_palette[8] : source_color;
//uniform vec3 key_color_light : source_color = vec3(0.56, 0.74, 0.56);
//uniform vec3 team_palette_dark[8] : source_color;
//uniform vec3 key_color_dark : source_color = vec3(0.45, 0.59, 0.45);
uniform float key_hue = 0.33;
uniform float tolerance = 0.05;
uniform float min_sat = 0.2;

varying flat vec4 inst;

vec3 rgb_to_hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

void vertex() {
    inst = INSTANCE_CUSTOM;
    UV = (UV + vec2(inst.z, inst.y)) / sheet_size;
}

void fragment() {
    vec4 base = texture(TEXTURE, UV);
    vec3 hsv = rgb_to_hsv(base.rgb);
    vec3 team = team_palette[clamp(int(inst.x), 0, 7)];

    // hue는 원형이라 감아서 거리 계산
    float dh = abs(hsv.x - key_hue);
    dh = min(dh, 1.0 - dh);

    float hue_ok = 1.0 - smoothstep(tolerance, tolerance * 2.0, dh);
    float sat_ok = smoothstep(min_sat * 0.5, min_sat, hsv.y);
    float keyness = hue_ok * sat_ok;

    // 명암(V)만 살려서 팀 컬러 곱하기
    vec3 tinted = team * hsv.z;

    COLOR = vec4(mix(base.rgb, tinted, keyness), base.a);
}

//void light() {
//	// Called for every pixel for every light affecting the material.
//	// Uncomment to replace the default light processing function with this one.
//}
