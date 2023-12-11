#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> width: f32;
@group(1) @binding(2) var<uniform> rows: u32;
@group(1) @binding(3) var<uniform> cols: u32;
@group(1) @binding(4) var<storage> grid: array<f32>;

fn grid_at(row: u32, col: u32) -> f32 {
    return grid[row * cols + col];
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // return color;
    var offset = vec4<f32>(0., 0., 0., 0.);
    offset.x = f32(cols) * width / 2.;
    offset.y = f32(rows) * width / 2.;
    let position = mesh.world_position + offset;

    let col = u32(position.x / f32(width));
    let row = u32(position.y / f32(width));

    // Sample visibility of nearby cells to get a smoother range of visibility.
    var alpha = 0.2 * grid_at(row, col);
    alpha += 0.2 * grid_at(row + 1u, col + 0u);
    alpha += 0.2 * grid_at(row + 0u, col + 1u);
    alpha += 0.2 * grid_at(row - 1u, col - 0u);
    alpha += 0.2 * grid_at(row - 0u, col - 1u);

    var output_color = color;
    output_color.a *= alpha;
    return output_color;
}
