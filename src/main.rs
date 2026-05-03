use anyhow::Result;
use chrono::{Local, Utc};
use clap::Parser;
mod font;
use crossterm::{
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{layout::Rect, prelude::CrosstermBackend};

mod widgets;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(version, about, long_about = None)]
struct Args {
    ///     -s            Show seconds
    #[clap(short = 's', long, default_value = "false")]
    show_seconds: bool,
    ///     -m [1-3]      Show milliseconds with the specified number of digits
    #[clap(short = 'm', long, value_parser = clap::value_parser!(u8).range(1..=3))]
    ms_digits: Option<u8>,
    ///     -d            Show the date
    #[clap(short = 'd', long, default_value = "false")]
    show_date: bool,
    ///     -w            Show the week number and weekday
    #[clap(short = 'w', long, default_value = "false")]
    show_week: bool,
    ///     -c            Set the clock at the center of the terminal
    #[clap(short = 'c', long, default_value = "false")]
    center: bool,
    ///     -b            Use bold font
    #[clap(short = 'b', long, default_value = "false")]
    bold: bool,
    ///     --hour-12     Not implemented yet: use 12-hour format
    #[clap(long, default_value = "false")]
    hour_12: bool,
    ///     -u            Use UTC time
    #[clap(short = 'u', long)]
    utc: bool,
    ///     -f format     Not implemented yet: set the date format
    #[clap(short = 'f', long, value_name = "FORMAT")]
    format: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;
    let utc = args.utc;
    let mut terminal = ratatui::init();
    terminal.hide_cursor()?;
    let mut root_widget = widgets::RootWidget::new(args);

    terminal = loop_run(terminal, &mut root_widget, utc).await?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(())
}

async fn loop_run(
    mut terminal: ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    root_widget: &mut widgets::RootWidget,
    utc: bool,
) -> Result<ratatui::Terminal<CrosstermBackend<std::io::Stdout>>> {
    loop {
        let area = terminal.get_frame().area();
        let now: chrono::NaiveDateTime = if utc {
            Utc::now().naive_utc()
        } else {
            Local::now().naive_local()
        };
        terminal_action(now, root_widget, &mut terminal, area)?;
        terminal.hide_cursor()?;
        let unix_time = widgets::get_unix_time()?;
        let precision_ms = match root_widget.clock_widget.time_widget.display {
            widgets::TimeDisplay::Minutes => 1000,
            widgets::TimeDisplay::Seconds => 1000,
            widgets::TimeDisplay::SecondsMs1 => 100,
            widgets::TimeDisplay::SecondsMs2 => 10,
            widgets::TimeDisplay::SecondsMs3 => 1,
        };
        if poll(widgets::get_next_timing(unix_time, precision_ms))?
            && let Event::Key(key) = read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(terminal),
                _ => {}
            }
        }
    }
}

fn terminal_action(
    now: chrono::NaiveDateTime,
    app: &mut widgets::RootWidget,
    terminal: &mut ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
) -> Result<()> {
    use ratatui::widgets::FrameExt;
    app.clock_widget.time_widget.time = now.into();
    app.clock_widget.date_widget.date = now.date();
    app.clock_widget.week_widget.weekday = now.date();
    terminal.draw(|frame| frame.render_widget_ref(app.clone(), area))?;
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.hour_12 {
        anyhow::bail!("--hour-12 is not implemented yet");
    }
    if args.format.is_some() {
        anyhow::bail!("--format is not implemented yet");
    }
    Ok(())
}
