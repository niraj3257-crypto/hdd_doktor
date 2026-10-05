#[derive(Clone, Copy, PartialEq)]
pub enum Dil {
    Tr,
    En,
}

impl Dil {
    pub fn window_title(&self) -> &'static str {
        match self {
            Self::Tr => "HDD Doktor - Disk Teşhis ve Onarım Merkezi",
            Self::En => "HDD Doktor - Disk Diagnostics & Repair Center",
        }
    }
    pub fn app_name(&self) -> &'static str { "HDD DOKTOR" }
    pub fn version(&self) -> &'static str { "  v0.3" }

    pub fn smartctl_init_fail(&self) -> &'static str {
        match self {
            Self::Tr => "smartctl başlatılamadı. Programı yönetici olarak çalıştırmayı deneyin.",
            Self::En => "smartctl failed to start. Try running as administrator.",
        }
    }
    pub fn smartctl_ready(&self) -> &'static str {
        match self {
            Self::Tr => "smartctl hazır. Diskler taranıyor...",
            Self::En => "smartctl ready. Scanning disks...",
        }
    }
    pub fn smartctl_missing(&self) -> &'static str {
        match self {
            Self::Tr => "smartctl YOK",
            Self::En => "smartctl MISSING",
        }
    }
    pub fn disks_found(&self, n: usize) -> String {
        match self {
            Self::Tr => format!("{} disk bulundu", n),
            Self::En => format!("{} disk(s) found", n),
        }
    }
    pub fn disk_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("Disk hatası: {}", e),
            Self::En => format!("Disk error: {}", e),
        }
    }
    pub fn smart_read_ok(&self, n: usize) -> String {
        match self {
            Self::Tr => format!("SMART okundu ({} attribute)", n),
            Self::En => format!("SMART read ({} attributes)", n),
        }
    }
    pub fn smart_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("SMART hatası: {}", e),
            Self::En => format!("SMART error: {}", e),
        }
    }
    pub fn n_disks(&self, n: usize) -> String {
        match self {
            Self::Tr => format!("{} disk", n),
            Self::En => format!("{} disk(s)", n),
        }
    }

    // Top bar
    pub fn light_theme(&self) -> &'static str {
        match self { Self::Tr => "Aydınlık Tema", Self::En => "Light Theme" }
    }
    pub fn dark_theme(&self) -> &'static str {
        match self { Self::Tr => "Karanlık Tema", Self::En => "Dark Theme" }
    }
    pub fn auto_refresh(&self) -> &'static str {
        match self { Self::Tr => "Otomatik yenile (30 sn)", Self::En => "Auto refresh (30s)" }
    }
    pub fn scan_disks(&self) -> &'static str {
        match self { Self::Tr => "Diskleri Tara", Self::En => "Scan Disks" }
    }
    pub fn language_label(&self) -> &'static str {
        match self { Self::Tr => "EN", Self::En => "TR" }
    }

    // Side panel
    pub fn disks_header(&self) -> &'static str {
        match self { Self::Tr => "DİSKLER", Self::En => "DISKS" }
    }
    pub fn no_disk_found(&self) -> &'static str {
        match self { Self::Tr => "Disk bulunamadı", Self::En => "No disk found" }
    }
    pub fn run_as_admin(&self) -> &'static str {
        match self { Self::Tr => "Yönetici olarak çalıştırın", Self::En => "Run as administrator" }
    }
    pub fn direct_conn(&self) -> &'static str {
        match self { Self::Tr => "Doğrudan", Self::En => "Direct" }
    }

    // Log panel
    pub fn log_header(&self) -> &'static str {
        match self { Self::Tr => "KAYIT", Self::En => "LOG" }
    }
    pub fn clear(&self) -> &'static str {
        match self { Self::Tr => "Temizle", Self::En => "Clear" }
    }
    pub fn n_records(&self, n: usize) -> String {
        match self {
            Self::Tr => format!("{} kayıt", n),
            Self::En => format!("{} entries", n),
        }
    }

    // Central panel
    pub fn select_disk(&self) -> &'static str {
        match self { Self::Tr => "Disk seçin", Self::En => "Select a disk" }
    }

    // Progress
    pub fn operation(&self) -> &'static str {
        match self { Self::Tr => "İŞLEM", Self::En => "OPERATION" }
    }
    pub fn cancel(&self) -> &'static str {
        match self { Self::Tr => "[ İPTAL ET ]", Self::En => "[ CANCEL ]" }
    }
    pub fn stop_test(&self) -> &'static str {
        match self { Self::Tr => "[ TESTI DURDUR ]", Self::En => "[ STOP TEST ]" }
    }
    pub fn elapsed_secs(&self, s: f32) -> String {
        match self {
            Self::Tr => format!("| Geçen: {:.0} sn", s),
            Self::En => format!("| Elapsed: {:.0}s", s),
        }
    }

    // Tabs
    pub fn tab_overview(&self) -> &'static str {
        match self { Self::Tr => "Özet", Self::En => "Overview" }
    }
    pub fn tab_smart(&self) -> &'static str {
        match self { Self::Tr => "SMART Detay", Self::En => "SMART Detail" }
    }
    pub fn tab_tests(&self) -> &'static str {
        match self { Self::Tr => "Testler & Çıktılar", Self::En => "Tests & Output" }
    }
    pub fn tab_maintenance(&self) -> &'static str {
        match self { Self::Tr => "Bakım & Onarım", Self::En => "Maintenance & Repair" }
    }
    pub fn tab_trend(&self) -> &'static str {
        match self { Self::Tr => "Trend", Self::En => "Trend" }
    }
    pub fn tab_report(&self) -> &'static str {
        match self { Self::Tr => "Rapor", Self::En => "Report" }
    }

    // Overview tab sections
    pub fn general(&self) -> &'static str {
        match self { Self::Tr => "GENEL", Self::En => "GENERAL" }
    }
    pub fn connection(&self) -> &'static str {
        match self { Self::Tr => "BAĞLANTI", Self::En => "CONNECTION" }
    }
    pub fn status(&self) -> &'static str {
        match self { Self::Tr => "DURUM", Self::En => "STATUS" }
    }
    pub fn critical_smart(&self) -> &'static str {
        match self { Self::Tr => "KRİTİK SMART", Self::En => "CRITICAL SMART" }
    }
    pub fn health_analysis(&self) -> &'static str {
        match self { Self::Tr => "SAĞLIK ANALİZİ", Self::En => "HEALTH ANALYSIS" }
    }
    pub fn recommended_action(&self) -> &'static str {
        match self { Self::Tr => "ÖNERİLEN AKSİYON", Self::En => "RECOMMENDED ACTION" }
    }
    pub fn temp_chart(&self) -> &'static str {
        match self { Self::Tr => "SICAKLIK GRAFİĞİ", Self::En => "TEMPERATURE CHART" }
    }

    // Field labels
    pub fn model(&self) -> &'static str { "Model" }
    pub fn serial(&self) -> &'static str {
        match self { Self::Tr => "Seri No", Self::En => "Serial" }
    }
    pub fn firmware(&self) -> &'static str { "Firmware" }
    pub fn capacity(&self) -> &'static str {
        match self { Self::Tr => "Kapasite", Self::En => "Capacity" }
    }
    pub fn disk_type(&self) -> &'static str {
        match self { Self::Tr => "Tip", Self::En => "Type" }
    }
    pub fn form_factor(&self) -> &'static str {
        match self { Self::Tr => "Form Faktörü", Self::En => "Form Factor" }
    }
    pub fn conn_label(&self) -> &'static str {
        match self { Self::Tr => "Bağlantı", Self::En => "Connection" }
    }
    pub fn interface(&self) -> &'static str {
        match self { Self::Tr => "Arayüz", Self::En => "Interface" }
    }
    pub fn sector(&self) -> &'static str {
        match self { Self::Tr => "Sektör", Self::En => "Sector" }
    }
    pub fn win_drive(&self) -> &'static str {
        match self { Self::Tr => "Windows Sürücüsü", Self::En => "Windows Drive" }
    }
    pub fn disk_path(&self) -> &'static str {
        match self { Self::Tr => "Disk Yolu", Self::En => "Disk Path" }
    }
    pub fn temperature(&self) -> &'static str {
        match self { Self::Tr => "Sıcaklık", Self::En => "Temperature" }
    }
    pub fn uptime(&self) -> &'static str {
        match self { Self::Tr => "Çalışma Süresi", Self::En => "Power-On Time" }
    }
    pub fn power_cycle(&self) -> &'static str {
        match self { Self::Tr => "Güç Döngüsü", Self::En => "Power Cycles" }
    }
    pub fn smart_health(&self) -> &'static str {
        match self { Self::Tr => "SMART Sağlık", Self::En => "SMART Health" }
    }
    pub fn loading(&self) -> &'static str {
        match self { Self::Tr => "Yükleniyor...", Self::En => "Loading..." }
    }
    pub fn hours_fmt(&self, h: u64) -> String {
        match self {
            Self::Tr => format!("{} saat  (~{:.1} yıl)", h, h as f64 / 8760.0),
            Self::En => format!("{} hours  (~{:.1} years)", h, h as f64 / 8760.0),
        }
    }
    pub fn est_life(&self, y: f32) -> String {
        match self {
            Self::Tr => format!("Tahmini ömür: ~{:.1} yıl", y),
            Self::En => format!("Est. lifespan: ~{:.1} years", y),
        }
    }

    // SMART table
    pub fn no_smart_data(&self) -> &'static str {
        match self { Self::Tr => "SMART verisi yok", Self::En => "No SMART data" }
    }
    pub fn col_id(&self) -> &'static str { "ID" }
    pub fn col_name(&self) -> &'static str {
        match self { Self::Tr => "İsim", Self::En => "Name" }
    }
    pub fn col_value(&self) -> &'static str {
        match self { Self::Tr => "Değer", Self::En => "Value" }
    }
    pub fn col_worst(&self) -> &'static str {
        match self { Self::Tr => "Kötü", Self::En => "Worst" }
    }
    pub fn col_thresh(&self) -> &'static str {
        match self { Self::Tr => "Eşik", Self::En => "Thresh" }
    }
    pub fn col_raw(&self) -> &'static str {
        match self { Self::Tr => "Ham", Self::En => "Raw" }
    }
    pub fn col_status(&self) -> &'static str {
        match self { Self::Tr => "Durum", Self::En => "Status" }
    }

    // Status labels
    pub fn status_error(&self) -> &'static str {
        match self { Self::Tr => "HATA", Self::En => "FAIL" }
    }
    pub fn status_critical(&self) -> &'static str {
        match self { Self::Tr => "KRİTİK", Self::En => "CRITICAL" }
    }
    pub fn status_warning(&self) -> &'static str {
        match self { Self::Tr => "DİKKAT", Self::En => "WARNING" }
    }
    pub fn status_ok(&self) -> &'static str {
        match self { Self::Tr => "Sorunsuz", Self::En => "OK" }
    }

    // Tests tab
    pub fn run_tests(&self) -> &'static str {
        match self { Self::Tr => "TESTLERİ BAŞLAT", Self::En => "RUN TESTS" }
    }
    pub fn short_test(&self) -> &'static str {
        match self { Self::Tr => "Kısa Self-Test (~2 dk)", Self::En => "Short Self-Test (~2 min)" }
    }
    pub fn long_test(&self) -> &'static str {
        match self { Self::Tr => "Uzun Self-Test (saatler)", Self::En => "Long Self-Test (hours)" }
    }
    pub fn get_results(&self) -> &'static str {
        match self { Self::Tr => "Test Sonuçlarını Getir", Self::En => "Get Test Results" }
    }
    pub fn show_error_log(&self) -> &'static str {
        match self { Self::Tr => "Hata Log'unu Göster", Self::En => "Show Error Log" }
    }
    pub fn shortcuts(&self) -> &'static str {
        match self {
            Self::Tr => "Kısayol: Ctrl+T kısa test | F5 yenile | Esc iptal",
            Self::En => "Shortcut: Ctrl+T short test | F5 refresh | Esc cancel",
        }
    }
    pub fn outputs(&self) -> &'static str {
        match self { Self::Tr => "ÇIKTILAR", Self::En => "OUTPUTS" }
    }
    pub fn no_output_yet(&self) -> &'static str {
        match self {
            Self::Tr => "Henüz çıktı yok. Bir işlem çalıştırın.",
            Self::En => "No output yet. Run an operation.",
        }
    }
    pub fn copy(&self) -> &'static str {
        match self { Self::Tr => "Kopyala", Self::En => "Copy" }
    }
    pub fn delete(&self) -> &'static str {
        match self { Self::Tr => "Sil", Self::En => "Delete" }
    }

    // Maintenance tab
    pub fn safe_ops(&self) -> &'static str {
        match self { Self::Tr => "GÜVENLİ İŞLEMLER", Self::En => "SAFE OPERATIONS" }
    }
    pub fn smart_refresh(&self) -> &'static str {
        match self { Self::Tr => "SMART Yenile (F5)", Self::En => "SMART Refresh (F5)" }
    }
    pub fn trend_save(&self) -> &'static str {
        match self { Self::Tr => "Trend Kaydet", Self::En => "Save Trend" }
    }
    pub fn trend_show(&self) -> &'static str {
        match self { Self::Tr => "Trend Göster", Self::En => "Show Trend" }
    }
    pub fn danger_title(&self) -> &'static str {
        match self {
            Self::Tr => "TEHLİKELİ ONARIM - VERİ SİLİNİR",
            Self::En => "DANGEROUS REPAIR - DATA WILL BE ERASED",
        }
    }
    pub fn zerofill_desc(&self) -> &'static str {
        match self {
            Self::Tr => "Zero-Fill diski baştan sona sıfırla doldurur; bad sector'ları remap tetikler.",
            Self::En => "Zero-Fill writes zeros to the entire disk; triggers bad sector remapping.",
        }
    }
    pub fn secure_erase_desc(&self) -> &'static str {
        match self {
            Self::Tr => "Secure Erase ATA komutu ile fabrika sıfırlaması yapar.",
            Self::En => "Secure Erase performs a factory reset via ATA command.",
        }
    }
    pub fn wipe_pattern(&self) -> &'static str {
        match self { Self::Tr => "Silme Deseni:", Self::En => "Wipe Pattern:" }
    }
    pub fn accept_wipe(&self) -> &'static str {
        match self {
            Self::Tr => "Silme için veri kaybını kabul ediyorum",
            Self::En => "I accept data loss for wiping",
        }
    }
    pub fn start_wipe(&self) -> &'static str {
        match self { Self::Tr => "Silme Başlat", Self::En => "Start Wipe" }
    }
    pub fn accept_erase(&self) -> &'static str {
        match self {
            Self::Tr => "Secure Erase için veri kaybını kabul ediyorum",
            Self::En => "I accept data loss for Secure Erase",
        }
    }
    pub fn start_erase(&self) -> &'static str {
        match self { Self::Tr => "Secure Erase Başlat", Self::En => "Start Secure Erase" }
    }
    pub fn chkdsk_title(&self) -> &'static str {
        match self { Self::Tr => "WINDOWS CHKDSK", Self::En => "WINDOWS CHKDSK" }
    }
    pub fn chkdsk_desc(&self) -> &'static str {
        match self {
            Self::Tr => "Windows dosya sistemi tarayıcısı. Sürücü harfi için çalıştırır (yönetici gerekli).",
            Self::En => "Windows file system scanner. Runs for the given drive letter (admin required).",
        }
    }
    pub fn drive_letter(&self) -> &'static str {
        match self { Self::Tr => "Sürücü Harfi:", Self::En => "Drive Letter:" }
    }
    pub fn chkdsk_quick(&self) -> &'static str {
        match self { Self::Tr => "Chkdsk /f (hızlı)", Self::En => "Chkdsk /f (quick)" }
    }
    pub fn chkdsk_full(&self) -> &'static str {
        match self { Self::Tr => "Chkdsk /r (tam tarama, uzun)", Self::En => "Chkdsk /r (full scan, slow)" }
    }

    // Trend tab
    pub fn trend_title(&self) -> &'static str {
        match self { Self::Tr => "TREND İZLEME", Self::En => "TREND TRACKING" }
    }
    pub fn trend_desc(&self) -> &'static str {
        match self {
            Self::Tr => "Zaman içinde SMART değerlerinin nasıl değiştiğini kaydeder ve karşılaştırır.",
            Self::En => "Records and compares how SMART values change over time.",
        }
    }
    pub fn save_snapshot(&self) -> &'static str {
        match self { Self::Tr => "Bu Anın Kaydını Al", Self::En => "Take Snapshot" }
    }
    pub fn compare(&self) -> &'static str {
        match self { Self::Tr => "Karşılaştır", Self::En => "Compare" }
    }
    pub fn tips(&self) -> &'static str {
        match self { Self::Tr => "İPUÇLARI", Self::En => "TIPS" }
    }
    pub fn tip_lines(&self) -> [&'static str; 5] {
        match self {
            Self::Tr => [
                "- Önce bugünün kaydını al",
                "- SATA/USB kablosunu değiştir",
                "- 1 hafta sonra tekrar kaydet + karşılaştır",
                "- Kritik SMART (5, 197, 198, 199) artmıyorsa disk sağlıklı",
                "- Otomatik yenile checkbox'ı ile 30 sn'de bir izlenir",
            ],
            Self::En => [
                "- Take today's snapshot first",
                "- Replace SATA/USB cable",
                "- Save again after 1 week + compare",
                "- If critical SMART (5, 197, 198, 199) isn't rising, disk is healthy",
                "- Auto refresh checkbox monitors every 30s",
            ],
        }
    }

    // Report tab
    pub fn report_text(&self) -> &'static str {
        match self { Self::Tr => "RAPOR (Metin)", Self::En => "REPORT (Text)" }
    }
    pub fn save_html(&self) -> &'static str {
        match self { Self::Tr => "HTML Rapor Kaydet", Self::En => "Save HTML Report" }
    }
    pub fn save_csv(&self) -> &'static str {
        match self { Self::Tr => "CSV Kaydet", Self::En => "Save CSV" }
    }

    // Worker messages
    pub fn test_started(&self, tip: &str) -> String {
        match self {
            Self::Tr => format!("Test başlatıldı ({})", tip),
            Self::En => format!("Test started ({})", tip),
        }
    }
    pub fn test_output_title(&self, tip: &str) -> String {
        match self {
            Self::Tr => format!("Self-Test Başlatma Çıktısı ({})", tip),
            Self::En => format!("Self-Test Start Output ({})", tip),
        }
    }
    pub fn test_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("Test hatası: {}", e),
            Self::En => format!("Test error: {}", e),
        }
    }
    pub fn test_cancelled(&self) -> &'static str {
        match self { Self::Tr => "Test İPTAL edildi", Self::En => "Test CANCELLED" }
    }
    pub fn test_result_cancel(&self) -> &'static str {
        match self { Self::Tr => "Self-Test Sonucu (iptal)", Self::En => "Self-Test Result (cancelled)" }
    }
    pub fn test_result(&self) -> &'static str {
        match self { Self::Tr => "Self-Test Sonucu", Self::En => "Self-Test Result" }
    }
    pub fn test_passed(&self) -> &'static str {
        match self { Self::Tr => "Test BAŞARIYLA tamamlandı", Self::En => "Test completed SUCCESSFULLY" }
    }
    pub fn test_failed(&self) -> &'static str {
        match self { Self::Tr => "Test HATALI bitti - okuma hatası var", Self::En => "Test FAILED - read errors detected" }
    }
    pub fn test_timeout(&self) -> &'static str {
        match self { Self::Tr => "Test zaman aşımına uğradı", Self::En => "Test timed out" }
    }
    pub fn test_done_short(&self) -> &'static str {
        match self { Self::Tr => "Test bitti", Self::En => "Test finished" }
    }
    pub fn test_done_notif(&self) -> &'static str {
        match self { Self::Tr => "Self-Test Bitti", Self::En => "Self-Test Done" }
    }
    pub fn cancel_sent(&self) -> &'static str {
        match self { Self::Tr => "İptal isteği gönderildi...", Self::En => "Cancel request sent..." }
    }
    pub fn starting(&self) -> &'static str {
        match self { Self::Tr => "Başlatılıyor...", Self::En => "Starting..." }
    }

    // Wipe/erase messages
    pub fn zerofill_done(&self) -> &'static str {
        match self { Self::Tr => "Zero-fill tamam. SMART 5 kontrol edin.", Self::En => "Zero-fill complete. Check SMART 5." }
    }
    pub fn zerofill_notif_title(&self) -> &'static str {
        match self { Self::Tr => "Zero-Fill Bitti", Self::En => "Zero-Fill Done" }
    }
    pub fn zerofill_notif_body(&self) -> &'static str {
        match self {
            Self::Tr => "Disk sıfırla dolduruldu. SMART kontrol edin.",
            Self::En => "Disk zeroed out. Check SMART values.",
        }
    }
    pub fn wipe_done(&self, label: &str) -> String {
        match self {
            Self::Tr => format!("{} tamamlandı", label),
            Self::En => format!("{} completed", label),
        }
    }
    pub fn wipe_done_notif(&self) -> &'static str {
        match self { Self::Tr => "Silme Bitti", Self::En => "Wipe Done" }
    }
    pub fn wipe_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("Silme hatası: {}", e),
            Self::En => format!("Wipe error: {}", e),
        }
    }

    // Temperature alarm
    pub fn temp_alarm(&self, t: i32) -> String {
        match self {
            Self::Tr => format!("SICAKLIK ALARMI: {} °C - Disk yanabilir!", t),
            Self::En => format!("TEMPERATURE ALARM: {} °C - Disk may overheat!", t),
        }
    }
    pub fn temp_critical(&self) -> &'static str {
        match self { Self::Tr => "SICAKLIK KRİTİK", Self::En => "TEMPERATURE CRITICAL" }
    }
    pub fn temp_critical_body(&self, t: i32) -> String {
        match self {
            Self::Tr => format!("Disk sıcaklığı {} °C - havalandırın", t),
            Self::En => format!("Disk temperature {} °C - improve cooling", t),
        }
    }

    // Trend messages
    pub fn trend_saved(&self, p: &str) -> String {
        match self {
            Self::Tr => format!("Trend kaydedildi: {}", p),
            Self::En => format!("Trend saved: {}", p),
        }
    }
    pub fn trend_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("Trend hatası: {}", e),
            Self::En => format!("Trend error: {}", e),
        }
    }
    pub fn trend_analysis(&self) -> &'static str {
        match self { Self::Tr => "Trend Analizi", Self::En => "Trend Analysis" }
    }
    pub fn no_trend_yet(&self) -> &'static str {
        match self {
            Self::Tr => "Henüz kayıt yok. Önce 'Trend Kaydet' çalıştırın.",
            Self::En => "No records yet. Run 'Save Trend' first.",
        }
    }
    pub fn need_two_records(&self, n: usize) -> String {
        match self {
            Self::Tr => format!("Karşılaştırma için en az 2 kayıt gerekli. Mevcut: {}", n),
            Self::En => format!("Need at least 2 records to compare. Current: {}", n),
        }
    }

    // Report messages
    pub fn html_saved(&self, p: &str) -> String {
        match self {
            Self::Tr => format!("HTML kaydedildi: {}", p),
            Self::En => format!("HTML saved: {}", p),
        }
    }
    pub fn html_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("HTML hatası: {}", e),
            Self::En => format!("HTML error: {}", e),
        }
    }
    pub fn csv_saved(&self, p: &str) -> String {
        match self {
            Self::Tr => format!("CSV kaydedildi: {}", p),
            Self::En => format!("CSV saved: {}", p),
        }
    }
    pub fn csv_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("CSV hatası: {}", e),
            Self::En => format!("CSV error: {}", e),
        }
    }

    // Health labels
    pub fn health_excellent(&self) -> &'static str {
        match self { Self::Tr => "MÜKEMMEL", Self::En => "EXCELLENT" }
    }
    pub fn health_good(&self) -> &'static str {
        match self { Self::Tr => "İYİ", Self::En => "GOOD" }
    }
    pub fn health_caution(&self) -> &'static str {
        match self { Self::Tr => "DİKKAT", Self::En => "CAUTION" }
    }
    pub fn health_bad(&self) -> &'static str {
        match self { Self::Tr => "KÖTÜ", Self::En => "BAD" }
    }
    pub fn health_critical(&self) -> &'static str {
        match self { Self::Tr => "KRİTİK - YEDEK AL", Self::En => "CRITICAL - BACKUP NOW" }
    }
    pub fn healthy(&self) -> &'static str {
        match self { Self::Tr => "SAĞLIKLI", Self::En => "HEALTHY" }
    }
    pub fn unhealthy(&self) -> &'static str {
        match self { Self::Tr => "SORUNLU (FAILED)", Self::En => "FAILED" }
    }
    pub fn unknown(&self) -> &'static str {
        match self { Self::Tr => "Bilinmiyor", Self::En => "Unknown" }
    }

    // Wipe pattern labels
    pub fn wp_zero(&self) -> &'static str {
        match self { Self::Tr => "Zero-Fill (1 geçiş, hızlı)", Self::En => "Zero-Fill (1 pass, fast)" }
    }
    pub fn wp_random(&self) -> &'static str {
        match self { Self::Tr => "Random (1 geçiş rastgele veri)", Self::En => "Random (1 pass random data)" }
    }
    pub fn wp_dod(&self) -> &'static str {
        match self { Self::Tr => "DoD 5220.22-M (3 geçiş: 0, 1, rastgele)", Self::En => "DoD 5220.22-M (3 pass: 0, 1, random)" }
    }
    pub fn wp_gutmann(&self) -> &'static str {
        match self { Self::Tr => "Gutmann (35 geçiş, çok yavaş)", Self::En => "Gutmann (35 pass, very slow)" }
    }

    // Health analysis strings
    pub fn reallocated_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} bozuk sektör remap edildi (SMART 5)", n),
            Self::En => format!("{} bad sectors remapped (SMART 5)", n),
        }
    }
    pub fn reallocated_action(&self) -> &'static str {
        match self {
            Self::Tr => "Zero-Fill çalıştır, pending sector'ları remap et",
            Self::En => "Run Zero-Fill to trigger pending sector remapping",
        }
    }
    pub fn pending_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} pending sector var (SMART 197) - okuma zorluyor", n),
            Self::En => format!("{} pending sectors (SMART 197) - read difficulties", n),
        }
    }
    pub fn pending_action(&self) -> &'static str {
        match self {
            Self::Tr => "ACIL: Yedek al. Zero-Fill remap tetikleyebilir",
            Self::En => "URGENT: Back up now. Zero-Fill may trigger remapping",
        }
    }
    pub fn uncorrectable_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} düzeltilemez hata (SMART 198)", n),
            Self::En => format!("{} uncorrectable errors (SMART 198)", n),
        }
    }
    pub fn uncorrectable_action(&self) -> &'static str {
        match self {
            Self::Tr => "Diski değiştirin. Güvenilir değil",
            Self::En => "Replace the disk. It is unreliable",
        }
    }
    pub fn crc_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} CRC hatası (SMART 199) - kablo sorunu", n),
            Self::En => format!("{} CRC errors (SMART 199) - cable issue", n),
        }
    }
    pub fn crc_action(&self) -> &'static str {
        match self {
            Self::Tr => "SATA/USB kablosunu değiştirin",
            Self::En => "Replace the SATA/USB cable",
        }
    }
    pub fn realloc_event_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} remap denemesi ama tamamlanmamış (SMART 196)", n),
            Self::En => format!("{} remap attempts but incomplete (SMART 196)", n),
        }
    }
    pub fn reported_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} rapor edilmiş düzeltilemez hata (SMART 187)", n),
            Self::En => format!("{} reported uncorrectable errors (SMART 187)", n),
        }
    }
    pub fn timeout_reason(&self, n: i64) -> String {
        match self {
            Self::Tr => format!("{} komut zaman aşımı (SMART 188)", n),
            Self::En => format!("{} command timeouts (SMART 188)", n),
        }
    }
    pub fn timeout_action(&self) -> &'static str {
        match self {
            Self::Tr => "USB istasyonun gücünü / kablosunu kontrol edin",
            Self::En => "Check USB dock power / cable",
        }
    }
    pub fn thresh_reason(&self, id: u32, v: i32, t: i32) -> String {
        match self {
            Self::Tr => format!("SMART {} eşiğin altına düştü (değer={} <= eşik={})", id, v, t),
            Self::En => format!("SMART {} fell below threshold (value={} <= thresh={})", id, v, t),
        }
    }
    pub fn thresh_action(&self, id: u32) -> String {
        match self {
            Self::Tr => format!("SMART {} kritik eşikte. Diski değiştirin", id),
            Self::En => format!("SMART {} at critical threshold. Replace the disk", id),
        }
    }
    pub fn hot_reason(&self, t: i32) -> String {
        match self {
            Self::Tr => format!("Çok sıcak: {} °C (>= 60)", t),
            Self::En => format!("Very hot: {} °C (>= 60)", t),
        }
    }
    pub fn hot_action(&self) -> &'static str {
        match self {
            Self::Tr => "Havalandırmayı artır. Sıcaklık arıza sebebi",
            Self::En => "Improve ventilation. Heat causes failures",
        }
    }
    pub fn warm_reason(&self, t: i32) -> String {
        match self {
            Self::Tr => format!("Sıcak: {} °C", t),
            Self::En => format!("Warm: {} °C", t),
        }
    }
    pub fn all_clean(&self) -> &'static str {
        match self {
            Self::Tr => "Bütün kritik değerler temiz. Disk sağlıklı.",
            Self::En => "All critical values clean. Disk is healthy.",
        }
    }
    pub fn default_advice(&self) -> &'static str {
        match self {
            Self::Tr => "Periyodik olarak trend kaydet, değişim yoksa endişe etme",
            Self::En => "Save trends periodically, don't worry if no change",
        }
    }
    pub fn high_read_error(&self) -> &'static str {
        match self {
            Self::Tr => "Yüksek raw read error rate (SMART 1)",
            Self::En => "High raw read error rate (SMART 1)",
        }
    }
    pub fn high_seek_error(&self) -> &'static str {
        match self {
            Self::Tr => "Yüksek seek error rate (SMART 7)",
            Self::En => "High seek error rate (SMART 7)",
        }
    }

    // SATA link warning
    pub fn sata_link_warn(&self, speed: &str) -> String {
        match self {
            Self::Tr => format!("Disk SATA 3 desteği olmasına rağmen daha düşük hızda çalışıyor ({}). Kablo/port sorunu olabilir.", speed),
            Self::En => format!("Disk supports SATA 3 but running at a lower speed ({}). Possible cable/port issue.", speed),
        }
    }

    // Display translations for raw DiskInfo fields
    pub fn translate_health(&self, raw: &str) -> &'static str {
        match raw {
            "PASSED" => match self { Self::Tr => "SAĞLIKLI", Self::En => "HEALTHY" },
            "FAILED" => match self { Self::Tr => "SORUNLU (FAILED)", Self::En => "FAILED" },
            _ => match self { Self::Tr => "Bilinmiyor", Self::En => "Unknown" },
        }
    }

    pub fn translate_conn(&self, raw: &str) -> String {
        if raw == "DIRECT" {
            return match self { Self::Tr => "Doğrudan", Self::En => "Direct" }.into();
        }
        raw.to_string()
    }

    pub fn conn_short(&self, raw: &str) -> &'static str {
        if raw.starts_with("USB") {
            return "USB";
        }
        if raw == "DIRECT" {
            return "SATA";
        }
        if raw == "NVMe" {
            return "NVMe";
        }
        "SATA"
    }

    // Chkdsk messages
    pub fn chkdsk_error(&self, e: &str) -> String {
        match self {
            Self::Tr => format!("Chkdsk hatası: {}", e),
            Self::En => format!("Chkdsk error: {}", e),
        }
    }

    // Error log title
    pub fn error_log_title(&self) -> &'static str {
        match self { Self::Tr => "Genişletilmiş Hata Log'u", Self::En => "Extended Error Log" }
    }
    pub fn test_log_title(&self) -> &'static str {
        match self { Self::Tr => "Self-Test Log Detayı", Self::En => "Self-Test Log Detail" }
    }

    // Attribute descriptions
    pub fn attr_desc(&self, id: u32) -> &'static str {
        match self {
            Self::Tr => match id {
                1 => "Raw Read Error Rate - Ham okuma hata oranı. Yüksekse okuma zorluk çekiliyor.",
                3 => "Spin-Up Time - Diskin dönmeye başlama süresi (ms).",
                4 => "Start/Stop Count - Başlama/durma sayısı. Çok yüksekse motor eskiyor.",
                5 => "Reallocated Sectors Count - Bozuk sektör sayısı. > 0 ise disk kötüye gidiyor.",
                7 => "Seek Error Rate - Kafa hareketi hatası. Yüksekse mekanik sorun.",
                9 => "Power-On Hours - Toplam çalışma saati.",
                10 => "Spin Retry Count - Dönme yeniden deneme. Motorun zayıfladığını gösterir.",
                12 => "Power Cycle Count - Güç döngüsü sayısı.",
                184 => "End-to-End Error - İç veri yolu hatası.",
                187 => "Reported Uncorrectable Errors - Düzeltilemez hata sayısı.",
                188 => "Command Timeout - Komut zaman aşımı. USB istasyonlarda sık görülür.",
                190 => "Airflow Temperature - Hava akımı sıcaklığı (°C).",
                193 => "Load Cycle Count - Kafa park sayısı. WD Green'lerde özellikle kritik.",
                194 => "Temperature - Disk sıcaklığı (°C).",
                196 => "Reallocation Event Count - Remap deneme sayısı.",
                197 => "Current Pending Sector - Okuma zorluğu çeken sektör. ACIL.",
                198 => "Offline Uncorrectable - Düzeltilemez arıza. Diski değiştir.",
                199 => "UDMA CRC Error Count - Kablo/arayüz hatası. Kablo değiştir.",
                200 => "Multi-Zone Error Rate - Yazma hata oranı.",
                _ => "SMART attribute",
            },
            Self::En => match id {
                1 => "Raw Read Error Rate - Raw read error rate. High values indicate read difficulty.",
                3 => "Spin-Up Time - Time for the disk to spin up (ms).",
                4 => "Start/Stop Count - Start/stop count. Very high means motor wear.",
                5 => "Reallocated Sectors Count - Bad sector count. > 0 means disk is degrading.",
                7 => "Seek Error Rate - Head movement error. High means mechanical issue.",
                9 => "Power-On Hours - Total operating hours.",
                10 => "Spin Retry Count - Spin retry count. Indicates weakening motor.",
                12 => "Power Cycle Count - Power cycle count.",
                184 => "End-to-End Error - Internal data bus error.",
                187 => "Reported Uncorrectable Errors - Uncorrectable error count.",
                188 => "Command Timeout - Command timeout. Common with USB docks.",
                190 => "Airflow Temperature - Airflow temperature (°C).",
                193 => "Load Cycle Count - Head park count. Especially critical on WD Green drives.",
                194 => "Temperature - Disk temperature (°C).",
                196 => "Reallocation Event Count - Remap attempt count.",
                197 => "Current Pending Sector - Sector with read difficulty. URGENT.",
                198 => "Offline Uncorrectable - Uncorrectable failure. Replace disk.",
                199 => "UDMA CRC Error Count - Cable/interface error. Replace cable.",
                200 => "Multi-Zone Error Rate - Write error rate.",
                _ => "SMART attribute",
            },
        }
    }
}
