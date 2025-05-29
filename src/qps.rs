use crate::config::{PROCESS_NUM, TOTAL_HTTP_NUM, HTTP_QUERY, EXPIRE_TIME_SEC};
use reqwest::{Client, ClientBuilder};
use tokio::time::{Duration, interval};
use tokio::task;
use chrono;
use ftlog::{info, error};
use std::sync::{Mutex, Arc};

// pub async fn send_request_by_qps(qps: u64) -> (Vec<u64>, u64)  {
//     // Build a client with customized settings
//     let expire_secs = EXPIRE_TIME_SEC.get().unwrap();
//
//     let client = ClientBuilder::new()
//         .http2_adaptive_window(true) // Enable adaptive window for HTTP/2
//         .pool_max_idle_per_host(500) // Allow up to 500 idle connections per host 同时处理500
//         .timeout(Duration::from_secs(*expire_secs as u64))// 连接超时时间
//         .build()
//         .expect("Failed to build client"); // [SAFE]
//
//     let interval_duration = Duration::from_secs_f64(1.0 / qps as f64);
//     let mut interval = interval(interval_duration); // qps interval
//
//     let success_cost = Arc::new(Mutex::new(Vec::new()));
//     let failed_cnt = Arc::new(Mutex::new(0));
//
//     let total_query = *TOTAL_HTTP_NUM.get().unwrap() as usize;
//     for i in 0..std::cmp::min(total_query, HTTP_QUERY.query.len()) {
//
//         let client = client.clone();
//         let req = HTTP_QUERY.query[i].clone(); // Ensure req can be moved into the async block
//         task::spawn(async move {
//             let begin_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//             match req.send_request(Some(&client)).await {
//                 Ok(resp) => {
//                     let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//                     let mut success_cost = success_cost.lock().unwrap();
//                     success_cost.push(end_timestamp - begin_timestamp);
//                     info!("OK. Response Cost: {} ms", end_timestamp - begin_timestamp);
//                 }
//                 Err(e) => {
//                     let mut failed_cnt = failed_cnt.lock().unwrap();
//                     *failed_cnt += 1;
//                     let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//                     error!("Error sending request: {}. Cost: {} ms", e,  end_timestamp - begin_timestamp);
//
//                 }
//             }
//         });
//         interval.tick().await;
//     }
//     let success_cost = Arc::try_unwrap(success_cost).unwrap().into_inner().unwrap();
//     let failed_cnt = Arc::try_unwrap(failed_cnt).unwrap().into_inner().unwrap();
//     (success_cost, failed_cnt)
// }


pub async fn send_request_by_qps(qps: u64) -> (Vec<u64>, u64) {
    // Build a client with customized settings
    let expire_secs = EXPIRE_TIME_SEC.get().unwrap();

    let client = ClientBuilder::new()
        .http2_adaptive_window(true) // Enable adaptive window for HTTP/2
        .pool_max_idle_per_host(500) // Allow up to 500 idle connections per host
        .timeout(Duration::from_secs(*expire_secs as u64)) // 连接超时时间
        .build()
        .expect("Failed to build client"); // [SAFE]

    let interval_duration = Duration::from_secs_f64(1.0 / qps as f64);
    let mut interval = interval(interval_duration); // qps interval

    let success_cost = Arc::new(Mutex::new(Vec::new()));
    let failed_cnt = Arc::new(Mutex::new(0));

    let total_query = *TOTAL_HTTP_NUM.get().unwrap() as usize;
    for i in 0..std::cmp::min(total_query, HTTP_QUERY.query.len()) {
        let client = client.clone();
        let req = HTTP_QUERY.query[i].clone(); // Ensure req can be moved into the async block
        let success_cost_clone = Arc::clone(&success_cost);
        let failed_cnt_clone = Arc::clone(&failed_cnt);

        task::spawn(async move {
            let begin_timestamp = chrono::Utc::now().timestamp_millis() as u64;
            match req.send_request(Some(&client)).await {
                Ok(_) => {
                    let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
                    let mut success_cost = success_cost_clone.lock().unwrap();
                    success_cost.push(end_timestamp - begin_timestamp);
                    info!("OK. Response Cost: {} ms", end_timestamp - begin_timestamp);
                }
                Err(e) => {
                    let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
                    let mut failed_cnt = failed_cnt_clone.lock().unwrap();
                    *failed_cnt += 1;
                    error!("Error sending request: {}. Cost: {} ms", e, end_timestamp - begin_timestamp);
                }
            }
        });

        interval.tick().await;
    }

    let success_cost = success_cost.lock().unwrap().clone();
    let failed_cnt = failed_cnt.lock().unwrap().clone();
    (success_cost, failed_cnt)
}
