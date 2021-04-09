mod checkerboard;
pub use checkerboard::Checkerboard;

pub trait Noise2D {
  /// Sample a point somewhere on the plane.
  /// Some Noise2Ds may have restrictions on the input coordinates.
  /// They should always output between 0 and 1; there may be unpredictable errors otherwise.
  fn get(&self, x: f32, y: f32) -> f32;

  /// Layer the same noise multiple times over itself, each time more detailed and with less effect.
  /// Count is the total number of octaves to apply. Each means more work for more complexity. Should be >0.
  /// Zoom is the amount to shrink each subsequent layer by (by "zooming out" on the original noise); it should be >1.
  /// Scale is the amount by which the effect shrinks every layer; it should be between 0 and 1.
  fn octaves(self, count: usize, zoom: f32, scale: f32) -> Octaves<Self>
  where
    Self: Sized
  {
    assert!(count > 0);
    assert!(zoom > 1.0);
    assert!(scale > 0.0 && scale < 1.0);
    println!("Making with count={} zoom={} scale={}", count, zoom, scale);
    Octaves { orig: self, count, zoom, scale }
  }
}

pub struct Octaves<N: Noise2D> {
  orig: N,
  count: usize,
  zoom: f32,
  scale: f32,
}

impl<N: Noise2D> Noise2D for Octaves<N> {
  fn get(&self, x: f32, y: f32) -> f32 {
    // base layer
    let mut max = 0.0;
    let mut sum = 0.0;
    let mut zoom = 1.0;
    let mut scale = 1.0;
    for _ in 0..self.count {
      sum += self.orig.get(x * zoom, y * zoom) * scale;
      max += scale;
      zoom *= self.zoom;
      scale *= self.scale;
    }
    sum / max
  }
}
