//! Click-placed, editable step markers and deterministic export rendering.
use crate::{Object, Tool};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum StepShape {
    #[default]
    Circle,
    Square,
    Diamond,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum StepSequence {
    #[default]
    Numbers,
    Uppercase,
    Lowercase,
}
impl StepSequence {
    pub fn value(self, label: &str) -> Option<u32> {
        if self == Self::Numbers {
            return label.parse::<u32>().ok().filter(|n| (1..=9999).contains(n));
        }
        let base = if self == Self::Uppercase { b'A' } else { b'a' };
        if label.is_empty() {
            return None;
        }
        let mut n = 0u32;
        for b in label.bytes() {
            if !(base..=base + 25).contains(&b) {
                return None;
            }
            n = n.checked_mul(26)?.checked_add(u32::from(b - base) + 1)?;
        }
        (n <= 9999).then_some(n)
    }
    pub fn label(self, value: u32) -> String {
        let mut n = value.clamp(1, 9999);
        if self == Self::Numbers {
            return n.to_string();
        }
        let mut letters = Vec::new();
        while n > 0 {
            n -= 1;
            letters.push((if self == Self::Uppercase { b'A' } else { b'a' }) + (n % 26) as u8);
            n /= 26;
        }
        letters.reverse();
        String::from_utf8(letters).unwrap()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepStyle {
    pub shape: StepShape,
    pub text_color: [u8; 4],
    pub border_color: [u8; 4],
    pub border_width: f32,
}
impl Default for StepStyle {
    fn default() -> Self {
        Self {
            shape: StepShape::Circle,
            text_color: [255, 255, 255, 255],
            border_color: [255, 255, 255, 255],
            border_width: 2.,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepPreset {
    pub name: String,
    pub style: StepStyle,
    pub fill: [u8; 4],
    pub size: f32,
    pub font_size: f32,
}
impl Default for StepPreset {
    fn default() -> Self {
        Self {
            name: "Teal".into(),
            style: StepStyle::default(),
            fill: [23, 111, 118, 255],
            size: 56.,
            font_size: 30.,
        }
    }
}
impl StepPreset {
    pub fn valid(&self) -> bool {
        self.name.len() <= 80
            && self.size.is_finite()
            && (24.0..=160.).contains(&self.size)
            && self.font_size.is_finite()
            && (10.0..=80.).contains(&self.font_size)
            && self.style.border_width.is_finite()
            && (0.0..=12.).contains(&self.style.border_width)
    }
    pub fn marker(&self, center: [f32; 2], label: String) -> Object {
        let r = self.size / 2.;
        Object {
            id: 0,
            tool: Tool::Step,
            a: [center[0] - r, center[1] - r],
            b: [center[0] + r, center[1] + r],
            color: self.fill,
            width: self.font_size,
            text: label,
            points: vec![],
            step: Some(self.style.clone()),
        }
    }
}
fn inside(shape: StepShape, x: f32, y: f32, r: f32) -> bool {
    match shape {
        StepShape::Circle => x * x + y * y <= r * r,
        StepShape::Square => x.abs() <= r && y.abs() <= r,
        StepShape::Diamond => x.abs() + y.abs() <= r,
    }
}
pub fn paint(img: &mut RgbaImage, o: &Object) {
    let [x, y, w, h] = o.bounds();
    let cx = x + w / 2.;
    let cy = y + h / 2.;
    let r = w.min(h) / 2.;
    let legacy = StepStyle {
        border_width: 0.,
        ..Default::default()
    };
    let style = o.step.as_ref().unwrap_or(&legacy);
    // Pixel coverage smooths marker edges at export size without changing geometry.
    let offsets = [0.25, 0.75];
    for py in
        ((cy - r - 1.).max(0.) as u32)..((cy + r + 1.).max(0.).min(img.height() as f32) as u32)
    {
        for px in
            ((cx - r - 1.).max(0.) as u32)..((cx + r + 1.).max(0.).min(img.width() as f32) as u32)
        {
            let mut rgb = [0f32; 3];
            let mut alpha = 0.;
            for dy in offsets {
                for dx in offsets {
                    let sx = px as f32 + dx - cx;
                    let sy = py as f32 + dy - cy;
                    if inside(style.shape, sx, sy, r) {
                        let c = if inside(style.shape, sx, sy, (r - style.border_width).max(0.)) {
                            o.color
                        } else {
                            style.border_color
                        };
                        let a = f32::from(c[3]) / 255. / 4.;
                        alpha += a;
                        for k in 0..3 {
                            rgb[k] += f32::from(c[k]) * a;
                        }
                    }
                }
            }
            if alpha > 0. {
                let old = img.get_pixel(px, py);
                let oa = f32::from(old[3]) / 255.;
                let a = alpha + oa * (1. - alpha);
                let mut c = [0; 4];
                for k in 0..3 {
                    c[k] = ((rgb[k] + f32::from(old[k]) * oa * (1. - alpha)) / a).round() as u8;
                }
                c[3] = (a * 255.).round() as u8;
                img.put_pixel(px, py, Rgba(c));
            }
        }
    }
    let font = if o.step.is_none() {
        crate::font()
    } else {
        ab_glyph::FontRef::try_from_slice(include_bytes!("../assets/Ubuntu-Bold.ttf")).unwrap()
    };
    let mut size = if o.step.is_none() { 25. } else { o.width };
    let max_width = if style.shape == StepShape::Diamond {
        r * 0.95
    } else {
        r * 1.55
    };
    let (tw, th) = imageproc::drawing::text_size(size, &font, &o.text);
    size *= (max_width / tw.max(1) as f32)
        .min((r * 1.35) / th.max(1) as f32)
        .min(1.);
    let (tw, th) = imageproc::drawing::text_size(size, &font, &o.text);
    imageproc::drawing::draw_text_mut(
        img,
        Rgba(style.text_color),
        (cx - tw as f32 / 2.).round() as i32,
        (cy - th as f32 / 2.).round() as i32,
        size,
        &font,
        &o.text,
    );
}
