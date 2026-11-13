//! Drawing strokes into a pixel buffer, and blurring one.

pub type Pt = (f32, f32);

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Canvas { w, h, px: vec![0.0; w * h] }
    }

    /// Draws a polyline with a round brush of the given radius (anti-aliased).
    pub fn stroke(&mut self, pts: &[Pt], radius: f32) {
        if pts.len() == 1 {
            self.segment(pts[0], pts[0], radius);
        }
        for pair in pts.windows(2) {
            self.segment(pair[0], pair[1], radius);
        }
    }

    fn segment(&mut self, a: Pt, b: Pt, radius: f32) {
        let reach = radius + 1.0;
        let x0 = (a.0.min(b.0) - reach).floor().max(0.0) as usize;
        let y0 = (a.1.min(b.1) - reach).floor().max(0.0) as usize;
        let x1 = ((a.0.max(b.0) + reach).ceil().max(0.0) as usize).min(self.w);
        let y1 = ((a.1.max(b.1) + reach).ceil().max(0.0) as usize).min(self.h);
        for y in y0..y1 {
            for x in x0..x1 {
                let d = distance_to_segment((x as f32 + 0.5, y as f32 + 0.5), a, b);
                let v = (radius + 0.5 - d).clamp(0.0, 1.0);
                let p = &mut self.px[y * self.w + x];
                *p = p.max(v);
            }
        }
    }
}

fn distance_to_segment(p: Pt, a: Pt, b: Pt) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

/// Separable Gaussian blur (zero outside the image).
pub fn blur(img: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    let reach = (sigma * 3.0).ceil() as isize;
    let kernel: Vec<f32> = (-reach..=reach).map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp()).collect();
    let total: f32 = kernel.iter().sum();

    let pass = |src: &[f32], horizontal: bool| -> Vec<f32> {
        let mut dst = vec![0.0; w * h];
        for y in 0..h as isize {
            for x in 0..w as isize {
                let mut sum = 0.0;
                for (k, weight) in kernel.iter().enumerate() {
                    let o = k as isize - reach;
                    let (sx, sy) = if horizontal { (x + o, y) } else { (x, y + o) };
                    if sx >= 0 && sy >= 0 && sx < w as isize && sy < h as isize {
                        sum += src[sy as usize * w + sx as usize] * weight;
                    }
                }
                dst[y as usize * w + x as usize] = sum / total;
            }
        }
        dst
    };
    pass(&pass(img, true), false)
}

pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// A terminal picture of a 28×28 image, for debugging.
pub fn ascii(img: &[f32], w: usize) -> String {
    let shades = [' ', '.', ':', 'o', '#'];
    let mut s = String::new();
    for row in img.chunks(w) {
        for v in row {
            s.push(shades[((v.clamp(0.0, 1.0) * 4.0).round()) as usize]);
        }
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_has_the_requested_thickness() {
        let mut c = Canvas::new(20, 20);
        c.stroke(&[(2.0, 10.0), (18.0, 10.0)], 1.5);
        let column: f32 = (0..20).map(|y| c.px[y * 20 + 10]).sum();
        assert!((column - 3.0).abs() < 0.6, "ink across the stroke = {column}");
        assert_eq!(c.px[2 * 20 + 10], 0.0);
    }

    #[test]
    fn blur_keeps_the_total_away_from_edges() {
        let mut img = vec![0.0; 28 * 28];
        img[14 * 28 + 14] = 1.0;
        let b = blur(&img, 28, 28, 2.0);
        assert!((b.iter().sum::<f32>() - 1.0).abs() < 1e-3);
        assert!(b[14 * 28 + 14] > b[14 * 28 + 16]);
    }
}
