use ncurses::*;
use std::fs::File;
use std::io::{self, Write, BufRead};
use std::env;
use std::process;
use std::ops::{Add, Mul};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;


#[derive(Default, Copy, Clone)]
struct Vec2 {
    x: i32,
    y: i32
}

impl Add for Vec2 {
    type Output = Vec2;
    
    fn add (self, rhs: Vec2) -> Vec2 {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul for Vec2 {
    type Output = Vec2;
    
    fn mul (self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Self{x, y}
    }
}

enum LayoutKind {
    Vert,
    Horz
}

struct Layout {
    kind: LayoutKind,
    pos: Vec2,
    size: Vec2,
}

impl Layout {
    fn available_pos(&self) -> Vec2 {
        use LayoutKind::*;
        match self.kind {
            Horz => self.pos + self.size * Vec2::new(1, 0),
            Vert => self.pos + self.size * Vec2::new(0,1),
        }
    }

    fn add_widget(&mut self, size: Vec2) {
        use LayoutKind::*;
        match self.kind {
            Horz => {
                self.size.x += size.x;
                self.size.y = std::cmp::max(self.size.y, size.y);
            },
            Vert => {
                self.size.x = std::cmp::max(self.size.x, size.x);
                self.size.y += size.y;    
            },
        }
    }
}


#[derive(Default)]
struct Ui{
    layouts: Vec<Layout>
}


impl Ui {
    fn begin(&mut self, pos: Vec2, kind: LayoutKind) {
        assert!(self.layouts.is_empty());
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0,0)
        }); 
    }

    fn begin_layout(&mut self, kind: LayoutKind) {
        let layout = self.layouts.last().expect("Can't create layout outside of Ui::begin() and Ui::end()");
        let pos = layout.available_pos();
        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2::new(0,0)
        })
    }

    fn end_layout(&mut self) {
        let layout = self.layouts.pop()
            .expect("Unbalanced UI::begin_layout() and UI::end_layout()");
        self.layouts.last_mut().expect("Unbalanced Ui::begin_layout() and Ui::end_layout() calls.")
            .add_widget(layout.size);
    }

    fn label_fixed_width(&mut self, text: &str, width: i32, pair: i16) {
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render label outside of any layout");

        let pos = layout.available_pos();

        mv(pos.y, pos.x);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));
        layout.add_widget(Vec2::new(width, 1));
    }

    fn label(&mut self, text: &str, pair: i16) {
        self.label_fixed_width(text, text.len() as i32, pair);
    }

    fn end(&mut self) {
        self.layouts
            .pop()
            .expect("Unbalanced UI::begin() and UI::end() calls");
    }
}

#[derive(Debug, PartialEq)]
enum Status {
    Todo,
    Done
}

impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";

    if line.starts_with(todo_prefix) {
        return Some((Status::Todo, &line[todo_prefix.len()..]))
    }
    
    if line.starts_with(done_prefix) {
        return Some((Status::Done, &line[done_prefix.len()..]))
    }

    return None;
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

fn list_transfer(list_dst: &mut Vec<String>, list_src: &mut Vec<String>, list_src_curr: &mut usize) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() && list_src.len() > 0 {
            *list_src_curr = list_src.len() -1;
        }    
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path.clone()).unwrap();   
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: ERROR:  ill-formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }    
}

fn save_state(todos: &Vec<String>, dones: &Vec<String>, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

// TODO: persist the state of the application
// TODO: add new items to TODO
// TODO: delete items
// TODO: edit the items
// TODO: keep track of date when the item was DONE
// TODO: undo system
// TODO: save the state on SIGINT

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            eprintln!("Usage: todo-rs <file-path>");
            eprintln!("ERROR: file path is not provided");
            process::exit(1);
        }
    };
    
    let mut todos = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut dones = Vec::<String>::new();
    let mut done_curr: usize = 0;
        
    load_state(&mut todos, &mut dones, &file_path);

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK)    ;
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    
    let mut quit = false;
    let mut tab = Status::Todo;

    let mut ui = Ui::default();

    while !quit {
        erase();
        ui.begin(Vec2::new(0,0), LayoutKind::Horz);
        {
            ui.begin_layout(LayoutKind::Vert);
            {
                ui.label("TODO",
                    if tab == Status::Todo {
                        HIGHLIGHT_PAIR
                    } else {
                        REGULAR_PAIR
                    });

                for (index, todo) in todos.iter().enumerate() {
                    ui.label(&format!("- [ ] {}", todo), 
                        if index == todo_curr && tab == Status::Todo {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        });
                }
            }  
            ui.end_layout();

            ui.begin_layout(LayoutKind::Vert);
            {
                ui.label("DONE", 
                    if tab == Status::Done {
                        HIGHLIGHT_PAIR
                    } else {
                        REGULAR_PAIR
                    });

                for (index, done) in dones.iter().enumerate() {
                    ui.label(&format!("- [x] {}", done), 
                        if index == done_curr && tab == Status::Done {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        });
                }
            }
            ui.end_layout();
        }
        ui.end();
        
        refresh();

        let key = getch();

        match key as u8 as char{
            'q' => quit = true,
            'w' => match tab {
                    Status::Todo => list_up(&mut todo_curr),
                    Status::Done => list_up(&mut done_curr),
             },
            's' => match tab {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut done_curr),
             },
            '\n' => match tab {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut done_curr)
            },
            '\t' => {
                tab = tab.toggle();
            },
            _ => {
                //todos.push(format!("{} ", key));
            }
        }
    }

    save_state(&todos, &dones, &file_path);
    endwin();
}
