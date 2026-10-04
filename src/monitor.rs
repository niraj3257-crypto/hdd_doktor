use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use crate::smart::{self, SmartAttribute};

#[derive(Debug, Serialize, Deserialize)]
struct TrendKayit {
    tarih: String,
    disk: String,
    attrs: Vec<SmartAttribute>,
}

fn trend_dir() -> PathBuf {
    let mut p = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|x| x.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    p.push("hdd_trend");
    let _ = fs::create_dir_all(&p);
    p
}

fn trend_dosyasi(disk: &str) -> PathBuf {
    let safe = disk.replace(['/', '\\', ':'], "_");
    let mut p = trend_dir();
    p.push(format!("trend_{}.jsonl", safe));
    p
}

pub fn trend_kaydet(disk: &str, tip_flag: &Option<String>) -> Result<String> {
    let attrs = smart::smart_oku(disk, tip_flag)?;
    let kayit = TrendKayit {
        tarih: Local::now().to_rfc3339(),
        disk: disk.to_string(),
        attrs,
    };
    let dosya = trend_dosyasi(disk);
    let mut f = OpenOptions::new().create(true).append(true).open(&dosya)?;
    writeln!(f, "{}", serde_json::to_string(&kayit)?)?;
    Ok(dosya.display().to_string())
}

pub fn trend_ozet(disk: &str) -> Result<String> {
    let dosya = trend_dosyasi(disk);
    if !dosya.exists() {
        return Ok("Henuz kayıt yok. Önce 'Trend Kaydet' çalıştırin.".into());
    }
    let icerik = fs::read_to_string(&dosya)?;
    let kayitlar: Vec<TrendKayit> = icerik
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    if kayitlar.len() < 2 {
        return Ok(format!(
            "Karşılaştırma için en az 2 kayıt gerekli. Mevcut: {}",
            kayitlar.len()
        ));
    }

    let ilk = &kayitlar[0];
    let son = &kayitlar[kayitlar.len() - 1];

    let mut s = String::new();
    s.push_str(&format!(
        "=== TREND ANALİZİ ({} -> {}) ===\n",
        ilk.tarih, son.tarih
    ));
    s.push_str(&format!("Kayıt sayısı: {}\n\n", kayitlar.len()));
    s.push_str(&format!(
        "{:>4} {:<28} {:>10} {:>10} {:>10}\n",
        "ID", "İsim", "Ilk", "Son", "Değişim"
    ));
    s.push_str(&"-".repeat(70));
    s.push('\n');

    let izle = [5u32, 187, 188, 197, 198, 199, 196, 9, 194];
    for id in izle {
        let ilk_v = ilk.attrs.iter().find(|a| a.id == id);
        let son_v = son.attrs.iter().find(|a| a.id == id);
        if let (Some(i), Some(sn)) = (ilk_v, son_v) {
            let fark = sn.raw - i.raw;
            let isaret = if fark > 0 {
                format!("+{}", fark)
            } else {
                fark.to_string()
            };
            s.push_str(&format!(
                "{:>4} {:<28} {:>10} {:>10} {:>10}\n",
                i.id, i.name, i.raw, sn.raw, isaret
            ));
        }
    }

    Ok(s)
}
