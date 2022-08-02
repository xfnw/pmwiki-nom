pub mod pmwiki;
pub mod parser;

fn main() {
    use parser::pmwikis;
    println!("{:?}", pmwikis("hello"));
    println!("{:?}", pmwikis("!! im a heading"));
    println!("{:?}", pmwikis("''goodbye'' hello '''again'''"));
}
