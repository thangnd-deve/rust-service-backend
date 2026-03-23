use chrono::Utc;

pub struct DateTime;

impl DateTime {
    pub fn now() -> chrono::DateTime<Utc> {
        Utc::now()
    }

    // pub fn format(dt: chrono::DateTime<Utc>, format: Option<&str>) -> String {
    //     let fmt = format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S");
    //     dt.format(fmt).to_string()
    // }
}
