//! Reads the MNIST files (IDX format, big-endian headers, unpacked).

use nn_core::preprocess::preprocess;
use std::path::Path;

pub struct Dataset {
    /// 784 values per image, 0..1, already preprocessed like a drawing on the site.
    pub pixels: Vec<f32>,
    pub labels: Vec<usize>,
}

#[allow(clippy::len_without_is_empty)]
impl Dataset {
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn image(&self, i: usize) -> &[f32] {
        &self.pixels[i * 784..(i + 1) * 784]
    }

    pub fn append(&mut self, other: Dataset) {
        self.pixels.extend(other.pixels);
        self.labels.extend(other.labels);
    }
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    bytes.get(at..at + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

/// Parses an images file and a labels file that belong together.
pub fn parse(images: &[u8], labels: &[u8]) -> Result<Dataset, String> {
    if be_u32(images, 0) != Some(2051) {
        return Err("not an MNIST images file (did you unpack the .gz?)".into());
    }
    if be_u32(labels, 0) != Some(2049) {
        return Err("not an MNIST labels file (did you unpack the .gz?)".into());
    }
    let count = be_u32(images, 4).ok_or("images file is truncated")? as usize;
    let rows = be_u32(images, 8).ok_or("images file is truncated")? as usize;
    let cols = be_u32(images, 12).ok_or("images file is truncated")? as usize;
    if (rows, cols) != (28, 28) {
        return Err(format!("expected 28×28 images, found {rows}×{cols}"));
    }
    if be_u32(labels, 4) != Some(count as u32) {
        return Err("images and labels files have different counts".into());
    }
    let image_bytes = images.get(16..16 + count * 784).ok_or("images file is truncated")?;
    let label_bytes = labels.get(8..8 + count).ok_or("labels file is truncated")?;
    if label_bytes.iter().any(|l| *l > 9) {
        return Err("labels file contains something that is not a digit".into());
    }

    let mut pixels = Vec::with_capacity(count * 784);
    let mut buffer = [0.0f32; 784];
    for image in image_bytes.chunks_exact(784) {
        for (b, v) in buffer.iter_mut().zip(image) {
            *b = *v as f32 / 255.0;
        }
        // The same crop / scale / centre step the site applies to a drawing,
        // so training images and drawings reach the network in the same form.
        pixels.extend(preprocess(&buffer, 28, 28));
    }
    Ok(Dataset { pixels, labels: label_bytes.iter().map(|l| *l as usize).collect() })
}

pub fn load(folder: &Path, prefix: &str) -> Result<Dataset, String> {
    let read = |name: String| {
        let path = folder.join(&name);
        std::fs::read(&path).map_err(|e| format!("{}: {e} (see the README for how to download MNIST)", path.display()))
    };
    let images = read(format!("{prefix}-images-idx3-ubyte"))?;
    let labels = read(format!("{prefix}-labels-idx1-ubyte"))?;
    parse(&images, &labels).map_err(|e| format!("{prefix}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(count: u32) -> (Vec<u8>, Vec<u8>) {
        let mut images = Vec::new();
        for v in [2051u32, count, 28, 28] {
            images.extend(v.to_be_bytes());
        }
        let mut labels = Vec::new();
        for v in [2049u32, count] {
            labels.extend(v.to_be_bytes());
        }
        for i in 0..count {
            let mut image = [0u8; 784];
            // a vertical bar, 20 pixels tall
            for y in 4..24 {
                image[y * 28 + 13] = 255;
                image[y * 28 + 14] = 255;
            }
            images.extend(image);
            labels.push((i % 10) as u8);
        }
        (images, labels)
    }

    #[test]
    fn parses_images_and_labels() {
        let (images, labels) = files(3);
        let data = parse(&images, &labels).unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data.labels, vec![0, 1, 2]);
        assert_eq!(data.image(2).len(), 784);
        assert_eq!(data.image(0)[14 * 28 + 13], 1.0);
        assert_eq!(data.image(0)[14 * 28 + 5], 0.0);
    }

    #[test]
    fn rejects_broken_files() {
        let (images, labels) = files(3);
        assert!(parse(&images[..100], &labels).is_err());
        assert!(parse(&labels, &images).is_err());
        assert!(parse(&images, &files(2).1).is_err());
        let mut bad = labels.clone();
        bad[8] = 42;
        assert!(parse(&images, &bad).is_err());
    }
}
