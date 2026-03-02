use modularitea_libs::infrastructure::news_parser::NewsParser;

fn main() {
    let parser = match NewsParser::new() {
        Ok(parser) => parser,
        Err(err) => {
            eprintln!("failed to init parser: {err}");
            return;
        }
    };

    let items = match parser.blackbox_fetcher() {
        Ok(items) => items,
        Err(err) => {
            eprintln!("failed to fetch feeds: {err}");
            return;
        }
    };

    println!("{:#?}", items);
}