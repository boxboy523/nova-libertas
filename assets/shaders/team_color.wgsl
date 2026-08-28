#import bevy_pbr::forward_io::VertexOutput

struct TeamColorMaterial {
    team_color: vec4<f32>,
    key_hue: f32,
    tolerance: f32,
    min_saturation: f32,
    alpha_cutoff: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: TeamColorMaterial;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var color_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var color_sampler: sampler;

fn rgb_to_hsv(color: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(
        0.0,
        -1.0 / 3.0,
        2.0 / 3.0,
        -1.0,
    );

    let p = mix(
        vec4<f32>(color.b, color.g, k.w, k.z),
        vec4<f32>(color.g, color.b, k.x, k.y),
        step(color.b, color.g),
    );

    let q = mix(
        vec4<f32>(p.x, p.y, p.w, color.r),
        vec4<f32>(color.r, p.y, p.z, p.x),
        step(p.x, color.r),
    );

    let difference = q.x - min(q.w, q.y);
    let epsilon = 1.0e-10;

    return vec3<f32>(
        abs(
            q.z
            + (q.w - q.y)
                / (6.0 * difference + epsilon)
        ),
        difference / (q.x + epsilon),
        q.x,
    );
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(
        color_texture,
        color_sampler,
        input.uv,
    );

    if base.a < material.alpha_cutoff {
        discard;
    }

    let hsv = rgb_to_hsv(base.rgb);

    // Hue는 0과 1이 이어지는 원형 값이다.
    let raw_hue_distance = abs(hsv.x - material.key_hue);
    let hue_distance = min(
        raw_hue_distance,
        1.0 - raw_hue_distance,
    );

    let hue_match = 1.0 - smoothstep(
        material.tolerance,
        material.tolerance * 2.0,
        hue_distance,
    );

    let saturation_match = smoothstep(
        material.min_saturation * 0.5,
        material.min_saturation,
        hsv.y,
    );

    let keyness = hue_match * saturation_match;

    // 원본의 명암(value)을 보존하면서 팀 색상을 적용한다.
    let tinted = material.team_color.rgb * hsv.z;
    let final_color = mix(base.rgb, tinted, keyness);

    return vec4<f32>(final_color, base.a);
}