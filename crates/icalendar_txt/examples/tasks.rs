//! Example reading tasks from file and showing priority

use icalendar_txt::TodoEntry;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    // file to read (or default todo.txt)
    let file = if let Some(x) = args.get(1) {
        x.as_str()
    } else {
        if std::fs::exists("todo.txt").unwrap_or_default() {
            "todo.txt"
        } else if std::fs::exists("~/todo.txt").unwrap_or_default() {
            "~/todo.txt"
        } else {
            eprintln!("Could not find todo.txt or ~/todo.txt");
            std::process::exit(1);
        }
    };

    let contents = match std::fs::read_to_string(file) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Could not read {file:?}: {err}");
            std::process::exit(1);
        }
    };

    let mut entries: Vec<_> = match TodoEntry::from_str(&contents) {
        // filter all completed todos
        Ok(x) => x.into_iter().filter(|x| !x.completed).collect(),
        Err(err) => {
            eprintln!("Error parsing {file:?}: {err}");
            std::process::exit(1);
        }
    };

    // sort the tasks by priority
    entries.sort_by(|x, y| {
        use std::cmp::Ordering;

        match (x.priority, y.priority) {
            (a, b) if a == b => Ordering::Equal,
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (Some(ref a), Some(ref b)) => b.cmp(a),
            (None, None) => Ordering::Equal,
        }
    });

    println!("## First 10 tasks by priority ({file}) ##");
    for (i, task) in entries.iter().rev().take(10).enumerate() {
        println!("{}. {}", i + 1, task.description);
    }
    println!()
}
