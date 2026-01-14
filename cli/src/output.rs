use serde_json::Value;

pub fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    // Print headers
    println!("{}", headers.join(" | "));
    println!("{}", "-".repeat(headers.len() * 15));

    // Print rows
    for row in rows {
        println!("{}", row.join(" | "));
    }
}
