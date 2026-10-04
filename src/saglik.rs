use crate::smart::SmartAttribute;

#[derive(Debug, Clone)]
pub struct SaglikSkoru {
    pub puan: u32,
    pub durum: SaglikDurumu,
    pub sebep: Vec<String>,
    pub oneri: Vec<String>,
    pub kalan_omur_yil: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaglikDurumu {
    Mukemmel,
    Iyi,
    Dikkat,
    Kotu,
    Kritik,
}

impl SaglikDurumu {
    pub fn etiket(&self) -> &'static str {
        match self {
            Self::Mukemmel => "MÜKEMMEL",
            Self::Iyi => "İYİ",
            Self::Dikkat => "DİKKAT",
            Self::Kotu => "KÖTÜ",
            Self::Kritik => "KRİTİK - YEDEK AL",
        }
    }

    pub fn renk(&self) -> (u8, u8, u8) {
        match self {
            Self::Mukemmel => (60, 220, 130),
            Self::Iyi => (120, 200, 120),
            Self::Dikkat => (240, 200, 80),
            Self::Kotu => (240, 140, 60),
            Self::Kritik => (240, 80, 80),
        }
    }
}

pub fn hesapla(attrs: &[SmartAttribute], cikis_saati: Option<u64>) -> SaglikSkoru {
    let mut puan: i32 = 100;
    let mut sebep = Vec::new();
    let mut oneri = Vec::new();

    let al = |id: u32| -> i64 { attrs.iter().find(|a| a.id == id).map(|a| a.raw).unwrap_or(0) };
    let val = |id: u32| -> i32 {
        attrs.iter().find(|a| a.id == id).map(|a| a.value).unwrap_or(100)
    };
    let thresh = |id: u32| -> i32 {
        attrs.iter().find(|a| a.id == id).map(|a| a.thresh).unwrap_or(0)
    };

    let reallocated = al(5);
    let pending = al(197);
    let uncorrectable = al(198);
    let crc = al(199);
    let realloc_event = al(196);
    let reported_uncorrect = al(187);
    let cmd_timeout = al(188);
    let seek_error = al(7);
    let raw_read = al(1);

    if reallocated > 0 {
        let ceza = ((reallocated as i32).min(1000) / 5).max(5);
        puan -= ceza;
        sebep.push(format!("{} bozuk sektör remap edildi (SMART 5)", reallocated));
        oneri.push("Zero-Fill çalıştır, pending sector'lari remap et".into());
    }

    if pending > 0 {
        puan -= 15 + (pending as i32 * 2).min(30);
        sebep.push(format!("{} pending sector var (SMART 197) - okuma zorluyor", pending));
        oneri.push("ACIL: Yedek al. Zero-Fill remap tetikleyebilir".into());
    }

    if uncorrectable > 0 {
        puan -= 25 + (uncorrectable as i32 * 3).min(40);
        sebep.push(format!("{} düzeltilemez hata (SMART 198)", uncorrectable));
        oneri.push("Diski değiştirin. Güvenilir değil".into());
    }

    if crc > 0 {
        let ceza = (crc as i32 / 2).min(20);
        puan -= ceza;
        sebep.push(format!("{} CRC hatası (SMART 199) - kablo sorunu", crc));
        oneri.push("SATA/USB kablosunu değiştirin".into());
    }

    if realloc_event > 0 && reallocated == 0 {
        puan -= 10;
        sebep.push(format!("{} remap denemesi ama tamamlanmamis (SMART 196)", realloc_event));
    }

    if reported_uncorrect > 0 {
        puan -= 15;
        sebep.push(format!("{} rapor edilmis düzeltilemez hata (SMART 187)", reported_uncorrect));
    }

    if cmd_timeout > 100 {
        puan -= 10;
        sebep.push(format!("{} komut zaman aşımı (SMART 188)", cmd_timeout));
        oneri.push("USB istasyonun gücünü / kablosunu kontrol edin".into());
    }

    for &id in &[5u32, 197, 198, 199, 196, 187, 188] {
        let v = val(id);
        let t = thresh(id);
        if t > 0 && v <= t {
            puan -= 30;
            sebep.push(format!("SMART {} eşiğin altina düştü (değer={} <= eşik={})", id, v, t));
            oneri.push(format!("SMART {} kritik eşikte. Diski değiştirin", id));
        }
    }

    if let Some(t_raw) = attrs.iter().find(|a| a.id == 194 || a.id == 190) {
        let sicaklik = t_raw.raw as i32;
        if sicaklik >= 60 {
            puan -= 20;
            sebep.push(format!("Çok sicak: {} C (>= 60)", sicaklik));
            oneri.push("Havalandırmayı artır. Sıcaklık arıza sebebi".into());
        } else if sicaklik >= 55 {
            puan -= 10;
            sebep.push(format!("Sıcak: {} C", sicaklik));
        }
    }

    if raw_read > 1_000_000 {
        puan -= 5;
        sebep.push("Yüksek raw read error rate (SMART 1)".into());
    }

    if seek_error > 100_000_000 {
        puan -= 5;
        sebep.push("Yüksek seek error rate (SMART 7)".into());
    }

    let kalan_omur_yil = cikis_saati.map(|saat| {
        let temel = 5.0f32;
        let kullanilan_yil = saat as f32 / 8760.0;
        let hata_carpani = if pending > 0 || uncorrectable > 0 {
            0.1
        } else if reallocated > 50 {
            0.3
        } else if reallocated > 0 || crc > 10 {
            0.6
        } else {
            1.0
        };
        (temel - kullanilan_yil).max(0.0) * hata_carpani
    });

    let puan = puan.max(0).min(100) as u32;

    let durum = match puan {
        90..=100 => SaglikDurumu::Mukemmel,
        70..=89 => SaglikDurumu::Iyi,
        50..=69 => SaglikDurumu::Dikkat,
        25..=49 => SaglikDurumu::Kotu,
        _ => SaglikDurumu::Kritik,
    };

    if sebep.is_empty() {
        sebep.push("Bütün kritik değerler temiz. Disk sağlıklı.".into());
    }
    if oneri.is_empty() && puan >= 90 {
        oneri.push("Periyodik olarak trend kaydet, değişim yoksa endişe etme".into());
    }

    SaglikSkoru {
        puan,
        durum,
        sebep,
        oneri,
        kalan_omur_yil,
    }
}
