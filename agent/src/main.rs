use std::time::Duration;

use serde::Serialize;
use tokio::time;
use uuid::Uuid;

#[derive(Debug)]
struct CollectedSysInfo {
    hostname: String,
    os_release: String,
    os: String,
    uptime_minutes: u64,
}
#[derive(Serialize)]
struct Checkin {
    os: String,
    hostname: String,
    mac: String,
    ip: String,
}

async fn collect_system_info() -> Result<CollectedSysInfo, Box<dyn std::error::Error>> {
    let hostname = sys_info::hostname().unwrap_or_else(|_| "unknown".to_string());
    let os_release = sys_info::os_release().unwrap_or_else(|_| "unknown".to_string());

    let boottime = sys_info::boottime().unwrap();
    let uptime_minutes =
        (time::Instant::now().elapsed().as_secs() as i64 - boottime.tv_sec).abs() as u64 / 60;
    let os = sys_info::os_type().unwrap();

    Ok(CollectedSysInfo {
        hostname,
        os_release,
        uptime_minutes,
        os,
    })
}

#[tokio::main]
async fn main() {
    let checkin = "http://127.0.0.1:8080/checkin";

    let clinet = reqwest::Client::new();

    let mut interval = time::interval(Duration::from_secs(20));
    let uuid: Uuid = Uuid::new_v4();

    loop {
        interval.tick().await;

        let info = collect_system_info().await.unwrap();
        let chekin_data = Checkin {
            hostname: info.hostname,
            os: info.os,
            mac: String::from("00000000.00000000.00000001"),
            ip: String::from("127.0.0.1"),
        };

        let response = clinet
            .post(checkin)
            .json(&chekin_data)
            .send()
            .await
            .unwrap();
        println!("{:#?}", response);

        // if let Err(_) = send_checkin(checkin) {

        // }
    }
}
