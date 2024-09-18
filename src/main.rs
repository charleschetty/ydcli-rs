mod result;
mod tools;
use result::info::{Example, Paraphrase, Pronounce, Variant};
use std::env;
use tools::color::_RED;
use tools::format_data;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2{
        println!("Invalid input!");
        return;
    }

    let url = format!("http://dict.youdao.com/search?q={}", &args[1]);
    let resp = match reqwest::blocking::get(url) {
        Ok(res) => res.text().unwrap(),
        _ => {
            println!("Can not connect to http://dict.youdao.com ");
            return;
        }
    };

    let doc = scraper::Html::parse_document(&resp);

    println!("{}", format_data(&args[1], _RED));
    let pronounce = Pronounce::get(&doc);
    pronounce.output();

    let paraphrase = Paraphrase::get(&doc);
    paraphrase.output();

    let rank = Variant::get(&doc);
    rank.output();
    println!("");

    let examples = Example::get(&doc);
    examples.output();
}
