use anyhow::Result;
use chrono::Local;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use crate::saglik::SaglikSkoru;
use crate::smart::{DiskInfo, SmartAttribute};

fn rapor_dir() -> PathBuf {
    let mut p = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|x| x.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    p.push("hdd_raporlar");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn sha256_hex(veri: &str) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut m: Vec<u8> = veri.as_bytes().to_vec();
    let uz = m.len() as u64 * 8;
    m.push(0x80);
    while m.len() % 64 != 56 {
        m.push(0);
    }
    m.extend_from_slice(&uz.to_be_bytes());

    for chunk in m.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e;
            e = d.wrapping_add(t1);
            d = c; c = b; b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    h.iter().map(|x| format!("{:08x}", x)).collect::<String>()
}

pub fn html_rapor(
    d: &DiskInfo,
    attrs: &[SmartAttribute],
    saglik: Option<&SaglikSkoru>,
) -> Result<PathBuf> {
    let tarih = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let dosya_adi = format!(
        "rapor_{}_{}.html",
        d.seri.replace(['/', '\\', ':', ' '], "_"),
        Local::now().format("%Y%m%d_%H%M%S")
    );
    let mut yol = rapor_dir();
    yol.push(&dosya_adi);

    let (skor, durum, renk) = if let Some(s) = saglik {
        let (r, g, b) = s.durum.renk();
        (s.puan, s.durum.etiket().to_string(), format!("rgb({},{},{})", r, g, b))
    } else {
        (0u32, "Bilinmiyor".to_string(), "#888".to_string())
    };

    let mut html = String::new();
    let _ = write!(html, r#"<!DOCTYPE html>
<html lang="tr">
<head>
<meta charset="UTF-8">
<title>HDD Doktor Raporu - {model}</title>
<style>
* {{ box-sizing: border-box; }}
body {{ font-family: -apple-system, "Segoe UI", Tahoma, sans-serif; margin: 0; padding: 30px; background: #f5f7fa; color: #222; }}
.container {{ max-width: 1100px; margin: 0 auto; }}
.header {{ background: linear-gradient(135deg, #2563eb, #1d4ed8); color: white; padding: 24px 32px; border-radius: 12px; margin-bottom: 24px; }}
.header h1 {{ margin: 0 0 8px 0; font-size: 28px; }}
.header .meta {{ opacity: 0.85; font-size: 14px; }}
.grid {{ display: grid; grid-template-columns: 2fr 1fr; gap: 20px; margin-bottom: 20px; }}
.card {{ background: white; padding: 20px 24px; border-radius: 10px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }}
.card h2 {{ margin: 0 0 14px 0; font-size: 16px; color: #2563eb; border-bottom: 2px solid #e5e7eb; padding-bottom: 8px; }}
.saglik-kart {{ background: white; padding: 24px; border-radius: 10px; text-align: center; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }}
.puan {{ font-size: 64px; font-weight: bold; color: {renk}; line-height: 1; }}
.durum {{ font-size: 18px; font-weight: bold; color: {renk}; margin-top: 8px; }}
table {{ width: 100%; border-collapse: collapse; font-size: 13px; }}
th, td {{ text-align: left; padding: 6px 10px; border-bottom: 1px solid #f0f0f0; }}
th {{ background: #f9fafb; font-weight: 600; color: #555; }}
tr:hover {{ background: #fafbfc; }}
.durum-ok {{ color: #10b981; font-weight: 600; }}
.durum-dikkat {{ color: #f59e0b; font-weight: 600; }}
.durum-kritik {{ color: #ef4444; font-weight: 600; }}
.bilgi-satir {{ display: flex; justify-content: space-between; padding: 6px 0; border-bottom: 1px solid #f0f0f0; }}
.bilgi-satir:last-child {{ border: 0; }}
.bilgi-satir span:first-child {{ color: #666; }}
.uyari-kutu {{ background: #fef3c7; border-left: 4px solid #f59e0b; padding: 12px 16px; margin: 10px 0; border-radius: 4px; }}
.oneri-kutu {{ background: #dbeafe; border-left: 4px solid #3b82f6; padding: 12px 16px; margin: 10px 0; border-radius: 4px; }}
.footer {{ margin-top: 30px; padding: 20px; background: white; border-radius: 10px; font-size: 12px; color: #666; text-align: center; }}
.hash {{ font-family: monospace; background: #f3f4f6; padding: 4px 8px; border-radius: 4px; font-size: 11px; word-break: break-all; }}
ul {{ margin: 8px 0; padding-left: 20px; }}
li {{ margin: 4px 0; }}
</style>
</head>
<body>
<div class="container">

<div class="header">
    <h1>HDD Doktor Raporu</h1>
    <div class="meta">Tarih: {tarih} | Disk: {model}</div>
</div>

<div class="grid">
    <div class="card">
        <h2>Genel Bilgi</h2>
        <div class="bilgi-satir"><span>Model</span><b>{model}</b></div>
        <div class="bilgi-satir"><span>Seri No</span><span>{seri}</span></div>
        <div class="bilgi-satir"><span>Firmware</span><span>{firmware}</span></div>
        <div class="bilgi-satir"><span>Kapasite</span><span>{kapasite:.1} GB</span></div>
        <div class="bilgi-satir"><span>Tip</span><span>{tip}</span></div>
        <div class="bilgi-satir"><span>Bağlantı</span><span>{baglanti}</span></div>
        <div class="bilgi-satir"><span>Arayüz</span><span>{sata_hiz}</span></div>
        <div class="bilgi-satir"><span>Sıcaklık</span><span>{sicaklik}</span></div>
        <div class="bilgi-satir"><span>Çalışma Süresi</span><span>{saat} saat</span></div>
        <div class="bilgi-satir"><span>Güç Döngüsü</span><span>{guc}</span></div>
        <div class="bilgi-satir"><span>Windows Sürücüsü</span><span>{surucu}</span></div>
    </div>

    <div class="saglik-kart">
        <div style="font-size:13px; color:#666; margin-bottom:8px;">SAĞLIK SKORU</div>
        <div class="puan">{skor}<span style="font-size:24px; color:#999;">/100</span></div>
        <div class="durum">{durum}</div>
"#,
        model = htmlesc(&d.model),
        tarih = tarih,
        renk = renk,
        seri = htmlesc(&d.seri),
        firmware = htmlesc(&d.firmware),
        kapasite = d.kapasite_gb,
        tip = htmlesc(&d.tip),
        baglanti = htmlesc(&d.baglanti),
        sata_hiz = htmlesc(&d.sata_hiz),
        sicaklik = d.sicaklik.map(|t| format!("{} °C", t)).unwrap_or("-".into()),
        saat = d.cikis_saati.unwrap_or(0),
        guc = d.guc_dongusu.unwrap_or(0),
        surucu = htmlesc(&d.aktif_alan),
        skor = skor,
        durum = htmlesc(&durum),
    );

    if let Some(s) = saglik {
        if let Some(y) = s.kalan_omur_yil {
            let _ = write!(html, r#"        <div style="margin-top:14px; font-size:13px; color:#666;">Tahmini kalan ömür<br><b style="font-size:20px; color:#333;">~{:.1} yıl</b></div>
"#, y);
        }
    }
    html.push_str("    </div>\n</div>\n");

    if let Some(s) = saglik {
        if !s.sebep.is_empty() {
            html.push_str(r#"<div class="card"><h2>Analiz Sebepleri</h2><ul>"#);
            for sb in &s.sebep {
                let _ = write!(html, "<li>{}</li>", htmlesc(sb));
            }
            html.push_str("</ul></div>\n");
        }
        if !s.oneri.is_empty() {
            html.push_str(r#"<div class="card"><h2>Öneriler</h2>"#);
            for o in &s.oneri {
                let _ = write!(html, r#"<div class="oneri-kutu">{}</div>"#, htmlesc(o));
            }
            html.push_str("</div>\n");
        }
    }

    html.push_str(r#"<div class="card"><h2>SMART Attributes</h2>
<table>
<thead><tr><th>ID</th><th>İsim</th><th>Değer</th><th>En Kötü</th><th>Eşik</th><th>Ham Değer</th><th>Durum</th></tr></thead>
<tbody>
"#);

    for a in attrs {
        let (dur, sinif) = durum_class(a);
        let _ = write!(
            html,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><b>{}</b></td><td class=\"{}\">{}</td></tr>\n",
            a.id, htmlesc(&a.name), a.value, a.worst, a.thresh, a.raw, sinif, dur
        );
    }
    html.push_str("</tbody></table></div>\n");

    let hash_veri = format!("{}|{}|{}|{}|{}", d.model, d.seri, tarih, attrs.len(), skor);
    let hash = sha256_hex(&hash_veri);

    let _ = write!(
        html,
        r#"<div class="footer">
Bu rapor <b>HDD Doktor v0.3</b> tarafından oluşturuldu.<br>
Doğrulama Hash (SHA-256): <span class="hash">{}</span>
</div>
</div>
</body>
</html>"#,
        hash
    );

    fs::write(&yol, html)?;
    Ok(yol)
}

pub fn csv_export(
    d: &DiskInfo,
    attrs: &[SmartAttribute],
) -> Result<PathBuf> {
    let dosya_adi = format!(
        "smart_{}_{}.csv",
        d.seri.replace(['/', '\\', ':', ' '], "_"),
        Local::now().format("%Y%m%d_%H%M%S")
    );
    let mut yol = rapor_dir();
    yol.push(&dosya_adi);

    let mut csv = String::new();
    csv.push_str("Tarih,Model,Seri,Firmware,Kapasite_GB,Sicaklik,Saglik\n");
    let _ = writeln!(
        csv,
        "{},{},{},{},{:.1},{},{}",
        Local::now().to_rfc3339(),
        d.model,
        d.seri,
        d.firmware,
        d.kapasite_gb,
        d.sicaklik.map(|t| t.to_string()).unwrap_or_default(),
        d.saglik
    );
    csv.push('\n');
    csv.push_str("ID,Isim,Deger,EnKotu,Esik,HamDeger,WhenFailed\n");
    for a in attrs {
        let _ = writeln!(
            csv,
            "{},\"{}\",{},{},{},{},\"{}\"",
            a.id, a.name, a.value, a.worst, a.thresh, a.raw, a.when_failed
        );
    }

    fs::write(&yol, csv)?;
    Ok(yol)
}

fn durum_class(a: &SmartAttribute) -> (&'static str, &'static str) {
    if !a.when_failed.is_empty() && a.when_failed != "-" {
        return ("HATA", "durum-kritik");
    }
    if a.thresh > 0 && a.value <= a.thresh {
        return ("KRİTİK", "durum-kritik");
    }
    let kritik = matches!(a.id, 5 | 187 | 188 | 197 | 198 | 199 | 196);
    if kritik && a.raw > 0 {
        return ("DİKKAT", "durum-dikkat");
    }
    ("Sorunsuz", "durum-ok")
}

fn htmlesc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
