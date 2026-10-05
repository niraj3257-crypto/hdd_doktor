#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod smart;
mod disk;
mod monitor;
mod saglik;
mod bildirim;
mod tema;
mod rapor;
mod embedded;
mod lang;

use eframe::egui::{self, Color32, RichText};
use smart::{DiskInfo, SmartAttribute, TestDurumu};
use saglik::SaglikSkoru;
use tema::Tema;
use lang::Dil;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("HDD Doktor"),
        ..Default::default()
    };

    eframe::run_native(
        "HDD Doktor",
        options,
        Box::new(|cc| {
            let tema = Tema::Karanlik;
            tema.uygula(&cc.egui_ctx);
            Ok(Box::new(HddDoktorApp::new(tema)))
        }),
    )
}

#[derive(Clone)]
enum LogMsg {
    Ok(String),
    Warn(String),
    Err(String),
}

enum WorkerMsg {
    Log(LogMsg),
    Diskler(Vec<DiskInfo>),
    Smart(Vec<SmartAttribute>),
    Progress(f32, String),
    ProgressDone,
    UzunMetin(String, String),
    TestIlerleme(TestDurumu),
    TestBitti,
    Bildirim(String, String, bool),
}

struct HddDoktorApp {
    diskler: Vec<DiskInfo>,
    secili: Option<usize>,
    smart_attrs: Vec<SmartAttribute>,
    saglik: Option<SaglikSkoru>,
    log_lines: Vec<LogMsg>,
    smartctl_ok: bool,
    calisiyor: bool,
    ilerleme: Option<(f32, String)>,
    onay_zerofill: bool,
    onay_erase: bool,
    aktif_sekme: Sekme,
    uzun_metinler: Vec<(String, String)>,
    tema: Tema,
    dil: Dil,
    iptal_bayragi: Arc<AtomicBool>,
    test_durumu: Option<TestDurumu>,
    test_baslama: Option<Instant>,
    sicaklik_gecmisi: Vec<(f32, i32)>,
    otomatik_yenile: bool,
    son_otomatik_yenile: Instant,
    secili_wipe: disk::WipePattern,
    chkdsk_surucu: String,
    tx: Sender<WorkerMsg>,
    rx: Receiver<WorkerMsg>,
}

#[derive(PartialEq, Clone, Copy)]
enum Sekme {
    Ozet,
    Smart,
    Testler,
    Bakim,
    Trend,
    Rapor,
}

impl HddDoktorApp {
    fn new(tema: Tema) -> Self {
        let (tx, rx) = channel();
        let smartctl_ok = smart::smartctl_kurulu_mu();
        let dil = Dil::Tr;

        let mut app = Self {
            diskler: Vec::new(),
            secili: None,
            smart_attrs: Vec::new(),
            saglik: None,
            log_lines: Vec::new(),
            smartctl_ok,
            calisiyor: false,
            ilerleme: None,
            onay_zerofill: false,
            onay_erase: false,
            aktif_sekme: Sekme::Ozet,
            uzun_metinler: Vec::new(),
            tema,
            dil,
            iptal_bayragi: Arc::new(AtomicBool::new(false)),
            test_durumu: None,
            test_baslama: None,
            sicaklik_gecmisi: Vec::new(),
            otomatik_yenile: false,
            son_otomatik_yenile: Instant::now(),
            secili_wipe: disk::WipePattern::Zero,
            chkdsk_surucu: "C".to_string(),
            tx: tx.clone(),
            rx,
        };

        if !smartctl_ok {
            app.log_lines.push(LogMsg::Err(dil.smartctl_init_fail().into()));
        } else {
            app.log_lines.push(LogMsg::Ok(dil.smartctl_ready().into()));
            app.diskleri_yenile();
        }

        app
    }

    fn diskleri_yenile(&mut self) {
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match smart::disk_listele() {
            Ok(d) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.disks_found(d.len()))));
                let _ = tx.send(WorkerMsg::Diskler(d));
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.disk_error(&e.to_string()))));
                let _ = tx.send(WorkerMsg::Diskler(Vec::new()));
            }
        });
    }

    fn smart_yukle(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match smart::smart_oku(&d.yol, &d.tip_flag) {
            Ok(a) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.smart_read_ok(a.len()))));
                let _ = tx.send(WorkerMsg::Smart(a));
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.smart_error(&e.to_string()))));
                let _ = tx.send(WorkerMsg::Smart(Vec::new()));
            }
        });
    }

    fn self_test_baslat(&mut self, tip: &'static str) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        self.iptal_bayragi.store(false, Ordering::Relaxed);
        self.test_baslama = Some(Instant::now());
        let iptal = self.iptal_bayragi.clone();

        thread::spawn(move || {
            match smart::self_test(&d.yol, tip, &d.tip_flag) {
                Ok(s) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.test_started(tip))));
                    let _ = tx.send(WorkerMsg::UzunMetin(dil.test_output_title(tip), s));
                }
                Err(e) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.test_error(&e.to_string()))));
                    let _ = tx.send(WorkerMsg::ProgressDone);
                    return;
                }
            }

            thread::sleep(Duration::from_secs(8));
            let baslama = std::time::Instant::now();
            let max_sure = if tip == "short" {
                Duration::from_secs(15 * 60)
            } else {
                Duration::from_secs(24 * 3600)
            };
            let mut calisir_gordum = false;
            let mut son_kalan: u32 = 100;

            loop {
                if iptal.load(Ordering::Relaxed) {
                    let _ = smart::self_test_iptal(&d.yol, &d.tip_flag);
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Warn(dil.test_cancelled().into())));
                    if let Ok(log) = smart::self_test_log(&d.yol, &d.tip_flag) {
                        let _ = tx.send(WorkerMsg::UzunMetin(dil.test_result_cancel().into(), log));
                    }
                    let _ = tx.send(WorkerMsg::TestBitti);
                    return;
                }

                match smart::test_durumu(&d.yol, &d.tip_flag) {
                    Ok(td) => {
                        if td.aktif || td.kalan_yuzde > 0 {
                            calisir_gordum = true;
                        }
                        son_kalan = td.kalan_yuzde;
                        let _ = tx.send(WorkerMsg::TestIlerleme(td.clone()));

                        let bitti_temiz = calisir_gordum && !td.aktif && td.kalan_yuzde == 0;
                        let zaman_asimi = baslama.elapsed() > max_sure;

                        if bitti_temiz || zaman_asimi {
                            thread::sleep(Duration::from_secs(2));
                            let log = smart::self_test_log(&d.yol, &d.tip_flag)
                                .unwrap_or_else(|e| format!("{}: {}", dil.test_error(""), e));

                            let _ = tx.send(WorkerMsg::UzunMetin(dil.test_result().into(), log.clone()));

                            let sonuc_ozeti = if log.contains("Completed without error") {
                                dil.test_passed()
                            } else if log.contains("read failure") || log.contains("failed") {
                                dil.test_failed()
                            } else if zaman_asimi {
                                dil.test_timeout()
                            } else {
                                dil.test_done_short()
                            };

                            let _ = tx.send(WorkerMsg::Bildirim(dil.test_done_notif().into(), sonuc_ozeti.into(), false));
                            let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(sonuc_ozeti.into())));
                            let _ = tx.send(WorkerMsg::TestBitti);
                            return;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(WorkerMsg::Log(LogMsg::Warn(format!("{}: {}", dil.test_error(""), e))));
                    }
                }

                let bekle = if son_kalan > 50 { 15 } else if son_kalan > 10 { 8 } else { 4 };
                thread::sleep(Duration::from_secs(bekle));
            }
        });
    }

    fn test_sonuclari(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match smart::self_test_log(&d.yol, &d.tip_flag) {
            Ok(s) => {
                let _ = tx.send(WorkerMsg::UzunMetin(dil.test_log_title().into(), s));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(e.to_string())));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn hata_log_goster(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match smart::error_log(&d.yol, &d.tip_flag) {
            Ok(s) => {
                let _ = tx.send(WorkerMsg::UzunMetin(dil.error_log_title().into(), s));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(e.to_string())));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn trend_kaydet(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match monitor::trend_kaydet(&d.yol, &d.tip_flag) {
            Ok(p) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.trend_saved(&p))));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.trend_error(&e.to_string()))));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn trend_goster(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match monitor::trend_ozet(&d.yol, dil) {
            Ok(s) => {
                let _ = tx.send(WorkerMsg::UzunMetin(dil.trend_analysis().into(), s));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Warn(e.to_string())));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn zero_fill_baslat(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        self.iptal_bayragi.store(false, Ordering::Relaxed);
        self.ilerleme = Some((0.0, dil.starting().into()));
        let iptal = self.iptal_bayragi.clone();

        thread::spawn(move || {
            let tx2 = tx.clone();
            let cb = move |oran: f32, msg: String| {
                let _ = tx2.send(WorkerMsg::Progress(oran, msg));
            };
            match disk::zero_fill(&d.yol, iptal, cb) {
                Ok(_) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.zerofill_done().into())));
                    let _ = tx.send(WorkerMsg::Bildirim(
                        dil.zerofill_notif_title().into(),
                        dil.zerofill_notif_body().into(),
                        false,
                    ));
                }
                Err(e) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.wipe_error(&e.to_string()))));
                }
            }
            let _ = tx.send(WorkerMsg::ProgressDone);
        });
    }

    fn secure_erase_baslat(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let tx = self.tx.clone();
        self.calisiyor = true;
        thread::spawn(move || match disk::secure_erase(&d.yol) {
            Ok(s) => {
                let _ = tx.send(WorkerMsg::UzunMetin("Secure Erase".into(), s));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(e.to_string())));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn iptal_et(&mut self) {
        self.iptal_bayragi.store(true, Ordering::Relaxed);
        self.log_lines.push(LogMsg::Warn(self.dil.cancel_sent().into()));
    }

    fn saglik_hesapla(&mut self) {
        if !self.smart_attrs.is_empty() {
            let saat = self
                .secili
                .and_then(|i| self.diskler.get(i))
                .and_then(|d| d.cikis_saati);
            self.saglik = Some(saglik::hesapla(&self.smart_attrs, saat, self.dil));
        }
    }

    fn mesaj_isle(&mut self, ctx: &egui::Context) {
        let dil = self.dil;
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Log(l) => self.log_lines.push(l),
                WorkerMsg::Diskler(d) => {
                    self.diskler = d;
                    if self.secili.is_none() && !self.diskler.is_empty() {
                        self.secili = Some(0);
                        self.smart_yukle();
                    }
                    self.calisiyor = false;
                }
                WorkerMsg::Smart(a) => {
                    self.smart_attrs = a;
                    self.saglik_hesapla();
                    if let Some(idx) = self.secili {
                        if let Some(d) = self.diskler.get(idx) {
                            if let Some(t) = d.sicaklik {
                                let ts = self.son_otomatik_yenile.elapsed().as_secs_f32();
                                self.sicaklik_gecmisi.push((ts, t));
                                if self.sicaklik_gecmisi.len() > 200 {
                                    self.sicaklik_gecmisi.remove(0);
                                }
                                if t >= 60 {
                                    self.log_lines.push(LogMsg::Err(dil.temp_alarm(t)));
                                    let _ = self.tx.send(WorkerMsg::Bildirim(
                                        dil.temp_critical().into(),
                                        dil.temp_critical_body(t),
                                        true,
                                    ));
                                }
                            }
                        }
                    }
                    self.calisiyor = false;
                }
                WorkerMsg::Progress(o, m) => {
                    self.ilerleme = Some((o, m));
                }
                WorkerMsg::ProgressDone => {
                    self.ilerleme = None;
                    self.calisiyor = false;
                }
                WorkerMsg::UzunMetin(baslik, icerik) => {
                    self.uzun_metinler.push((baslik, icerik));
                    self.aktif_sekme = Sekme::Testler;
                }
                WorkerMsg::TestIlerleme(td) => {
                    self.test_durumu = Some(td);
                }
                WorkerMsg::TestBitti => {
                    self.test_durumu = None;
                    self.test_baslama = None;
                    self.calisiyor = false;
                }
                WorkerMsg::Bildirim(b, m, k) => {
                    bildirim::bildir(&b, &m, k);
                }
            }
        }

        if self.otomatik_yenile
            && !self.calisiyor
            && self.son_otomatik_yenile.elapsed() >= Duration::from_secs(30)
        {
            self.son_otomatik_yenile = Instant::now();
            self.smart_yukle();
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::F5) && !self.calisiyor {
                self.smart_yukle();
            }
            if i.key_pressed(egui::Key::Escape) && self.calisiyor {
                self.iptal_et();
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::T) && !self.calisiyor {
                self.self_test_baslat("short");
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                self.tema = if self.tema == Tema::Karanlik {
                    Tema::Aydinlik
                } else {
                    Tema::Karanlik
                };
                self.tema.uygula(ctx);
            }
        });
    }
}

impl eframe::App for HddDoktorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.mesaj_isle(ctx);
        ctx.request_repaint_after(Duration::from_millis(300));

        self.ust_serit(ctx);
        self.sol_disk_paneli(ctx);
        self.alt_log_paneli(ctx);
        self.orta_panel(ctx);
    }
}

impl HddDoktorApp {
    fn ust_serit(&mut self, ctx: &egui::Context) {
        let p = self.tema.palet();
        let dil = self.dil;
        egui::TopBottomPanel::top("ust")
            .exact_height(52.0)
            .frame(egui::Frame::default().fill(p.arka_ikinci).inner_margin(egui::Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(dil.app_name()).size(20.0).strong().color(p.mavi));
                    ui.label(RichText::new(dil.version()).color(p.metin_soluk));
                    ui.separator();
                    ui.label(RichText::new(dil.n_disks(self.diskler.len())).color(p.metin_soluk));

                    if !self.smartctl_ok {
                        ui.separator();
                        ui.colored_label(p.kirmizi, RichText::new(dil.smartctl_missing()).strong());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(dil.language_label()).clicked() {
                            self.dil = match self.dil {
                                Dil::Tr => Dil::En,
                                Dil::En => Dil::Tr,
                            };
                            self.saglik_hesapla();
                        }
                        let tema_metin = if self.tema == Tema::Karanlik { dil.light_theme() } else { dil.dark_theme() };
                        if ui.button(tema_metin).clicked() {
                            self.tema = if self.tema == Tema::Karanlik {
                                Tema::Aydinlik
                            } else {
                                Tema::Karanlik
                            };
                            self.tema.uygula(ctx);
                        }
                        ui.checkbox(&mut self.otomatik_yenile, dil.auto_refresh());
                        if ui.add_enabled(!self.calisiyor, egui::Button::new(dil.scan_disks())).clicked() {
                            self.diskleri_yenile();
                        }
                        if self.calisiyor {
                            ui.spinner();
                        }
                    });
                });
            });
    }

    fn sol_disk_paneli(&mut self, ctx: &egui::Context) {
        let p = self.tema.palet();
        let dil = self.dil;
        egui::SidePanel::left("sol")
            .exact_width(280.0)
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(p.arka_ikinci)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new(dil.disks_header()).size(12.0).strong().color(p.metin_soluk));
                ui.add_space(6.0);

                if self.diskler.is_empty() {
                    ui.label(RichText::new(dil.no_disk_found()).color(p.metin_soluk));
                    ui.label(RichText::new(dil.run_as_admin()).color(p.sari).size(11.0));
                    return;
                }

                let mut yeni_secim: Option<usize> = None;
                for (i, d) in self.diskler.iter().enumerate() {
                    let secili = self.secili == Some(i);
                    if disk_kart(ui, d, secili, &p, dil) {
                        yeni_secim = Some(i);
                    }
                    ui.add_space(6.0);
                }
                if let Some(i) = yeni_secim {
                    if self.secili != Some(i) {
                        self.secili = Some(i);
                        self.smart_yukle();
                        self.sicaklik_gecmisi.clear();
                    }
                }
            });
    }

    fn alt_log_paneli(&mut self, ctx: &egui::Context) {
        let p = self.tema.palet();
        let dil = self.dil;
        egui::TopBottomPanel::bottom("alt")
            .resizable(true)
            .default_height(140.0)
            .min_height(80.0)
            .frame(egui::Frame::default().fill(p.arka_ikinci).inner_margin(egui::Margin::same(10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(dil.log_header()).strong().color(p.metin_soluk));
                    ui.separator();
                    if ui.small_button(dil.clear()).clicked() {
                        self.log_lines.clear();
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(dil.n_records(self.log_lines.len())).color(p.metin_soluk).size(11.0));
                    });
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for l in &self.log_lines {
                            match l {
                                LogMsg::Ok(s) => { ui.colored_label(p.yesil, format!("[OK]  {}", s)); }
                                LogMsg::Warn(s) => { ui.colored_label(p.sari, format!("!  {}", s)); }
                                LogMsg::Err(s) => { ui.colored_label(p.kirmizi, format!("[X]   {}", s)); }
                            }
                        }
                    });
            });
    }

    fn orta_panel(&mut self, ctx: &egui::Context) {
        let p = self.tema.palet();
        let dil = self.dil;
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(p.arka_ana).inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                let Some(idx) = self.secili else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(RichText::new(dil.select_disk()).size(20.0).color(p.metin_soluk));
                    });
                    return;
                };
                let d = self.diskler[idx].clone();

                self.disk_baslik_karti(ui, &d, &p);
                ui.add_space(12.0);

                if let Some((o, msg)) = self.ilerleme.clone() {
                    self.ilerleme_karti(ui, o, &msg, &p);
                    ui.add_space(12.0);
                }

                if let Some(td) = self.test_durumu.clone() {
                    self.test_ilerleme_karti(ui, &td, &p);
                    ui.add_space(12.0);
                }

                self.sekme_bari(ui, &p);
                ui.add_space(8.0);

                match self.aktif_sekme {
                    Sekme::Ozet => self.sekme_ozet(ui, &d, &p),
                    Sekme::Smart => self.sekme_smart(ui, &p),
                    Sekme::Testler => self.sekme_testler(ui, &p),
                    Sekme::Bakim => self.sekme_bakim(ui, &p),
                    Sekme::Trend => self.sekme_trend(ui, &p),
                    Sekme::Rapor => self.sekme_rapor(ui, &d, &p),
                }
            });
    }

    fn disk_baslik_karti(&self, ui: &mut egui::Ui, d: &DiskInfo, p: &tema::Palet) {
        let dil = self.dil;
        egui::Frame::default()
            .fill(p.arka_kart)
            .rounding(8.0)
            .inner_margin(egui::Margin::same(16.0))
            .stroke(egui::Stroke::new(1.0, p.cizgi))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&d.model).size(20.0).strong().color(p.metin));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            etiket_kart(ui, &format!("{:.0} GB", d.kapasite_gb), p.mavi);
                            etiket_kart(ui, &d.tip, p.mor);
                            etiket_kart(ui, &dil.translate_conn(&d.baglanti), p.metin_soluk);
                            if let Some(t) = d.sicaklik {
                                let renk = if t >= 60 { p.kirmizi } else if t >= 55 { p.turuncu } else if t >= 45 { p.sari } else { p.yesil };
                                etiket_kart(ui, &format!("{} °C", t), renk);
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(s) = &self.saglik {
                            let (r, g, b) = s.durum.renk();
                            let renk = Color32::from_rgb(r, g, b);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}", s.puan)).size(38.0).strong().color(renk));
                                    ui.label(RichText::new("/100").size(18.0).color(p.metin_soluk));
                                });
                                ui.label(RichText::new(s.durum.etiket(dil)).strong().color(renk));
                                if let Some(y) = s.kalan_omur_yil {
                                    ui.label(RichText::new(dil.est_life(y)).color(p.metin_soluk).size(11.0));
                                }
                            });
                        } else {
                            ui.spinner();
                        }
                    });
                });
            });
    }

    fn ilerleme_karti(&mut self, ui: &mut egui::Ui, oran: f32, msg: &str, p: &tema::Palet) {
        let dil = self.dil;
        egui::Frame::default()
            .fill(p.arka_kart)
            .rounding(8.0)
            .inner_margin(egui::Margin::same(14.0))
            .stroke(egui::Stroke::new(1.0, p.turuncu))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(dil.operation()).strong().color(p.turuncu));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new(dil.cancel()).color(p.kirmizi)).clicked() {
                            self.iptal_et();
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new(msg).color(p.metin));
                ui.add(egui::ProgressBar::new(oran).show_percentage().fill(p.mavi));
            });
    }

    fn test_ilerleme_karti(&mut self, ui: &mut egui::Ui, td: &TestDurumu, p: &tema::Palet) {
        let dil = self.dil;
        let ilerleme = (100 - td.kalan_yuzde) as f32 / 100.0;
        egui::Frame::default()
            .fill(p.arka_kart)
            .rounding(8.0)
            .inner_margin(egui::Margin::same(14.0))
            .stroke(egui::Stroke::new(1.0, p.mavi))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("SELF-TEST").strong().color(p.mavi));
                    if let Some(bas) = self.test_baslama {
                        ui.label(RichText::new(dil.elapsed_secs(bas.elapsed().as_secs_f32())).color(p.metin_soluk));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new(dil.stop_test()).color(p.kirmizi)).clicked() {
                            self.iptal_et();
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new(&td.durum_metni).color(p.metin));
                ui.add(egui::ProgressBar::new(ilerleme).show_percentage().fill(p.mavi));
            });
    }

    fn sekme_bari(&mut self, ui: &mut egui::Ui, p: &tema::Palet) {
        let dil = self.dil;
        ui.horizontal(|ui| {
            for (s, ad) in [
                (Sekme::Ozet, dil.tab_overview()),
                (Sekme::Smart, dil.tab_smart()),
                (Sekme::Testler, dil.tab_tests()),
                (Sekme::Bakim, dil.tab_maintenance()),
                (Sekme::Trend, dil.tab_trend()),
                (Sekme::Rapor, dil.tab_report()),
            ] {
                let secili = self.aktif_sekme == s;
                let renk = if secili { p.mavi } else { p.metin_soluk };
                let btn = egui::Button::new(RichText::new(ad).color(renk).strong())
                    .fill(if secili { p.arka_vurgu } else { Color32::TRANSPARENT });
                if ui.add(btn).clicked() {
                    self.aktif_sekme = s;
                }
            }
        });
        ui.separator();
    }

    fn sekme_ozet(&self, ui: &mut egui::Ui, d: &DiskInfo, p: &tema::Palet) {
        let dil = self.dil;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.columns(2, |kols| {
                kart_ic(&mut kols[0], dil.general(), p, |ui| {
                    self.bilgi_satir(ui, dil.model(), &d.model, p);
                    self.bilgi_satir(ui, dil.serial(), &d.seri, p);
                    self.bilgi_satir(ui, dil.firmware(), &d.firmware, p);
                    self.bilgi_satir(ui, dil.capacity(), &format!("{:.1} GB", d.kapasite_gb), p);
                    self.bilgi_satir(ui, dil.disk_type(), &d.tip, p);
                    self.bilgi_satir(ui, dil.form_factor(), &d.form_faktoru, p);
                });

                kart_ic(&mut kols[1], dil.connection(), p, |ui| {
                    self.bilgi_satir(ui, dil.conn_label(), &dil.translate_conn(&d.baglanti), p);
                    self.bilgi_satir(ui, dil.interface(), &d.arayuz, p);
                    self.bilgi_satir(ui, "SATA", &d.sata_hiz, p);
                    self.bilgi_satir(ui, dil.sector(), &d.sektor_boyutu, p);
                    self.bilgi_satir(ui, dil.win_drive(), &d.aktif_alan, p);
                    self.bilgi_satir(ui, dil.disk_path(), &d.yol, p);

                    if smart::sata_link_yavas(d) {
                        ui.add_space(4.0);
                        ui.colored_label(p.sari, format!("!  {}", dil.sata_link_warn(&d.sata_hiz)));
                    }
                });
            });

            ui.add_space(10.0);

            ui.columns(2, |kols| {
                kart_ic(&mut kols[0], dil.status(), p, |ui| {
                    let sic = d.sicaklik.map(|t| format!("{} °C", t)).unwrap_or("-".into());
                    self.bilgi_satir(ui, dil.temperature(), &sic, p);
                    let saat = d.cikis_saati.map(|s| dil.hours_fmt(s)).unwrap_or("-".into());
                    self.bilgi_satir(ui, dil.uptime(), &saat, p);
                    let guc = d.guc_dongusu.map(|g| g.to_string()).unwrap_or("-".into());
                    self.bilgi_satir(ui, dil.power_cycle(), &guc, p);
                    self.bilgi_satir(ui, dil.smart_health(), dil.translate_health(&d.saglik), p);
                });

                kart_ic(&mut kols[1], dil.critical_smart(), p, |ui| {
                    if self.smart_attrs.is_empty() {
                        ui.label(RichText::new(dil.loading()).italics().color(p.metin_soluk));
                    } else {
                        let onemli = [(5u32, "Reallocated"), (197, "Pending"), (198, "Uncorrect"), (199, "CRC/Cable"), (196, "Realloc Event"), (187, "Reported"), (188, "Timeout")];
                        for (id, isim) in onemli {
                            if let Some(a) = self.smart_attrs.iter().find(|x| x.id == id) {
                                let renk = if a.raw > 0 {
                                    if id == 199 { p.sari } else { p.kirmizi }
                                } else { p.yesil };
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{} ({}):", isim, id)).color(p.metin_soluk).monospace());
                                    ui.colored_label(renk, RichText::new(a.raw.to_string()).monospace().strong());
                                });
                            }
                        }
                    }
                });
            });

            if let Some(s) = &self.saglik {
                ui.add_space(10.0);
                kart_ic(ui, dil.health_analysis(), p, |ui| {
                    for sb in &s.sebep {
                        ui.label(format!("- {}", sb));
                    }
                    if !s.oneri.is_empty() {
                        ui.add_space(6.0);
                        ui.label(RichText::new(dil.recommended_action()).strong().color(p.mavi));
                        for o in &s.oneri {
                            ui.colored_label(p.sari, format!("-> {}", o));
                        }
                    }
                });
            }

            if !self.sicaklik_gecmisi.is_empty() && self.sicaklik_gecmisi.len() > 3 {
                ui.add_space(10.0);
                kart_ic(ui, dil.temp_chart(), p, |ui| {
                    use egui_plot::{Line, Plot, PlotPoints};
                    let noktalar: PlotPoints = self.sicaklik_gecmisi.iter().map(|(x, y)| [*x as f64, *y as f64]).collect();
                    Plot::new("temp_plot")
                        .height(140.0)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .show(ui, |pui| {
                            pui.line(Line::new(noktalar).color(p.turuncu));
                        });
                });
            }
        });
    }

    fn sekme_smart(&self, ui: &mut egui::Ui, p: &tema::Palet) {
        let dil = self.dil;
        if self.smart_attrs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new(dil.no_smart_data()).color(p.metin_soluk));
            });
            return;
        }

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::default()
                .fill(p.arka_kart)
                .rounding(8.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    egui::Grid::new("smart_grid")
                        .num_columns(7)
                        .striped(true)
                        .spacing([14.0, 4.0])
                        .show(ui, |ui| {
                            for h in [dil.col_id(), dil.col_name(), dil.col_value(), dil.col_worst(), dil.col_thresh(), dil.col_raw(), dil.col_status()] {
                                ui.label(RichText::new(h).strong().color(p.mavi));
                            }
                            ui.end_row();

                            for a in &self.smart_attrs {
                                let (durum, renk) = durum_hesapla(a, p, dil);
                                ui.label(RichText::new(a.id.to_string()).monospace())
                                    .on_hover_text(dil.attr_desc(a.id));
                                ui.label(RichText::new(&a.name).monospace())
                                    .on_hover_text(dil.attr_desc(a.id));
                                ui.label(RichText::new(a.value.to_string()).monospace());
                                ui.label(RichText::new(a.worst.to_string()).monospace());
                                ui.label(RichText::new(a.thresh.to_string()).monospace());
                                ui.label(RichText::new(a.raw.to_string()).monospace().strong());
                                ui.colored_label(renk, RichText::new(durum).strong());
                                ui.end_row();
                            }
                        });
                });
        });
    }

    fn sekme_testler(&mut self, ui: &mut egui::Ui, p: &tema::Palet) {
        let dil = self.dil;
        let disk_var = self.secili.is_some() && !self.calisiyor;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            kart_ic(ui, dil.run_tests(), p, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.add_enabled(disk_var, egui::Button::new(dil.short_test())).clicked() {
                        self.self_test_baslat("short");
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.long_test())).clicked() {
                        self.self_test_baslat("long");
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.get_results())).clicked() {
                        self.test_sonuclari();
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.show_error_log())).clicked() {
                        self.hata_log_goster();
                    }
                });
                ui.add_space(4.0);
                ui.label(RichText::new(dil.shortcuts()).size(11.0).color(p.metin_soluk));
            });

            ui.add_space(10.0);

            if self.uzun_metinler.is_empty() {
                kart_ic(ui, dil.outputs(), p, |ui| {
                    ui.label(RichText::new(dil.no_output_yet()).color(p.metin_soluk));
                });
            } else {
                let mut sil: Option<usize> = None;
                for (i, (baslik, icerik)) in self.uzun_metinler.iter().enumerate().rev() {
                    egui::CollapsingHeader::new(RichText::new(baslik).strong())
                        .default_open(i == self.uzun_metinler.len() - 1)
                        .id_salt(i)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.small_button(dil.copy()).clicked() {
                                    ui.output_mut(|o| o.copied_text = icerik.clone());
                                }
                                if ui.small_button(dil.delete()).clicked() {
                                    sil = Some(i);
                                }
                            });
                            egui::ScrollArea::vertical()
                                .max_height(360.0)
                                .id_salt(("scroll", i))
                                .show(ui, |ui| {
                                    ui.monospace(icerik);
                                });
                        });
                }
                if let Some(i) = sil {
                    self.uzun_metinler.remove(i);
                }
            }
        });
    }

    fn sekme_bakim(&mut self, ui: &mut egui::Ui, p: &tema::Palet) {
        let dil = self.dil;
        let disk_var = self.secili.is_some() && !self.calisiyor;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            kart_ic(ui, dil.safe_ops(), p, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.add_enabled(disk_var, egui::Button::new(dil.smart_refresh())).clicked() {
                        self.smart_yukle();
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.trend_save())).clicked() {
                        self.trend_kaydet();
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.trend_show())).clicked() {
                        self.trend_goster();
                    }
                });
            });

            ui.add_space(10.0);

            egui::Frame::default()
                .fill(p.arka_kart)
                .rounding(8.0)
                .inner_margin(egui::Margin::same(14.0))
                .stroke(egui::Stroke::new(2.0, p.kirmizi))
                .show(ui, |ui| {
                    ui.label(RichText::new(dil.danger_title()).strong().color(p.kirmizi).size(14.0));
                    ui.add_space(6.0);
                    ui.label(RichText::new(dil.zerofill_desc()).color(p.metin_soluk));
                    ui.label(RichText::new(dil.secure_erase_desc()).color(p.metin_soluk));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(dil.wipe_pattern()).strong());
                        egui::ComboBox::from_id_salt("wipe_pattern")
                            .selected_text(self.secili_wipe.etiket(dil))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.secili_wipe, disk::WipePattern::Zero, disk::WipePattern::Zero.etiket(dil));
                                ui.selectable_value(&mut self.secili_wipe, disk::WipePattern::Random, disk::WipePattern::Random.etiket(dil));
                                ui.selectable_value(&mut self.secili_wipe, disk::WipePattern::DoD3, disk::WipePattern::DoD3.etiket(dil));
                                ui.selectable_value(&mut self.secili_wipe, disk::WipePattern::Gutmann, disk::WipePattern::Gutmann.etiket(dil));
                            });
                    });
                    ui.add_space(4.0);

                    ui.checkbox(&mut self.onay_zerofill, dil.accept_wipe());
                    if ui.add_enabled(disk_var && self.onay_zerofill, egui::Button::new(RichText::new(dil.start_wipe()).color(p.kirmizi))).clicked() {
                        self.wipe_baslat();
                        self.onay_zerofill = false;
                    }
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.onay_erase, dil.accept_erase());
                    if ui.add_enabled(disk_var && self.onay_erase, egui::Button::new(RichText::new(dil.start_erase()).color(p.kirmizi))).clicked() {
                        self.secure_erase_baslat();
                        self.onay_erase = false;
                    }
                });

            ui.add_space(10.0);

            kart_ic(ui, dil.chkdsk_title(), p, |ui| {
                ui.label(dil.chkdsk_desc());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(dil.drive_letter());
                    let cvp = egui::TextEdit::singleline(&mut self.chkdsk_surucu).desired_width(40.0);
                    ui.add(cvp);
                    ui.label(":");
                    if ui.add_enabled(!self.calisiyor, egui::Button::new(dil.chkdsk_quick())).clicked() {
                        self.chkdsk_baslat(false);
                    }
                    if ui.add_enabled(!self.calisiyor, egui::Button::new(dil.chkdsk_full())).clicked() {
                        self.chkdsk_baslat(true);
                    }
                });
            });
        });
    }

    fn sekme_trend(&mut self, ui: &mut egui::Ui, p: &tema::Palet) {
        let dil = self.dil;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            kart_ic(ui, dil.trend_title(), p, |ui| {
                ui.label(dil.trend_desc());
                ui.add_space(6.0);
                let disk_var = self.secili.is_some() && !self.calisiyor;
                ui.horizontal(|ui| {
                    if ui.add_enabled(disk_var, egui::Button::new(dil.save_snapshot())).clicked() {
                        self.trend_kaydet();
                    }
                    if ui.add_enabled(disk_var, egui::Button::new(dil.compare())).clicked() {
                        self.trend_goster();
                    }
                });
            });

            ui.add_space(10.0);
            kart_ic(ui, dil.tips(), p, |ui| {
                for line in dil.tip_lines() {
                    ui.label(line);
                }
            });
        });
    }

    fn sekme_rapor(&mut self, ui: &mut egui::Ui, d: &DiskInfo, p: &tema::Palet) {
        let dil = self.dil;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            kart_ic(ui, dil.report_text(), p, |ui| {
                let rapor = self.rapor_metni(d);
                ui.horizontal(|ui| {
                    if ui.button(dil.copy()).clicked() {
                        ui.output_mut(|o| o.copied_text = rapor.clone());
                    }
                    if ui.button(dil.save_html()).clicked() {
                        self.html_rapor_kaydet();
                    }
                    if ui.button(dil.save_csv()).clicked() {
                        self.csv_kaydet();
                    }
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(400.0).id_salt("rapor_scroll").show(ui, |ui| {
                    ui.monospace(rapor);
                });
            });
        });
    }

    fn html_rapor_kaydet(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let attrs = self.smart_attrs.clone();
        let saglik = self.saglik.clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        thread::spawn(move || match rapor::html_rapor(&d, &attrs, saglik.as_ref(), dil) {
            Ok(p) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.html_saved(&p.display().to_string()))));
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.html_error(&e.to_string()))));
            }
        });
    }

    fn csv_kaydet(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let attrs = self.smart_attrs.clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        thread::spawn(move || match rapor::csv_export(&d, &attrs) {
            Ok(p) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.csv_saved(&p.display().to_string()))));
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.csv_error(&e.to_string()))));
            }
        });
    }

    fn wipe_baslat(&mut self) {
        let Some(idx) = self.secili else { return };
        let d = self.diskler[idx].clone();
        let pattern = self.secili_wipe;
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        self.iptal_bayragi.store(false, Ordering::Relaxed);
        self.ilerleme = Some((0.0, format!("{} {}", pattern.etiket(dil), dil.starting())));
        let iptal = self.iptal_bayragi.clone();

        thread::spawn(move || {
            let tx2 = tx.clone();
            let cb = move |oran: f32, msg: String| {
                let _ = tx2.send(WorkerMsg::Progress(oran, msg));
            };
            match disk::wipe(&d.yol, pattern, iptal, cb) {
                Ok(_) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Ok(dil.wipe_done(pattern.etiket(dil)))));
                    let _ = tx.send(WorkerMsg::Bildirim(
                        dil.wipe_done_notif().into(),
                        dil.wipe_done(pattern.etiket(dil)),
                        false,
                    ));
                }
                Err(e) => {
                    let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.wipe_error(&e.to_string()))));
                }
            }
            let _ = tx.send(WorkerMsg::ProgressDone);
        });
    }

    fn chkdsk_baslat(&mut self, tam_tara: bool) {
        let harf = self.chkdsk_surucu.clone();
        let tx = self.tx.clone();
        let dil = self.dil;
        self.calisiyor = true;
        thread::spawn(move || match disk::chkdsk_calistir(&harf, tam_tara) {
            Ok(s) => {
                let _ = tx.send(WorkerMsg::UzunMetin(
                    format!("CHKDSK {}: ({})", harf, if tam_tara { "/r" } else { "/f" }),
                    s,
                ));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::Log(LogMsg::Err(dil.chkdsk_error(&e.to_string()))));
                let _ = tx.send(WorkerMsg::ProgressDone);
            }
        });
    }

    fn rapor_metni(&self, d: &DiskInfo) -> String {
        use std::fmt::Write;
        let dil = self.dil;
        let mut s = String::new();
        let rapor_baslik = match dil { Dil::Tr => "HDD DOKTOR RAPORU", Dil::En => "HDD DOKTOR REPORT" };
        let tarih_lbl = match dil { Dil::Tr => "Tarih", Dil::En => "Date" };
        let _ = writeln!(s, "=== {} ===", rapor_baslik);
        let _ = writeln!(s, "{}: {}", tarih_lbl, chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        let _ = writeln!(s);
        let _ = writeln!(s, "-- {} --", dil.general());
        let _ = writeln!(s, "{:<18}: {}", dil.model(), d.model);
        let _ = writeln!(s, "{:<18}: {}", dil.serial(), d.seri);
        let _ = writeln!(s, "{:<18}: {}", dil.firmware(), d.firmware);
        let _ = writeln!(s, "{:<18}: {:.1} GB", dil.capacity(), d.kapasite_gb);
        let _ = writeln!(s, "{:<18}: {}", dil.disk_type(), d.tip);
        let _ = writeln!(s, "{:<18}: {}", dil.conn_label(), dil.translate_conn(&d.baglanti));
        let _ = writeln!(s, "{:<18}: {} °C", dil.temperature(), d.sicaklik.unwrap_or(0));
        let _ = writeln!(s, "{:<18}: {} h", dil.uptime(), d.cikis_saati.unwrap_or(0));
        let _ = writeln!(s, "{:<18}: {}", dil.power_cycle(), d.guc_dongusu.unwrap_or(0));
        let _ = writeln!(s);
        if let Some(sa) = &self.saglik {
            let _ = writeln!(s, "-- {} --", dil.health_analysis());
            let _ = writeln!(s, "{:<18}: {}/100  ({})", "Score", sa.puan, sa.durum.etiket(dil));
            if let Some(y) = sa.kalan_omur_yil {
                let _ = writeln!(s, "{}", dil.est_life(y));
            }
            for sb in &sa.sebep {
                let _ = writeln!(s, "  - {}", sb);
            }
            for o in &sa.oneri {
                let _ = writeln!(s, "  -> {}", o);
            }
            let _ = writeln!(s);
        }
        let _ = writeln!(s, "-- SMART Attributes --");
        let _ = writeln!(s, "{:>4} {:<28} {:>5} {:>5} {:>5} {:>10}", "ID", dil.col_name(), dil.col_value(), dil.col_worst(), dil.col_thresh(), dil.col_raw());
        for a in &self.smart_attrs {
            let _ = writeln!(s, "{:>4} {:<28} {:>5} {:>5} {:>5} {:>10}", a.id, a.name, a.value, a.worst, a.thresh, a.raw);
        }
        s
    }

    fn bilgi_satir(&self, ui: &mut egui::Ui, etiket: &str, deger: &str, p: &tema::Palet) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{:<18}", etiket)).monospace().color(p.metin_soluk));
            ui.label(RichText::new(deger).monospace().color(p.metin));
        });
    }
}

fn disk_kart(ui: &mut egui::Ui, d: &DiskInfo, secili: bool, p: &tema::Palet, dil: Dil) -> bool {
    let baski = 1.0 + if secili { 1.0 } else { 0.0 };
    let ic_renk = if secili { p.arka_vurgu } else { p.arka_kart };
    let stroke_renk = if secili { p.mavi } else { p.cizgi };

    let resp = egui::Frame::default()
        .fill(ic_renk)
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .stroke(egui::Stroke::new(baski, stroke_renk))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let saglik_renk = if d.saglik == "PASSED" { p.yesil } else if d.saglik == "FAILED" { p.kirmizi } else { p.metin_soluk };
                ui.label(RichText::new("*").size(18.0).color(saglik_renk));
                ui.vertical(|ui| {
                    let kisa = if d.model.len() > 26 { format!("{}...", &d.model[..26]) } else { d.model.clone() };
                    ui.label(RichText::new(kisa).strong().color(p.metin));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{:.0} GB", d.kapasite_gb)).size(11.0).color(p.metin_soluk));
                        ui.label(RichText::new("|").size(11.0).color(p.metin_soluk));
                        ui.label(RichText::new(&d.tip).size(11.0).color(p.metin_soluk));
                    });
                    let baglanti_kisa = dil.conn_short(&d.baglanti);
                    ui.label(RichText::new(baglanti_kisa).size(10.0).color(p.mavi));
                });
            });
        })
        .response;

    resp.interact(egui::Sense::click()).clicked()
}

fn etiket_kart(ui: &mut egui::Ui, metin: &str, renk: Color32) {
    egui::Frame::default()
        .fill(renk.linear_multiply(0.15))
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .stroke(egui::Stroke::new(1.0, renk))
        .show(ui, |ui| {
            ui.label(RichText::new(metin).color(renk).size(12.0).strong());
        });
}

fn kart_ic<R>(ui: &mut egui::Ui, baslik: &str, p: &tema::Palet, ic: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::default()
        .fill(p.arka_kart)
        .rounding(8.0)
        .inner_margin(egui::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0, p.cizgi))
        .show(ui, |ui| {
            ui.label(RichText::new(baslik).strong().color(p.mavi).size(13.0));
            ui.separator();
            ui.add_space(4.0);
            ic(ui)
        })
        .inner
}

fn durum_hesapla(a: &SmartAttribute, p: &tema::Palet, dil: Dil) -> (String, Color32) {
    if !a.when_failed.is_empty() && a.when_failed != "-" {
        return (dil.status_error().into(), p.kirmizi);
    }
    if a.thresh > 0 && a.value <= a.thresh {
        return (dil.status_critical().into(), p.kirmizi);
    }
    let kritik = matches!(a.id, 5 | 187 | 188 | 197 | 198 | 199 | 196);
    if kritik && a.raw > 0 {
        return (dil.status_warning().into(), p.sari);
    }
    (dil.status_ok().into(), p.yesil)
}
