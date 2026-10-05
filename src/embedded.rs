use std::path::PathBuf;
use std::sync::OnceLock;

const SMARTCTL_BYTES: &[u8] = include_bytes!("../assets/smartctl.exe");

static SMARTCTL_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn smartctl_yol() -> &'static str {
    SMARTCTL_PATH
        .get_or_init(|| {
            let dir = std::env::temp_dir().join("hdd_doktor");
            std::fs::create_dir_all(&dir).ok();
            let path = dir.join("smartctl.exe");
            if !path.exists() || std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) != SMARTCTL_BYTES.len() as u64 {
                std::fs::write(&path, SMARTCTL_BYTES).expect("smartctl.exe temp dizinine yazılamadı");
            }
            path
        })
        .to_str()
        .expect("geçersiz temp yolu")
}
