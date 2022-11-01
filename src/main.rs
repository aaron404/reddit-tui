use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use roux::{response::BasicThing, submission::SubmissionData, Subreddit};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App<'a> {
    items: StatefulList<(&'a str, usize)>,
    events: Vec<(&'a str, &'a str)>,
    submissions: StatefulList<Submission>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![("Item0", 1)]),
            events: vec![("Event1", "INFO")],
            submissions: StatefulList::with_items(Vec::new()),
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {
        // let sub = Subreddit::new("rust");

        // let mut state = ListState::default();
        // // state.select(Some(1));

        // let items: Vec<ListItem> = sub
        //     .top(25, None)
        //     .unwrap()
        //     .data
        //     .children
        //     .iter()
        //     .map(|c| ListItem::new(c.data.title.clone()))
        //     .collect();
        // let event = self.events.remove(0);
        // self.events.push(event);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Some(evt) = event::read().ok() {
                if let Event::Key(key) = evt {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => app.submissions.unselect(),
                        KeyCode::Down => app.submissions.next(),
                        KeyCode::Up => app.submissions.previous(),
                        _ => {}
                    }
                } else if let Event::Resize(w, h) = evt {
                    println!("resized to {w} {h}");
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            if app.submissions.items.len() < 10 {
                let sub = Subreddit::new("rust");

                // let items = vec![ListItem::new("Item 1"), ListItem::new("Item 2")];
                app.submissions = StatefulList::with_items(
                    sub.top(25, None)
                        .unwrap()
                        .data
                        .children
                        .iter_mut()
                        .map(|c| Submission {
                            title: c.data.title.clone(),
                            score: c.data.score,
                            id: c.data.id.clone(),
                        })
                        .collect(),
                );
                app.submissions.state.select(Some(0));
            }
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

struct Submission {
    title: String,
    score: f64,
    id: String,
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let list: Vec<ListItem> = app
        .submissions
        .items
        .iter()
        .map(|i| ListItem::new(i.title.clone()))
        .collect();
    let list = List::new(list)
        .block(Block::default().borders(Borders::ALL).title("Posts"))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    let area = frame.size();
    frame.render_stateful_widget(list, area, &mut app.submissions.state);
}

fn ui2<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.0)];
            for _ in 0..i.1 {
                lines.push(Spans::from(Span::styled(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
            }
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    // Let's do the same for the events.
    // The event list doesn't have any state and only displays the current state of the list.
    let events: Vec<ListItem> = app
        .events
        .iter()
        .rev()
        .map(|&(event, level)| {
            // Colorcode the level depending on its type
            let s = match level {
                "CRITICAL" => Style::default().fg(Color::Red),
                "ERROR" => Style::default().fg(Color::Magenta),
                "WARNING" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };
            // Add a example datetime and apply proper spacing between them
            let header = Spans::from(vec![
                Span::styled(format!("{:<9}", level), s),
                Span::raw(" "),
                Span::styled(
                    "2020-01-01 10:00:00",
                    Style::default().add_modifier(Modifier::ITALIC),
                ),
            ]);
            // The event gets its own line
            let log = Spans::from(vec![Span::raw(event)]);

            // Here several things happen:
            // 1. Add a `---` spacing line above the final list entry
            // 2. Add the Level + datetime
            // 3. Add a spacer line
            // 4. Add the actual event
            ListItem::new(vec![
                Spans::from("-".repeat(chunks[1].width as usize)),
                header,
                Spans::from(""),
                log,
            ])
        })
        .collect();
    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .start_corner(Corner::BottomLeft);
    f.render_widget(events_list, chunks[1]);
}
