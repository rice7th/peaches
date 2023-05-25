struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
};

fn sdf_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p - vec2(500.0, 500.0)) - r;
}

fn sdf_rectangle(p: vec2<f32>, b: vec2<f32>) -> f32{
    let d = abs(p) - b;
    let a: vec2<f32> = max(d, vec2(0.0, 0.0));
    let b: f32 = max(d.x, d.y);
    let c: f32 = min(b, 0.0);
    return length(a) + c;
}

fn sdf_round_rect(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    return sdf_rectangle(p, b - vec2(r, r)) - r;
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 6.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 6.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment // Fragment. SDFs go here lmao
    
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sdf = sdf_round_rect(in.clip_position.xy - vec2(500.0, 500.0), vec2(300.0, 100.0), 20.0);
    if sdf < 0.0 {
        return vec4<f32>(0.9, 0.9, 0.9,1.0);
    }
    return vec4<f32>(0.3, 0.2, 0.1,1.0);
}