struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance_position: vec2<f32>,
) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-0.05, -0.05),
        vec2<f32>(0.05, -0.05),
        vec2<f32>(0.0, 0.05)
    );

    let pos = positions[vertex_index] + instance_position;
    
    var output: VertexOutput;
    output.position = vec4<f32>(pos, 0.0, 1.0);
    
    // Create different colors based on position
    output.color = vec3<f32>(
        (instance_position.x + 1.0) * 0.5,
        (instance_position.y + 1.0) * 0.5,
        0.5
    );
    
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}