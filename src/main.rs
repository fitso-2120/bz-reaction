extern crate getopts;
extern crate rand;
extern crate serde;
extern crate toml;

use getopts::Options;
use image::{ImageBuffer, Rgb};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::Write;

// ベロウソフ・ジャボチンスキー(Belousov-Zhabotinsky, BZ)反応のシミュレーション
// [Qiita BZ反応のシミュレーション](https://qiita.com/STInverSpinel/items/a7dcfbde0a08063f4d41)を参照
// オリジナルはJuliaで書かれていたものをRustに変換

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    // 領域の大きさ
    height: usize,
    width: usize,

    // 反応の強度
    alpha: f64,
    beta: f64,
    gamma: f64,

    // 繰返回数
    times: u32,

    // イメージファイルのプリフィックス
    file_prefix: String,
}

// 確認用に作成。普段は使用しない
#[allow(dead_code)]
fn sum(a: &Vec<Vec<f64>>) -> f64 {
    let mut s = 0.0;
    for i in 0..a.len() {
        for j in 0..a[0].len() {
            s += a[i as usize][j as usize].abs();
        }
    }
    return s;
}

// 配列は、スタック上に確保されるため大きな配列はスタックオーバーフローになる。
// なので、Vecを使用しているが、固定サイズで使用するため、最初の確保のみここに集約する
// 2次元の1次元目の大きさは、`a.len()`、2次元目は、`a[i].len()`のようにその添字での大きさでもよいが、
// 大きさ固定なので、`a[0].len()`としておく
fn mat_init(h: usize, w: usize) -> Vec<Vec<f64>> {
    let mut a: Vec<Vec<f64>>;

    a = Vec::new();
    a.resize(h, Vec::new());
    for i in 0..h {
        a[i] = Vec::new();
        a[i].resize(w, 0.0);
    }

    return a;
}

fn print_usage(pgname: &String, opt: Options) {
    let brief = format!("Usage: {} FILE [options]", pgname);
    print!("{}", opt.usage(&brief));
    print!(
        r#"\
The configuration file has the same directory as this command, and the command name.toml is assumed.\n
To change the configuration file to use, use `-c` or` --config-file`.\n
If there is no config file, `-g` or` --generate-config-file` will generate a config file template. 
"#
    );
}

fn getenv() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let pgname = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config-file", "change configuration-file", "NAME");
    opts.optflag(
        "g",
        "generate-config-file",
        "create template of settings-file",
    );
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&pgname, opts);
        return None;
    }

    // 設定ファイル名を確定
    let config_file_opt = matches.opt_str("c");
    // 拡張子に".toml"を無条件に追加
    let config_file = if config_file_opt == None {
        pgname
    } else {
        config_file_opt.unwrap()
    } + ".toml";

    // 設定ファイル書き出しか？
    if matches.opt_present("g") {
        // 設定ファイルを確定したファイル名で出力
        let config = Config {
            height: 400,
            width: 400,
            alpha: 0.8,
            beta: 1.0,
            gamma: 1.0,
            times: 200,
            file_prefix: "images/file-".to_string(),
        };
        let mut file = match File::create(config_file) {
            Ok(it) => it,
            Err(err) => panic!("{}", err),
        };
        match write!(file, "{}", toml::to_string(&config).unwrap()) {
            Ok(it) => it,
            Err(err) => panic!("{}", err),
        };
        match file.flush() {
            Ok(it) => it,
            Err(err) => panic!("{}", err),
        }
        return None;
    }

    return Some(config_file);
}

struct Graphics {
    imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl Graphics {
    fn new(width: usize, height: usize) -> Self {
        return Graphics {
            imgbuf: ImageBuffer::new(width as u32, height as u32),
        };
    }

    fn set_pixel(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        let pixel = self
            .imgbuf
            .get_pixel_mut(x.try_into().unwrap(), y.try_into().unwrap());
        *pixel = Rgb(rgb);
    }

    fn write(&self, fname: String) {
        self.imgbuf.save(fname).unwrap();
    }
}

fn image_write(config: &Config, t: u32, a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>, c: &Vec<Vec<f64>>) {
    // 各回の領域を画像(PNG)で出力するファイル名
    let fname = format!("{}{:04}.png", config.file_prefix, t);

    // 画像のバッファーを確保
    let mut g = Graphics::new(a[0].len(), a.len());
    // 領域内の各セル（反応の最小単位領域）で各化学種の濃度を色にする
    // 各化学種とも大きさは同じなので代表でaのサイズでループを形成
    for x in 0..a.len() {
        for y in 0..a[x].len() {
            let rgb = [
                (a[x][y] * 256.0) as u8,
                (b[x][y] * 256.0) as u8,
                (c[x][y] * 256.0) as u8,
            ];

            g.set_pixel(x.try_into().unwrap(), y.try_into().unwrap(), rgb);
        }
    }
    g.write(fname);
}

fn main() {
    let config_file_opt = getenv();
    if config_file_opt == None {
        return;
    }
    let config_file = config_file_opt.unwrap();
    // Read file and parse to Setting
    if !std::path::Path::new(&config_file).exists() {
        print!("config-file '{}' is not found\nIf this is the first execution, execute it with `-g` to create a configuration file.", config_file);
        return;
    };
    let config_str: String = match fs::read_to_string(config_file) {
        Ok(it) => it,
        Err(err) => panic!("{}", err),
    };
    let config: Config = toml::from_str(&config_str).unwrap();

    // 領域に３つの化学種を用意
    let mut a = mat_init(config.height, config.width);
    let mut b = mat_init(config.height, config.width);
    let mut c = mat_init(config.height, config.width);

    // 各々の化学種の濃度を[0..1]の範囲で乱数で配置
    rand_area(&mut a);
    rand_area(&mut b);
    rand_area(&mut c);

    // 必要回数反応を繰り返す
    for t in 0..config.times {
        image_write(&config, t, &a, &b, &c);

        // 反応は同期的に行うため、今の濃度を保存する
        let tempa = copy(&a);
        let tempb = copy(&b);
        let tempc = copy(&c);

        //print!("step {:04}, sum(a) = {:.1}, sum(b)= {:.1}, sum(c) = {:.1}\n", t, sum(&a), sum(&b), sum(&c));

        // 領域の各セルについて次の世代への濃度の計算を行う
        for x in 0..a.len() {
            for y in 0..a[x].len() {
                let mut ca = 0.0;
                let mut cb = 0.0;
                let mut cc = 0.0;

                // 近傍の座標を補正
                let i = bound(x, 0, a.len());
                let j = bound(y, 0, a[0].len());
                // 自身と隣接するセルの計9つのセルで濃度を集計
                for ii in i {
                    for jj in j {
                        ca += tempa[ii][jj];
                        cb += tempb[ii][jj];
                        cc += tempc[ii][jj];
                    }
                }

                // 平均の濃度を算出
                ca /= 9.0;
                cb /= 9.0;
                cc /= 9.0;
                //print!("step {:04}, ({:03},{:03}) ca) = {:.1}, cb = {:.1}, cc = {:.1}\n", t, x, y, ca, cb, cc);

                // 次の世代での濃度を計算
                a[x as usize][y as usize] = ca * (1.0 + (config.alpha * cb - config.gamma * cc));
                b[x as usize][y as usize] = cb * (1.0 + (config.beta * cc - config.alpha * ca));
                c[x as usize][y as usize] = cc * (1.0 + (config.gamma * ca - config.beta * cb));
            }
        }

        // 領域全体の計算後、各セルで濃度が[0..1]に収まるように調整
        clamp(&mut a);
        clamp(&mut b);
        clamp(&mut c);
    }
}

// 隣接するセルの計算する際、領域の端を超える場合は、反対側を示す。
// 領域がトーラスとなり、端がない状態になる
fn bound(x: usize, min: usize, max: usize) -> [usize; 3] {
    let xm1 = if x <= min { max - 1 } else { x - 1 };
    let xp1 = if x >= max - 1 { min } else { x + 1 };

    return [xm1, x, xp1];
}

fn copy(a: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut tempa: Vec<Vec<f64>>;

    tempa = Vec::new();
    tempa.resize(a.len(), Vec::new());
    for i in 0..a.len() {
        tempa[i] = Vec::new();
        tempa[i].resize(a[0].len(), 0.0);
        for j in 0..a[i].len() {
            tempa[i][j] = a[i][j];
        }
    }
    return tempa;
}

fn clamp(a: &mut Vec<Vec<f64>>) {
    for i in 0..a.len() {
        for j in 0..a[0].len() {
            a[i][j] = constrain(a[i][j]);
        }
    }
}

fn rand_area(a: &mut Vec<Vec<f64>>) {
    let mut rng = rand::thread_rng();

    for i in 0..a.len() {
        for j in 0..a[0].len() {
            a[i][j] = rng.gen();
        }
    }
}

fn constrain(d: f64) -> f64 {
    if d < 0.0 {
        return 0.0;
    } else if d > 1.0 {
        return 1.0;
    }

    return d;
}
