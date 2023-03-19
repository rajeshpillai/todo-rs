use ncurses::*;

fn main() {
    initscr();
    
    let mut quit = false;

    let todos = vec![
        "Learn Rust", 
        "Learn Zig",
        "Learn Kubernetes"
    ];

    while !quit {
        for (row, todo) in todos.iter().enumerate() {
            mv(row as i32, 1);
            addstr(*todo);
        }  
        
        refresh();
        let key = getch();

        match key as u8 as char{
            'q' => quit = true,
            _ => {}
        }
    }
    endwin();
}
