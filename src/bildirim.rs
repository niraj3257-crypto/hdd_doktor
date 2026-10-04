pub fn bildir(baslik: &str, mesaj: &str, kritik: bool) {
    let mut n = notify_rust::Notification::new();
    n.summary(baslik).body(mesaj).appname("HDD Doktor");
    if kritik {
        n.urgency(notify_rust::Urgency::Critical);
    }
    let _ = n.show();
}

pub fn attribute_aciklamasi(id: u32) -> &'static str {
    match id {
        1 => "Raw Read Error Rate - Ham okuma hata orani. Yüksekse okuma zorluk çekiliyor.",
        3 => "Spin-Up Time - Diskin donmeye başlama süresi (ms).",
        4 => "Start/Stop Count - Başlama/durma sayısı. Çok yüksekse motor eskiyor.",
        5 => "Reallocated Sectors Count - Bozuk sektör sayısı. > 0 ise disk kötüye gidiyor.",
        7 => "Seek Error Rate - Kafa hareketi hatası. Yüksekse mekanik sorun.",
        9 => "Power-On Hours - Toplam çalışma saati. Yasi belirtir.",
        10 => "Spin Retry Count - Donme yeniden deneme. Motorun zayifladigini gösterir.",
        12 => "Power Cycle Count - Güç döngüsu sayısı.",
        184 => "End-to-End Error - İç veri yolu hatası.",
        187 => "Reported Uncorrectable Errors - Düzeltilemez hata sayısı.",
        188 => "Command Timeout - Komut zaman aşımı. USB istasyonlarda sik görülür.",
        190 => "Airflow Temperature - Hava akımı sıcaklığı (C).",
        193 => "Load Cycle Count - Kafa park sayısı. WD Green'lerde özellikle kritik.",
        194 => "Temperature - Disk sıcaklığı (C).",
        196 => "Reallocation Event Count - Remap deneme sayısı.",
        197 => "Current Pending Sector - Okuma zorluğu çeken sektör. ACIL.",
        198 => "Offline Uncorrectable - Düzeltilemez arıza. Diski değiştir.",
        199 => "UDMA CRC Error Count - Kablo/arayüz hatası. Kablo değiştir.",
        200 => "Multi-Zone Error Rate - Yazma hata orani.",
        _ => "SMART attribute - Detay icin üretici belgeleri.",
    }
}
