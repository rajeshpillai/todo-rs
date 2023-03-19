use ncurses::*;

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

fn main() {
    initscr();

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK)    ;
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    
    let mut quit = false;

    let todos = vec![
        "Learn Rust", 
        "Learn Zig",
        "Learn Kubernetes"
    ];

    let todo_curr: usize = 0;

    while !quit {
        for (index, todo) in todos.iter().enumerate() {
            let pair = {
                if todo_curr == index {
                    // Render in a different style
                    HIGHLIGHT_PAIR
                } else {
                    REGULAR_PAIR
                }
            };
            attron(COLOR_PAIR(pair));
            mv(index as i32, 1);
            addstr(*todo);
            attroff (COLOR_PAIR(pair));
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
