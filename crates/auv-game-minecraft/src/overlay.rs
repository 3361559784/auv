use image::{Rgb, RgbImage};

use crate::types::{MinecraftProjectedPoint, RaycastHit};

pub fn render_projection_overlay(
  mut image: RgbImage,
  projected: &MinecraftProjectedPoint,
  raycast_hit: Option<&RaycastHit>,
) -> RgbImage {
  if let Some(screen_point) = projected.screen_point {
    let center_x = screen_point.x.round() as i32;
    let center_y = screen_point.y.round() as i32;
    let radius = projected.match_radius_px.round().max(1.0) as i32;
    draw_crosshair(&mut image, center_x, center_y, radius, Rgb([255, 0, 0]));
    draw_box(&mut image, center_x, center_y, radius, Rgb([255, 255, 0]));
  }

  if raycast_hit.is_some() {
    draw_marker(&mut image, 6, 6, Rgb([0, 255, 255]));
  }

  image
}

fn draw_crosshair(image: &mut RgbImage, center_x: i32, center_y: i32, radius: i32, color: Rgb<u8>) {
  for delta in -radius..=radius {
    draw_pixel(image, center_x + delta, center_y, color);
    draw_pixel(image, center_x, center_y + delta, color);
  }
}

fn draw_box(image: &mut RgbImage, center_x: i32, center_y: i32, radius: i32, color: Rgb<u8>) {
  let min_x = center_x - radius;
  let max_x = center_x + radius;
  let min_y = center_y - radius;
  let max_y = center_y + radius;

  for x in min_x..=max_x {
    draw_pixel(image, x, min_y, color);
    draw_pixel(image, x, max_y, color);
  }
  for y in min_y..=max_y {
    draw_pixel(image, min_x, y, color);
    draw_pixel(image, max_x, y, color);
  }
}

fn draw_marker(image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) {
  for dx in 0..4 {
    for dy in 0..4 {
      draw_pixel(image, x + dx, y + dy, color);
    }
  }
}

fn draw_pixel(image: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) {
  if x < 0 || y < 0 {
    return;
  }
  let x = x as u32;
  let y = y as u32;
  if x >= image.width() || y >= image.height() {
    return;
  }
  image.put_pixel(x, y, color);
}

#[cfg(test)]
mod tests {
  use image::RgbImage;

  use super::*;
  use crate::types::{BlockFace, BlockPosition, ProjectionVisibility};

  #[test]
  fn overlay_marks_projected_region_and_raycast_badge() {
    let image = RgbImage::from_pixel(32, 32, Rgb([0, 0, 0]));
    let projected = MinecraftProjectedPoint {
      screen_point: Some(auv_driver::geometry::Point::new(16.0, 16.0)),
      visibility: ProjectionVisibility::Visible,
      match_radius_px: 4.0,
      basis_frame_id: "frame-1".to_string(),
      confidence: 1.0,
    };
    let raycast_hit = RaycastHit {
      block_pos: BlockPosition::new(1, 2, 3),
      face: BlockFace::North,
      block_id: "minecraft:stone".to_string(),
    };

    let overlay = render_projection_overlay(image, &projected, Some(&raycast_hit));

    assert_eq!(overlay.width(), 32);
    assert_eq!(overlay.height(), 32);
    assert_eq!(overlay.get_pixel(16, 16), &Rgb([255, 0, 0]));
    assert_eq!(overlay.get_pixel(6, 6), &Rgb([0, 255, 255]));
  }
}
