use crate::parsers::ScoutMetadata;

pub fn parse_metadata(content: &str) -> ScoutMetadata {
    let (team_home, team_away) = parse_teams(content);
    let season_year = parse_season_year(content);
    let has_video = parse_has_video(content);
    let match_date = parse_match_date(content);
    ScoutMetadata {
        team_home,
        team_away,
        season_year,
        has_video,
        match_date,
    }
}

pub fn extract_video_reference(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let marker_index = lines.iter().position(|line| line.trim() == "[3VIDEO]")?;

    for line in &lines[marker_index + 1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') {
            break;
        }

        let parts: Vec<&str> = trimmed.split(';').map(str::trim).collect();
        for part in parts {
            let candidate = part
                .split_once('=')
                .map(|(_, value)| value.trim())
                .unwrap_or(part);
            if is_video_reference(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

fn parse_has_video(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let marker_index = match lines.iter().position(|line| line.trim() == "[3VIDEO]") {
        Some(i) => i,
        None => return false,
    };

    for line in &lines[marker_index + 1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') {
            break;
        }

        let parts: Vec<&str> = trimmed.split(';').map(str::trim).collect();
        if parts.iter().any(|part| is_video_reference(part)) {
            return true;
        }
    }

    false
}

fn is_video_reference(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    if lower == "-" || lower == "none" || lower == "null" || lower == "n/a" {
        return false;
    }
    lower.contains(".mp4")
        || lower.contains(".mov")
        || lower.contains(".mkv")
        || lower.contains(".avi")
        || lower.contains(".wmv")
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.contains("\\")
        || lower.contains('/')
}

fn parse_teams(content: &str) -> (Option<String>, Option<String>) {
    let lines: Vec<&str> = content.lines().collect();
    let marker_index = match lines.iter().position(|line| line.trim() == "[3TEAMS]") {
        Some(i) => i,
        None => return (None, None),
    };
    let team_lines: Vec<&str> = lines[marker_index + 1..]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .take(2)
        .collect();
    let parse_name = |line: &str| -> Option<String> {
        line.split(';')
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let team_home = team_lines.first().and_then(|l| parse_name(l));
    let team_away = team_lines.get(1).and_then(|l| parse_name(l));
    (team_home, team_away)
}

fn parse_season_year(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let marker_index = lines.iter().position(|line| line.trim() == "[3MATCH]")?;
    let date_line = lines[marker_index + 1..]
        .iter()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())?;

    let mut best: Option<String> = None;
    for part in date_line.split(|c: char| !c.is_ascii_digit()) {
        if part.len() == 4 {
            if let Ok(year) = part.parse::<u16>() {
                if (1900..=2099).contains(&year) {
                    best = Some(part.to_string());
                    break;
                }
            }
        }
    }
    best
}

fn parse_match_date(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let marker_index = lines.iter().position(|line| line.trim() == "[3MATCH]")?;
    let date_line = lines[marker_index + 1..]
        .iter()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())?;
    if date_line.is_empty() {
        return None;
    }
    Some(date_line.to_string())
}

pub fn derive_match_name(team_home: Option<&str>, team_away: Option<&str>, filename_stem: &str) -> String {
    match (team_home, team_away) {
        (Some(home), Some(away)) => format!("{home} vs {away}"),
        (Some(home), None) => home.to_string(),
        _ => filename_stem.to_string(),
    }
}
