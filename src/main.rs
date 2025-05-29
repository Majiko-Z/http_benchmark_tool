mod config;
mod process;
mod qps;

use std::process::exit;
use clap::Parser;
use env_logger;
use config::{HTTP_QUERY, CONFIG_FILE, CMDs, PROCESS_NUM, TOTAL_HTTP_NUM, QPS_NUM, EXPIRE_TIME_SEC};
use ftlog::{info, error};

#[tokio::main]
async fn main() {
    info!("BEGIN");
    env_logger::init();
    let app_cmds = CMDs::parse();
    let mode = app_cmds.mode;
    let process_num = app_cmds.process_num;
    let http_file = app_cmds.http_file;
    let total_num = app_cmds.total_num;
    let qps = app_cmds.qps;
    let expire_secs = app_cmds.expire_secs;

    CONFIG_FILE.set(http_file).unwrap();
    PROCESS_NUM.set(process_num).unwrap();
    QPS_NUM.set(qps).unwrap();
    EXPIRE_TIME_SEC.set(expire_secs).unwrap();

    if HTTP_QUERY.query.len() < total_num as usize {
        error!("{} file. http query num {} less than std input num:{}",
            CONFIG_FILE.get().unwrap(), HTTP_QUERY.query.len(), total_num);
        exit(1);
    }

    TOTAL_HTTP_NUM.set(total_num).unwrap();
    let begin_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let (mut success_time_sec, fail_cnt) = match mode.to_lowercase().as_str() {
        "process" => {
            info!("PROCESS mode enabled");
            process::multi_process_request().await
        }
        "qps" => {
            info!("QPS mode enabled");
            qps::send_request_by_qps(qps as u64).await
        }
        _ => {
            error!("Invalid mode: {}", mode);
            exit(1);
        }
    };
    let end_timestamp = chrono::Utc::now().timestamp_millis() as u64;
    let total_time_diff = end_timestamp - begin_timestamp;
    info!("请求失败数目:{}", fail_cnt);
    info!("请求成功数目:{}", success_time_sec.len());
    success_time_sec.sort();
    if success_time_sec.len() != 0 {
        let sum_cost:u64 = success_time_sec.iter().sum();
        let avg_cost = sum_cost as f64 / success_time_sec.len() as f64;
        let median_cost:f64 = if success_time_sec.len() % 2 == 0 {
            (success_time_sec[success_time_sec.len() / 2 - 1] + success_time_sec[success_time_sec.len() / 2]) as f64 / 2.0
        } else {
            success_time_sec[success_time_sec.len() / 2] as f64
        };

        let p90_cost =success_time_sec[(0.9 * success_time_sec.len() as f64).ceil() as usize - 1] as f64;
        let p99_cost =success_time_sec[(0.99 * success_time_sec.len() as f64).ceil() as usize - 1] as f64;



        info!("发送请求总耗时:{:.3}ms", total_time_diff);
        info!("最小耗时:{:.3}ms", success_time_sec[0]);
        info!("最大耗时:{:.3}ms",success_time_sec[success_time_sec.len() - 1]);
        info!("平均耗时:{:.3}ms", avg_cost);
        info!("耗时中位数:{:.3}ms", median_cost);
        info!("耗时90值:{:.3}ms", p90_cost);
        info!("耗时99值:{:.3}ms", p99_cost);
    }


}
