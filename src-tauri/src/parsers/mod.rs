pub mod dvw;

pub struct ScoutMetadata {
    pub team_home: Option<String>,
    pub team_away: Option<String>,
    pub season_year: Option<String>,
    pub has_video: bool,
    pub match_date: Option<String>,
}
