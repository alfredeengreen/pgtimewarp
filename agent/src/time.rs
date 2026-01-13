use chrono::{DateTime, Utc};

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn parse_pg_lsn(lsn: &str) -> Option<u64> {
    let parts: Vec<&str> = lsn.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let high = parts[0].parse::<u64>().ok()?;
    let low = parts[1].parse::<u64>().ok()?;
    
    Some((high << 32) | low)
}

pub fn format_pg_lsn(lsn: u64) -> String {
    let high = lsn >> 32;
    let low = lsn & 0xFFFFFFFF;
    format!("{:X}/{:X}", high, low)
}
