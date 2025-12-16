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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_count_4_segments() {
        let square = generate_square(1.0, 0.0, 4);
        let circle = generate_circle(1.0, 0.0, 4);

        assert_eq!(square.len(), 4);
        assert_eq!(circle.len(), 4);
    }

    #[test]
    fn test_vertex_count_8_segments() {
        let square = generate_square(1.0, 0.0, 8);
        let circle = generate_circle(1.0, 0.0, 8);

        assert_eq!(square.len(), 8);
        assert_eq!(circle.len(), 8);
    }

    #[test]
    fn test_square_corners_4_segments() {
        let square = generate_square(1.0, 0.0, 4);

        // Corner 0: (-1, -1)
        assert!((square[0].pos.x - (-1.0)).abs() < 1e-10);
        assert!((square[0].pos.y - (-1.0)).abs() < 1e-10);

        // Corner 1: (1, -1)
        assert!((square[1].pos.x - 1.0).abs() < 1e-10);
        assert!((square[1].pos.y - (-1.0)).abs() < 1e-10);

        // Corner 2: (1, 1)
        assert!((square[2].pos.x - 1.0).abs() < 1e-10);
        assert!((square[2].pos.y - 1.0).abs() < 1e-10);

        // Corner 3: (-1, 1)
        assert!((square[3].pos.x - (-1.0)).abs() < 1e-10);
        assert!((square[3].pos.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle_4_segments() {
        let r = 1.0;
        let circle = generate_circle(r, 0.0, 4);

        // Vertex 0: angle -135° (-3π/4)
        let a0 = -3.0 * std::f64::consts::PI / 4.0;
        assert!((circle[0].pos.x - r * a0.cos()).abs() < 1e-10);
        assert!((circle[0].pos.y - r * a0.sin()).abs() < 1e-10);

        // Vertex 1: angle -45° (-π/4)
        let a1 = -std::f64::consts::PI / 4.0;
        assert!((circle[1].pos.x - r * a1.cos()).abs() < 1e-10);
        assert!((circle[1].pos.y - r * a1.sin()).abs() < 1e-10);

        // Vertex 2: angle 45° (π/4)
        let a2 = std::f64::consts::PI / 4.0;
        assert!((circle[2].pos.x - r * a2.cos()).abs() < 1e-10);
        assert!((circle[2].pos.y - r * a2.sin()).abs() < 1e-10);

        // Vertex 3: angle 135° (3π/4)
        let a3 = 3.0 * std::f64::consts::PI / 4.0;
        assert!((circle[3].pos.x - r * a3.cos()).abs() < 1e-10);
        assert!((circle[3].pos.y - r * a3.sin()).abs() < 1e-10);
    }

    #[test]
    fn test_connecting_vectors_4_segments() {
        let half_size = 1.0;
        let radius = 1.0;
        let z_bottom = 0.0;
        let z_top = 10.0;

        let square = generate_square(half_size, z_bottom, 4);
        let circle = generate_circle(radius, z_top, 4);

        // Precompute expected vectors from square corners to circle points
        // Square corner 0: (-1, -1), Circle point 0: (-√2/2, -√2/2)
        let sqrt2_2 = std::f64::consts::FRAC_1_SQRT_2;
        let expected = [
            // vertex 0: (-1, -1) -> (-√2/2, -√2/2)
            (-sqrt2_2 - (-1.0), -sqrt2_2 - (-1.0), z_top - z_bottom),
            // vertex 1: (1, -1) -> (√2/2, -√2/2)
            (sqrt2_2 - 1.0, -sqrt2_2 - (-1.0), z_top - z_bottom),
            // vertex 2: (1, 1) -> (√2/2, √2/2)
            (sqrt2_2 - 1.0, sqrt2_2 - 1.0, z_top - z_bottom),
            // vertex 3: (-1, 1) -> (-√2/2, √2/2)
            (-sqrt2_2 - (-1.0), sqrt2_2 - 1.0, z_top - z_bottom),
        ];

        for i in 0..4 {
            let dx = circle[i].pos.x - square[i].pos.x;
            let dy = circle[i].pos.y - square[i].pos.y;
            let dz = circle[i].pos.z - square[i].pos.z;

            assert!(
                (dx - expected[i].0).abs() < 1e-10,
                "Vertex {i}: dx {dx:.6} != expected {:.6}",
                expected[i].0
            );
            assert!(
                (dy - expected[i].1).abs() < 1e-10,
                "Vertex {i}: dy {dy:.6} != expected {:.6}",
                expected[i].1
            );
            assert!(
                (dz - expected[i].2).abs() < 1e-10,
                "Vertex {i}: dz {dz:.6} != expected {:.6}",
                expected[i].2
            );
        }
    }
}
