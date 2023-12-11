#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals;
// we can import items from shader modules in the assets folder with a quoted path
#import "shaders/perlin_noise_2d.wgsl"::{perlin_noise_2d}

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
    let gxy = mesh.world_position + offset;

    let gx = gxy.x / width;
    let gx_floor = floor(gx);
    let gx_frac = gx - gx_floor;
    let col = u32(gx_floor);

    let gy = gxy.y / width;
    let gy_floor = floor(gy);
    let gy_frac = gy - gy_floor;
    let row = u32(gy_floor);

    // Sample visibility of nearby cells to get a smoother range of visibility.
    var alpha = grid_at(row, col);
    alpha += grid_at(row + 1u, col + 0u) * gy_frac;
    alpha += grid_at(row + 0u, col + 1u) * gx_frac;
    alpha += grid_at(row - 1u, col - 0u) * (1. - gy_frac);
    alpha += grid_at(row - 0u, col - 1u) * (1. - gx_frac);
    alpha *= 0.3;

    var output_color = color;
    let noise_amount = 0.13;
    let sin_t = sin(0.2 * globals.time);
    let sin_xt = sin(0.13 * globals.time);
    let noise = (2. + sin_t) / 3. * perlin_noise_2d(vec2<f32>(gx + sin_t, gy - sin_xt));

    output_color.a *= (1.0 - noise_amount) * alpha + noise_amount * noise;
    return output_color;
}
