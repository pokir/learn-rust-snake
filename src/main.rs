use rand::Rng;

const START_SNAKE_SIZE: usize = 5;

const START_SLEEP_TIME: i32 = 150;
const MINIMUM_SLEEP_TIME: i32 = 50;
const SLEEP_DECREASE_PER_FOOD: i32 = 3;

#[derive(Clone)]
enum Cell {
    Empty,
    Snake,
    Wall,
    Food,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum SnakeEvent {
    Bump,
    Eat,
    Nothing,
}

fn create_grid(width: usize, height: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::Empty; width]; height]
}

fn format_grid(grid: &Vec<Vec<Cell>>) -> String {
    let mut output = String::from("");

    for row in grid {
        for cell in row {
            match cell {
                Cell::Empty => output += " ",
                Cell::Snake => output += "o",
                Cell::Wall => output += "M",
                Cell::Food => output += "x",
            }
        }
    }

    output
}

fn create_walls(grid: &mut Vec<Vec<Cell>>) {
    let height = grid.len();
    let width = grid[0].len();

    for x in 0..grid[0].len() {
        grid[0][x] = Cell::Wall;
        grid[height - 1][x] = Cell::Wall;
    }

    for y in 0..grid.len() {
        grid[y][0] = Cell::Wall;
        grid[y][width - 1] = Cell::Wall;
    }
}

fn create_snake(grid: &mut Vec<Vec<Cell>>) -> (Vec<(usize, usize)>, Direction) {
    // Create snake and add it to the grid

    let mut snake = Vec::new();

    for i in 0..START_SNAKE_SIZE {
        snake.push((i + grid[0].len() / 2, grid.len() / 2));
    }

    update_snake_on_grid(grid, &snake);

    (snake, Direction::Right)
}

fn update_snake_on_grid(grid: &mut Vec<Vec<Cell>>, snake: &Vec<(usize, usize)>) {
    // remove all snake cells first
    for y in 0..grid.len() {
        for x in 0..grid[0].len() {
            match grid[y][x] {
                Cell::Snake => grid[y][x] = Cell::Empty,
                _ => {}
            }
        }
    }

    // then add the new snake cells
    for snake_cell in snake {
        grid[snake_cell.1][snake_cell.0] = Cell::Snake;
    }
}

fn move_snake(
    grid: &mut Vec<Vec<Cell>>,
    snake: &mut Vec<(usize, usize)>,
    direction: &Direction,
    grow: bool,
) -> SnakeEvent {
    // Move snake and update it on the grid

    if !grow {
        snake.remove(0);
    }

    // Snake's head:
    let last_snake_cell = snake.last().unwrap();

    match direction {
        Direction::Up => snake.push((last_snake_cell.0, last_snake_cell.1 - 1)),
        Direction::Down => snake.push((last_snake_cell.0, last_snake_cell.1 + 1)),
        Direction::Left => snake.push((last_snake_cell.0 - 1, last_snake_cell.1)),
        Direction::Right => snake.push((last_snake_cell.0 + 1, last_snake_cell.1)),
    }

    let new_snake_cell = snake.last().unwrap();

    // check if snake bumped into a wall or itself, or ate food
    let snake_event = match grid[new_snake_cell.1][new_snake_cell.0] {
        Cell::Snake => SnakeEvent::Bump,
        Cell::Wall => SnakeEvent::Bump,
        Cell::Food => SnakeEvent::Eat,
        _ => SnakeEvent::Nothing,
    };

    update_snake_on_grid(grid, snake);

    snake_event
}

fn create_food(grid: &mut Vec<Vec<Cell>>) {
    let height = grid.len();
    let width = grid[0].len();

    let mut rng = rand::thread_rng();

    loop {
        let x: usize = rng.gen_range(0..width);
        let y: usize = rng.gen_range(0..height);

        match grid[y][x] {
            Cell::Empty => {
                grid[y][x] = Cell::Food;
                break;
            }
            _ => {}
        }
    }
}

fn main() {
    let mut window = yacurses::Curses::init();

    window.set_timeout(0); // don't block program for key events

    let result = window.set_cursor_visibility(yacurses::CursorVisibility::Invisible);

    if result.is_err() {
        // do nothing, just don't make the cursor invisible
    }

    let dimensions = window.get_terminal_size();

    let mut grid = create_grid(dimensions.x_count as usize, dimensions.y_count as usize);

    let (mut snake, mut snake_direction) = create_snake(&mut grid);

    create_walls(&mut grid);

    create_food(&mut grid);

    let mut grow = false;

    loop {
        // display the game
        let result = window.move_cursor(yacurses::Position { x: 0, y: 0 });

        if result.is_err() {
            let result = window.clear();

            if result.is_err() {
                panic!("Error: Cannot clear the terminal");
            }
        }

        let result = window.print_str(&format_grid(&grid));

        if result.is_err() {
            // TODO: fix this (it is err even though it works)
            //panic!("Error: Cannot print to the terminal using curses");
        }

        // TODO: display the score

        match window.poll_events() {
            Some(key) => {
                snake_direction = match key {
                    yacurses::CursesKey::ArrowUp if snake_direction != Direction::Down => {
                        Direction::Up
                    }
                    yacurses::CursesKey::ArrowDown if snake_direction != Direction::Up => {
                        Direction::Down
                    }
                    yacurses::CursesKey::ArrowLeft if snake_direction != Direction::Right => {
                        Direction::Left
                    }
                    yacurses::CursesKey::ArrowRight if snake_direction != Direction::Left => {
                        Direction::Right
                    }
                    _ => snake_direction,
                }
            }
            None => { /* do nothing if no key pressed */ }
        }

        let snake_event = move_snake(&mut grid, &mut snake, &snake_direction, grow);

        grow = false;

        match snake_event {
            SnakeEvent::Bump => break,
            SnakeEvent::Eat => {
                create_food(&mut grid);
                grow = true;
            }
            _ => {}
        }

        let sleep_decrease = (snake.len() - START_SNAKE_SIZE) as i32 * SLEEP_DECREASE_PER_FOOD;

        let sleep_time = std::cmp::max(MINIMUM_SLEEP_TIME, START_SLEEP_TIME - sleep_decrease);

        std::thread::sleep(std::time::Duration::from_millis(sleep_time as u64));
    }
}
