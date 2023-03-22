use ncurses::*;
use std::fs::File;
use std::io::{self, BufRead, ErrorKind,  Write};
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

fn list_drag_up(list: &mut [String], list_curr: &mut usize) {
    if *list_curr > 0 {
        list.swap(*list_curr, *list_curr - 1);
        *list_curr -= 1;
    }
}

fn list_drag_down (list: &mut [String], list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        list.swap(*list_curr, *list_curr + 1);
        *list_curr += 1;
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

fn list_first(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr = 0;
    }
}

fn list_last(list: &[String], list_curr: &mut usize) {
    if !list.is_empty(){
        *list_curr = list.len() - 1;
    }
}

fn list_delete(list: &mut Vec<String>, list_curr: &mut usize) {
    if *list_curr < list.len() {
        list.remove(*list_curr);
        if *list_curr >= list.len() && !list.is_empty() {
            *list_curr = list.len() - 1;
        }
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

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) -> io::Result<()> {
    let file = File::open(file_path)?;   
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line?) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: ERROR:  ill-formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }    
    Ok(())
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
// TODO: move items up and down (reorder)
// TODO: keep track of date when the item was DONE
// TODO: implement jumping to first and last element
// TODO: delete todo item
// TODO: allow non existing input files via command line and add notification
// TODO: add new items to TODO
// TODO: edit the items
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
        
    let mut notification;

    match load_state(&mut todos, &mut dones, &file_path) {
        Ok(()) => notification = format!("Loaded file {}", file_path),
        // Err(error) => if error.kind() == ErrorKind::NotFound {
        //     notification= format!("New file {}", file_path)
        // } else {
        //     panic!("Could not load state from file `{}` : {:?}", file_path, error);
        // }
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                notification = format!("New file {}", file_path)
            } else {
                panic!("Could not load state from file `{}` : {:?}", file_path, error);
            }
        }
    };


    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK)    ;
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    
    let mut quit = false;
    let mut panel = Status::Todo;

    let mut ui = Ui::default();

    while !quit {
        erase();

        let mut x = 0;
        let mut y = 0;
        getmaxyx(stdscr(), &mut y, &mut x);

        ui.begin(Vec2::new(0, 0), LayoutKind::Vert);
        {
            ui.label_fixed_width(&notification, x, REGULAR_PAIR);
            notification.clear();
            ui.label_fixed_width("", x, REGULAR_PAIR);

            ui.begin_layout(LayoutKind::Horz);
            {
                ui.begin_layout(LayoutKind::Vert);
                {
                    ui.label_fixed_width(
                        "TODO",
                        x / 2,
                        if panel == Status::Todo {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        },
                    );
                    for (index, todo) in todos.iter().enumerate() {
                        ui.label_fixed_width(
                            &format!("- [ ] {}", todo),
                            x / 2,
                            if index == todo_curr && panel == Status::Todo {
                                HIGHLIGHT_PAIR
                            } else {
                                REGULAR_PAIR
                            },
                        );
                    }
                }
                ui.end_layout();

                ui.begin_layout(LayoutKind::Vert);
                {
                    ui.label_fixed_width(
                        "DONE",
                        x / 2,
                        if panel == Status::Done {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        },
                    );
                    for (index, done) in dones.iter().enumerate() {
                        ui.label_fixed_width(
                            &format!("- [x] {}", done),
                            x / 2,
                            if index == done_curr && panel == Status::Done {
                                HIGHLIGHT_PAIR
                            } else {
                                REGULAR_PAIR
                            },
                        );
                    }
                }
                ui.end_layout();
            }
            ui.end_layout();
        }
        ui.end();

        refresh();

        let key = getch();

        match key as u8 as char{
            'q' => quit = true,
           
            'W' => match panel {
                Status::Todo => list_drag_up(&mut todos, &mut todo_curr),
                Status::Done => list_drag_up(&mut dones, &mut done_curr),
             },

             'S' => match panel {
                Status::Todo => list_drag_down(&mut todos, &mut todo_curr),
                Status::Done => list_drag_down(&mut dones, &mut done_curr),
             },
           
            'g' => match panel {
                Status::Todo => list_first(&mut todo_curr),
                Status::Done => list_first(&mut done_curr),
            },
           
            'G' => match panel {
                Status::Todo => list_last(&todos, &mut todo_curr),
                Status::Done => list_last(&dones, &mut done_curr),
            },

            'w' => match panel {
                    Status::Todo => list_up(&mut todo_curr),
                    Status::Done => list_up(&mut done_curr),
             },
            's' => match panel {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut done_curr),
             },
             'd' => match panel {
                Status::Todo => {
                    list_delete(&mut todos, &mut todo_curr);
                    notification.push_str("Done!")
                }
                Status::Done => { 
                    list_delete(&mut dones, &mut done_curr);
                    notification.push_str("No, not done yet...");
                }
            },
            '\n' => match panel {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut done_curr)
            },
            '\t' => {
                panel = panel.toggle();
            },
            _ => {
                // todos.push(format!("{} ", key));
            }
        }
    }

    endwin();
    save_state(&todos, &dones, &file_path);
    println!("Saved state to {}", file_path);
}
