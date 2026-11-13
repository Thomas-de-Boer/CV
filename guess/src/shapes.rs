//! How digits are written, described as named strokes.
//!
//! This is the "knowledge" behind the guessed weights: nothing here
//! comes from data. Every digit is a few strokes in a unit box (x to the
//! right, y downwards, both 0..1), plus how wide the digit usually is.
//! Angles are in degrees on screen: 0 = right, 90 = down, 180 = left, 270 = up.

use crate::raster::{Canvas, Pt};

pub const CANVAS: usize = 280;

pub struct Piece {
    /// Name of the stroke (for readability only; the network never sees it).
    pub name: &'static str,
    pub detector: bool,
    pub pts: Vec<Pt>,
}

pub struct Shape {
    /// Typical width ÷ height.
    pub aspect: f32,
    pub pieces: Vec<Piece>,
}

fn arc(cx: f32, cy: f32, rx: f32, ry: f32, from: f32, to: f32) -> Vec<Pt> {
    let steps = ((to - from).abs() / 10.0).ceil().max(1.0) as usize;
    (0..=steps)
        .map(|i| {
            let a = (from + (to - from) * i as f32 / steps as f32).to_radians();
            (cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

fn line(a: Pt, b: Pt) -> Vec<Pt> {
    (0..=8).map(|i| (a.0 + (b.0 - a.0) * i as f32 / 8.0, a.1 + (b.1 - a.1) * i as f32 / 8.0)).collect()
}

/// Quadratic Bézier curve from `a` to `b`, pulled towards `ctrl`.
fn curve(a: Pt, ctrl: Pt, b: Pt) -> Vec<Pt> {
    (0..=12)
        .map(|i| {
            let t = i as f32 / 12.0;
            let u = 1.0 - t;
            (u * u * a.0 + 2.0 * u * t * ctrl.0 + t * t * b.0, u * u * a.1 + 2.0 * u * t * ctrl.1 + t * t * b.1)
        })
        .collect()
}

fn det(name: &'static str, pts: Vec<Pt>) -> Piece {
    Piece { name, detector: true, pts }
}

fn plain(name: &'static str, pts: Vec<Pt>) -> Piece {
    Piece { name, detector: false, pts }
}

/// How many ways of writing `digit` are described.
pub fn variant_count(digit: usize) -> usize {
    match digit {
        1 => 3,
        4 | 7 | 9 => 2,
        _ => 1,
    }
}

pub fn shape(digit: usize, variant: usize) -> Shape {
    match (digit, variant) {
        (0, _) => Shape {
            aspect: 0.68,
            pieces: vec![
                det("0: top arc", arc(0.5, 0.5, 0.5, 0.5, 215.0, 325.0)),
                det("0: right arc", arc(0.5, 0.5, 0.5, 0.5, -55.0, 55.0)),
                det("0: bottom arc", arc(0.5, 0.5, 0.5, 0.5, 35.0, 145.0)),
                det("0: left arc", arc(0.5, 0.5, 0.5, 0.5, 125.0, 235.0)),
            ],
        },
        // A plain vertical line.
        (1, 0) => Shape {
            aspect: 0.02,
            pieces: vec![
                det("1: upper stem", line((0.5, 0.0), (0.5, 0.5))),
                det("1: lower stem", line((0.5, 0.5), (0.5, 1.0))),
            ],
        },
        // With a flag at the top.
        (1, 1) => Shape {
            aspect: 0.3,
            pieces: vec![
                det("1: flag", line((0.0, 0.3), (0.65, 0.0))),
                plain("stem", line((0.65, 0.0), (0.65, 1.0))),
            ],
        },
        // With a flag and a base.
        (1, _) => Shape {
            aspect: 0.5,
            pieces: vec![
                plain("flag", line((0.05, 0.27), (0.55, 0.0))),
                plain("stem", line((0.55, 0.0), (0.55, 1.0))),
                plain("base", line((0.1, 1.0), (1.0, 1.0))),
            ],
        },
        (2, _) => Shape {
            aspect: 0.7,
            pieces: vec![
                det("2: top hook", arc(0.5, 0.27, 0.45, 0.27, 195.0, 390.0)),
                det("2: diagonal", line((0.89, 0.405), (0.03, 1.0))),
                det("2: base bar", line((0.03, 1.0), (1.0, 1.0))),
            ],
        },
        (3, _) => Shape {
            aspect: 0.6,
            pieces: vec![
                det("3: upper bowl", arc(0.4, 0.25, 0.5, 0.25, 215.0, 450.0)),
                det("3: lower bowl", arc(0.4, 0.75, 0.6, 0.25, 270.0, 510.0)),
            ],
        },
        // Open top.
        (4, 0) => Shape {
            aspect: 0.7,
            pieces: vec![
                det("4: left stem", line((0.12, 0.0), (0.0, 0.62))),
                det("4: crossbar", line((0.0, 0.62), (1.0, 0.62))),
                det("4: right stem, upper", line((0.68, 0.0), (0.68, 0.62))),
                det("4: right stem, lower", line((0.68, 0.62), (0.68, 1.0))),
            ],
        },
        // Closed (pointed) top.
        (4, _) => Shape {
            aspect: 0.7,
            pieces: vec![
                det("4: diagonal roof", line((0.68, 0.0), (0.0, 0.65))),
                plain("crossbar", line((0.0, 0.65), (1.0, 0.65))),
                plain("stem", line((0.68, 0.0), (0.68, 1.0))),
            ],
        },
        (5, _) => Shape {
            aspect: 0.65,
            pieces: vec![
                det("5: top bar", line((0.95, 0.0), (0.2, 0.0))),
                det("5: upper-left stem", line((0.2, 0.0), (0.105, 0.418))),
                det("5: lower bowl", arc(0.42, 0.68, 0.55, 0.32, 235.0, 500.0)),
            ],
        },
        (6, _) => Shape {
            aspect: 0.62,
            pieces: vec![
                det("6: upper sweep", curve((0.8, 0.0), (0.15, 0.15), (0.08, 0.68))),
                det("6: loop, left half", arc(0.5, 0.7, 0.42, 0.3, 90.0, 270.0)),
                det("6: loop, right half", arc(0.5, 0.7, 0.42, 0.3, -90.0, 90.0)),
            ],
        },
        (7, 0) => Shape {
            aspect: 0.7,
            pieces: vec![
                det("7: top bar", line((0.0, 0.0), (1.0, 0.0))),
                det("7: diagonal, upper", line((1.0, 0.0), (0.7, 0.5))),
                det("7: diagonal, lower", line((0.7, 0.5), (0.4, 1.0))),
            ],
        },
        // With a crossbar through the diagonal.
        (7, _) => Shape {
            aspect: 0.7,
            pieces: vec![
                plain("top bar", line((0.0, 0.0), (1.0, 0.0))),
                plain("diagonal", line((1.0, 0.0), (0.4, 1.0))),
                plain("crossbar", line((0.35, 0.52), (1.0, 0.52))),
            ],
        },
        (8, _) => Shape {
            aspect: 0.58,
            pieces: vec![
                det("8: upper loop, left half", arc(0.5, 0.24, 0.4, 0.24, 90.0, 270.0)),
                det("8: upper loop, right half", arc(0.5, 0.24, 0.4, 0.24, -90.0, 90.0)),
                det("8: lower loop, left half", arc(0.5, 0.74, 0.5, 0.26, 90.0, 270.0)),
                det("8: lower loop, right half", arc(0.5, 0.74, 0.5, 0.26, -90.0, 90.0)),
            ],
        },
        // Straight tail.
        (9, 0) => Shape {
            aspect: 0.6,
            pieces: vec![
                det("9: loop, left half", arc(0.5, 0.28, 0.45, 0.28, 90.0, 270.0)),
                det("9: loop, right half", arc(0.5, 0.28, 0.45, 0.28, -90.0, 90.0)),
                det("9: tail", line((0.95, 0.28), (0.85, 1.0))),
            ],
        },
        // Tail curling to the left.
        (9, _) => Shape {
            aspect: 0.6,
            pieces: vec![
                plain("loop", arc(0.5, 0.28, 0.45, 0.28, 0.0, 360.0)),
                plain("tail", curve((0.95, 0.28), (0.95, 0.95), (0.25, 1.0))),
            ],
        },
        _ => panic!("no shape for digit {digit}"),
    }
}

/// How a shape is drawn on the 280×280 canvas.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// Height of the digit in canvas pixels.
    pub height: f32,
    /// Multiplies the shape's usual width.
    pub widen: f32,
    /// Shear: positive leans to the right, like most handwriting.
    pub slant: f32,
    /// Brush radius in canvas pixels.
    pub radius: f32,
    /// Rotation in radians.
    pub rotate: f32,
    /// Size of a smooth wobble, as a fraction of the height.
    pub wobble: f32,
    pub seed: u32,
    pub centre: Pt,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            height: 190.0,
            widen: 1.0,
            slant: 0.0,
            radius: 12.0,
            rotate: 0.0,
            wobble: 0.0,
            seed: 0,
            centre: (140.0, 140.0),
        }
    }
}

/// A shape's strokes in canvas coordinates.
pub fn place(shape: &Shape, style: &Style) -> Vec<Vec<Pt>> {
    let h = style.height;
    let w = h * shape.aspect * style.widen;
    let phase = |k: u32| (style.seed.wrapping_mul(2654435761).wrapping_add(k * 40503) % 1000) as f32 / 1000.0;
    let tau = std::f32::consts::TAU;
    let (sin, cos) = style.rotate.sin_cos();
    shape
        .pieces
        .iter()
        .map(|piece| {
            piece
                .pts
                .iter()
                .map(|&(u, v)| {
                    let mut x = (u - 0.5) * w + style.slant * (0.5 - v) * h;
                    let mut y = (v - 0.5) * h;
                    x += style.wobble * h * (tau * (1.3 * v + phase(1))).sin();
                    y += style.wobble * h * (tau * (1.1 * u + phase(2))).sin();
                    (style.centre.0 + x * cos - y * sin, style.centre.1 + x * sin + y * cos)
                })
                .collect()
        })
        .collect()
}

/// Draws a shape on a 280×280 canvas, like a visitor would.
pub fn render(shape: &Shape, style: &Style) -> Vec<f32> {
    let mut canvas = Canvas::new(CANVAS, CANVAS);
    for stroke in place(shape, style) {
        canvas.stroke(&stroke, style.radius);
    }
    canvas.px
}
