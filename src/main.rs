use std::cmp;

use ncurses::*;

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;


#[derive(Default)]
struct Ui{
    list_curr: Option<Id>,
    row: usize,
    col: usize,
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
    fn begin_list(&mut self, id: Id) {
        assert!(self.list_curr.is_none(), "Nested lists are not allowed!");
        self.list_curr = Some(id);
    }

   

    fn list_element(&mut self, label: &str, id: Id) -> bool {
        let id_curr = self.list_curr.expect("Not allowed to create list elements outside of a list!");
        self.label( label,  {
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                REGULAR_PAIR
            }
        });
        return false;
    }

    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }

    fn end_list(&mut self) {
        self.list_curr = None;
    }

    fn end(&mut self) {
    }
}

enum Focus {
    Todo,
    Done
}

impl Focus {
    fn toggle(&self) -> Self {
        match self {
            Focus::Todo => Focus::Done,
            Focus::Done => Focus::Todo,
        }
    }
}

fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1;
    }
}

fn list_down(list: &Vec<String>, list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1;
   }
}



fn main() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK)    ;
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    
    let mut quit = false;

    let mut todo_curr: usize = 0;
    let mut todos: Vec<String> = vec![
        "Learn Rust".to_string(), 
        "Learn Zig".to_string(),
        "Learn Kubernetes".to_string()
    ];

    let mut done_curr: usize = 0;
    let mut dones = vec![
        "Learn C".to_string(), 
        "Learn C++".to_string(),
        "Learn C#".to_string()
    ];

    let mut focus = Focus::Todo;

    let mut ui = Ui::default();

    while !quit {
        erase();
        ui.begin(0, 0);
        {
            match focus {
                Focus::Todo => {
                    ui.label("TODO: ", REGULAR_PAIR);
                    ui.begin_list(todo_curr);
                    for (index, todo) in todos.iter().enumerate() {
                        ui.list_element(&format!("- [ ] {}", todo), index);
                    }  
                    ui.end_list();
                },
                Focus::Done => {
                    ui.label("DONE: ", REGULAR_PAIR);
                    ui.begin_list(done_curr);
                    for(index, done) in dones.iter().enumerate() {
                        ui.list_element(&format!("- [x] {}", done), index);
        
                    }
                    ui.end_list();
                }
            }

        }

        ui.end();
        
        refresh();

        let key = getch();

        match key as u8 as char{
            'q' => quit = true,
            'w' => match focus {
                    Focus::Todo => list_up(&mut todo_curr),
                    Focus::Done => list_up(&mut done_curr),
             },
            's' => match focus {
                Focus::Todo => list_down(&todos, &mut todo_curr),
                Focus::Done => list_down(&dones, &mut done_curr),
             },
            '\n' => match focus {
                Focus::Todo => if todo_curr < todos.len() {
                    dones.push(todos.remove(todo_curr));
                },
                Focus::Done => if done_curr < dones.len() {
                    todos.push(dones.remove(done_curr));
                }
            },
            '\t' => {
                focus = focus.toggle();
            },
            _ => {
                //todos.push(format!("{} ", key));
            }
        }
    }
    endwin();
}
