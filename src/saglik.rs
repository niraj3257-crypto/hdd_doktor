use crate::smart::SmartAttribute;
use crate::lang::Dil;

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
    pub fn etiket(&self, dil: Dil) -> &'static str {
        match self {
            Self::Mukemmel => dil.health_excellent(),
            Self::Iyi => dil.health_good(),
            Self::Dikkat => dil.health_caution(),
            Self::Kotu => dil.health_bad(),
            Self::Kritik => dil.health_critical(),
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

pub fn hesapla(attrs: &[SmartAttribute], cikis_saati: Option<u64>, dil: Dil) -> SaglikSkoru {
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
        sebep.push(dil.reallocated_reason(reallocated));
        oneri.push(dil.reallocated_action().into());
    }

    if pending > 0 {
        puan -= 15 + (pending as i32 * 2).min(30);
        sebep.push(dil.pending_reason(pending));
        oneri.push(dil.pending_action().into());
    }

    if uncorrectable > 0 {
        puan -= 25 + (uncorrectable as i32 * 3).min(40);
        sebep.push(dil.uncorrectable_reason(uncorrectable));
        oneri.push(dil.uncorrectable_action().into());
    }

    if crc > 0 {
        let ceza = (crc as i32 / 2).min(20);
        puan -= ceza;
        sebep.push(dil.crc_reason(crc));
        oneri.push(dil.crc_action().into());
    }

    if realloc_event > 0 && reallocated == 0 {
        puan -= 10;
        sebep.push(dil.realloc_event_reason(realloc_event));
    }

    if reported_uncorrect > 0 {
        puan -= 15;
        sebep.push(dil.reported_reason(reported_uncorrect));
    }

    if cmd_timeout > 100 {
        puan -= 10;
        sebep.push(dil.timeout_reason(cmd_timeout));
        oneri.push(dil.timeout_action().into());
    }

    for &id in &[5u32, 197, 198, 199, 196, 187, 188] {
        let v = val(id);
        let t = thresh(id);
        if t > 0 && v <= t {
            puan -= 30;
            sebep.push(dil.thresh_reason(id, v, t));
            oneri.push(dil.thresh_action(id));
        }
    }

    if let Some(t_raw) = attrs.iter().find(|a| a.id == 194 || a.id == 190) {
        let sicaklik = t_raw.raw as i32;
        if sicaklik >= 60 {
            puan -= 20;
            sebep.push(dil.hot_reason(sicaklik));
            oneri.push(dil.hot_action().into());
        } else if sicaklik >= 55 {
            puan -= 10;
            sebep.push(dil.warm_reason(sicaklik));
        }
    }

    if raw_read > 1_000_000 {
        puan -= 5;
        sebep.push(dil.high_read_error().into());
    }

    if seek_error > 100_000_000 {
        puan -= 5;
        sebep.push(dil.high_seek_error().into());
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
        sebep.push(dil.all_clean().into());
    }
    if oneri.is_empty() && puan >= 90 {
        oneri.push(dil.default_advice().into());
    }

    SaglikSkoru {
        puan,
        durum,
        sebep,
        oneri,
        kalan_omur_yil,
    }
}
