struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) in: vec3<f32>) -> VertexOutput {
    var result: VertexOutput;
    result.position = vec4(in, 1.0);
    result.color = in;
    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = in.position;
    var x = position.x;
    var y = 600 - position.y;
    var q = x * x + y * y;
    var g = round(sin(q / 100) - 0.45);
    return vec4<f32>(0, g, 0, 1.0);
}
