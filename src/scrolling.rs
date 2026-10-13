//! Original bounded vertical panorama compositor.
use crate::Result;
use image::RgbaImage;
pub struct Panorama {
    pixels: RgbaImage,
    previous: RgbaImage,
}
impl Panorama {
    pub fn new(first: RgbaImage) -> Result<Self> {
        if first.width() < 32
            || first.height() < 32
            || first.width() > 16000
            || first.height() > 16000
            || u64::from(first.width()) * u64::from(first.height()) > 40_000_000
        {
            return Err("Scrolling area must be at least 32 pixels per side and within the image size limit.".into());
        }
        Ok(Self {
            previous: first.clone(),
            pixels: first,
        })
    }
    pub fn append(&mut self, next: RgbaImage) -> Result<bool> {
        if next.dimensions() != self.previous.dimensions() {
            return Err("Scrolling area changed size. Start a new capture.".into());
        }
        if next == self.previous {
            return Ok(false);
        }
        let (w, h) = next.dimensions();
        let mut best = (f64::INFINITY, 0);
        let stride_x = (w / 100).max(1);
        let stride_y = (h / 100).max(1);
        for shift in 0..=h * 4 / 5 {
            let mut error = 0_u64;
            let mut count = 0_u64;
            let mut detail = 0_u64;
            for y in (0..h - shift).step_by(stride_y as usize) {
                for x in (2..w - 2).step_by(stride_x as usize) {
                    let a = self.previous.get_pixel(x, y + shift);
                    let b = next.get_pixel(x, y);
                    let neighbor = next.get_pixel(x + 1, y);
                    if (0..3).any(|c| b[c].abs_diff(neighbor[c]) > 20) {
                        detail += 1;
                    }
                    for c in 0..3 {
                        error += u64::from(a[c].abs_diff(b[c]));
                        count += 1;
                    }
                }
            }
            if detail < 20 {
                continue;
            }
            let score = error as f64 / count.max(1) as f64;
            if score < best.0 {
                best = (score, shift);
            }
        }
        if best.0 > 4.0 {
            return Err("Frames do not align. Scroll downward slowly, keep at least 20% overlap, and exclude fixed headers and scrollbars.".into());
        }
        let shift = best.1;
        if shift == 0 {
            self.previous = next;
            return Ok(false);
        }
        let new_height = self.pixels.height() + shift;
        if new_height > 16000 || u64::from(w) * u64::from(new_height) > 40_000_000 {
            return Err(
                "Scrolling image reached the 40 million pixel / 16000 pixel height limit.".into(),
            );
        }
        let mut joined = RgbaImage::new(w, new_height);
        image::imageops::overlay(&mut joined, &self.pixels, 0, 0);
        let tail = image::imageops::crop_imm(&next, 0, h - shift, w, shift).to_image();
        image::imageops::overlay(&mut joined, &tail, 0, i64::from(self.pixels.height()));
        self.pixels = joined;
        self.previous = next;
        Ok(true)
    }
    pub fn finish(self) -> RgbaImage {
        self.pixels
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn content() -> RgbaImage {
        RgbaImage::from_fn(120, 240, |x, y| {
            image::Rgba([
                ((x * 31 + y * 17) % 251) as u8,
                ((y * 53 + x * 11) % 253) as u8,
                ((x * 7 + y * 29) % 255) as u8,
                255,
            ])
        })
    }
    fn frame(source: &RgbaImage, y: u32) -> RgbaImage {
        image::imageops::crop_imm(source, 0, y, 120, 100).to_image()
    }
    #[test]
    fn overlapping_scrolls_preserve_every_pixel_and_do_not_duplicate_rows() {
        let source = content();
        let mut p = Panorama::new(frame(&source, 0)).unwrap();
        assert!(p.append(frame(&source, 60)).unwrap());
        assert!(p.append(frame(&source, 110)).unwrap());
        assert_eq!(
            p.finish(),
            image::imageops::crop_imm(&source, 0, 0, 120, 210).to_image()
        );
    }
    #[test]
    fn repeated_frame_does_not_increase_output_height() {
        let frame = frame(&content(), 0);
        let mut p = Panorama::new(frame.clone()).unwrap();
        assert!(!p.append(frame.clone()).unwrap());
        assert_eq!(p.finish(), frame);
    }
    #[test]
    fn gaps_or_changed_dimensions_are_rejected() {
        let source = content();
        let mut p = Panorama::new(frame(&source, 0)).unwrap();
        assert!(p.append(frame(&source, 130)).is_err());
        assert!(p.append(RgbaImage::new(119, 100)).is_err());
    }
    #[test]
    fn flat_changed_content_is_not_guessed_as_an_overlap() {
        let mut p = Panorama::new(RgbaImage::from_pixel(
            120,
            100,
            image::Rgba([255, 255, 255, 255]),
        ))
        .unwrap();
        assert!(
            p.append(RgbaImage::from_pixel(
                120,
                100,
                image::Rgba([210, 210, 210, 255])
            ))
            .is_err()
        );
    }
    #[test]
    fn oversized_initial_frame_is_rejected() {
        assert!(Panorama::new(RgbaImage::new(16001, 3)).is_err());
    }
}
