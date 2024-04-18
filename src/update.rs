use std::{
    env::var_os,
    fs::read_to_string,
    io::Result,
    process::{Child, Command, Stdio},
};

use log::info;

use crate::{
    config::Feed,
    models::{sqlite, Content, Rss, Source},
    writer,
};

use std::time::Duration;
use rand::{thread_rng, Rng};

pub fn update(feed: &Feed) {
    let connection = sqlite::open();

    loop {
        // 随机延迟0到600秒
        let delay = &feed.delay;
        if *delay == true {
            let sleep_time = thread_rng().gen_range(0..600);
            std::thread::sleep(Duration::from_secs(sleep_time));
        }

        let url = &feed.url;

        match Rss::new(url) {
            Ok(rss) => {
                let channel = &rss.channel;
                let source = Source::query_where(&connection, url)
                    .unwrap_or_else(|_| Source::insert(&connection, url, &channel.title));

                let mut is_update = false;

                for item in &channel.item {
                    if Content::query_where(&connection, &item.link).is_err() {
                        info!("[{}] 更新了一个新视频：{}", &source.title, &item.title);
                        if let Ok(output) = download(&item.link, feed) {
                            writer::bilili(&source.title, &item.link);
                            let out = output.wait_with_output().unwrap();
                            let out = String::from_utf8_lossy(&out.stdout);
                            for line in out.split('\n') {
                                writer::bilili(&source.title, line);
                            }
                            info!("\"{}\" 下载成功", &item.title);
                            Content::insert(&connection, source.id, &item.link, &item.title);
                            is_update = true;
                        } else {
                            log::error!("下载错误");
                        }
                    }
                }

                if is_update {
                    info!("[{}] 已更新！", &source.title);
                } else {
                    info!("[{}] 没有更新！", &source.title);
                }
                
                let interval = feed.interval * 60;
                std::thread::sleep(Duration::from_secs(interval));
            },
            Err(e) => {
                log::error!("解析 RSS 失败: {:?}", e);
                let interval = feed.interval * 60;
                std::thread::sleep(Duration::from_secs(interval));
            }
        }
    }
}

fn download(url: &str, feed: &Feed) -> Result<Child> {
    let mut cmd = Command::new("yutto");
    let args = feed.option.split(' ');
    for arg in args {
        cmd.arg(arg);
    }

    // 先从本地读取 sessdata
    if let Ok(sessdata) = read_to_string("config/SESSDATA.txt") {
        cmd.arg("-c").arg(sessdata);
        // 如果本地不存在，则从系统环境变量获取 sessdata
    } else if let Some(sessdata) = var_os("SESSDATA") {
        cmd.arg("-c").arg(sessdata);
    }

    let download_dir = format!("downloads/{}", &feed.path);
    cmd.args(["-d", &download_dir]).arg(url).stdout(Stdio::piped()).spawn()
}
