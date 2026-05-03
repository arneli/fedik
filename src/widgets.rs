use crate::font::{RenderedText, convert_style_vec};
use chrono::{Datelike, NaiveDate};
use std::convert::TryFrom;

use super::Args;
use chrono::Timelike;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Offset, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Widget, WidgetRef},
};

pub fn get_unix_time() -> anyhow::Result<std::time::Duration> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.into())
}

pub fn get_next_timing(unix_epoch: std::time::Duration, precision_ms: u64) -> std::time::Duration {
    let now_ms = unix_epoch.as_secs() * 1000 + unix_epoch.subsec_millis() as u64;
    let precision_ms = precision_ms.max(1);
    let time = precision_ms - (now_ms % precision_ms);
    std::time::Duration::from_millis(time)
}

pub fn format_week_string(weekday: NaiveDate) -> String {
    let iso_week = weekday.iso_week().week();
    let weekday = weekday.weekday().number_from_monday();
    let weekday_str = format!("{:?}", weekday as u8);
    format!("w{}-{}", iso_week, weekday_str)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millisecond: u32,
}

impl From<chrono::NaiveTime> for Time {
    fn from(time: chrono::NaiveTime) -> Self {
        Self {
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            millisecond: time.nanosecond() / 1_000_000,
        }
    }
}

impl From<chrono::NaiveDateTime> for Time {
    fn from(date_time: chrono::NaiveDateTime) -> Self {
        date_time.time().into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WidgetState {
    //    Disable,
    Enable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootWidget {
    pub clock_widget: Box<ClockWidget>,
}

impl RootWidget {
    pub fn new(args: Args) -> Self {
        let display = match (args.show_seconds, args.ms_digits) {
            (false, None) => TimeDisplay::Minutes,
            (true, None) => TimeDisplay::Seconds,
            (_, Some(1)) => TimeDisplay::SecondsMs1,
            (_, Some(2)) => TimeDisplay::SecondsMs2,
            (_, Some(3)) => TimeDisplay::SecondsMs3,
            (_, Some(_)) => unreachable!(),
        };
        Self {
            clock_widget: Box::new(ClockWidget::new(
                display,
                args.show_date,
                args.show_week,
                args.bold,
                args.center,
            )),
        }
    }
}

impl WidgetRef for RootWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.clock_widget.render_ref(area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockWidget {
    pub time_widget: TimeWidget,
    pub date_widget: DateWidget,
    pub week_widget: WeekWidget,
    pub layout: Layout,
    pub show_date: bool,
    pub show_week: bool,
    pub bold: bool,
    pub center: bool,
}

impl ClockWidget {
    pub fn new(display: TimeDisplay, show_date: bool, show_week: bool, bold: bool, center: bool) -> Self {
        Self {
            time_widget: TimeWidget::new(display, bold, center),
            date_widget: DateWidget::new(
                chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                bold,
                center,
            ),
            week_widget: WeekWidget::new(
                chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                bold,
                center,
            ),
            layout: Layout::default(),
            show_date,
            show_week,
            bold,
            center,
        }
    }
}

impl WidgetRef for ClockWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area_1 = area.height / 2 + 3;
        let mut constraints = vec![Constraint::Length(area_1)];
        if self.show_date {
            constraints.push(Constraint::Length(1));
        }
        if self.show_week {
            constraints.push(Constraint::Length(1));
        }
        let used_height =
            area_1 + if self.show_date { 1 } else { 0 } + if self.show_week { 1 } else { 0 };
        let remaining = area.height.saturating_sub(used_height);
        constraints.push(Constraint::Length(remaining));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        self.time_widget.render_ref(layout[0], buf);
        let mut idx = 1;
        if self.show_date {
            self.date_widget.render_ref(layout[idx], buf);
            idx += 1;
        }
        if self.show_week {
            self.week_widget.render_ref(layout[idx], buf);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateWidget {
    pub state: WidgetState,
    pub date: chrono::NaiveDate,
    pub bold: bool,
    pub center: bool,
}

impl DateWidget {
    pub fn new(date: chrono::NaiveDate, bold: bool, center: bool) -> Self {
        Self {
            state: WidgetState::Enable,
            date,
            bold,
            center,
        }
    }
}

impl WidgetRef for DateWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let date_string = self.date.to_string();
        let offset_x = if self.center {
            area.width.overflowing_sub(date_string.len() as u16).0 as i32 / 2
        } else {
            0
        };
        let area = area.offset(Offset {
            x: offset_x,
            y: 0,
        });
        let segment = Span::raw(date_string);
        let line = Line::from(vec![segment]);

        let style = if self.bold {
            Style::new().green().bold()
        } else {
            Style::new().green()
        };
        line.style(style).render(area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekWidget {
    pub state: WidgetState,
    pub weekday: chrono::NaiveDate,
    pub bold: bool,
    pub center: bool,
}

impl WeekWidget {
    pub fn new(weekday: chrono::NaiveDate, bold: bool, center: bool) -> Self {
        Self {
            state: WidgetState::Enable,
            weekday,
            bold,
            center,
        }
    }
}

impl WidgetRef for WeekWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let week_str = format_week_string(self.weekday);
        let offset_x = if self.center {
            area.width.overflowing_sub(week_str.len() as u16).0 as i32 / 2
        } else {
            0
        };
        let area = area.offset(Offset {
            x: offset_x,
            y: 0,
        });
        let segment = Span::raw(week_str);
        let line = Line::from(vec![segment]);
        let style = if self.bold {
            Style::new().green().bold()
        } else {
            Style::new().green()
        };
        line.style(style).render(area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWidget {
    pub state: WidgetState,
    pub time: Time,
    pub display: TimeDisplay,
    pub bold: bool,
    pub center: bool,
}

impl TimeWidget {
    pub fn new(display: TimeDisplay, bold: bool, center: bool) -> Self {
        Self {
            state: WidgetState::Enable,
            time: Time::default(),
            display,
            bold,
            center,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeDisplay {
    Minutes,
    Seconds,
    SecondsMs1,
    SecondsMs2,
    SecondsMs3,
}

fn format_time_string(time: Time, display: TimeDisplay) -> String {
    match display {
        TimeDisplay::Minutes => format!("{:02}:{:02}", time.hour, time.minute),
        TimeDisplay::Seconds => format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second),
        TimeDisplay::SecondsMs1 => format!(
            "{:02}:{:02}:{:02}.{:01}",
            time.hour,
            time.minute,
            time.second,
            time.millisecond / 100
        ),
        TimeDisplay::SecondsMs2 => format!(
            "{:02}:{:02}:{:02}.{:02}",
            time.hour,
            time.minute,
            time.second,
            time.millisecond / 10
        ),
        TimeDisplay::SecondsMs3 => format!(
            "{:02}:{:02}:{:02}.{:03}",
            time.hour, time.minute, time.second, time.millisecond
        ),
    }
}

impl WidgetRef for TimeWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let time_str = format_time_string(self.time, self.display);
        let rendered = RenderedText::try_from(time_str.as_str()).unwrap();
        let style_vec = convert_style_vec(rendered.lines, self.bold);
        let str_width = style_vec[0].len() as u16;
        let offset_x = if self.center {
            (area.width.overflowing_sub(str_width).0) as i32 / 2
        } else {
            0
        };
        let offset_y = if self.center {
            area.height.overflowing_sub(style_vec.len() as u16).0 as i32
        } else {
            0
        };
        let mut _offset: Offset = Offset::default();
        _offset.x = offset_x;
        _offset.y = offset_y;
        style_vec.iter().enumerate().for_each(|(i, line)| {
            line.iter().enumerate().for_each(|(j, style)| {
                let y = _offset.y + i as i32;
                let x = _offset.x + j as i32;
                let offset = Offset { x, y };
                Span::styled(" ", style.clone()).render(area.offset(offset), buf);
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_time_from_naive_time() {
        let time = "12:34:56.789".parse::<chrono::NaiveTime>().unwrap();
        let rendered = Time::from(time);

        assert_eq!(rendered.hour, 12);
        assert_eq!(rendered.minute, 34);
        assert_eq!(rendered.second, 56);
        assert_eq!(rendered.millisecond, 789);
    }

    #[test]
    fn test_time_widget_new_sets_display() {
        let widget = TimeWidget::new(TimeDisplay::SecondsMs2, true, true);
        assert_eq!(widget.display, TimeDisplay::SecondsMs2);
        assert!(widget.bold);
        assert!(widget.center);
        assert_eq!(widget.time, Time::default());
    }

    #[test]
    fn test_clock_widget_new_sets_flags() {
        let widget = ClockWidget::new(TimeDisplay::SecondsMs1, false, true, false, true);
        assert_eq!(widget.time_widget.display, TimeDisplay::SecondsMs1);
        assert!(!widget.show_date);
        assert!(widget.show_week);
        assert!(!widget.bold);
        assert!(widget.center);
    }

    #[test]
    fn test_get_unix_time() {
        let unix_time = get_unix_time().unwrap();
        assert!(unix_time.as_secs() > 0);
    }

    #[test]
    fn test_next_duration() {
        let unix_time = get_unix_time().unwrap();
        let next = get_next_timing(unix_time, 1000);
        assert!(next.as_millis() > 0);
        assert!(next.as_millis() <= 1000);
    }

    #[test]
    fn test_get_next_timing_rollover_at_boundary() {
        let unix_epoch = std::time::Duration::from_millis(1000);
        let next = get_next_timing(unix_epoch, 1000);
        assert_eq!(next.as_millis(), 1000);
    }

    #[test]
    fn test_get_next_timing_near_second_boundary() {
        let unix_epoch = std::time::Duration::from_millis(1999);
        let next = get_next_timing(unix_epoch, 1000);
        assert_eq!(next.as_millis(), 1);
    }

    #[test]
    fn test_get_next_timing_with_ms_precision() {
        let unix_epoch = std::time::Duration::from_millis(1999);
        let next = get_next_timing(unix_epoch, 1);
        assert_eq!(next.as_millis(), 1);
    }

    #[test]
    fn test_format_week_string() {
        let date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let week_str = format_week_string(date);
        assert_eq!(week_str, "w53-5");
    }

    #[test]
    fn test_root_widget_sets_time_display_from_args() {
        let root = RootWidget::new(super::Args {
            show_seconds: false,
            ms_digits: None,
            center: true,
            bold: true,
            hour_12: false,
            utc: false,
            format: None,
            show_date: true,
            show_week: false,
        });
        assert_eq!(root.clock_widget.time_widget.display, TimeDisplay::Minutes);
        assert!(root.clock_widget.show_date);
        assert!(!root.clock_widget.show_week);

        let root = RootWidget::new(super::Args {
            show_seconds: true,
            ms_digits: None,
            center: true,
            bold: true,
            hour_12: false,
            utc: false,
            format: None,
            show_date: false,
            show_week: true,
        });
        assert_eq!(root.clock_widget.time_widget.display, TimeDisplay::Seconds);
        assert!(!root.clock_widget.show_date);
        assert!(root.clock_widget.show_week);

        let root = RootWidget::new(super::Args {
            show_seconds: false,
            ms_digits: Some(2),
            center: true,
            bold: true,
            hour_12: false,
            utc: false,
            format: None,
            show_date: true,
            show_week: true,
        });
        assert_eq!(
            root.clock_widget.time_widget.display,
            TimeDisplay::SecondsMs2
        );
        assert!(root.clock_widget.show_date);
        assert!(root.clock_widget.show_week);
        assert!(root.clock_widget.bold);
    }

    #[test]
    fn test_format_time_string_for_each_display_mode() {
        let time = Time {
            hour: 12,
            minute: 34,
            second: 56,
            millisecond: 789,
        };

        assert_eq!(format_time_string(time, TimeDisplay::Minutes), "12:34");
        assert_eq!(format_time_string(time, TimeDisplay::Seconds), "12:34:56");
        assert_eq!(
            format_time_string(time, TimeDisplay::SecondsMs1),
            "12:34:56.7"
        );
        assert_eq!(
            format_time_string(time, TimeDisplay::SecondsMs2),
            "12:34:56.78"
        );
        assert_eq!(
            format_time_string(time, TimeDisplay::SecondsMs3),
            "12:34:56.789"
        );
    }
}
