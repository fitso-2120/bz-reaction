use eframe::{egui, egui::ColorImage, emath::History};
use egui::TextureHandle;
// use image::{ImageBuffer, Rgba};
use rand::{rngs::ThreadRng, Rng};

/// release/debugモードの表示
/// debugモードでは、警告表示
/// releaseモードでは、パッケージ名とバージョンを表示する
fn show_build_mode(ui: &mut egui::Ui) {
    if cfg!(debug_assertions) {
        ui.label(
            egui::RichText::new("‼ Debug build ‼")
                .small()
                .color(egui::Color32::RED),
        )
        .on_hover_text("This module is created in debug mode");
    } else {
        ui.label(
            egui::RichText::new(format!(
                "‼ {} Ver.{} ‼",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .small()
            .color(egui::Color32::GREEN),
        )
        .on_hover_text("This mode is created in release mode.");
    }
}

// このままだと、"0.1"で、"."を消すと"1"になる。一桁では小数点を挿入できないなど、数値入力として不自然な動作になる。
fn input_f32(ui: &mut egui::Ui, mut s: f32) -> f32 {
    let mut w = s.to_string();
    let res = ui.text_edit_singleline(&mut w);
    if res.changed() {
        s = if let Ok(x) = w.parse() { x } else { 0.0 };
    }
    return s;
}

fn input_usize(ui: &mut egui::Ui, mut s: usize) -> usize {
    let mut w = s.to_string();
    let res = ui.text_edit_singleline(&mut w);
    if res.changed() {
        s = if let Ok(x) = w.parse() { x } else { 0 };
    }
    return s;
}

// ジェネリックでまとめたいのだけれど、コンパイルが通らない・・・
// fn input<T>(ui: &mut egui::Ui, mut s: T) -> T {
//     let mut w = match  std::any::type_name::<T>() {
//         f32 => s.to_string(),
//         usize => s.to_string(),
//     };
//     let res = ui.text_edit_singleline(&mut w);
//     if res.changed() {
//         s = if let Ok(x) = w.parse() { x } else { 0 };
//     }
//     return s;
// }

fn popup_window(ctx: &egui::Context, s: &mut BzReactionApp) -> bool {
    let mut ret = true;
    egui::Window::new("Settings Window").show(ctx, |ui| {
        ui.heading("BZ-Reaction Settings");
        ui.horizontal(|ui| {
            ui.label("Width: ");
            s.width = input_usize(ui, s.width);
        });
        ui.horizontal(|ui| {
            ui.label("Height: ");
            s.height = input_usize(ui, s.height);
        });
        ui.horizontal(|ui| {
            ui.label("Alpha: ");
            s.alpha = input_f32(ui, s.alpha);
        });
        ui.horizontal(|ui| {
            ui.label("Beta: ");
            s.beta = input_f32(ui, s.beta);
        });
        ui.horizontal(|ui| {
            ui.label("Gamma: ");
            s.gamma = input_f32(ui, s.gamma);
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                // 内容を保存してウィンドウを閉じる
                ret = false;
            }
            if ui.button("Cancel").clicked() {
                // 何もせずウィンドウを閉じる
                ret = false;
            }
        });
    });
    return ret;
}

/// アプリケーションで使用するメンバー変数を定義する。
pub struct BzReactionApp {
    width: usize,
    height: usize,
    times: u32,
    update_cycle: u32,
    alpha: f32,
    beta: f32,
    gamma: f32,
    a: Vec<Vec<f32>>,
    b: Vec<Vec<f32>>,
    c: Vec<Vec<f32>>,
    tmp_a: Vec<Vec<f32>>,
    tmp_b: Vec<Vec<f32>>,
    tmp_c: Vec<Vec<f32>>,

    popup_dialog: bool,
    is_running: bool,
    texture: Option<TextureHandle>,

    rng: ThreadRng,

    frame_times: History<f32>,
}

/// アプリケーションで使用するメソッドを定義する
impl Default for BzReactionApp {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;

        Self {
            width: 0,
            height: 0,
            times: 0,
            update_cycle: 1,
            alpha: 0.0,
            beta: 0.0,
            gamma: 0.0,
            a: Vec::new(),
            b: Vec::new(),
            c: Vec::new(),
            tmp_a: Vec::new(),
            tmp_b: Vec::new(),
            tmp_c: Vec::new(),
            popup_dialog: false,
            is_running: false,
            texture: None,
            rng: rand::thread_rng(),

            frame_times: History::new(0..max_len, max_age),
        }
    }
}

impl BzReactionApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let mut w = Self::default();
        w.width = 400;
        w.height = 400;

        // w.update_cycle = 10;
        w.alpha = 0.8;
        w.beta = 1.0;
        w.gamma = 1.0;
        w.mat_init(w.width, w.height);

        return w;
    }

    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previous_frame_time); // projected
    }
    pub fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }

    pub fn mat_init(&mut self, w: usize, h: usize) {
        fn init(w: usize, h: usize) -> Vec<Vec<f32>> {
            let mut a: Vec<Vec<f32>>;

            a = Vec::new();
            a.resize(h, Vec::new());
            for i in 0..h {
                a[i] = Vec::new();
                a[i].resize(w, 0.0);
            }

            return a;
        }

        self.a = init(w, h);
        self.b = init(w, h);
        self.c = init(w, h);
        self.tmp_a = init(w, h);
        self.tmp_b = init(w, h);
        self.tmp_c = init(w, h);
    }

    pub fn calc(&mut self) {
        // 隣接するセルの計算する際、領域の端を超える場合は、反対側を示す。
        // 領域がトーラスとなり、端がない状態になる
        pub fn bound(x: usize, min: usize, max: usize) -> [usize; 3] {
            let xm1 = if x <= min { max - 1 } else { x - 1 };
            let xp1 = if x >= max - 1 { min } else { x + 1 };

            return [xm1, x, xp1];
        }

        // 反応は同期的に行うため、今の濃度を保存する
        self.copy();

        //print!("step {:04}, sum(a) = {:.1}, sum(b)= {:.1}, sum(c) = {:.1}\n", t, sum(&a), sum(&b), sum(&c));

        // 領域の各セルについて次の世代への濃度の計算を行う
        for x in 0..self.a.len() {
            for y in 0..self.a[x].len() {
                let mut ca = 0.0;
                let mut cb = 0.0;
                let mut cc = 0.0;

                // 近傍の座標を補正
                let i = bound(x, 0, self.a.len());
                let j = bound(y, 0, self.a[0].len());
                // 自身と隣接するセルの計9つのセルで濃度を集計
                for ii in i {
                    for jj in j {
                        ca += self.tmp_a[ii][jj];
                        cb += self.tmp_b[ii][jj];
                        cc += self.tmp_c[ii][jj];
                    }
                }

                // 平均の濃度を算出
                ca /= 9.0;
                cb /= 9.0;
                cc /= 9.0;
                //print!("step {:04}, ({:03},{:03}) ca) = {:.1}, cb = {:.1}, cc = {:.1}\n", t, x, y, ca, cb, cc);

                // 次の世代での濃度を計算
                self.a[x][y] = ca * (1.0 + (self.alpha * cb - self.gamma * cc));
                self.b[x][y] = cb * (1.0 + (self.beta * cc - self.alpha * ca));
                self.c[x][y] = cc * (1.0 + (self.gamma * ca - self.beta * cb));
            }
        }

        // 領域全体の計算後、各セルで濃度が[0..1]に収まるように調整
        self.clamp();

        // println!("{:.3},{:.3},{:.3}", self.a[0][0], self.b[0][0], self.c[0][0]);
    }

    fn copy(&mut self) {
        for i in 0..self.a.len() {
            for j in 0..self.a[i].len() {
                self.tmp_a[i][j] = self.a[i][j];
                self.tmp_b[i][j] = self.b[i][j];
                self.tmp_c[i][j] = self.c[i][j];
            }
        }
    }

    pub fn clamp(&mut self) {
        pub fn constrain(d: f32) -> f32 {
            if d < 0.0 {
                return 0.0;
            } else if d > 1.0 {
                return 1.0;
            }

            return d;
        }

        for i in 0..self.a.len() {
            for j in 0..self.a[0].len() {
                self.a[i][j] = constrain(self.a[i][j]);
                self.b[i][j] = constrain(self.b[i][j]);
                self.c[i][j] = constrain(self.c[i][j]);
            }
        }
    }

    pub fn rand_area(&mut self) {
        for i in 0..self.a.len() {
            for j in 0..self.a[0].len() {
                self.a[i][j] = self.rng.gen();
                self.b[i][j] = self.rng.gen();
                self.c[i][j] = self.rng.gen();
            }
        }
    }

    pub fn paint_area(&self) -> ColorImage {
        // 画像のバッファーを確保。eguiのimageとの関連で、カラーはRGBAした。
        // 領域内の各セル（反応の最小単位領域）で各化学種の濃度を色にする
        // 各化学種とも大きさは同じなので代表でaのサイズでループを形成
        let mut pixels = Vec::new();

        for iy in 0..self.height {
            for ix in 0..self.width {
                pixels.push((self.a[ix][iy] * 256.0) as u8);
                pixels.push((self.b[ix][iy] * 256.0) as u8);
                pixels.push((self.c[ix][iy] * 256.0) as u8);
            }
        }

        let imgbuf = ColorImage::from_rgb([self.width, self.height], pixels.as_slice());
        return imgbuf;
    }
}

/// EGUIのアプリケーション用拡張を定義する。
/// name()が名前（タイトル名）取得用、update()が画面更新用に必要。残りはデフォルトに任せる
impl eframe::App for BzReactionApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        /*
        上にメニューバーとビルド情報 Debug/Release
        項目は、Settings,Reset,Start,Pause,Stop,Quit
        下にステータスバー。表示項目はSettingsの内容

         */
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                    if ui.button("🔧 Settings").clicked() {
                        self.popup_dialog = true;
                    }
                    if self.popup_dialog {
                        self.popup_dialog = popup_window(ctx, self);
                    };
                    if self.is_running {
                        if ui.button("Stop").clicked() {
                            self.is_running = false;
                        }
                    } else {
                        if ui.button("Run").clicked() {
                            self.is_running = true;
                            if self.times == 0 {
                                self.mat_init(self.width, self.height);
                                self.rand_area();
                            }
                        }
                        if ui.button("Reset").clicked() {
                            self.times = 0;
                            self.rand_area();
                        }
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    show_build_mode(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_running {
                self.times += 1;
                if self.times % self.update_cycle == 0 {
                    self.calc();
                }
                let image_buffer = self.paint_area();

                let texture =
                    ui.ctx()
                        .load_texture("BZ-Reaction-image", image_buffer, Default::default());

                self.texture = Some(texture.clone());
            }

            if self.texture != None {
                if let Some(texture) = &self.texture {
                    ui.image(texture, texture.size_vec2());
                }
            }
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("({}, {}) ", self.width, self.height));
                ui.label(format!("times={}", self.times));
                ui.label(format!("({}, {}, {})", self.alpha, self.beta, self.gamma));

                if self.is_running {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        ui.label(format!("FPS {:.1}", self.fps()));
                        ui.separator();
                    });
                }
            });
        });

        if self.is_running {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        follow_system_theme: false,

        ..Default::default()
    };

    eframe::run_native(
        "BZ-Reaction",
        options,
        Box::new(|cc| Box::new(BzReactionApp::new(cc))),
    )
}
