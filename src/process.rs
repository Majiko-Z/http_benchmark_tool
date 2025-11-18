use ftlog::{error, info};
use crate::config::{PROCESS_NUM, TOTAL_HTTP_NUM, HTTP_QUERY, EXPIRE_TIME_SEC};
use reqwest::{Client, ClientBuilder};
use chrono;
use std::sync::{Mutex, Arc};
use std::time::Duration;

// pub async fn multi_process_request() -> (Vec<u64>, u64) {
//     let process_num = PROCESS_NUM.get().unwrap();
//     let total_num = TOTAL_HTTP_NUM.get().unwrap();
//     // total_num % process_num = 0;
//     let send_segment = total_num / process_num;
//     let expire_secs = EXPIRE_TIME_SEC.get().unwrap();
//
//     let client = ClientBuilder::new()
//         .http2_adaptive_window(true) // Enable adaptive window for HTTP/2
//         .pool_max_idle_per_host(500) // Allow up to 500 idle connections per host 同时处理500
//         .timeout(Duration::from_secs(*expire_secs as u64))// 连接超时时间
//         .build()
//         .expect("Failed to build client"); // [SAFE]
//
//     let mut handles = vec![];
//
//     let success_cost = Arc::new(Mutex::new(Vec::new()));
//     let failed_cnt = Arc::new(Mutex::new(0));
//
//     for i in 0..*process_num {
//         let client_clone = client.clone();
//         let begin_idx = i * send_segment;
//         let end_idx = (i + 1) * send_segment;
//         let handler = tokio::spawn(async move {
//             for j in begin_idx..end_idx {
//                 let begin_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//                 match HTTP_QUERY.query[j as usize].send_request(Some(&client_clone)).await {
//
//                     Ok(resp) => {
//                         let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//                         let mut success_cost = success_cost.lock().unwrap();
//                         success_cost.push(end_timestamp - begin_timestamp);
//                         info!("OK. Response Cost: {} ms", end_timestamp - begin_timestamp);
//                     }
//                     Err(e) => {
//                         let mut failed_cnt = failed_cnt.lock().unwrap();
//                         *failed_cnt += 1;
//                         let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
//                         error!("Error sending request: {}. Cost: {} ms", e,  end_timestamp - begin_timestamp);
//                     }
//                 }
//             }
//         });
//         handles.push(handler);
//     }
//     for handler in handles {
//         match handler.await {
//             Ok(_) => {}
//             Err(e) => {}
//         }
//     }
//     let success_cost = Arc::try_unwrap(success_cost).unwrap().into_inner().unwrap();
//     let failed_cnt = Arc::try_unwrap(failed_cnt).unwrap().into_inner().unwrap();
//     (success_cost, failed_cnt)
// }

// use std::sync::{Arc, Mutex};
// use tokio::task;
// use tokio::time::Duration;
// use reqwest::ClientBuilder;

pub async fn multi_process_request() -> (Vec<u64>, u64) {
    let process_num = *PROCESS_NUM.get().expect("PROCESS_NUM is not set");
    let total_num = *TOTAL_HTTP_NUM.get().expect("TOTAL_HTTP_NUM is not set");
    let send_segment = total_num / process_num;
    let expire_secs = *EXPIRE_TIME_SEC.get().expect("EXPIRE_TIME_SEC is not set");



    let mut handles = vec![];

    let success_cost = Arc::new(Mutex::new(Vec::new()));
    let failed_cnt = Arc::new(Mutex::new(0));

    for i in 0..process_num {
        let client = ClientBuilder::new()
            .http2_adaptive_window(true) // Enable adaptive window for HTTP/2
            .pool_max_idle_per_host(500) // Allow up to 500 idle connections per host
            .timeout(Duration::from_secs(expire_secs as u64)) // 连接超时时间
            .build()
            .expect("Failed to build client");
        let client_clone = client;
        let begin_idx = i * send_segment;
        let end_idx = (i + 1) * send_segment;
        let success_cost_clone = Arc::clone(&success_cost);
        let failed_cnt_clone = Arc::clone(&failed_cnt);

        let handler = tokio::spawn(async move {
            for j in begin_idx..end_idx {
                let begin_timestamp = chrono::Utc::now().timestamp_millis() as u64;
                match HTTP_QUERY.query[j as usize].send_request(Some(&client_clone)).await {
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
            }
        });
        handles.push(handler);
    }

    for handler in handles {
        match handler.await {
            Ok(_) => {}
            Err(e) => {
                error!("Task failed: {:?}", e);
            }
        }
    }

    let success_cost = success_cost.lock().unwrap().clone();
    let failed_cnt = failed_cnt.lock().unwrap().clone();
    (success_cost, failed_cnt)
}
