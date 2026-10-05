use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(windows)]
fn cmd(program: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut c = Command::new(program);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

#[cfg(not(windows))]
fn cmd(program: &str) -> Command {
    Command::new(program)
}

pub fn smartctl_kurulu_mu() -> bool {
    cmd(crate::embedded::smartctl_yol()).arg("--version").output().is_ok()
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub yol: String,
    pub tip_flag: Option<String>,
    pub goster: String,
    pub model: String,
    pub seri: String,
    pub kapasite_gb: f64,
    pub tip: String,
    pub firmware: String,
    pub arayuz: String,
    pub sicaklik: Option<i32>,
    pub saglik: String,
    pub form_faktoru: String,
    pub sata_hiz: String,
    pub sektor_boyutu: String,
    pub cikis_saati: Option<u64>,
    pub guc_dongusu: Option<u64>,
    pub aktif_alan: String,
    pub baglanti: String,
}

pub fn smart_args<'a>(yol: &'a str, tip_flag: &'a Option<String>) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(t) = tip_flag {
        v.push("-d".to_string());
        v.push(t.clone());
    }
    v.push(yol.to_string());
    v
}

fn dene_bilgi(yol: &str, tip_flag: Option<&str>) -> Option<serde_json::Value> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-i", "-H", "-A", "-j", yol]);
    let out = c.output().ok()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let has_model = v.get("model_name").and_then(|x| x.as_str()).is_some()
        || v.get("model_family").and_then(|x| x.as_str()).is_some();
    let has_smart = v.pointer("/ata_smart_attributes/table").is_some()
        || v.pointer("/nvme_smart_health_information_log").is_some();
    if has_model || has_smart {
        Some(v)
    } else {
        None
    }
}

fn tespit_et(yol: &str) -> Option<(String, Option<String>)> {
    let denemeler: &[Option<&str>] = &[
        None,
        Some("sat"),
        Some("sat,auto"),
        Some("sat,12"),
        Some("sat,16"),
        Some("usbjmicron"),
        Some("usbsunplus"),
        Some("usbprolific"),
        Some("usbcypress"),
        Some("nvme"),
        Some("scsi"),
        Some("ata"),
    ];
    for tf in denemeler {
        if let Some(_v) = dene_bilgi(yol, *tf) {
            return Some((yol.to_string(), tf.map(|s| s.to_string())));
        }
    }
    None
}

fn bridge_chipseti_mi(model: &str) -> bool {
    let m = model.to_lowercase();
    m.contains("jmicron") || m.contains("asmedia") || m.contains("sunplus")
        || m.contains("prolific") || m.contains("cypress") || m.contains("bridge")
        || m.contains("initio") || m == "usb" || m.trim().is_empty()
}

pub fn disk_listele() -> Result<Vec<DiskInfo>> {
    let mut diskler: Vec<DiskInfo> = Vec::new();

    if !cfg!(windows) {
        let out = cmd(crate::embedded::smartctl_yol()).args(["--scan-open", "-j"]).output()?;
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            if let Some(arr) = v.get("devices").and_then(|d| d.as_array()) {
                for (idx, d) in arr.iter().enumerate() {
                    if let Some(name) = d.get("name").and_then(|n| n.as_str()) {
                        let tp = d.get("type").and_then(|x| x.as_str()).map(|s| s.to_string());
                        if let Some(v2) = dene_bilgi(name, tp.as_deref()) {
                            diskler.push(json_to_diskinfo(name.to_string(), tp, idx, &v2));
                        }
                    }
                }
            }
        }
        return Ok(diskler);
    }

    for i in 0..16usize {
        let yol = format!("/dev/pd{}", i);
        let denemeler: &[Option<&str>] = &[
            None,
            Some("sat"),
            Some("sat,12"),
            Some("sat,16"),
            Some("nvme"),
            Some("scsi"),
            Some("usbjmicron"),
            Some("usbsunplus"),
            Some("usbprolific"),
            Some("usbcypress"),
            Some("ata"),
        ];

        let mut secilen: Option<(Option<String>, serde_json::Value)> = None;

        for tf in denemeler {
            if let Some(v) = dene_bilgi(&yol, *tf) {
                let model = v
                    .get("model_name")
                    .and_then(|x| x.as_str())
                    .or_else(|| v.get("model_family").and_then(|x| x.as_str()))
                    .unwrap_or("");
                let kap = v.pointer("/user_capacity/bytes").and_then(|x| x.as_u64()).unwrap_or(0);

                if bridge_chipseti_mi(model) || kap == 0 {
                    if secilen.is_none() {
                        secilen = Some((tf.map(|s| s.to_string()), v.clone()));
                    }
                    continue;
                }

                secilen = Some((tf.map(|s| s.to_string()), v));
                break;
            }
        }

        if let Some((tf, v)) = secilen {
            let info = json_to_diskinfo(yol.clone(), tf, i, &v);
            let ayni_yol = diskler.iter().any(|d| d.yol == info.yol);
            let bos_kopru = info.kapasite_gb == 0.0 && bridge_chipseti_mi(&info.model);
            if !ayni_yol && !bos_kopru {
                diskler.push(info);
            }
        }
    }

    Ok(diskler)
}

fn json_to_diskinfo(
    yol: String,
    tip_flag: Option<String>,
    idx: usize,
    v: &serde_json::Value,
) -> DiskInfo {
    let model = v
        .get("model_name")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("model_family").and_then(|x| x.as_str()))
        .unwrap_or("Unknown")
        .to_string();

    let seri = v
        .get("serial_number")
        .and_then(|x| x.as_str())
        .unwrap_or("-")
        .to_string();

    let kapasite_bayt = v
        .pointer("/user_capacity/bytes")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let kapasite_gb = kapasite_bayt as f64 / 1_000_000_000.0;

    let rpm = v.get("rotation_rate").and_then(|x| x.as_i64()).unwrap_or(-1);
    let tip = if rpm == 0 {
        "SSD".to_string()
    } else if rpm > 0 {
        format!("HDD ({} RPM)", rpm)
    } else {
        "-".to_string()
    };

    let firmware = v
        .get("firmware_version")
        .and_then(|x| x.as_str())
        .unwrap_or("-")
        .to_string();

    let arayuz = v
        .pointer("/device/protocol")
        .and_then(|x| x.as_str())
        .unwrap_or("-")
        .to_string();

    let form_faktoru = v
        .pointer("/form_factor/name")
        .and_then(|x| x.as_str())
        .unwrap_or("-")
        .to_string();

    let sata_v = v
        .pointer("/sata_version/string")
        .and_then(|x| x.as_str())
        .unwrap_or("-");
    let sata_hiz_str = v
        .pointer("/interface_speed/current/string")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let sata_hiz = if sata_hiz_str.is_empty() {
        sata_v.to_string()
    } else {
        format!("{} @ {}", sata_v, sata_hiz_str)
    };

    let mantiksal = v
        .pointer("/logical_block_size")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let fiziksel = v
        .pointer("/physical_block_size")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let sektor_boyutu = if mantiksal > 0 {
        format!("{} / {} B", mantiksal, fiziksel)
    } else {
        "-".into()
    };

    let saglik = v
        .pointer("/smart_status/passed")
        .and_then(|x| x.as_bool())
        .map(|b| {
            if b {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            }
        })
        .unwrap_or_else(|| "UNKNOWN".into());

    let mut sicaklik = None;
    let mut cikis_saati = None;
    let mut guc_dongusu = None;

    if let Some(t) = v.pointer("/temperature/current").and_then(|x| x.as_i64()) {
        sicaklik = Some(t as i32);
    }

    if let Some(arr) = v
        .pointer("/ata_smart_attributes/table")
        .and_then(|t| t.as_array())
    {
        for a in arr {
            let id = a.get("id").and_then(|x| x.as_u64()).unwrap_or(0);
            let raw = a.pointer("/raw/value").and_then(|x| x.as_i64()).unwrap_or(0);
            match id {
                194 | 190 if sicaklik.is_none() => sicaklik = Some(raw as i32),
                9 => cikis_saati = Some(raw as u64),
                12 => guc_dongusu = Some(raw as u64),
                _ => {}
            }
        }
    }

    let aktif_alan = windows_surucu_harfleri(idx);

    let baglanti = match tip_flag.as_deref() {
        Some(t) if t.starts_with("usb") || t.starts_with("sat") => {
            format!("USB Kopru ({})", t)
        }
        Some("nvme") => "NVMe".into(),
        Some(t) => t.to_string(),
        None => "DIRECT".into(),
    };

    let goster = if kapasite_gb > 0.0 {
        format!(
            "Disk {}: {}  |  {:.0} GB  |  {}",
            idx, model, kapasite_gb, baglanti
        )
    } else {
        format!("Disk {}: {}  |  {}", idx, model, baglanti)
    };

    DiskInfo {
        yol,
        tip_flag,
        goster,
        model,
        seri,
        kapasite_gb,
        tip,
        firmware,
        arayuz,
        sicaklik,
        saglik,
        form_faktoru,
        sata_hiz,
        sektor_boyutu,
        cikis_saati,
        guc_dongusu,
        aktif_alan,
        baglanti,
    }
}

#[cfg(windows)]
fn windows_surucu_harfleri(disk_num: usize) -> String {
    let ps = format!(
        "Get-Partition -DiskNumber {} -ErrorAction SilentlyContinue | Where-Object {{ $_.DriveLetter }} | Select-Object -ExpandProperty DriveLetter",
        disk_num
    );
    let out = cmd("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .output();
    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout);
        let harfler: Vec<String> = s
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .map(|l| format!("{}:", l))
            .collect();
        if !harfler.is_empty() {
            return harfler.join(", ");
        }
    }
    "-".into()
}

#[cfg(not(windows))]
fn windows_surucu_harfleri(_disk_num: usize) -> String {
    "-".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    pub id: u32,
    pub name: String,
    pub value: i32,
    pub worst: i32,
    pub thresh: i32,
    pub raw: i64,
    pub when_failed: String,
}

pub fn nvme_smart_oku(disk: &str, tip_flag: &Option<String>) -> Option<Vec<SmartAttribute>> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-A", "-j", disk]);
    let out = c.output().ok()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let log = v.get("nvme_smart_health_information_log")?;
    let obj = log.as_object()?;
    let mut sonuc = Vec::new();
    let mut id = 1u32;
    for (k, val) in obj {
        let raw = val.as_i64().unwrap_or(0);
        sonuc.push(SmartAttribute {
            id,
            name: k.clone(),
            value: 100,
            worst: 100,
            thresh: 0,
            raw,
            when_failed: "-".to_string(),
        });
        id += 1;
    }
    if sonuc.is_empty() { None } else { Some(sonuc) }
}

pub fn smart_oku(disk: &str, tip_flag: &Option<String>) -> Result<Vec<SmartAttribute>> {
    if let Some(nvme) = nvme_smart_oku(disk, tip_flag) {
        return Ok(nvme);
    }

    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-A", "-j", disk]);
    let out = c.output().context("SMART read failed")?;

    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let mut sonuc = Vec::new();

    if let Some(table) = v
        .pointer("/ata_smart_attributes/table")
        .and_then(|t| t.as_array())
    {
        for attr in table {
            sonuc.push(SmartAttribute {
                id: attr.get("id").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                name: attr.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                value: attr.get("value").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
                worst: attr.get("worst").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
                thresh: attr.get("thresh").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
                raw: attr
                    .pointer("/raw/value")
                    .and_then(|x| x.as_i64())
                    .unwrap_or(0),
                when_failed: attr
                    .get("when_failed")
                    .and_then(|x| x.as_str())
                    .unwrap_or("-")
                    .to_string(),
            });
        }
    }

    Ok(sonuc)
}

pub fn error_log(disk: &str, tip_flag: &Option<String>) -> Result<String> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-l", "xerror", disk]);
    let out = c.output()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        Ok("Hata log'u bos veya erisilemedi.".into())
    } else {
        Ok(s)
    }
}

pub fn sata_link_yavas(v: &crate::smart::DiskInfo) -> bool {
    let h = v.sata_hiz.to_lowercase();
    if h.contains("sata 3") || h.contains("6.0 gb/s") || h.contains("sata iii") {
        if h.contains("3.0 gb/s") || h.contains("1.5 gb/s") {
            return true;
        }
    }
    false
}

pub fn self_test(disk: &str, tip: &str, tip_flag: &Option<String>) -> Result<String> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-t", tip, disk]);
    let out = c.output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn self_test_iptal(disk: &str, tip_flag: &Option<String>) -> Result<String> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-X", disk]);
    let out = c.output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn self_test_log(disk: &str, tip_flag: &Option<String>) -> Result<String> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-l", "selftest", disk]);
    let out = c.output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[derive(Debug, Clone)]
pub struct TestDurumu {
    pub aktif: bool,
    pub kalan_yuzde: u32,
    pub durum_metni: String,
    pub tahmini_dakika: Option<u32>,
}

pub fn test_durumu(disk: &str, tip_flag: &Option<String>) -> Result<TestDurumu> {
    let mut c = cmd(crate::embedded::smartctl_yol());
    if let Some(t) = tip_flag {
        c.args(["-d", t]);
    }
    c.args(["-c", "-j", disk]);
    let out = c.output()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;

    let status = v.pointer("/ata_smart_data/self_test/status");

    let aktif = status
        .and_then(|s| s.get("passed"))
        .is_none()
        && status.is_some();

    let kalan = status
        .and_then(|s| s.get("remaining_percent"))
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as u32;

    let deger = status
        .and_then(|s| s.get("value"))
        .and_then(|x| x.as_u64())
        .unwrap_or(0);

    let calisiyor = (0xF0..=0xFF).contains(&deger) && deger != 0xFF;

    let durum_metni = status
        .and_then(|s| s.get("string"))
        .and_then(|x| x.as_str())
        .unwrap_or("bilinmiyor")
        .to_string();

    let tahmini = v
        .pointer("/ata_smart_data/self_test/polling_minutes/short")
        .and_then(|x| x.as_u64())
        .map(|x| x as u32);

    Ok(TestDurumu {
        aktif: calisiyor || (aktif && kalan > 0),
        kalan_yuzde: kalan,
        durum_metni,
        tahmini_dakika: tahmini,
    })
}
