use std::{path::PathBuf, time::Instant};

use clap::{Command, Parser, Subcommand, builder::styling};
use cli_todo::{TodoDb, TodoError, TodoItem, delete_db};
use time::UtcDateTime;

#[derive(Parser)]
#[command(version, about, long_about = None, override_usage = "cli-todo [COMMAND] [OPTIONS] <args>", styles = STYLES)]

struct Cli {
    /// Optional name to operate on
    // #[arg(short, long)]
    // add: String,

    // #[arg(short, long)]
    // delete: usize,

    // #[arg(short, long)]
    // edit: usize,

    // #[arg(short, long)]
    // show: usize,

    // #[arg(short, long)]
    // list: bool
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands{
    /// Adds a todo task
    Add{
        // #[arg(short,long)]
        todo: Vec<String>
    },

    /// Checks a todo task
    Done{
        id: i32,
    },

    /// List all todo tasks
    List,

    /// Deletes a todo task
    Delete{
        id: i32
    },

    /// Edits a todo task
    Edit{
        id: i32,
        
            #[arg(num_args = 1..)]
        task: Vec<String>
        
    },

    /// Shows a todo task
    Show{
        id: i32
    },

    /// Clear all tasks
    Clear
}

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());


fn main() -> Result<(),TodoError>{
    let cli = Cli::parse();
    let db= TodoDb::new()?;

    match cli.command{
        Some(Commands::Add { todo }) => {
            let task = TodoItem::new(todo.join(" ").to_string(), UtcDateTime::now().date().to_string(), false);

            db.add(task)?;

        },

        Some(Commands::Done { id })=>{
            db.done(id as i32)?
        },

        Some(Commands::List) =>{
            db.list().expect("Failed to list todos");
        },

        Some(Commands::Clear) =>{
            delete_db().expect("Failed to delete database");
        },

        Some(Commands::Delete { id })=>{
            db.delete(id)?
        },

        Some(Commands::Edit { id, task })=>{
            let task = task.join(" ").to_string();
            db.edit(id, task)?
        },

        Some(Commands::Show { id })=>{
            db.show(id)?
        }

        _=>{
            println!("None")
        }
    }
    

    Ok(())
    // println!("All todos: {:?}", db);
}