#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import "shaders/constants.wgsl"::{COLOR_MULTIPLIER, HIGHLIGHT_LEVEL, CHECKERBOARD_LIGHT, CHECKERBOARD_DARK};
#import "shaders/grid.wgsl"::{GridSize, grid_index, grid_offset, grid_coords, grid_uv};

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> size: GridSize;
@group(1) @binding(2) var<storage> grid: array<u32>;
@group(1) @binding(3) var<uniform> offset: vec2<f32>;
@group(1) @binding(4) var<uniform> viewport: vec2<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var offset = vec2<f32>(viewport.x, viewport.y) / 2. + vec2<f32>(offset.x, -offset.y);
    // offset.y *= viewport.y / viewport.x;

    let frag_coord = mesh.position.xy;
    var sfrag_coord = (frag_coord - offset) / viewport.xy;
    var uv = sfrag_coord;
    uv.y *= viewport.y / viewport.x;
    let width = 1. / 8.;
    uv /= width;
    uv *= vec2<f32>(1., -1.);
    uv += 0.5;

    let g = grid_uv(size, uv);
    let row = u32(g.y);
    let col = u32(g.x);

    var output_color = vec4<f32>(.1, .1, .1, 1.);
    output_color *= f32((col + row) % u32(2)) * (0.5 - 0.4) + 0.4;
    let highlight = f32(grid[grid_index(size, row, col)]) * HIGHLIGHT_LEVEL;

    output_color.r += 2. * highlight;
    output_color.g += 2. * highlight;
    output_color.b += 30. * highlight;

    return output_color;
}
