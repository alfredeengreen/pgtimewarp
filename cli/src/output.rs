use serde_json::Value;

pub fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    use tabled::{Table, Tabled};
    
    println!("{}", Table::new(rows).with(tabled::settings::Style::modern()));
}
