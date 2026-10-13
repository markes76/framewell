//! Original Framewell document and rendering engine. No vendor implementation code.
use image::{Rgba, RgbaImage};
use imageproc::drawing::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Arrow,
    Rectangle,
    Highlight,
    Redact,
    Text,
    Callout,
    Step,
    Pen,
    Blur,
}
impl Tool {
    pub const ALL: [Tool; 10] = [
        Self::Select,
        Self::Arrow,
        Self::Rectangle,
        Self::Highlight,
        Self::Redact,
        Self::Text,
        Self::Callout,
        Self::Step,
        Self::Pen,
        Self::Blur,
    ];
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    pub id: u64,
    pub tool: Tool,
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub color: [u8; 4],
    pub width: f32,
    pub text: String,
    pub points: Vec<[f32; 2]>,
}
impl Object {
    pub fn bounds(&self) -> [f32; 4] {
        [
            self.a[0].min(self.b[0]),
            self.a[1].min(self.b[1]),
            (self.a[0] - self.b[0]).abs().max(1.),
            (self.a[1] - self.b[1]).abs().max(1.),
        ]
    }
    pub fn hit(&self, p: [f32; 2]) -> bool {
        fn distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = dx * dx + dy * dy;
            let t = if length > 0. {
                ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length
            } else {
                0.
            }
            .clamp(0., 1.);
            ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
        }
        if self.tool == Tool::Arrow {
            return distance(p, self.a, self.b) <= self.width / 2. + 8.;
        }
        if self.tool == Tool::Pen {
            return self
                .points
                .windows(2)
                .any(|s| distance(p, s[0], s[1]) <= self.width / 2. + 8.);
        }
        let [x, y, w, h] = self.bounds();
        p[0] >= x - 8. && p[0] <= x + w + 8. && p[1] >= y - 8. && p[1] <= y + h + 8.
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub title: String,
    pub objects: Vec<Object>,
    pub caption: String,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            title: "Untitled capture".into(),
            objects: vec![],
            caption: String::new(),
        }
    }
}
#[derive(Clone)]
pub struct State {
    pub pixels: Arc<RgbaImage>,
    pub document: Document,
}
pub struct Editor {
    pub state: State,
    undo: Vec<State>,
    redo: Vec<State>,
    pub revision: u64,
}
impl Editor {
    pub fn new(pixels: RgbaImage) -> Self {
        Self {
            state: State {
                pixels: Arc::new(pixels),
                document: Document::default(),
            },
            undo: vec![],
            redo: vec![],
            revision: 0,
        }
    }
    pub fn checkpoint(&mut self) {
        self.undo.push(self.state.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.revision += 1;
    }
    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(self.state.clone());
            self.state = s;
            self.revision += 1;
        }
    }
    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(self.state.clone());
            self.state = s;
            self.revision += 1;
        }
    }
    pub fn add(&mut self, mut o: Object) {
        self.checkpoint();
        let ids: std::collections::HashSet<_> =
            self.state.document.objects.iter().map(|o| o.id).collect();
        let mut id = ids
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .unwrap_or(1);
        while ids.contains(&id) {
            id = id.checked_add(1).unwrap_or(1);
        }
        o.id = id;
        self.state.document.objects.push(o);
    }
    pub fn render(&self) -> RgbaImage {
        render(&self.state.pixels, &self.state.document.objects)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        // Each commit has an immutable PNG asset. Publish the manifest last atomically.
        let name = format!("source-{}.png", nonce());
        self.state.pixels.save(path.join(&name))?;
        let manifest = serde_json::json!({"document":self.state.document,"source":name});
        let tmp = path.join(format!(".document-{}.tmp", nonce()));
        fs::write(&tmp, serde_json::to_vec_pretty(&manifest)?)?;
        fs::File::open(&tmp)?.sync_all()?;
        fs::rename(tmp, path.join("document.json"))?;
        Ok(())
    }
    pub fn open(path: &Path) -> Result<Self> {
        let manifest_path = path.join("document.json");
        if fs::metadata(&manifest_path)?.len() > 8 * 1024 * 1024 {
            return Err("Project manifest is too large".into());
        }
        let v: serde_json::Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
        let document: Document = serde_json::from_value(v["document"].clone())?;
        if document.version != 1 || document.objects.len() > 5000 {
            return Err("Unsupported project version or object count".into());
        }
        let source = v["source"].as_str().ok_or("Missing source")?;
        if Path::new(source).components().count() != 1
            || !source.starts_with("source-")
            || !source.ends_with(".png")
        {
            return Err("Invalid project asset path".into());
        }
        let asset = path.join(source);
        if fs::symlink_metadata(&asset)?.file_type().is_symlink() {
            return Err("Project asset cannot be a symbolic link".into());
        }
        let pixels = load_image(&asset)?;
        validate(&document, pixels.width(), pixels.height())?;
        let mut e = Self::new(pixels);
        e.state.document = document;
        Ok(e)
    }
}
pub fn nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
pub fn load_image(path: &Path) -> Result<RgbaImage> {
    let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16000);
    limits.max_image_height = Some(16000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode()?.into_rgba8();
    if image.width() as u64 * image.height() as u64 > 40_000_000 {
        return Err("Image exceeds 40 million pixels".into());
    }
    Ok(image)
}
pub fn validate(d: &Document, w: u32, h: u32) -> Result<()> {
    if d.objects.len() > 5000 {
        return Err("Too many objects".into());
    }
    for o in &d.objects {
        if !o
            .a
            .iter()
            .chain(o.b.iter())
            .chain(o.points.iter().flatten())
            .all(|x| x.is_finite() && x.abs() < 1_000_000.)
            || !o.width.is_finite()
            || o.width < 1.
            || o.width > 120.
            || o.text.len() > 5000
            || o.points.len() > 20000
        {
            return Err("Invalid annotation geometry or text".into());
        }
    }
    if w == 0 || h == 0 {
        return Err("Empty image".into());
    }
    Ok(())
}
fn region(o: &Object, img: &RgbaImage) -> (u32, u32, u32, u32) {
    let [x, y, w, h] = o.bounds();
    let x = x.max(0.).min(img.width() as f32) as u32;
    let y = y.max(0.).min(img.height() as f32) as u32;
    (
        x,
        y,
        (w as u32).min(img.width() - x),
        (h as u32).min(img.height() - y),
    )
}
fn line(img: &mut RgbaImage, a: [f32; 2], b: [f32; 2], width: f32, c: Rgba<u8>) {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let n = dx.abs().max(dy.abs()).ceil() as usize;
    for i in 0..=n {
        let t = i as f32 / n.max(1) as f32;
        draw_filled_circle_mut(
            img,
            ((a[0] + dx * t) as i32, (a[1] + dy * t) as i32),
            (width / 2.).max(1.) as i32,
            c,
        );
    }
}
pub fn font() -> ab_glyph::FontRef<'static> {
    ab_glyph::FontRef::try_from_slice(include_bytes!("../assets/Ubuntu-Light.ttf"))
        .expect("Bundled open font")
}
fn protect(img: &mut RgbaImage, objects: &[Object]) {
    // Privacy masks render last, regardless of layer ordering, fully opaque.
    for o in objects.iter().filter(|o| o.tool == Tool::Redact) {
        let (x, y, w, h) = region(o, img);
        if w > 0 && h > 0 {
            draw_filled_rect_mut(
                img,
                imageproc::rect::Rect::at(x as i32, y as i32).of_size(w, h),
                Rgba([18, 24, 30, 255]),
            );
        }
    }
}
pub fn render(source: &RgbaImage, objects: &[Object]) -> RgbaImage {
    let mut img = source.clone();
    protect(&mut img, objects);
    for o in objects.iter().filter(|o| o.tool != Tool::Redact) {
        let c = Rgba(o.color);
        let [x, y, w, h] = o.bounds();
        match o.tool {
            Tool::Arrow => {
                line(&mut img, o.a, o.b, o.width, c);
                let t = (o.b[1] - o.a[1]).atan2(o.b[0] - o.a[0]);
                for d in [-0.5, 0.5] {
                    let l = 16. + o.width;
                    line(
                        &mut img,
                        o.b,
                        [o.b[0] - l * (t + d).cos(), o.b[1] - l * (t + d).sin()],
                        o.width,
                        c,
                    );
                }
            }
            Tool::Pen => {
                for p in o.points.windows(2) {
                    line(&mut img, p[0], p[1], o.width, c);
                }
            }
            Tool::Rectangle => {
                let a = [x, y];
                let b = [x + w, y];
                let d = [x, y + h];
                let e = [x + w, y + h];
                for (p, q) in [(a, b), (b, e), (e, d), (d, a)] {
                    line(&mut img, p, q, o.width, c);
                }
            }
            Tool::Highlight => {
                let (x, y, w, h) = region(o, &img);
                for py in y..y + h {
                    for px in x..x + w {
                        let p = img.get_pixel_mut(px, py);
                        for k in 0..3 {
                            p[k] = ((p[k] as u16 * 2 + o.color[k] as u16) / 3) as u8;
                        }
                    }
                }
            }
            Tool::Blur => {
                // Never sample pixels concealed by a privacy mask, including prior annotations.
                protect(&mut img, objects);
                let (x, y, w, h) = region(o, &img);
                if w > 0 && h > 0 {
                    let crop = image::imageops::crop_imm(&img, x, y, w, h).to_image();
                    let blur = image::imageops::blur(&crop, o.width.clamp(1., 48.));
                    image::imageops::overlay(&mut img, &blur, x as i64, y as i64);
                }
            }
            Tool::Text | Tool::Callout | Tool::Step => {
                if o.tool == Tool::Callout {
                    draw_filled_rect_mut(
                        &mut img,
                        imageproc::rect::Rect::at(x as i32, y as i32)
                            .of_size(w.max(1.) as u32, h.max(1.) as u32),
                        c,
                    );
                }
                if o.tool == Tool::Step {
                    draw_filled_circle_mut(&mut img, (x as i32 + 20, y as i32 + 20), 20, c);
                }
                let text_color = if o.tool == Tool::Text {
                    c
                } else {
                    Rgba([255, 255, 255, 255])
                };
                let size = if o.tool == Tool::Step {
                    25.
                } else {
                    o.width.max(14.)
                };
                let f = font();
                if o.tool == Tool::Step {
                    let (tw, th) = text_size(size, &f, &o.text);
                    draw_text_mut(
                        &mut img,
                        text_color,
                        x as i32 + 20 - tw as i32 / 2,
                        y as i32 + 20 - th as i32 / 2,
                        size,
                        &f,
                        &o.text,
                    );
                } else {
                    let (rx, ry, rw, rh) = region(o, &img);
                    if rw > 16 && rh > 12 {
                        let mut patch = RgbaImage::new(rw, rh);
                        let mut row = String::new();
                        let mut ty = 6_i32;
                        for paragraph in o.text.split('\n') {
                            for word in paragraph.split_whitespace() {
                                let candidate = if row.is_empty() {
                                    word.to_owned()
                                } else {
                                    format!("{row} {word}")
                                };
                                if text_size(size, &f, &candidate).0 > rw - 16 && !row.is_empty() {
                                    draw_text_mut(&mut patch, text_color, 8, ty, size, &f, &row);
                                    ty += (size * 1.25) as i32;
                                    row.clear();
                                }
                                if !row.is_empty() {
                                    row.push(' ');
                                }
                                row.push_str(word);
                            }
                            draw_text_mut(&mut patch, text_color, 8, ty, size, &f, &row);
                            ty += (size * 1.25) as i32;
                            row.clear();
                            if ty >= rh as i32 {
                                break;
                            }
                        }
                        image::imageops::overlay(&mut img, &patch, i64::from(rx), i64::from(ry));
                    }
                }
            }
            _ => {}
        }
    }
    protect(&mut img, objects);
    img
}
pub fn sample() -> RgbaImage {
    // Original synthetic software screen: a practical subject for annotations.
    let mut img = RgbaImage::from_pixel(1080, 680, Rgba([245, 247, 250, 255]));
    let f = font();
    fn block(img: &mut RgbaImage, x: i32, y: i32, w: u32, h: u32, c: [u8; 4]) {
        draw_filled_rect_mut(img, imageproc::rect::Rect::at(x, y).of_size(w, h), Rgba(c));
    }
    let ink = Rgba([37, 44, 52, 255]);
    let muted = Rgba([85, 96, 107, 255]);
    block(&mut img, 0, 0, 1080, 62, [255, 255, 255, 255]);
    draw_text_mut(&mut img, ink, 28, 20, 25., &f, "Team workspace");
    draw_text_mut(
        &mut img,
        muted,
        775,
        24,
        17.,
        &f,
        "Example software / synthetic data",
    );
    block(&mut img, 0, 62, 200, 618, [233, 238, 243, 255]);
    for (i, title) in ["Overview", "Projects", "Shared files", "People", "Settings"]
        .iter()
        .enumerate()
    {
        let y = 104 + i as i32 * 54;
        if i == 1 {
            block(&mut img, 16, y - 9, 168, 42, [215, 234, 234, 255]);
        }
        draw_text_mut(&mut img, ink, 32, y, 20., &f, title);
    }
    draw_text_mut(&mut img, ink, 238, 98, 34., &f, "Projects");
    draw_text_mut(
        &mut img,
        muted,
        238,
        147,
        18.,
        &f,
        "Keep the team aligned and share what happens next.",
    );
    block(&mut img, 872, 96, 165, 43, [27, 105, 111, 255]);
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        892,
        107,
        20.,
        &f,
        "New project",
    );
    block(&mut img, 238, 195, 800, 42, [255, 255, 255, 255]);
    draw_text_mut(&mut img, muted, 254, 208, 18., &f, "Search projects...");
    block(&mut img, 238, 262, 800, 328, [255, 255, 255, 255]);
    for (x, title) in [
        (256, "Project"),
        (646, "Owner"),
        (820, "Status"),
        (958, "Updated"),
    ] {
        draw_text_mut(&mut img, muted, x, 278, 17., &f, title);
    }
    for (i, (title, owner, status)) in [
        ("New colleague onboarding", "Alex", "In progress"),
        ("Software setup guide", "Sam", "In review"),
        ("Quarterly planning", "Jordan", "In progress"),
        ("Knowledge library", "Taylor", "Complete"),
    ]
    .iter()
    .enumerate()
    {
        let y = 330 + i as i32 * 63;
        block(&mut img, 254, y - 12, 766, 1, [230, 235, 239, 255]);
        draw_text_mut(&mut img, ink, 256, y, 20., &f, title);
        draw_text_mut(&mut img, muted, 646, y, 18., &f, owner);
        draw_text_mut(&mut img, Rgba([27, 105, 111, 255]), 820, y, 17., &f, status);
        draw_text_mut(&mut img, muted, 958, y, 17., &f, "Today");
    }
    draw_text_mut(
        &mut img,
        muted,
        238,
        619,
        17.,
        &f,
        "Practice: point out New project, highlight a row, then add this image to a guide.",
    );
    img
}
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn export_guide(steps: &[(String, RgbaImage)], path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Framewell guide</title><style>body{font:18px system-ui;max-width:1000px;margin:60px auto;color:#1d3741}img{max-width:100%;border:1px solid #ddd;border-radius:12px}section{margin:50px 0}h1{font-size:42px}</style><h1>Visual guide</h1>",
    );
    for (i, (caption, img)) in steps.iter().enumerate() {
        let name = format!("step-{}.png", i + 1);
        img.save(path.join(&name))?;
        html += &format!(
            "<section><h2>{}. {}</h2><img alt=\"Step {}\" src=\"{}\"></section>",
            i + 1,
            escape(caption),
            i + 1,
            name
        );
    }
    fs::write(path.join("index.html"), html)?;
    Ok(())
}
