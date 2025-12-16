use csgrs::mesh::polygon::Polygon;
use csgrs::mesh::vertex::Vertex;
use csgrs::sketch::Sketch;
use nalgebra::{Point3, Vector3};

/// Generate a square polygon with `segments` vertices at given z height.
/// Vertices are distributed evenly (segments/4 per side), starting at
/// (-half, -half) and going counter-clockwise.
fn generate_square(half_size: f64, z: f64, segments: u64) -> Vec<Vertex> {
    let pts_per_side = segments / 4;
    let mut pts: Vec<Vertex> = Vec::with_capacity(segments as usize);

    // Bottom edge: (-half, -half) to (half, -half)
    for i in 0..pts_per_side {
        let t = i as f64 / pts_per_side as f64;
        let x = -half_size + t * (2.0 * half_size);
        pts.push(Vertex::new(
            Point3::new(x, -half_size, z),
            Vector3::new(0.0, -1.0, 0.0),
        ));
    }
    // Right edge: (half, -half) to (half, half)
    for i in 0..pts_per_side {
        let t = i as f64 / pts_per_side as f64;
        let y = -half_size + t * (2.0 * half_size);
        pts.push(Vertex::new(
            Point3::new(half_size, y, z),
            Vector3::new(1.0, 0.0, 0.0),
        ));
    }
    // Top edge: (half, half) to (-half, half)
    for i in 0..pts_per_side {
        let t = i as f64 / pts_per_side as f64;
        let x = half_size - t * (2.0 * half_size);
        pts.push(Vertex::new(
            Point3::new(x, half_size, z),
            Vector3::new(0.0, 1.0, 0.0),
        ));
    }
    // Left edge: (-half, half) to (-half, -half)
    for i in 0..pts_per_side {
        let t = i as f64 / pts_per_side as f64;
        let y = half_size - t * (2.0 * half_size);
        pts.push(Vertex::new(
            Point3::new(-half_size, y, z),
            Vector3::new(-1.0, 0.0, 0.0),
        ));
    }

    pts
}

/// Generate a circle polygon with `segments` vertices at given z height.
/// Starts at -135° (-3π/4) to align with square's starting corner.
fn generate_circle(radius: f64, z: f64, segments: u64) -> Vec<Vertex> {
    let start_angle = -3.0 * std::f64::consts::PI / 4.0;
    let mut pts: Vec<Vertex> = Vec::with_capacity(segments as usize);

    for i in 0..segments {
        let angle = start_angle + 2.0 * std::f64::consts::PI * (i as f64) / (segments as f64);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        pts.push(Vertex::new(
            Point3::new(x, y, z),
            Vector3::new(angle.cos(), angle.sin(), 0.0),
        ));
    }

    pts
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _tube_od = 12.0;
    let tube_id = 8.0;
    let length = 20.0;
    let inside_wall_thickness = 2.0;
    let square_sz = 2.397; // Default from rerf-bd-bobbins.py `default_cube_size = round_to_resolution(2.4, default_bed_resolution)`
    let segments_string = std::env::args().nth(1).unwrap_or("50".to_string());
    let segments: u64 = match segments_string.parse() {
        Ok(v) => v,
        _ => {
            println!("Expected segments to be a number");
            return Ok(());
        }
    };
    if segments < 4 || ((segments % 4) != 0) {
        println!("Segements must be >= 4 and a multiple of 4");
        return Ok(());
    }

    // Add lofting a square to circle with square_sz and circle radius of (tube_id / 2) - inside_wall_thickness
    // Both polygons need the same number of vertices (segments)
    let circle_radius = (tube_id / 2.0) - inside_wall_thickness;
    let half_square = square_sz / 2.0;

    let square_pts = generate_square(half_square, 0.0, segments);
    let circle_pts = generate_circle(circle_radius, length, segments);

    let bottom_poly: Polygon<()> = Polygon::new(square_pts, None);
    let top_poly: Polygon<()> = Polygon::new(circle_pts, None);
    let loft_obj = Sketch::loft(&bottom_poly, &top_poly, false)
        .map_err(|e| format!("{:?}", e))?;

    let name: String = format!("csgrs-logfting_segments-{segments}").into();
    let shape: Vec<u8> = loft_obj.to_stl_ascii(&name).into();
    let file_name: String = name + ".stl";
    println!("Writing file: {}", file_name);
    std::fs::write(file_name, shape).unwrap();

    Ok(())
}
