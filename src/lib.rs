use std::{io, path::PathBuf, time::Instant};
use clap::{Parser, Subcommand,Command};
use rusqlite::{Connection, ErrorCode, Result, params};
use tabled::{Table, Tabled, assert::assert_table, grid::records::vec_records::Cell, settings::{Alignment, Style, Width, object::{Columns, Rows}, split::{self, Split}}};
use thiserror::Error;
use time::UtcDateTime;
use std::io::Write;

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("failed to connect to database")]
    FailedToConnectDB,
    #[error("Id not found {0}")]
    NotFound(i32),
    #[error("Invalid id provided {0}")]
    InvalidId(i32),
    #[error("Unexpected error occurred")]
    UnExpectedError,
    #[error("Failed to add todos")]
    FailedToCreateTodos
}

impl From<rusqlite::Error> for TodoError{
    fn from(value: rusqlite::Error) -> Self {
        match value{
            rusqlite::Error::SqliteFailure(_,_)=> TodoError::FailedToConnectDB,
            rusqlite::Error::QueryReturnedNoRows => TodoError::FailedToConnectDB,
            _ =>{
                TodoError::FailedToCreateTodos
            }
        }
    }
}

#[derive(Debug)]
pub struct TodoDb{
    pub tasks: Vec<TodoItem>,
    pub connection: Connection
}

impl TodoDb{
    pub fn new() -> Result<Self,TodoError>{
        let connection = open_db()?;
        Ok(Self { tasks: vec![], connection })
    }

    pub fn add(&self, tasks: TodoItem)->Result<(),TodoError>{
        let check = self.connection.execute(
            "INSERT INTO todo (task, created_at, status) VALUES (?1, ?2, ?3)",
            (&tasks.task, &tasks.created_at, &tasks.status),
        );

        match check{
            Ok(1)=>{
                println!("Successfully added task!");
                Ok(())
            },
            Ok(0)=>{
                Err(TodoError::FailedToCreateTodos)
            },
            _ =>{
                Err(TodoError::UnExpectedError)
            }
        }
    }

    pub fn done(&self, id: i32)->Result<(),TodoError>{
        let check = self.connection.execute("UPDATE todo SET status = 1 WHERE id = ?", [id]);

        match check{
            Ok(1)=>{
                println!("Successfully marked task #{} as done!", id);
                Ok(())
            },
            Ok(0) =>{
                Err(TodoError::InvalidId(id))
            },
            _=>{
                Err(TodoError::UnExpectedError)
            }
        }
    }

    pub fn list(&self) -> Result<()>{
        let mut count = 1;
        let mut stmt = self.connection.prepare("SELECT * FROM todo")?;
        let todo_iter: Vec<ListTodo>= stmt.query_map([], |row|{
            Ok(ListTodo{
                id: row.get(0)?,
                task: row.get(1)?,
                created_at: row.get(2)?,
                status: row.get(3)?,
            })
        })?.map(|f|{
            f.unwrap()
        }).collect();

        if todo_iter.len()<1{
            println!("Todo list is empty :(");
            return Ok(())
        }
        let mut table = Table::new(todo_iter);

        table.modify(Rows::new(1..), Width::wrap(40).keep_words(true));
        table.with(Style::modern());
        table.modify(Columns::first(), Alignment::right());

        println!("{}", table);
        Ok(())
    }

    pub fn delete(&self, id:i32) -> Result<(),TodoError>{
        let check = self.connection.execute("DELETE FROM todo WHERE id=?", [id]);

        match check{
            Ok(1)=>{
                println!("Successfully deleted todos #{}",id);
                Ok(())
            },
            Ok(0) =>{
                Err(TodoError::NotFound(id))
            },
            _=>{
                Err(TodoError::UnExpectedError)
            }
        }
    }

    pub fn edit(&self, id:i32, task: String)-> Result<(),TodoError>{
        if task.len()<1{
            println!("Current task: {}", self.connection.query_row("SELECT task FROM todo WHERE id=?", [id], |row| row.get::<_,String>(0))?);
            let task = readline().unwrap();

            let check = self.connection.execute("UPDATE todo SET task=?1 WHERE id =?2", params![task, id]);

            match check{
                Ok(1)=>{
                    println!("Task #{} updated", id);
                    return Ok(())
                },
                Ok(0) =>{
                    return Err(TodoError::NotFound(id))
                },
                _=>{
                    return Err(TodoError::UnExpectedError)
                }
            }            
        }
        
        let check = self.connection.execute("UPDATE todo SET task=?1 WHERE id =?2", params![task, id]);

        match check{
            Ok(1)=>{
                println!("Task #{} updated", id);
                Ok(())
            },
            Ok(0) =>{
                Err(TodoError::NotFound(id))
            },
            _=>{
                Err(TodoError::UnExpectedError)
            }
        }   

    }

    pub fn show(&self, id: i32)-> Result<()>{
        let mut stmt = self.connection.prepare("SELECT * FROM todo WHERE id=?")?;
        let todo_iter: Vec<ListTodo>= stmt.query_map([id], |row|{
            Ok(ListTodo{
                id: row.get(0)?,
                task: row.get(1)?,
                created_at: row.get(2)?,
                status: row.get(3)?,
            })
        })?.map(|f|{
            f.unwrap()
        }).collect();


        let mut table = Table::new(todo_iter);

        table.modify(Rows::new(1..), Width::wrap(40).keep_words(true));
        table.with(Style::modern());
        table.modify(Columns::first(), Alignment::right());

        println!("{}", table);
        Ok(())
    }
}
#[derive(Debug)]
pub struct TodoItem{
    task: String,
    created_at: String,
    status: bool
}

impl TodoItem{
    pub fn new(task: String, created_at: String,status: bool) -> Self{
        Self {
            task,
            created_at,
            status
        }
    }
}

#[derive(Debug,Clone,Tabled)]

pub struct ListTodo{
    id: i32,
    task: String,
    created_at: String,
    status: bool
}

fn open_db()-> Result<Connection, TodoError> {
    let db_path = "/this/path/does/not/exist/app.db";

    let conn = Connection::open("todo.db")?;
    
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS todo (
            id INTEGER PRIMARY KEY,
            task TEXT NOT NULL,
            created_at TEXT NOT NULL,
            status BOOLEAN NOT NULL
        )
        ", ()
    )?;
    
    Ok(conn)
}

pub fn delete_db()-> Result<()>{
    let conn = Connection::open("todo.db")?;

    conn.execute("DROP TABLE IF EXISTS todo", [])?;

    Ok(())
}

pub fn readline() -> Result<String, String> {
    write!(std::io::stdout(), "$ New task: ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}

// fn respond(line: &str) -> Result<bool, String> {
//     let args = shlex::split(line).ok_or("error: Invalid quoting")?;
//     let matches = cli()
//         .try_get_matches_from(args)
//         .map_err(|e| e.to_string())?;
//     match matches.subcommand() {
//         Some(("ping", _matches)) => {
//             write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
//             std::io::stdout().flush().map_err(|e| e.to_string())?;
//         }
//         Some(("quit", _matches)) => {
//             write!(std::io::stdout(), "Exiting ...").map_err(|e| e.to_string())?;
//             std::io::stdout().flush().map_err(|e| e.to_string())?;
//             return Ok(true);
//         }
//         Some((name, _matches)) => unimplemented!("{name}"),
//         None => unreachable!("subcommand required"),
//     }

//     Ok(false)
// }