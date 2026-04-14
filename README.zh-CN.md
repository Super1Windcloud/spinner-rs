# spinner-rs

[English README](./README.md)

一个用于命令行程序的轻量级 Rust spinner 库，适合在任务执行期间展示进度状态，并在结束时输出成功、失败、警告或提示结果。

## 功能

- 后台线程驱动的终端动画
- 运行时动态更新文本、前缀和后缀
- 内置 `succeed`、`fail`、`warn`、`info` 状态输出
- 支持自定义帧序列和刷新间隔
- 支持颜色、缩进、光标显示和输出流配置
- 自动检测 TTY

## 安装

cargo add spinner-rs

在发布到 crates.io 之前，建议先使用本地路径依赖。

## 快速开始

```rust
use std::{io, thread, time::Duration};

use spinner_rs::Spinner;

fn main() -> io::Result<()> {
    let mut spinner = Spinner::new("Connecting to service");

    spinner.start()?;
    thread::sleep(Duration::from_secs(1));

    spinner.set_text("Downloading data");
    thread::sleep(Duration::from_secs(1));

    spinner.succeed(Some("Completed"))?;
    Ok(())
}
```

## Builder API

```rust
use std::{io, thread, time::Duration};

use spinner_rs::{Color, Spinner, SpinnerStyle, Stream};

fn main() -> io::Result<()> {
    let mut spinner = Spinner::builder()
        .text("Building assets")
        .prefix_text("worker-1")
        .suffix_text("queued")
        .color(Color::Yellow)
        .spinner(SpinnerStyle::arrow())
        .indent(2)
        .hide_cursor(true)
        .stream(Stream::Stderr)
        .build();

    spinner.start()?;
    thread::sleep(Duration::from_secs(2));
    spinner.succeed(Some("Assets ready"))?;
    Ok(())
}
```

如果你想先组合配置，再延后创建 spinner，可以使用 `SpinnerBuilder::build_options()` 或 `SpinnerOptions::builder()`。

## 示例

```bash
cargo run --example basic
cargo run --example custom_style
```

## 测试

```bash
cargo test
```

## API 概览

- `Spinner::new(text)`
- `Spinner::builder()`
- `Spinner::with_options(options)`
- `SpinnerBuilder::build()`
- `SpinnerBuilder::build_options()`
- `start()`
- `stop()`
- `clear()`
- `snapshot()`
- `set_text(...)`
- `set_prefix_text(...)`
- `set_suffix_text(...)`
- `set_color(...)`
- `set_spinner(...)`
- `succeed(...)`
- `fail(...)`
- `warn(...)`
- `info(...)`
- `stop_and_persist(...)`
