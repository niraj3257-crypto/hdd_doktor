<div align="center">
  <img src="assets/logo.png" width="180" alt="HDD Doktor logosu">

# HDD Doktor

**Rust ile yazılmış profesyonel HDD teşhis, izleme ve bad-sector onarım aracı.**

![Rust](https://img.shields.io/badge/rust-1.75+-orange?logo=rust) ![egui](https://img.shields.io/badge/UI-egui_0.29-blue) ![Platform](https://img.shields.io/badge/platform-Windows-informational) ![License](https://img.shields.io/badge/license-MIT-green)

</div>

---

## 🩺 Ne işe yarar?

Bir doktor gibi — diskinin sağlığını canlı okur, kritik değerleri işaretler, bozulan sektörleri tespit edip yedek havuzdan değiştirmeyi sağlar. WD5000AVDS ve benzeri SATA/ATA diskler ile test edilmiştir, standart SMART destekleyen her diskte çalışır.

## ✨ Özellikler

- 📊 **SMART Değer Okuma** — renk kodlu panel, kritik değerleri (05/C5/C6) işaretler
- 🧪 **Kısa / Uzun Self-Test** — dahili ATA SMART disk testleri
- 🔍 **Yüzey Taraması** — her sektörü sırayla okuyarak okuma hatalarını listeler
- 🩹 **Zero-Fill (Bad Sector Onarımı)** — her sektöre sıfır yazarak zayıf sektörleri yedek havuzdan değiştirir (realokasyonu tetikler)
- 💣 **Secure Erase** — ATA Secure Erase komutuyla fabrika sıfırlaması
- 📈 **Trend Kaydı ve Analizi** — SMART değerlerinin zaman içindeki değişimini JSON olarak kaydeder, grafik olarak gösterir
- 🔔 **Masaüstü bildirimleri** — kritik eşik aşılırsa `notify-rust` ile uyarır
- 🖼 **egui tabanlı GUI** — hızlı, native, karanlık tema
- 📝 **PDF/JSON rapor çıktısı**

## 🚀 Kurulum

### Önkoşullar

1. **Rust** 1.75+ → https://rustup.rs
2. **smartmontools** → https://www.smartmontools.org/wiki/Download#Windows
   - Kurulumdan sonra `C:\Program Files\smartmontools\bin` **PATH**'e eklenmeli
   - Doğrulama: `smartctl --version` komutu sürüm göstermeli
3. **Yönetici hakları** — zero-fill, secure erase, raw disk I/O için gerekli

### Derleme

```bash
git clone https://github.com/r00tastic/hdd_doktor.git
cd hdd_doktor
cargo build --release
```

Derleme ~2-3 dakika sürer. Çıktı: `target\release\hdd_doktor.exe`

### Çalıştırma

Yönetici olarak PowerShell aç:

```powershell
.\target\release\hdd_doktor.exe
```

> ⚠ Normal kullanıcı modunda SMART okunabilir ama zero-fill / secure erase başarısız olur.

## 📖 Tipik iş akışı

Örnek: **WD5000AVDS** disk için bad sector onarımı

1. **Teşhis** — SMART panelinde:
   - `05 Reallocated Sector Count` — yeniden eşlenmiş sektör sayısı
   - `C5 Current Pending Sector` — henüz eşlenmemiş zayıf sektör
   - `C6 Offline Uncorrectable` — düzeltilemeyen hata
   - C5/C6 > 0 → zayıf sektör var, Zero-Fill çözer
2. **Kısa Self-Test** çalıştır (~2 dk)
3. **Yüzey Taraması** — hatalı sektör LBA'larını listele
4. **Zero-Fill** — tüm diski sıfırla (saatler sürer). HDD her sektöre yazarken zayıf olanı spare havuzdan değiştirir
5. **Uzun Self-Test** — doğrulama
6. **Trend izleme** — her hafta aynı değerleri karşılaştır

## 🛠 Mimari

```
src/
├── main.rs      # eframe App — menü, routing
├── smart.rs     # smartctl JSON parse (ATTRIBUTES + health)
├── disk.rs      # Win32 disk I/O (zero-fill, read sector)
├── saglik.rs    # SMART threshold değerlendirme
├── monitor.rs   # Trend kaydı (JSON append-only)
├── bildirim.rs  # notify-rust desktop notification
├── rapor.rs     # JSON/PDF rapor çıktısı
└── tema.rs      # egui dark theme + renk paleti
```

## 🧩 Bağımlılıklar

| Crate | Amaç |
|---|---|
| `eframe` + `egui` 0.29 | Native GUI |
| `egui_plot` | Trend grafikleri |
| `windows` 0.58 | Win32 disk I/O (IOCTL) |
| `serde` + `serde_json` | Config + trend serialization |
| `chrono` | Timestamp |
| `notify-rust` | Masaüstü bildirim |
| `anyhow` | Error handling |

## ⚠ Güvenlik

- **Zero-Fill** ve **Secure Erase** **TÜM VERİYİ YOK EDER**. Yanlış disk seçimi geri dönülemez — tekrar tekrar sor.
- Önce `wmic diskdrive list brief` ile disk ID'yi doğrula
- Prod sistemde değil, yedek alınmış/test disklerde kullan

## 🤝 Katkı

Issue ve PR'lar hoş karşılanır. SMART attribute veritabanı için eşik değerleri `saglik.rs` içinde — yeni disk modelleri için PR gönderilebilir.

## 📄 Lisans

[MIT](LICENSE)
