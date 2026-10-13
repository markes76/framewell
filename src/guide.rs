use framewell::{Result, load_image, nonce};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    steps: Vec<Step>,
}
#[derive(Serialize, Deserialize)]
struct Step {
    caption: String,
    image: String,
}

pub fn save(steps: &[(String, RgbaImage)], root: &Path) -> Result<()> {
    if steps.len() > 50 {
        return Err("A guide supports up to 50 steps".into());
    }
    fs::create_dir_all(root)?;
    let generation = nonce();
    let mut manifest = Manifest {
        version: 1,
        steps: vec![],
    };
    for (i, (caption, pixels)) in steps.iter().enumerate() {
        if caption.len() > 20_000 {
            return Err("A guide caption is too long".into());
        }
        let image = format!("step-{generation}-{i}.png");
        pixels.save(root.join(&image))?;
        manifest.steps.push(Step {
            caption: caption.clone(),
            image,
        });
    }
    let temporary = root.join(format!("guide-{generation}.tmp"));
    fs::write(&temporary, serde_json::to_vec_pretty(&manifest)?)?;
    fs::rename(temporary, root.join("guide.json"))?;
    Ok(())
}
pub fn load(root: &Path) -> Result<Vec<(String, RgbaImage)>> {
    let path = root.join("guide.json");
    if fs::metadata(&path)?.len() > 2_000_000 {
        return Err("Guide manifest is too large".into());
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(path)?)?;
    if manifest.version != 1 || manifest.steps.len() > 50 {
        return Err("Unsupported guide format".into());
    }
    let mut result = vec![];
    let mut total_pixels = 0_u64;
    for step in manifest.steps {
        if step.caption.len() > 20_000
            || !step.image.starts_with("step-")
            || !step.image.ends_with(".png")
            || Path::new(&step.image).components().count() != 1
            || step.image.contains(['/', '\\'])
        {
            return Err("Invalid guide step".into());
        }
        let path = root.join(step.image);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err("Guide images cannot be symbolic links".into());
        }
        let pixels = load_image(&path)?;
        total_pixels += u64::from(pixels.width()) * u64::from(pixels.height());
        if total_pixels > 50_000_000 {
            return Err("Guide exceeds the 50 million pixel budget".into());
        }
        result.push((step.caption, pixels));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_guide_restores_order_captions_and_pixels() {
        let root = std::env::temp_dir().join(format!("framewell-guide-test-{}", nonce()));
        let steps = vec![
            (
                "First <step>".into(),
                RgbaImage::from_pixel(10, 8, image::Rgba([30, 90, 120, 255])),
            ),
            ("Second".into(), RgbaImage::new(5, 6)),
        ];
        save(&steps, &root).unwrap();
        assert_eq!(load(&root).unwrap(), steps);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn external_image_reference_is_rejected() {
        let root = std::env::temp_dir().join(format!("framewell-guide-test-{}", nonce()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("guide.json"),
            r#"{"version":1,"steps":[{"caption":"","image":"step-../../secret.png"}]}"#,
        )
        .unwrap();
        assert!(load(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
