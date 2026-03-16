use std::{
    fmt, fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone)]
pub(crate) struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Debug)]
pub(crate) struct IcoImage {
    pub path: PathBuf,
    pub file_name: String,
    pub resolution: Resolution,
    pub size: u64,
}

impl IcoImage {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();

        let size = fs::metadata(path)?.len();
        let (width, height) = image::image_dimensions(path)?;

        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "-".to_owned());

        Ok(Self {
            path: path.to_path_buf(),
            file_name,
            resolution: Resolution { width, height },
            size,
        })
    }

    pub(crate) fn formatted_size(&self) -> String {
        const UNITS: &[&str] = &["Б", "КБ", "МБ", "ГБ"];
        let mut size = self.size as f64;

        for unit in UNITS.iter().take(UNITS.len() - 1) {
            if size < 1024.0 {
                return format!("{:.1} {}", size, unit);
            }
            size /= 1024.0;
        }

        format!("{:.1} {}", size, UNITS.last().unwrap())
    }
}
