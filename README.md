# bz-reaction

## Usage
Add this to you `Cargo.toml`:
```toml
[dependencies]
getopts = "0.2.21"
image = "0.23.14"
rand = "0.8.4"
serde = { version = "1.0.132", features = ["derive"] }
toml = "0.5.8"
```

Argument description...
```bash
cargo run --release -- -h
```

First, create a configuration file.<br/>
Configuration file does not create automatically.

```bash
cargo run --release -- -h
```

Configuretion file is ...
```toml
height = 400 # image area's height
width = 400  # image area's width
alpha = 0.8  # Chemical species `A`'s relative velocity of reaction.
beta = 1.0   # Chemical species `B`'s relative velocity of reaction.
gamma = 1.0  # Chemical species `C`'s relative velocity of reaction.
times = 200  # The number of times to simulate. Number of image files.
file_prefix = "images/file-" # Image file prefix
```

Image file is in png format.<br/>
You can combine multiple PNG files into one APNG file using tools such as `APNG Assembler`.

In Japanese

- 化学反応のなかで時計反応と呼ばれる周期的な反応のシミュレーションを行う。
- 元ネタの[BZ反応のシミュレーション](https://qiita.com/STInverSpinel/items/a7dcfbde0a08063f4d41)を元に使用言語を`Julia`から`Rust`に変更した。
- BZ反応の詳細は[BZ反応 in WikiPedia](https://ja.wikipedia.org/wiki/%E3%83%99%E3%83%AD%E3%82%A6%E3%82%BD%E3%83%95%E3%83%BB%E3%82%B8%E3%83%A3%E3%83%9C%E3%83%81%E3%83%B3%E3%82%B9%E3%82%AD%E3%83%BC%E5%8F%8D%E5%BF%9C)を参照。
<br/>
現象の発見が1951年。再発見が1961年。詳細な機構発表が1972年。結構新しい(?)知見。<br/>
非平衡熱力学の代表例らしい。<br/>
化学反応は、一方向に進むか、双方の反応速度の比率から平衡状態になると習った身からは、青天の霹靂。
<br/>
<br/>
物理でも、2体問題は解析的に解けるが、3体問題は基本解けないので<br/>
<br/>
３つ以上関わるものは複雑になる！<br/>
<br/>
ということなのか？？