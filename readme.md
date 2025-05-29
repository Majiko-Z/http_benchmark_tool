# ft_http_test

用于HTTP性能测试，目前市面上压测工具不支持从文件中读取请求并压测。

而实际业务测试时，经常遇到批量POST请求，且每个请求的携带数据不同, 如批量创建一批用户等

# 使用说明

## 编译

项目用到openssl相关，可能需要下载openssl相关库，并设置OpenSSL相关库，这里以mac为为例。

`brew install openssl`
`rustup target add x86_64-unknown-linux-musl`

**编译命令**

`cargo zigbuild --target x86_64-unknown-linux-musl --release`

## process模式

模拟一种压测模式，有N个进程，每个进程按顺序发HTTP请求，直到请求返回再处理下一个。

## qps模式

每秒发送qps个请求

## 命令

```bash
RUST_LOG=info ./http_benchmark_tool --mode qps  --http_file output.json --process_num 5 --total 5 --qps 5 --expire_secs 300
```

1. `RUST_LOG=info`是开启日志打印
2. `--mode` 代表使用哪种模式
3. `--http_file` 代表传入的请求文件
4. `--process_num` process模式下进程数
5. `--qps` qps模式下每秒qps
5. `--total` 总共的请求数。
6. `--expire_secs` qps模式下，每个连接会维持的时间(秒)

   如果是process模式, 则每个进程会发送`total/process_num`个请求；

   如果是qps模式,进程会持续`total`次发送, 每次发送后sleep `1/qps`秒；

**http_file格式**

http_file需要是json文件，这种json文件很容易通过脚本生成

格式示例如下:

```bash
{
    "query": [
        {
            "method": "POST",
            "url": "http://0.0.0.0:9998/task",
            "headers": [
                [
                    "Content-Type",
                    "application/json"
                ]
            ],
            "body": "{\"accountType\": \"1\"}"
        },
        {
            "method": "POST",
            "url": "http://0.0.0.0:9998/task",
            "headers": [
                [
                    "Content-Type",
                    "application/json"
                ]
            ],
            "body": "{\"accountType\": \"1\"}"
        },
    ]
}
```

