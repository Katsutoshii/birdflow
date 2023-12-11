#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK}

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> width: f32;
@group(1) @binding(2) var<uniform> rows: u32;
@group(1) @binding(3) var<uniform> cols: u32;
@group(1) @binding(4) var<storage> grid: array<u32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var offset = vec4<f32>(0., 0., 0., 0.);
    offset.x = f32(cols) * width / 2.;
    offset.y = f32(rows) * width / 2.;
    let position = mesh.world_position + offset;

    let col = u32(position.x / f32(width));
    let row = u32(position.y / f32(width));

    var output_color = color;
    output_color *= f32((col + row) % u32(2)) * (CHECKERBOARD_LIGHT - CHECKERBOARD_DARK) + CHECKERBOARD_DARK;
    let highlight = f32(grid[row * cols + col]) * HIGHLIGHT_LEVEL;
    output_color.r += highlight;
    output_color.g += highlight;
    output_color.b += highlight;
    return output_color;
}
