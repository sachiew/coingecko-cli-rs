//! Interactive TUI mode — ratatui-based market and trending coin browser.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, TableState,
    },
};
use std::io;

use crate::api::{CoinDetail, MarketEntry};

// ─── Brand Colors ─────────────────────────────────────────────────────────────

const GECKO_GREEN: Color = Color::Rgb(140, 195, 81);
const GOLD: Color = Color::Rgb(255, 215, 0);

// ─── App State ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum ListMode {
    Markets,
    Trending,
}

enum View {
    List,
    Loading(usize),
    Detail(usize),
}

struct App {
    coins: Vec<MarketEntry>,
    table_state: TableState,
    view: View,
    mode: ListMode,
    category: Option<String>,
    chart_data: Option<Vec<(f64, f64)>>,
    chart_error: Option<String>,
    coin_detail: Option<CoinDetail>,
}

impl App {
    fn new(coins: Vec<MarketEntry>, mode: ListMode, category: Option<String>) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        App {
            coins,
            table_state,
            view: View::List,
            mode,
            category,
            chart_data: None,
            chart_error: None,
            coin_detail: None,
        }
    }

    fn next(&mut self) {
        let len = self.coins.len();
        let i = match self.table_state.selected() {
            Some(i) => (i + 1).min(len.saturating_sub(1)),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.table_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.table_state.select(Some(i));
    }
}

// ─── Entry Point ──────────────────────────────────────────────────────────────

pub async fn run_tui(category: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match category {
        Some(cat) => print!("  Fetching top 50 coins in category: {cat}…"),
        None => print!("  Fetching top 50 coins…"),
    }
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let coins = crate::api::fetch_top_coins(50, "usd", category).await?;
    println!(" done ({} coins).", coins.len());
    run_tui_inner(coins, ListMode::Markets, category.map(str::to_owned)).await
}

pub async fn run_trending_tui() -> Result<(), Box<dyn std::error::Error>> {
    print!("  Fetching trending coins…");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let coins = crate::api::fetch_trending_coins().await?;
    println!(" done ({} coins).", coins.len());
    run_tui_inner(coins, ListMode::Trending, None).await
}

async fn run_tui_inner(
    coins: Vec<MarketEntry>,
    mode: ListMode,
    category: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(coins, mode, category);
    let result = event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

// ─── Event Loop ───────────────────────────────────────────────────────────────

async fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| render(f, app))?;

        // block_in_place lets tokio know this thread will block briefly on keyboard input
        let event = tokio::task::block_in_place(event::read)?;

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match app.view {
                View::List => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Enter => {
                        if let Some(idx) = app.table_state.selected() {
                            let coin_id = app.coins[idx].id.clone();
                            app.chart_data = None;
                            app.chart_error = None;
                            app.coin_detail = None;
                            app.view = View::Loading(idx);
                            terminal.draw(|f| render(f, app))?;
                            // Fetch chart and detail concurrently.
                            let (chart_res, detail_res) = tokio::join!(
                                crate::api::fetch_coin_chart(&coin_id, 7, "usd"),
                                crate::api::fetch_coin_detail(&coin_id, "usd"),
                            );
                            match chart_res {
                                Ok(data) => app.chart_data = Some(data),
                                Err(e) => app.chart_error = Some(e.to_string()),
                            }
                            app.coin_detail = detail_res.ok();
                            app.view = View::Detail(idx);
                        }
                    }
                    _ => {}
                },
                View::Loading(_) | View::Detail(_) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                        app.chart_data = None;
                        app.chart_error = None;
                        app.coin_detail = None;
                        app.view = View::List;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

// ─── Rendering ────────────────────────────────────────────────────────────────

fn render(f: &mut ratatui::Frame, app: &mut App) {
    match app.view {
        View::List => match app.mode {
            ListMode::Markets => render_markets_list(f, app),
            ListMode::Trending => render_trending_list(f, app),
        },
        View::Loading(idx) => render_loading(f, app, idx),
        View::Detail(idx) => render_detail(f, app, idx),
    }
}

fn header_style() -> Style {
    Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
}

fn outer_block(subtitle: &str, category: Option<&str>) -> Block<'static> {
    let mut spans = vec![
        Span::styled(
            " ◆ CoinGecko ",
            Style::default()
                .fg(GECKO_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{subtitle} "), Style::default().fg(Color::DarkGray)),
    ];
    if let Some(cat) = category {
        spans.push(Span::styled(
            format!("[{cat}] "),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));
    }
    Block::default()
        .title(Line::from(spans))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GECKO_GREEN))
}

// ─── List Views ───────────────────────────────────────────────────────────────

fn render_markets_list(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let block = outer_block("TUI — Top 50 by Market Cap", app.category.as_deref());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new("  ↑/k  ↓/j  navigate    Enter  detail    q/Esc  quit")
            .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );

    let rows: Vec<Row> = app
        .coins
        .iter()
        .map(|c| {
            let (cs, ct) = change_fmt(c.change_24h);
            Row::new(vec![
                Cell::from(c.rank.to_string()),
                Cell::from(c.name.clone()),
                Cell::from(c.symbol.to_uppercase()),
                Cell::from(crate::ui::format_usd(c.price)),
                Cell::from(crate::ui::format_large_usd(c.market_cap)),
                Cell::from(crate::ui::format_large_usd(c.volume)),
                Cell::from(ct).style(cs),
            ])
        })
        .collect();

    let headers = Row::new(vec![
        Cell::from("#").style(header_style()),
        Cell::from("Name").style(header_style()),
        Cell::from("Symbol").style(header_style()),
        Cell::from("Price (USD)").style(header_style()),
        Cell::from("Market Cap").style(header_style()),
        Cell::from("Volume").style(header_style()),
        Cell::from("24h").style(header_style()),
    ])
    .height(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(headers)
    .block(Block::default())
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(GECKO_GREEN)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[0], &mut app.table_state);
}

fn render_trending_list(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let block = outer_block("TUI — Top 30 Trending Coins (24h)", None);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new("  ↑/k  ↓/j  navigate    Enter  detail    q/Esc  quit")
            .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );

    let rows: Vec<Row> = app
        .coins
        .iter()
        .map(|c| {
            let (cs, ct) = change_fmt(c.change_24h);
            let trend_rank = c
                .trending_rank
                .map_or_else(|| "—".to_string(), |r| format!("#{r}"));
            let mcap_rank = if c.rank > 0 {
                format!("#{}", c.rank)
            } else {
                "—".to_string()
            };
            Row::new(vec![
                Cell::from(trend_rank),
                Cell::from(mcap_rank),
                Cell::from(c.name.clone()),
                Cell::from(c.symbol.to_uppercase()),
                Cell::from(crate::ui::format_usd(c.price)),
                Cell::from(ct).style(cs),
            ])
        })
        .collect();

    let headers = Row::new(vec![
        Cell::from("Trend").style(header_style()),
        Cell::from("MCap #").style(header_style()),
        Cell::from("Name").style(header_style()),
        Cell::from("Symbol").style(header_style()),
        Cell::from("Price (USD)").style(header_style()),
        Cell::from("24h").style(header_style()),
    ])
    .height(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
    )
    .header(headers)
    .block(Block::default())
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(GECKO_GREEN)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[0], &mut app.table_state);
}

// ─── Loading View ─────────────────────────────────────────────────────────────

fn render_loading(f: &mut ratatui::Frame, app: &App, idx: usize) {
    let name = app.coins.get(idx).map_or("coin", |c| c.name.as_str());
    let area = f.area();
    let block = outer_block("Loading…", app.category.as_deref());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(1),
            Constraint::Percentage(50),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("  Fetching 7-day chart for {name}…"))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        v[1],
    );
}

// ─── Detail View ──────────────────────────────────────────────────────────────

fn render_detail(f: &mut ratatui::Frame, app: &App, idx: usize) {
    let Some(coin) = app.coins.get(idx) else {
        return;
    };
    let area = f.area();

    let title = format!("{} ({}) — Detail", coin.name, coin.symbol.to_uppercase());
    let block = outer_block(&title, app.category.as_deref());
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Vertical: content on top, hint on bottom.
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new("  Esc / q / ← back to list").style(Style::default().fg(Color::DarkGray)),
        v[1],
    );

    // Horizontal: 30% metadata | 70% chart.
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(v[0]);

    render_metadata(f, coin, app.coin_detail.as_ref(), h[0]);
    render_chart(
        f,
        app.chart_data.as_deref(),
        app.chart_error.as_deref(),
        coin,
        h[1],
    );
}

#[allow(clippy::similar_names)] // ath_* vs atl_* are semantically distinct (all-time high vs low)
fn render_metadata(
    f: &mut ratatui::Frame,
    coin: &MarketEntry,
    detail: Option<&CoinDetail>,
    area: ratatui::layout::Rect,
) {
    let (cs, ct) = change_fmt(coin.change_24h);
    let lbl = Style::default().fg(GOLD).add_modifier(Modifier::BOLD);
    let val = Style::default().fg(Color::White);
    let dim = Style::default().fg(Color::DarkGray);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Rank    ", lbl),
            Span::styled(coin.rank.to_string(), val),
        ]),
        Line::from(vec![
            Span::styled(" Name    ", lbl),
            Span::styled(coin.name.clone(), val),
        ]),
        Line::from(vec![
            Span::styled(" Symbol  ", lbl),
            Span::styled(coin.symbol.to_uppercase(), val),
        ]),
        Line::from(vec![
            Span::styled(" ID      ", lbl),
            Span::styled(coin.id.clone(), val),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Price   ", lbl),
            Span::styled(crate::ui::format_usd(coin.price), val),
        ]),
        Line::from(vec![
            Span::styled(" Mkt Cap ", lbl),
            Span::styled(crate::ui::format_large_usd(coin.market_cap), val),
        ]),
        Line::from(vec![
            Span::styled(" Vol 24h ", lbl),
            Span::styled(crate::ui::format_large_usd(coin.volume), val),
        ]),
        Line::from(vec![Span::styled(" 24h Chg ", lbl), Span::styled(ct, cs)]),
    ];

    if let Some(d) = detail {
        let (ath_cs, ath_ct) = change_fmt(d.ath_change_pct);
        let (atl_cs, atl_ct) = change_fmt(d.atl_change_pct);

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Hi 24h  ", lbl),
            Span::styled(crate::ui::format_usd(d.high_24h), val),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" Lo 24h  ", lbl),
            Span::styled(crate::ui::format_usd(d.low_24h), val),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" ATH     ", lbl),
            Span::styled(crate::ui::format_usd(d.ath), val),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  date     ", dim),
            Span::styled(d.ath_date.clone(), dim),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  from ATH ", dim),
            Span::styled(ath_ct, ath_cs),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" ATL     ", lbl),
            Span::styled(crate::ui::format_usd(d.atl), val),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  date     ", dim),
            Span::styled(d.atl_date.clone(), dim),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  from ATL ", dim),
            Span::styled(atl_ct, atl_cs),
        ]));
    }

    lines.push(Line::from(""));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(Span::styled(
                    " Info ",
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(GECKO_GREEN)),
        ),
        area,
    );
}

fn render_chart(
    f: &mut ratatui::Frame,
    chart_data: Option<&[(f64, f64)]>,
    chart_error: Option<&str>,
    coin: &MarketEntry,
    area: ratatui::layout::Rect,
) {
    let block = Block::default()
        .title(Span::styled(
            " 7-Day Price (USD) ",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GECKO_GREEN));

    if let Some(err) = chart_error {
        f.render_widget(
            Paragraph::new(format!("  ✖  {err}"))
                .style(Style::default().fg(Color::Red))
                .block(block),
            area,
        );
        return;
    }

    let Some(data) = chart_data else {
        f.render_widget(
            Paragraph::new("  No data.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    };

    if data.is_empty() {
        f.render_widget(
            Paragraph::new("  No data points.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    }

    let min_y = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let max_y = data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    // Chart data has at most a few hundred points — well within f64 precision.
    #[allow(clippy::cast_precision_loss)]
    let x_max = data.len().saturating_sub(1) as f64;
    let margin = (max_y - min_y) * 0.05;
    let y_lo = (min_y - margin).max(0.0);
    let y_hi = max_y + margin;

    let (trend_style, _) = change_fmt(coin.change_24h);

    let dataset = Dataset::default()
        .name(coin.symbol.to_uppercase())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(trend_style)
        .data(data);

    let x_mid = x_max / 2.0;
    let y_mid = f64::midpoint(y_lo, y_hi);
    let gray = Style::default().fg(Color::DarkGray);

    let chart = Chart::new(vec![dataset])
        .block(block)
        .x_axis(
            Axis::default()
                .style(gray)
                .bounds([0.0, x_max])
                .labels(vec![
                    Span::styled("Day 1", gray),
                    Span::styled(
                        {
                            let mid_day = (x_mid / x_max * 6.0 + 1.0).round();
                            format!("Day {mid_day:.0}")
                        },
                        gray,
                    ),
                    Span::styled("Day 7", gray),
                ]),
        )
        .y_axis(
            Axis::default()
                .style(gray)
                .bounds([y_lo, y_hi])
                .labels(vec![
                    Span::styled(crate::ui::format_large_usd(min_y), gray),
                    Span::styled(crate::ui::format_large_usd(y_mid), gray),
                    Span::styled(crate::ui::format_large_usd(max_y), gray),
                ]),
        );

    f.render_widget(chart, area);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn change_fmt(pct: f64) -> (Style, String) {
    if pct >= 0.0 {
        (
            Style::default().fg(Color::Green),
            format!("▲ {:.2}%", pct.abs()),
        )
    } else {
        (
            Style::default().fg(Color::Red),
            format!("▼ {:.2}%", pct.abs()),
        )
    }
}
