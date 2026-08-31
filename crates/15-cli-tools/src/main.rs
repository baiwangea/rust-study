//! CLI 工具示例：用 clap 4 derive 模式构建一个任务清单命令行工具。
//!
//! 试用：
//! ```bash
//! cargo run -p cli-tools -- add "写周报" --priority high
//! cargo run -p cli-tools -- list
//! cargo run -p cli-tools -- done 1
//! cargo run -p cli-tools -- stats
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 一个简单的任务清单命令行工具
#[derive(Parser)]
#[command(name = "tasky", version, about, long_about = None)]
struct Cli {
    /// 任务存储文件路径（全局参数，所有子命令可用）
    #[arg(long, global = true, default_value = "tasks.json")]
    file: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 添加一个新任务
    Add {
        /// 任务标题
        title: String,
        /// 优先级
        #[arg(short, long, value_enum, default_value_t = Priority::Normal)]
        priority: Priority,
    },
    /// 列出任务
    List {
        /// 包含已完成的任务
        #[arg(long)]
        all: bool,
    },
    /// 标记任务为已完成
    Done {
        /// 任务 ID
        id: u32,
    },
    /// 删除任务
    Remove {
        /// 任务 ID
        id: u32,
    },
    /// 统计任务数量
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
enum Priority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    priority: Priority,
    done: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TaskStore {
    tasks: Vec<Task>,
    next_id: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = load(&cli.file)?;

    match cli.command {
        Command::Add { title, priority } => {
            store.next_id += 1;
            let task = Task {
                id: store.next_id,
                title: title.clone(),
                priority,
                done: false,
            };
            store.tasks.push(task);
            println!("已添加任务 #{}: {} ({:?})", store.next_id, title, priority);
        }
        Command::List { all } => {
            let items: Vec<&Task> = store
                .tasks
                .iter()
                .filter(|t| all || !t.done)
                .collect();
            if items.is_empty() {
                println!("暂无任务");
            }
            for task in items {
                let status = if task.done { "[x]" } else { "[ ]" };
                println!("{} #{} {} ({:?})", status, task.id, task.title, task.priority);
            }
        }
        Command::Done { id } => {
            match store.tasks.iter_mut().find(|t| t.id == id) {
                Some(task) => {
                    task.done = true;
                    println!("任务 #{} 已完成", id);
                }
                None => anyhow::bail!("找不到任务 #{}", id),
            }
        }
        Command::Remove { id } => {
            let before = store.tasks.len();
            store.tasks.retain(|t| t.id != id);
            if store.tasks.len() == before {
                anyhow::bail!("找不到任务 #{}", id);
            }
            println!("已删除任务 #{}", id);
        }
        Command::Stats => {
            let total = store.tasks.len();
            let done = store.tasks.iter().filter(|t| t.done).count();
            println!("共 {} 个任务，已完成 {}，待办 {}", total, done, total - done);
        }
    }

    save(&cli.file, &store)?;
    Ok(())
}

fn load(path: &PathBuf) -> Result<TaskStore> {
    if !path.exists() {
        return Ok(TaskStore::default());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("读取任务文件 {:?} 失败", path))?;
    serde_json::from_str(&content)
        .with_context(|| format!("解析任务文件 {:?} 失败", path))
}

fn save(path: &PathBuf, store: &TaskStore) -> Result<()> {
    let content = serde_json::to_string_pretty(store)?;
    fs::write(path, content)
        .with_context(|| format!("写入任务文件 {:?} 失败", path))
}
