use std::env;

const HELP: &str = "\
有道词典命令行版 — 从 dict.youdao.com 查询单词

用法:
  ydcli <WORD>      查询指定单词
  ydcli --help      显示帮助信息
  ydcli --version   显示版本号

示例:
  ydcli world
  ydcli hello";

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None => {
            eprintln!("错误：请输入要查询的单词");
            eprintln!("尝试 'ydcli --help' 查看更多信息");
            std::process::exit(1);
        }
        Some("--help") | Some("-h") => {
            println!("{HELP}");
        }
        Some("--version") | Some("-V") => {
            println!("ydcli-rs {}", env!("CARGO_PKG_VERSION"));
        }
        Some(word) => {
            if let Err(e) = run(word) {
                eprintln!("错误：{e}");
                std::process::exit(1);
            }
        }
    }
}

fn run(word: &str) -> ydcli_rs::Result<()> {
    let html = ydcli_rs::fetch_word(word)?;
    ydcli_rs::output_all(&html, word);
    Ok(())
}
