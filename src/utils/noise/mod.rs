mod checkerboard;
pub use checkerboard::Checkerboard;
mod worley;
pub use worley::Worley;

// #[allow(unused_imports)]
use std::ops::*;

pub enum Quadrant {
  NW = 0,
  NE = 1,
  SE = 2,
  SW = 3,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Pos {
  pub x: f32,
  pub y: f32,
}

#[allow(unused)]
impl Pos {
  pub fn zero() -> Self {
    Pos { x: 0.0, y: 0.0 }
  }

  pub fn of(x: f32, y: f32) -> Self {
    Pos { x, y }
  }

  pub fn quadrant(&self) -> Quadrant {
    if self.x > 0.0 {
      if self.y > 0.0 {
        Quadrant::NW
      } else {
        Quadrant::SW
      }
    } else {
      if self.y > 0.0 {
        Quadrant::NE
      } else {
        Quadrant::SE
      }
    }
  }

  pub fn len_sq(&self) -> f32 {
    self.x * self.x + self.y * self.y
  }

  pub fn len(&self) -> f32 {
    self.len_sq().sqrt()
  }
}

macro_rules! impl_pos_op {
  (normal, $trait:ident, $fn:ident, $op:tt) => {
    impl $trait<Self> for Pos {
      type Output = Self;
      fn $fn(self, rhs: Self) -> Self {
        Pos {
          x: self.x $op rhs.x,
          y: self.y $op rhs.y,
        }
      }
    }
    impl $trait<f32> for Pos {
      type Output = Self;
      fn $fn(self, rhs: f32) -> Self {
        Pos {
          x: self.x $op rhs,
          y: self.y $op rhs,
        }
      }
    }
  };
  (assign, $trait:ident, $fn:ident, $op:tt) => {
    impl $trait<Self> for Pos {
      fn $fn(&mut self, rhs: Self) {
        self.x $op rhs.x;
        self.y $op rhs.y;
      }
    }
    impl $trait<f32> for Pos {
      fn $fn(&mut self, rhs: f32) {
        self.x $op rhs;
        self.y $op rhs;
      }
    }
  };
}

impl_pos_op!(normal, Add, add, +);
impl_pos_op!(normal, Sub, sub, -);
impl_pos_op!(normal, Mul, mul, *);
impl_pos_op!(normal, Div, div, /);
impl_pos_op!(normal, Rem, rem, %);
impl_pos_op!(assign, AddAssign, add_assign, +=);
impl_pos_op!(assign, SubAssign, sub_assign, -=);
impl_pos_op!(assign, MulAssign, mul_assign, *=);
impl_pos_op!(assign, DivAssign, div_assign, /=);
impl_pos_op!(assign, RemAssign, rem_assign, %=);

impl Neg for Pos {
  type Output = Pos;
  fn neg(self) -> Self {
    Pos {
      x: -self.x,
      y: -self.y,
    }
  }
}

impl From<f32> for Pos {
  fn from(n: f32) -> Pos {
    Pos::of(n, n)
  }
}

pub trait Noise2D {
  /// Sample a point somewhere on the plane.
  /// Some Noise2Ds may have restrictions on the input coordinates.
  /// They should always output between 0 and 1; there may be unpredictable errors otherwise.
  fn get(&self, p: Pos) -> f32;

  /// Layer the same noise multiple times over itself, each time more detailed and with less effect.
  /// Variables really _should_ be set with the setters in Octaves.
  fn octaves(self) -> Octaves<Self>
  where
    Self: Sized
  {
    Octaves { orig: self, count: 0, zoom: 0.0, scale: 0.0, offset: Pos::zero() }
  }

  fn invert(self) -> Invert<Self>
  where
    Self: Sized
  {
    Invert { orig: self }
  }
}

pub struct Octaves<N: Noise2D> {
  orig: N,
  count: usize,
  zoom: f32,
  scale: f32,
  offset: Pos,
}

macro_rules! fluent_setters {
  ($($name:ident: $type:ty),+) => {
    $(
      pub fn $name(mut self, new: $type) -> Self {
        self.$name = new;
        self
      }
    )+
  };
}

impl<N: Noise2D> Octaves<N> {
  fluent_setters!{ count: usize, zoom: f32, scale: f32, offset: Pos }
}

impl<N: Noise2D> Noise2D for Octaves<N> {
  fn get(&self, p: Pos) -> f32 {
    // base layer
    let mut max = 0.0;
    let mut sum = 0.0;
    let mut zoom = 1.0;
    let mut scale = 1.0;
    let mut offset = Pos::zero();
    for _ in 0..self.count {
      sum += self.orig.get((p + offset) * zoom) * scale;
      max += scale;
      zoom *= self.zoom;
      scale *= self.scale;
      offset += self.offset
    }
    sum / max
  }
}

pub struct Invert<N: Noise2D> {
  orig: N,
}

impl<N: Noise2D> Noise2D for Invert<N> {
  fn get(&self, p: Pos) -> f32 {
    1.0 - self.orig.get(p)
  }
}
