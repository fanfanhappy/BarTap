//! 日历事件模型与 iCalendar 解析。
//!
//! 原先放在邮件模块里；移除邮件功能后独立出来，供 CalDAV、`.ics` 导入和本地日程共用。

/// 会议邀请里界面和提醒要用的字段。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    pub all_day: bool,
    pub title: String,
    /// 开始时间（Unix 秒，本地时区解释）。
    pub start: i64,
    /// 来源提供的结束时间；缺失或无效时不推测时长。
    pub end: Option<i64>,
    /// 本地时间 `MM-DD HH:MM`，直接显示。
    pub at: String,
    /// iCalendar 的 `UID`；多来源靠它 + 开始时间去重。
    pub uid: String,
}

/// 从一段 iCalendar 里取出所有带时刻的 VEVENT。
///
/// CalDAV 的 `calendar-query` 一次会返回多个事件；支持带时刻和全天日期，
/// 没有有效 DTSTART 就跳过。
pub fn meetings_of(calendar: &str) -> Vec<Meeting> {
    // RFC 5545 折行：以空格/制表符开头的行是上一行的续行。
    let mut lines: Vec<String> = Vec::new();
    for raw in calendar.lines() {
        let line = raw.trim_end_matches('\r');
        if (line.starts_with(' ') || line.starts_with('\t')) && !lines.is_empty() {
            if let Some(last) = lines.last_mut() {
                last.push_str(&line[1..]);
            }
        } else {
            lines.push(line.to_string());
        }
    }

    let mut out = Vec::new();
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut in_event = false;
    let mut cancelled = false;
    let mut nested = 0usize;
    for line in &lines {
        let up = line.to_ascii_uppercase();
        if up == "BEGIN:VCALENDAR" {
            cancelled = false;
        }
        if !in_event && up == "METHOD:CANCEL" {
            cancelled = true;
        }
        if up == "BEGIN:VEVENT" {
            in_event = true;
            nested = 0;
            fields.clear();
            continue;
        }
        if up == "END:VEVENT" {
            if in_event && !cancelled {
                if let Some(m) = event_of(&fields) {
                    out.push(m);
                }
            }
            in_event = false;
            continue;
        }
        if !in_event {
            continue;
        }
        if up.starts_with("BEGIN:") {
            nested += 1;
            continue;
        }
        if up.starts_with("END:") {
            nested = nested.saturating_sub(1);
            continue;
        }
        if nested > 0 {
            continue;
        }
        let Some((head, value)) = line.split_once(':') else { continue };
        let name = head.split(';').next().unwrap_or("").to_ascii_uppercase();
        if name == "DTSTART" || name == "DTEND" {
            if let Some(tz) = head.split(';').skip(1).find_map(|p| {
                p.split_once('=')
                    .filter(|(k, _)| k.eq_ignore_ascii_case("TZID"))
                    .map(|(_, v)| v.trim_matches('"'))
            }) {
                fields.push((format!("{name}-TZID"), tz.to_string()));
            }
        }
        fields.push((name, value.trim().to_string()));
    }
    out
}

fn event_of(fields: &[(String, String)]) -> Option<Meeting> {
    use chrono::TimeZone;
    let get = |name: &str| fields.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone());
    if get("STATUS").is_some_and(|v| v.eq_ignore_ascii_case("CANCELLED")) {
        return None;
    }
    let parse_time = |name: &str| -> Option<chrono::DateTime<chrono::Local>> {
        let value = get(name)?;
        Some(if let Some(s) = value.strip_suffix('Z') {
            chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S")
                .ok()?
                .and_utc()
                .with_timezone(&chrono::Local)
        } else {
            if !value.contains('T') {
                return chrono::NaiveDate::parse_from_str(&value, "%Y%m%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)?
                    .and_local_timezone(chrono::Local)
                    .earliest();
            }
            let n = chrono::NaiveDateTime::parse_from_str(&value, "%Y%m%dT%H%M%S").ok()?;
            if let Some(tz) = get(&format!("{name}-TZID")) {
                let zone: chrono_tz::Tz = tz.parse().ok()?;
                zone.from_local_datetime(&n).single()?.with_timezone(&chrono::Local)
            } else {
                chrono::Local.from_local_datetime(&n).single()?
            }
        })
    };
    let naive = parse_time("DTSTART")?;
    let all_day = get("DTSTART").is_some_and(|value| !value.contains('T'));
    let end = parse_time("DTEND")
        .map(|at| at.timestamp())
        .filter(|end| *end > naive.timestamp())
        .or_else(|| {
            all_day
                .then(|| {
                    naive
                        .date_naive()
                        .succ_opt()?
                        .and_hms_opt(0, 0, 0)?
                        .and_local_timezone(chrono::Local)
                        .earliest()
                        .map(|at| at.timestamp())
                })
                .flatten()
        });
    let title = get("SUMMARY")
        .map(|v| ics_text(&v))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "会议".to_string());
    Some(Meeting {
        all_day,
        title,
        start: naive.timestamp(),
        end,
        at: naive
            .format(if all_day { "%m-%d 全天" } else { "%m-%d %H:%M" })
            .to_string(),
        uid: get("UID").unwrap_or_default(),
    })
}

/// iCalendar 文本转义：`\n` / `\,` / `\;` / `\\`。
fn ics_text(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut it = v.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') | Some('N') => out.push(' '),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_meetings_are_not_active_events() {
        let event = "BEGIN:VCALENDAR\nMETHOD:CANCEL\nBEGIN:VEVENT\nSUMMARY:取消\nDTSTART:20261030T063000Z\nEND:VEVENT\nEND:VCALENDAR";
        assert!(meetings_of(event).is_empty());
        let message = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:取消\nDTSTART:20261030T063000Z\nSTATUS:CANCELLED\nEND:VEVENT\nEND:VCALENDAR";
        assert!(meetings_of(message).is_empty());
    }

    #[test]
    fn alarm_fields_do_not_replace_meeting_fields() {
        let event = "BEGIN:VEVENT\nSUMMARY:正式会议\nDTSTART:20261030T063000Z\nBEGIN:VALARM\nSUMMARY:提醒\nDTSTART:20261030T060000Z\nEND:VALARM\nEND:VEVENT";
        assert_eq!(meetings_of(event)[0].title, "正式会议");
    }

    #[test]
    fn ics_meeting_title_and_start() {
        let ics = "BEGIN:VEVENT\nUID:evt-1\nSUMMARY:项目 评审\nDTSTART:20261030T063000Z\nDTEND:20261030T073000Z\nEND:VEVENT";
        let m = meetings_of(ics).into_iter().next().expect("应当识别出会议");
        assert_eq!(m.title, "项目 评审");
        assert_eq!(m.uid, "evt-1");
        assert!(m.end.is_some());
    }

    #[test]
    fn all_day_events_are_supported() {
        let event = "BEGIN:VEVENT\nUID:all-day\nSUMMARY:全天\nDTSTART;VALUE=DATE:20261103\nDTEND;VALUE=DATE:20261105\nEND:VEVENT";
        let m = meetings_of(event).into_iter().next().expect("全天日程应可展示");
        assert!(m.all_day);
        assert_eq!(m.at, "11-03 全天");
    }

    #[test]
    fn no_dtstart_means_no_meeting() {
        assert!(meetings_of("").is_empty());
        assert!(meetings_of("会议时间：10月30日 14:30").is_empty());
    }
}
