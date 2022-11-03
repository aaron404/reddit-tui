use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use roux::Subreddit;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
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

enum SubSort {
    Hot,
    Rising,
    Popular,
}

enum ViewState {
    Subreddit,
    Post,
}

struct App {
    submissions: StatefulList<Submission>,
    view_state: ViewState,
    selection: Option<Submission>,
    subreddit: Subreddit,
    // comments: roux::Comments,
}

impl App {
    fn new() -> App {
        App {
            submissions: StatefulList::with_items(Vec::new()),
            view_state: ViewState::Subreddit,
            selection: None,
            subreddit: Subreddit::new("rust"),
            // comments: roux::Comments::from(""),
        }
    }

    fn on_tick(&mut self) {}

    fn select(&mut self) {
        match self.view_state {
            ViewState::Subreddit => {
                if self.submissions.items.len() > 0 {
                    if let Some(i) = self.submissions.state.selected() {
                        self.selection = Some(self.submissions.items.get(i).unwrap().clone());
                        let article = self.subreddit.article_comments(
                            &self.selection.as_ref().unwrap().id,
                            Some(8),
                            Some(100),
                        );
                        // eprintln!(
                        //     "{:?}",
                        //     article.unwrap().data.children.first().unwrap().data.body
                        // );
                        self.view_state = ViewState::Post;
                    }
                }
            }
            ViewState::Post => todo!(),
        }
    }

    fn back(&mut self) {
        self.view_state = ViewState::Subreddit;
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
                        KeyCode::Enter => app.select(),
                        KeyCode::Esc | KeyCode::Backspace => app.back(),
                        _ => {}
                    }
                } else if let Event::Resize(w, h) = evt {
                    println!("resized to {w} {h}");
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            if app.submissions.items.len() < 10 {
                app.submissions = StatefulList::with_items(
                    app.subreddit
                        .top(25, None)
                        .unwrap()
                        .data
                        .children
                        .iter_mut()
                        .map(|c| Submission {
                            title: c.data.title.clone(),
                            score: c.data.score,
                            id: c.data.id.clone(),
                            selftext: c.data.selftext.clone(),
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

#[derive(Clone)]
struct Submission {
    title: String,
    score: f64,
    id: String,
    selftext: String,
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    match app.view_state {
        ViewState::Subreddit => {
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
        ViewState::Post => {
            // let paragraph = frame.render_widget();
            let paragraph = tui::widgets::Paragraph::new(
                app.selection.as_ref().unwrap().selftext.clone(),
            )
            .block(Block::default().borders(Borders::ALL).title("Post"));
            let area = frame.size();
            frame.render_widget(paragraph, area);
        }
    }
}
