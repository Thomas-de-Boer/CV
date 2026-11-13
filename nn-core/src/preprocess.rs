//! Turns a drawing into the 28×28 image the network sees (MNIST style):
//!
//! 1. find the bounding box of the ink,
//! 2. scale it so the longest side fits a 20×20 box (aspect ratio kept),
//! 3. place it in a 28×28 image, centred by centre of mass.
//!
//! Lives in `nn-core` (it is plain maths on a pixel buffer) so the browser,
//! the tests and the hand-designed weights all share exactly one version.

pub const SIZE: usize = 28;
pub const BOX: usize = 20;

/// Pixels at or below this value count as background.
const INK_THRESHOLD: f32 = 0.1;

/// Where a drawing ended up: source `(x, y)` maps to
/// `(x * scale + off_x, y * scale + off_y)` in 28×28 space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub scale: f32,
    pub off_x: f32,
    pub off_y: f32,
}

impl Placement {
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.scale + self.off_x, y * self.scale + self.off_y)
    }
}

/// `pixels`: `width × height` values in 0..=1, row-major, ink = high.
/// Returns 784 values in 0..=1 (all zero for an empty drawing).
pub fn preprocess(pixels: &[f32], width: usize, height: usize) -> Vec<f32> {
    preprocess_with_placement(pixels, width, height).0
}

pub fn preprocess_with_placement(
    pixels: &[f32],
    width: usize,
    height: usize,
) -> (Vec<f32>, Option<Placement>) {
    assert_eq!(pixels.len(), width * height, "pixel buffer has the wrong length");
    let mut out = vec![0.0; SIZE * SIZE];

    // 1. Bounding box of the ink.
    let (mut x0, mut y0, mut x1, mut y1) = (width, height, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if pixels[y * width + x] > INK_THRESHOLD {
                x0 = x0.min(x);
                x1 = x1.max(x + 1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return (out, None);
    }
    let (bw, bh) = (x1 - x0, y1 - y0);

    // 2. Scale so the longest side becomes 20 pixels.
    let scale = BOX as f32 / bw.max(bh) as f32;
    let tw = ((bw as f32 * scale).round() as usize).clamp(1, BOX);
    let th = ((bh as f32 * scale).round() as usize).clamp(1, BOX);
    let cols = spans(x0, bw, tw);
    let rows = spans(y0, bh, th);
    let mut small = vec![0.0f32; tw * th];
    for (ty, row_span) in rows.iter().enumerate() {
        for (tx, col_span) in cols.iter().enumerate() {
            let mut sum = 0.0;
            for &(sy, wy) in row_span {
                for &(sx, wx) in col_span {
                    sum += pixels[sy * width + sx] * wx * wy;
                }
            }
            small[ty * tw + tx] = sum.clamp(0.0, 1.0);
        }
    }

    // 3. Centre of mass goes to the middle of the 28×28 image.
    let (mut mass, mut mx, mut my) = (0.0f32, 0.0f32, 0.0f32);
    for ty in 0..th {
        for tx in 0..tw {
            let v = small[ty * tw + tx];
            mass += v;
            mx += v * (tx as f32 + 0.5);
            my += v * (ty as f32 + 0.5);
        }
    }
    let (cx, cy) = if mass > 0.0 { (mx / mass, my / mass) } else { (tw as f32 / 2.0, th as f32 / 2.0) };
    let half = SIZE as f32 / 2.0;
    let ox = ((half - cx).round() as isize).clamp(0, (SIZE - tw) as isize) as usize;
    let oy = ((half - cy).round() as isize).clamp(0, (SIZE - th) as isize) as usize;
    for ty in 0..th {
        for tx in 0..tw {
            out[(oy + ty) * SIZE + ox + tx] = small[ty * tw + tx];
        }
    }

    let sx = tw as f32 / bw as f32;
    let sy = th as f32 / bh as f32;
    let placement = Placement {
        scale: (sx + sy) / 2.0,
        off_x: ox as f32 - x0 as f32 * sx,
        off_y: oy as f32 - y0 as f32 * sy,
    };
    (out, Some(placement))
}

/// Box-filter resampling along one axis: for each of the `target` output
/// pixels, the source pixels it covers and their weights (summing to 1).
fn spans(start: usize, len: usize, target: usize) -> Vec<Vec<(usize, f32)>> {
    let step = len as f32 / target as f32;
    (0..target)
        .map(|t| {
            let a = t as f32 * step;
            let b = (t + 1) as f32 * step;
            let first = a.floor() as usize;
            let last = (b.ceil() as usize).min(len).max(first + 1);
            (first..last)
                .map(|s| {
                    let covered = (b.min(s as f32 + 1.0) - a.max(s as f32)).max(0.0);
                    (start + s, covered / step)
                })
                .filter(|&(_, w)| w > 0.0)
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Draws a filled "L" shape: asymmetric, so centring is actually tested.
    fn draw_l(width: usize, height: usize, left: usize, top: usize, unit: usize) -> Vec<f32> {
        let mut px = vec![0.0; width * height];
        let mut rect = |x0: usize, y0: usize, w: usize, h: usize| {
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    px[y * width + x] = 1.0;
                }
            }
        };
        rect(left, top, unit, unit * 5); // stem
        rect(left, top + unit * 4, unit * 3, unit); // foot
        px
    }

    fn mean_abs_diff(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.len() as f32
    }

    #[test]
    fn empty_drawing_gives_empty_image() {
        let (out, placement) = preprocess_with_placement(&vec![0.0; 100 * 100], 100, 100);
        assert_eq!(out.len(), SIZE * SIZE);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(placement.is_none());
    }

    #[test]
    fn longest_side_fills_twenty_pixels() {
        let out = preprocess(&draw_l(280, 280, 60, 40, 30), 280, 280);
        let rows: Vec<usize> = (0..SIZE).filter(|y| (0..SIZE).any(|x| out[y * SIZE + x] > 0.1)).collect();
        let cols: Vec<usize> = (0..SIZE).filter(|x| (0..SIZE).any(|y| out[y * SIZE + x] > 0.1)).collect();
        assert_eq!(rows.len(), 20);
        assert_eq!(cols.len(), 12); // 3 units wide vs 5 tall
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    #[test]
    fn centre_of_mass_lands_in_the_middle() {
        let out = preprocess(&draw_l(280, 280, 20, 100, 24), 280, 280);
        let mass: f32 = out.iter().sum();
        let cx: f32 = (0..SIZE * SIZE).map(|i| out[i] * ((i % SIZE) as f32 + 0.5)).sum::<f32>() / mass;
        let cy: f32 = (0..SIZE * SIZE).map(|i| out[i] * ((i / SIZE) as f32 + 0.5)).sum::<f32>() / mass;
        assert!((cx - 14.0).abs() <= 0.5, "cx = {cx}");
        assert!((cy - 14.0).abs() <= 0.5, "cy = {cy}");
    }

    #[test]
    fn shifting_the_drawing_changes_nothing() {
        let a = preprocess(&draw_l(280, 280, 20, 30, 30), 280, 280);
        let b = preprocess(&draw_l(280, 280, 150, 110, 30), 280, 280);
        assert!(mean_abs_diff(&a, &b) < 1e-6);
    }

    #[test]
    fn scaling_the_drawing_changes_almost_nothing() {
        let a = preprocess(&draw_l(280, 280, 20, 30, 40), 280, 280);
        let b = preprocess(&draw_l(280, 280, 100, 100, 17), 280, 280);
        assert!(mean_abs_diff(&a, &b) < 0.01, "diff = {}", mean_abs_diff(&a, &b));
    }

    #[test]
    fn placement_maps_the_bounding_box_into_the_image() {
        let (_, placement) = preprocess_with_placement(&draw_l(280, 280, 60, 40, 30), 280, 280);
        let p = placement.unwrap();
        let (x0, y0) = p.apply(60.0, 40.0);
        let (x1, y1) = p.apply(150.0, 190.0);
        assert!((y1 - y0 - 20.0).abs() < 0.01);
        assert!((x1 - x0 - 12.0).abs() < 0.01);
        assert!(x0 >= 0.0 && y0 >= 0.0 && x1 <= 28.0 && y1 <= 28.0);
    }
}
