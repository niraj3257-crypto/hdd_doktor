use eframe::egui::{self, Color32, Visuals};

#[derive(Clone, Copy, PartialEq)]
pub enum Tema {
    Karanlik,
    Aydinlik,
}

pub struct Palet {
    pub arka_ana: Color32,
    pub arka_ikinci: Color32,
    pub arka_kart: Color32,
    pub arka_vurgu: Color32,
    pub metin: Color32,
    pub metin_soluk: Color32,
    pub cizgi: Color32,
    pub mavi: Color32,
    pub yesil: Color32,
    pub sari: Color32,
    pub turuncu: Color32,
    pub kirmizi: Color32,
    pub mor: Color32,
}

impl Tema {
    pub fn palet(&self) -> Palet {
        match self {
            Tema::Karanlik => Palet {
                arka_ana: Color32::from_rgb(24, 26, 32),
                arka_ikinci: Color32::from_rgb(32, 35, 42),
                arka_kart: Color32::from_rgb(40, 44, 52),
                arka_vurgu: Color32::from_rgb(52, 58, 70),
                metin: Color32::from_rgb(230, 233, 240),
                metin_soluk: Color32::from_rgb(150, 155, 165),
                cizgi: Color32::from_rgb(60, 65, 75),
                mavi: Color32::from_rgb(80, 155, 255),
                yesil: Color32::from_rgb(80, 210, 130),
                sari: Color32::from_rgb(240, 200, 80),
                turuncu: Color32::from_rgb(245, 140, 60),
                kirmizi: Color32::from_rgb(240, 90, 90),
                mor: Color32::from_rgb(180, 130, 240),
            },
            Tema::Aydinlik => Palet {
                arka_ana: Color32::from_rgb(245, 247, 250),
                arka_ikinci: Color32::from_rgb(235, 238, 242),
                arka_kart: Color32::from_rgb(255, 255, 255),
                arka_vurgu: Color32::from_rgb(220, 225, 235),
                metin: Color32::from_rgb(30, 33, 40),
                metin_soluk: Color32::from_rgb(110, 115, 125),
                cizgi: Color32::from_rgb(210, 215, 225),
                mavi: Color32::from_rgb(35, 110, 220),
                yesil: Color32::from_rgb(30, 160, 90),
                sari: Color32::from_rgb(200, 160, 40),
                turuncu: Color32::from_rgb(215, 110, 30),
                kirmizi: Color32::from_rgb(210, 60, 60),
                mor: Color32::from_rgb(140, 90, 200),
            },
        }
    }

    pub fn uygula(&self, ctx: &egui::Context) {
        let p = self.palet();
        let mut vis = match self {
            Tema::Karanlik => Visuals::dark(),
            Tema::Aydinlik => Visuals::light(),
        };
        vis.panel_fill = p.arka_ana;
        vis.window_fill = p.arka_kart;
        vis.extreme_bg_color = p.arka_ikinci;
        vis.faint_bg_color = p.arka_ikinci;
        vis.override_text_color = Some(p.metin);
        vis.widgets.noninteractive.bg_fill = p.arka_ikinci;
        vis.widgets.inactive.bg_fill = p.arka_kart;
        vis.widgets.hovered.bg_fill = p.arka_vurgu;
        vis.widgets.active.bg_fill = p.arka_vurgu;
        vis.selection.bg_fill = p.mavi.linear_multiply(0.4);
        vis.selection.stroke.color = p.mavi;
        vis.hyperlink_color = p.mavi;
        ctx.set_visuals(vis);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.window_margin = egui::Margin::same(10.0);
        style.spacing.indent = 20.0;
        ctx.set_style(style);
    }
}
