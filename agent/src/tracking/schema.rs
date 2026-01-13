use anyhow::Result;

pub async fn detect_drift(
    _schema: &str,
    _table: &str,
    _expected_cols: &[String],
) -> Result<bool> {
    Ok(false)
}
