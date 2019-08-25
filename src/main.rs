#![allow(unused)]
/// baseball 
///     
/// **TODO** Add Documention to the root here

use isahc::prelude::*;
use rayon::prelude::*;
use serde::{Serialize, Deserialize, Deserializer};
use std::{error, fmt, num};
use std::collections::HashMap;

// use csv::Writer;

// // Enumerate all the baseball levels so that we can loop through a range of levels
// const LEVELS: [baseball::Level; 8] = [
//     baseball::Level{code: "mlb", name: "Majors", class: "MLB", rank: 0},
//     baseball::Level{code: "aaa", name: "Triple A", class: "AAA", rank: 1},
//     baseball::Level{code: "aax", name: "Double A", class: "AA", rank: 2},
//     baseball::Level{code: "afa", name: "High A", class: "A+", rank: 3},
//     baseball::Level{code: "afx", name: "Single A", class: "A", rank: 4},
//     baseball::Level{code: "asx", name: "Low A", class: "A-", rank: 5},
//     baseball::Level{code: "rok", name: "Rookie", class: "R", rank: 6},
//     baseball::Level{code: "win", name: "Winter", class: "W", rank: 7},
// ];


/// There are 7 levels to major league baseball, as well as winter ball. 
/// You can use LEVEL_CODES to iterate through all the levels for any date range
/// These codes show up in the root of the mlb gameday directories
pub const LEVEL_CODES: [&'static str; 8] = ["mlb", "aaa", "aax", "afa", "afx", "asx", "rok", "win"];

/// Level Names correspond to the long form representation of the levels
pub const LEVEL_NAMES: [&'static str; 8] = ["Majors", "Triple A", "Double A", "High A", "Single A", "Low A", "Rookie", "Winter"];

/// Level Class corresponds to the short form representation of the the levels
pub const LEVEL_CLASS: [&'static str; 8] = ["MLB", "AAA", "AA", "A+", "A", "A-", "R", "W"];

pub struct Levels {
    codes: [&'static str; 8],
    names: [&'static str; 8],
    class: [&'static str; 8],
}

/// For convenience the codes, names and classes are gathered in to a LEVELS const struct
pub const LEVELS:Levels = Levels {
    codes: LEVEL_CODES,
    names: LEVEL_NAMES,
    class: LEVEL_CLASS,
};

type GameDayLinks =  Vec<String>;

#[derive(Deserialize, Serialize, Debug)]
enum HomeAway {
    #[serde(rename="home")]
    Home,
    #[serde(rename="away")]
    Away,
}

#[derive(Deserialize, Serialize, Debug)]
struct Team {
    #[serde(rename="type")]
    home_away: HomeAway,
    id: String,
    name: String,
    #[serde(rename="player")]
    players: Vec<Player>,
    #[serde(rename="coach")]
    coaches: Vec<Coach>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Player {
    id: u32,
    #[serde(rename="first")]
    name_first: String,
    #[serde(rename="last")]
    name_last: String,
    game_position: Option<String>,
    bat_order: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Coach {
    position: String,
    #[serde(rename="first")]
    name_first: String,
    #[serde(rename="last")]
    name_last: String,
    id: u32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Umpire {
    position: String,
    name: String,
    #[serde(deserialize_with = "empty_string_is_none")]
    id: Option<u32>,
}


// This Struct is a waste, but is neccesary to get serde_xml_rs to work
// A from/into impl is provided to transform it into the format we need
#[derive(Deserialize, Serialize, Debug)]
struct Umpires {
    #[serde(rename="umpire")]
    umpires: Vec<Umpire>
}

#[derive(Deserialize, Serialize, Debug)]
struct Game {
    #[serde(rename="team")]
    teams: Vec<Team>,
    umpires: Umpires
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct GameUmpires {
    ump_HP_id: Option<u32>,
    ump_1B_id: Option<u32>,
    ump_2B_id: Option<u32>,
    ump_3B_id: Option<u32>,
    ump_LF_id: Option<u32>,
    ump_RF_id: Option<u32>,
    ump_HP_name: String,
    ump_1B_name: String,
    ump_2B_name: String,
    ump_3B_name: String,
    ump_LF_name: String,
    ump_RF_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct LineScoreData {
    game_pk: u32,
    game_type: char,
    venue: String,
    venue_w_chan_loc: String,
    venue_id: u32,
    time: String,
    time_zone: String,
    ampm: String,
    home_team_id: u32,
    home_team_city: String,
    home_team_name: String,
    home_league_id: u32,
    away_team_id: u32,
    away_team_city: String,
    away_team_name: String,
    away_league_id: u32,
    #[serde(rename="linescore", skip_serializing)]
    innings: Vec<LineScore>,
}

#[derive(Deserialize, Serialize, Debug)]
struct LineScore {
    #[serde(rename="away_inning_runs")]
    away_runs: u32,
    #[serde(rename="home_inning_runs")]
    home_runs: u32,
    // Keeping the inning as a string, since we'll need it to construct URLs later
    inning: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct LineScores {
    #[serde(rename="linescore")]
    linescores: Vec<LineScore>,
}

#[derive(Deserialize, Serialize, Debug)]
struct BoxScoreData {
    weather_temp: u8,
    weather_condition: String,
    wind_speed: u8,
    wind_direction: String,
    attendance: Option <u32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct PlateAppearance {
    #[serde(rename="num")]
    at_bat_num: u16,
    batter: u32,
    #[serde(rename="stand")]
    batter_stands: char,
    pitcher: u32,
    #[serde(rename="p_throws")]
    pitcher_throws: char,
    #[serde(rename="des")]
    at_bat_des: String,
    #[serde(skip_deserializing)]
    outs_start: u8,
    #[serde(rename="o")]
    outs_end: u8,
    #[serde(rename="event")]
    at_bat_result: String,
    #[serde(rename="pitch")]
    pitches: Vec<Pitch>,
    #[serde(rename="runner")]
    runners: Vec<Runner>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Inning {
//   #[serde(rename="action")]
//   actions: Vec<Action>,
//   #[serde(rename="atbat")]
//   plate_apps: Vec<PlateAppearance>,
    at_bat_action: Vec<AtBatOrAction>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AtBatOrAction {
    action: Option<Action>,
    #[serde(rename="atbat")]
    atbat: Option<PlateAppearance>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Action {
    #[serde(rename="b")]
    balls: u8,
    #[serde(rename="s")]
    strikes: u8,
    #[serde(rename="o")]
    outs: u8,
    #[serde(rename="des")]
    action_description: String,
    player: u32,
    pitch: u8,
    event: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Pitch {
    des: String,
    #[serde(rename="type")]
    pitch_result: char,
    #[serde(rename="x")]
    pixels_x: f32,
    #[serde(rename="y")]
    pixels_y: f32,

    // The fields below are the MLB specific fields and will all be wrapped in Options
    ax: Option<f32>,
    ay: Option<f32>,
    az: Option<f32>,
    vx_0: Option<f32>,
    vy_0: Option<f32>,
    vz_0: Option<f32>,
    x_0: Option<f32>,
    y_0: Option<f32>,
    z_0: Option<f32>,

    #[serde(rename="px")]
    plate_x: Option<f32>,
    #[serde(rename="pz")]
    plate_z: Option<f32>,

    break_angle: Option<f32>,
    break_length: Option<f32>,
    break_y: Option<f32>,
    
    #[serde(rename="code")]
    pitch_code: Option<char>,
    #[serde(rename="des")]
    pitch_description: Option<String>,
    #[serde(rename="start_speed")]
    pitch_speed_start: Option<f32>,
    #[serde(rename="end_speed")]
    pitch_speed_end: Option<f32>,
    pitch_type: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
struct Runner {
    code: Option<char>,
    id: u32,
    start: String,
    end: String,
    event: String,
}


#[derive(Deserialize, Serialize, Debug)]
struct GameData {
    linescore_data: LineScoreData,
    boxscore_data: BoxScoreData,
    game_umps: GameUmpires,
}

impl GameData {
    fn new(boxscore_data: BoxScoreData, linescore_data: LineScoreData, game_umps: GameUmpires) -> Self {
        GameData {
            boxscore_data,
            linescore_data,
            game_umps,
        }
    }
}

#[derive(Debug)]
struct WeatherMissingError {
    err_msg: String,
}

#[derive(Debug)]
struct GameDayMissingLinksError {
    err_msg: String,
}

impl fmt::Display for WeatherMissingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing Weather Data for: {}", self.err_msg.to_owned())
    }
}

impl WeatherMissingError {
    fn error(msg: &str) -> Self {
        WeatherMissingError {
            err_msg: msg.to_string()
        }
    }
}

impl error::Error for WeatherMissingError {
    fn description(&self) -> &str {
        &self.err_msg
    }
}

impl fmt::Display for GameDayMissingLinksError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No Valid GameDay Links found for: {}", self.err_msg.to_owned())
    }
}

impl GameDayMissingLinksError {
    fn error(msg: &str) -> Self {
        GameDayMissingLinksError {
            err_msg: msg.to_string()
        }
    }
}

impl error::Error for GameDayMissingLinksError {
    fn description(&self) -> &str {
        &self.err_msg
    }
}

#[derive(Debug)]
enum GameDayError {
    Request(isahc::Error),
    XMLParse(serde_xml_rs::Error),
    ParseIntError(num::ParseIntError),
    Weather(WeatherMissingError),
    GameDayLinks(GameDayMissingLinksError),        
}

impl fmt::Display for GameDayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameDayError::Request(ref err) => write!(f, "isahc Error: {}", err),
            GameDayError::XMLParse(ref err) => write!(f, "XML Parse Error: {}", err),
            GameDayError::Weather(ref err) => write!(f, "Weather Error: {}", err),
            GameDayError::ParseIntError(ref err) => write!(f, "Interger Parse Error: {}", err),
            GameDayError::GameDayLinks(ref err) => write!(f, "Missing GameDay Links Error: {}", err),
        }
    }
}

impl error::Error for GameDayError {
    fn description(&self) -> &str {
        match *self {
            GameDayError::Request(ref err) => err.description(),
            GameDayError::XMLParse(ref err) => err.description(),
            GameDayError::Weather(ref err) => err.description(),
            GameDayError::ParseIntError(ref err) => err.description(),
            GameDayError::GameDayLinks(ref err) => err.description(),
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            GameDayError::Request(ref err) => Some(err),
            GameDayError::XMLParse(ref err) => Some(err),
            GameDayError::Weather(ref err) => Some(err),
            GameDayError::ParseIntError(ref err) => Some(err),
            GameDayError::GameDayLinks(ref err) => Some(err),
        }
    }
}


impl From<isahc::Error> for GameDayError {
    fn from(err: isahc::Error) -> GameDayError {
        GameDayError::Request(err)
    }
}

impl From<serde_xml_rs::Error> for GameDayError {
    fn from(err: serde_xml_rs::Error) -> GameDayError {
        GameDayError::XMLParse(err)
    }
}

impl From<WeatherMissingError> for GameDayError {
    fn from(err: WeatherMissingError) -> GameDayError {
        GameDayError::Weather(err)
    }
}

impl From<GameDayMissingLinksError> for GameDayError {
    fn from(err: GameDayMissingLinksError) -> GameDayError {
        GameDayError::GameDayLinks(err)
    }
}

impl From<num::ParseIntError> for GameDayError {
    fn from(err: num::ParseIntError) -> GameDayError {
        GameDayError::ParseIntError(err)
    }
}

// Converts the Umpires struct into the GameUmpires struct
// We need to pivot the umpires into defined fields, to flatten it out for our game metadata
// The From impl automatically provides an Into, which allows for a very readable .into()
#[allow(non_snake_case)]
impl From<Umpires> for GameUmpires {
    fn from(umpires: Umpires) -> GameUmpires {
        let umps: HashMap<String, (Option<u32>, String)> = umpires.umpires
            .into_iter()
            .map(|ump| (ump.position,(ump.id, ump.name)))
            .collect();

        let default = (None, String::new());

        let (ump_HP_id, ump_HP_name) = umps.get("home").unwrap_or(&default).to_owned();
        let (ump_1B_id, ump_1B_name) = umps.get("first").unwrap_or(&default).to_owned();
        let (ump_2B_id, ump_2B_name) = umps.get("second").unwrap_or(&default).to_owned();
        let (ump_3B_id, ump_3B_name) = umps.get("third").unwrap_or(&default).to_owned();
        let (ump_LF_id, ump_LF_name) = umps.get("left").unwrap_or(&default).to_owned();
        let (ump_RF_id, ump_RF_name) = umps.get("right").unwrap_or(&default).to_owned();

        GameUmpires {
            ump_HP_id, ump_HP_name,
            ump_1B_id, ump_1B_name,
            ump_2B_id, ump_2B_name,
            ump_3B_id, ump_3B_name,
            ump_LF_id, ump_LF_name,
            ump_RF_id, ump_RF_name,
        }
    }
}

// Overwrite default serde_xml behaviour to throw an error when trying to parse an empty
// string to a u32. Instead we return None, which makes a lot more sense
fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error> 
    where D: Deserializer<'de>,
{
    let n = u32::deserialize(deserializer);

    if n.is_ok() {
        Ok(Some(n.unwrap()))
    }
    else {
        Ok(None)
    }
}

fn game_day_url (level: &str, year: &str, month: &str, day: &str) -> String {

    String::from("http://gd2.mlb.com/components/game/") + level 
                    + "/year_" + year 
                    + "/month_" + month 
                    + "/day_" + day
}

// TODO - Rewrite this at some point to make it more idiomatic
// Originally wanted to return an empty Vec when there were no links, but would rather
// this return a custom error "No Games Found" rather than an empty Vec which is ambiguous, as it could be an unwrap issue,
// or a parsing issue

fn game_day_links (url: &str) -> Result<GameDayLinks, GameDayError> {

    let response = isahc::get(url)?.text()?;

    let links = response
        .split("<li>")
        .filter(|line| line.contains("gid_"))
        .map(|line| url.to_string().clone() + "/" 
            + line.split(|c| c == '>'|| c == '<').nth(2).unwrap().trim()
        )
        .collect()
        ;

    Ok(links)
}


fn split_boxscore_xml (xml: &str) -> Result<Vec<&str>, WeatherMissingError> {
    
    let items = xml
            .split("<b>")
            .filter(|item| item.starts_with("Weather") || item.starts_with("Wind") || item.starts_with("Att") )
            .filter(|item| item.contains(":"))
            //We can unwrap safely, since we're guaranteed that split will return at least 2 elements
            .map(|item| item.split(":").nth(1).unwrap().trim())
            .collect::<Vec<&str>>();

    // We want to throw an error if item[0] isn't weather or item[1] isn't "wind" 
    // since this will cause errors downstream, or worse, we'll parse the wrong data

    if  items.len() < 2 {return Err(WeatherMissingError::error("Not enough weather items"))};
    if !items[0].to_lowercase().contains(" degrees,") || !items[0].contains("<")
        {return Err(WeatherMissingError::error("Weather data has wrong format"))};
    if !items[1].to_lowercase().contains("mph,")  || !items[0].contains("<")
        {return Err(WeatherMissingError::error("Wind data has wrong format"))};

    Ok(items)
}

fn parse_weather (item: &str) -> Result<(u8, String), num::ParseIntError> {
    
    // The split_boxscore_xml function checks that the characters we're splitting on are there so we 
    // can safely unwrap any .nth() calls that we do after splitting
    Ok((
        item.split(" ").nth(0).unwrap().parse()?,
        item.split(",").nth(1).unwrap()
            .split("<").nth(0).unwrap()
            .trim_end_matches(".").trim().to_string()
    ))
}

fn parse_attendance (att: &str) -> Result<Option<u32>, num::ParseIntError> {
    let attendance: u32 = att
                .replace(":", "").replace(",", "").replace(".", "")
                .split("<").nth(0).unwrap()
                .trim().to_string()
                .parse()?;
    Ok( Some (attendance))
}

fn create_inning_links (base: &str, innings: &Vec<LineScore>) -> Vec<String> {

    innings
        .iter()
        .map(|i| base.to_string() + "inning/inning_" + &i.inning + ".xml")
        .collect()
}


// TODO - create the struct first
fn parse_inning (url: &str) -> Option<u32> {

    None

}

/// Takes in a base url for each game, downloads all the relevant xml files and parses them
/// 
/// The linescore.xml file contains imporant metadata about the game, such as venue and date
/// 
/// The boxscore.xml file contains weather and attendance data, as well as a "linescore" section which we
/// can use at some point to make sure we gathered correct play-play data. It will also tell us how many
/// inning.xml files we'll need to pull
/// 
/// The players.xml file gives us the initial position of all the players as well as the coaches and umpires for the game
/// 
/// 

fn game_download_parse (url: &str) -> Result <GameData, GameDayError> {

    let linescore_url = format!("{}linescore.xml", url);
    let boxscore_url = format!("{}boxscore.xml", url);
    let players_url = format!("{}players.xml", url);

    let linescore_xml = isahc::get(&linescore_url)?.text()?.replace('&', "&amp;");
    let linescore_data: LineScoreData = serde_xml_rs::from_str(&linescore_xml)?;

    let inning_links = create_inning_links(url, &linescore_data.innings);
    // dbg!(inning_links);

    let boxscore_xml = isahc::get(&boxscore_url)?.text()?;
    
    let items = split_boxscore_xml(&boxscore_xml)?;

    let (weather_temp, weather_condition) = parse_weather(items[0])?;
    let (wind_speed, wind_direction) = parse_weather(items[1])?;
    let attendance: Option<u32> = if items.len() > 2 {parse_attendance(items[2])?} else {None};

    let players_xml = isahc::get(&players_url)?.text()?;
    let player_data: Game = serde_xml_rs::from_str(&players_xml)?;

    let game_umps: GameUmpires = player_data.umpires.into();
    
    let boxscore_data = BoxScoreData {weather_temp, weather_condition, wind_speed, wind_direction, attendance};

    let game_data = GameData::new(boxscore_data, linescore_data, game_umps);

    Ok(game_data)
}

fn main () {

    let url = game_day_url("mlb", "2008", "06", "10");
    let games = game_day_links(&url).unwrap();

    dbg!(game_download_parse(&games[0]));

    let at_bat_xml = r#"<atbat num="77" b="0" s="0" o="1" batter="544881" stand="R" b_height="6-0" pitcher="501640" p_throws="R" des="Donell Linares doubles (4) on a line drive to left fielder Guillermo Pimentel.    Erick Epifano to 3rd.  " event="Double"><pitch des="In play, no out" id="446" type="X" x="97.85" y="120.88" on_1b="542573"/><runner id="542573" start="1B" end="3B" event="Double"/><runner id="544881" start="" end="2B" event="Double"/></atbat>"#;

    let at_bat_xml = r#"<atbat num="4" b="1" s="0" o="3" batter="544881" stand="R" b_height="6-0" pitcher="542653" p_throws="R" des="Donell Linares flies out to center fielder Ariel Ventura.  " event="Fly Out"><pitch des="Ball" id="18" type="B" x="56.65" y="127.79" on_1b="501592"/><runner id="501592" start="1B" end="2B" event="Stolen Base 2B"/><pitch des="In play, out(s)" id="22" type="X" x="105.58" y="139.02" on_2b="501592"/><runner id="501592" start="2B" end="" event="Fly Out"/></atbat>"#;

    // let at_bat: PlateAppearance = serde_xml_rs::from_str(at_bat_xml).unwrap();

    // let inning_xml = r#"<action away_team_runs="6" b="0" des="Pitching Change: Brandon Workman replaces Nathan Eovaldi." des_es="Pitching Change: Brandon Workman replaces Nathan Eovaldi." event="Pitching Substitution" event_es="Pitching Substitution" event_num="29" home_team_runs="5" o="0" pitch="1" player="519443" s="0" tfs="023625" tfs_zulu="2019-08-14T02:36:25.000Z"/><atbat away_team_runs="6" b="1" b_height="6' 0" batter="656185" des="Greg Allen singles on a line drive to right fielder Mookie Betts." des_es="Greg Allen singles on a line drive to right fielder Mookie Betts." end_tfs_zulu="2019-08-14T02:38:15.000Z" event="Single" event_es="Single" event_num="30" home_team_runs="5" num="78" o="0" p_throws="R" pitcher="519443" play_guid="5f528c85-3549-494b-be49-8b6f08d5568a" s="2" stand="L" start_tfs="023625" start_tfs_zulu="2019-08-14T02:36:25.000Z"><pitch ax="4.27" ay="21.64" az="-43.86" break_angle="6.0" break_length="12.0" break_y="24.0" cc="" code="C" des="Called Strike" des_es="Pitching Change: Brandon Workman replaces Nathan Eovaldi." end_speed="75.5" event_num="30" id="30" mt="" nasty="" pfx_x="3.0" pfx_z="-8.22" pitch_type="KC" play_guid="e06f83b0-a3ed-4c62-89bf-e84ee9796854" px="-0.68" pz="2.27" spin_dir="placeholder" spin_rate="placeholder" start_speed="81.1" sv_id="190814_023708" sz_bot="1.63" sz_top="3.4" tfs="023703" tfs_zulu="2019-08-14T02:37:03.000Z" type="S" type_confidence="placeholder" vx0="1.68" vy0="-118.25" vz0="-0.19" x="143.09" x0="-1.79" y="177.54" y0="50.0" z0="6.36" zone="placeholder"/><pitch ax="3.45" ay="22.07" az="-44.84" break_angle="4.8" break_length="13.2" break_y="24.0" cc="" code="S" des="Swinging Strike" des_es="Called Strike" end_speed="75.1" event_num="30" id="30" mt="" nasty="" pfx_x="2.47" pfx_z="-9.07" pitch_type="KC" play_guid="32bd833f-e3d1-488f-829e-3ce6d6656494" px="0.87" pz="1.55" spin_dir="placeholder" spin_rate="placeholder" start_speed="80.6" sv_id="190814_023724" sz_bot="1.57" sz_top="3.37" tfs="023718" tfs_zulu="2019-08-14T02:37:18.000Z" type="S" type_confidence="placeholder" vx0="5.28" vy0="-117.38" vz0="-1.59" x="83.79" x0="-1.73" y="196.97" y0="50.0" z0="6.41" zone="placeholder"/><pitch ax="5.44" ay="22.93" az="-45.58" break_angle="7.2" break_length="13.2" break_y="24.0" cc="" code="B" des="Ball" des_es="Swinging Strike" end_speed="74.6" event_num="30" id="30" mt="" nasty="" pfx_x="3.88" pfx_z="-9.56" pitch_type="KC" play_guid="1d69625b-c575-408f-ac2e-407a77f96065" px="-1.44" pz="3.05" spin_dir="placeholder" spin_rate="placeholder" start_speed="80.9" sv_id="190814_023748" sz_bot="1.61" sz_top="3.27" tfs="023743" tfs_zulu="2019-08-14T02:37:43.000Z" type="B" type_confidence="placeholder" vx0="-0.44" vy0="-117.86" vz0="1.7" x="172.0" x0="-1.76" y="156.35" y0="50.0" z0="6.54" zone="placeholder"/><pitch ax="4.25" ay="23.6" az="-45.69" break_angle="6.0" break_length="13.2" break_y="24.0" cc="" code="D" des="In play, no out" des_es="Ball" end_speed="75.4" event_num="30" id="30" mt="" nasty="" pfx_x="3.01" pfx_z="-9.55" pitch_type="KC" play_guid="5f528c85-3549-494b-be49-8b6f08d5568a" px="0.2" pz="1.37" spin_dir="placeholder" spin_rate="placeholder" start_speed="81.4" sv_id="190814_023817" sz_bot="1.57" sz_top="3.37" tfs="023804" tfs_zulu="2019-08-14T02:38:04.000Z" type="X" type_confidence="placeholder" vx0="3.18" vy0="-118.47" vz0="-1.71" x="109.55" x0="-1.56" y="201.66" y0="50.0" z0="6.29" zone="placeholder"/><runner end="1B" event="Single" event_num="31" id="656185" start=""/></atbat><action away_team_runs="6" b="1" des="Greg Allen steals (3) 2nd base." des_es="Greg Allen steals (3) 2nd base." event="Stolen Base 2B" event_es="Stolen Base 2B" event_num="32" home_team_runs="5" o="0" pitch="2" player="656185" s="1" tfs="023942" tfs_zulu="2019-08-14T02:39:42.000Z"/><atbat away_team_runs="6" b="1" b_height="6' 2" batter="571980" des="Tyler Naquin strikes out on a foul tip." des_es="Tyler Naquin strikes out on a foul tip." end_tfs_zulu="2019-08-14T02:41:02.000Z" event="Strikeout" event_es="Strikeout" event_num="33" home_team_runs="5" num="79" o="1" p_throws="R" pitcher="519443" play_guid="8e2b5eb2-8bd9-4675-bb8e-ad18d1525b75" s="3" stand="L" start_tfs="023857" start_tfs_zulu="2019-08-14T02:38:57.000Z"><pitch ax="-1.54" ay="24.97" az="-18.03" break_angle="1.2" break_length="3.6" break_y="24.0" cc="" code="B" des="Ball" des_es="Foul Tip" end_speed="86.4" event_num="33" id="33" mt="" nasty="" pfx_x="-0.81" pfx_z="7.47" pitch_type="FF" play_guid="debd1e32-0d24-4b02-9f24-3ec24117be11" px="0.45" pz="4.87" spin_dir="placeholder" spin_rate="placeholder" start_speed="92.9" sv_id="190814_023904" sz_bot="1.53" sz_top="3.33" tfs="023900" tfs_zulu="2019-08-14T02:39:00.000Z" type="B" type_confidence="placeholder" vx0="5.46" vy0="-135.59" vz0="-1.02" x="99.75" x0="-1.47" y="107.21" y0="50.0" z0="6.49" zone="placeholder"/><pitch ax="-2.74" ay="25.51" az="-16.88" break_angle="6.0" break_length="3.6" break_y="24.0" cc="" code="C" des="Called Strike" des_es="Ball" end_speed="85.3" event_num="33" id="33" mt="" nasty="" pfx_x="-1.48" pfx_z="8.28" pitch_type="FF" play_guid="1ec93890-d8ea-48e3-b2d4-105330a33377" px="-0.11" pz="3.45" spin_dir="placeholder" spin_rate="placeholder" start_speed="92.0" sv_id="190814_023933" sz_bot="1.53" sz_top="3.28" tfs="023926" tfs_zulu="2019-08-14T02:39:26.000Z" type="S" type_confidence="placeholder" vx0="4.17" vy0="-134.2" vz0="-4.39" x="121.35" x0="-1.48" y="145.72" y0="50.0" z0="6.28" zone="placeholder"/><pitch ax="-1.2" ay="26.98" az="-14.81" break_angle="2.4" break_length="3.6" break_y="24.0" cc="" code="F" des="Foul" des_es="Greg Allen steals (3) 2nd base." end_speed="85.9" event_num="33" id="33" mt="" nasty="" pfx_x="-0.64" pfx_z="9.24" pitch_type="FF" play_guid="b0857f2e-f82b-4b6f-9c5c-79279e643502" px="-0.24" pz="3.72" spin_dir="placeholder" spin_rate="placeholder" start_speed="93.0" sv_id="190814_024025" sz_bot="1.62" sz_top="3.41" tfs="024018" tfs_zulu="2019-08-14T02:40:18.000Z" type="S" type_confidence="placeholder" vx0="3.29" vy0="-135.61" vz0="-4.23" x="126.27" x0="-1.39" y="138.34" y0="50.0" z0="6.32" zone="placeholder"/><pitch ax="-0.8" ay="25.65" az="-13.69" break_angle="1.2" break_length="3.6" break_y="24.0" cc="" code="T" des="Foul Tip" des_es="Foul" end_speed="87.7" event_num="33" id="33" mt="" nasty="" pfx_x="-0.41" pfx_z="9.49" pitch_type="FF" play_guid="8e2b5eb2-8bd9-4675-bb8e-ad18d1525b75" px="0.64" pz="3.6" spin_dir="placeholder" spin_rate="placeholder" start_speed="94.3" sv_id="190814_024059" sz_bot="1.62" sz_top="3.41" tfs="024054" tfs_zulu="2019-08-14T02:40:54.000Z" type="S" type_confidence="placeholder" vx0="5.62" vy0="-137.54" vz0="-5.04" x="92.75" x0="-1.37" y="141.64" y0="50.0" z0="6.36" zone="placeholder"/><runner end="2B" event="Stolen Base 2B" event_num="34" id="656185" start="1B"/><runner end="" event="Strikeout" event_num="35" id="571980" start=""/></atbat><atbat away_team_runs="6" b="2" b_height="5' 11" batter="596019" des="Francisco Lindor doubles (29) on a line drive to left fielder Andrew Benintendi.   Greg Allen scores." des_es="Francisco Lindor doubles (29) on a line drive to left fielder Andrew Benintendi.   Greg Allen scores." end_tfs_zulu="2019-08-14T02:43:41.000Z" event="Double" event_es="Double" event_num="36" home_team_runs="6" num="80" o="1" p_throws="R" pitcher="519443" play_guid="19c1d038-a8b0-446b-b0a9-4850cd832976" s="1" stand="L" start_tfs="024140" start_tfs_zulu="2019-08-14T02:41:40.000Z"><pitch ax="6.1" ay="19.28" az="-42.84" break_angle="8.4" break_length="13.2" break_y="24.0" cc="" code="B" des="Ball" des_es="In play, run(s)" end_speed="73.8" event_num="36" id="36" mt="" nasty="" pfx_x="4.47" pfx_z="-7.81" pitch_type="KC" play_guid="24389988-3580-4203-a2c0-5a9a67bb27ef" px="-0.91" pz="4.15" spin_dir="placeholder" spin_rate="placeholder" start_speed="79.3" sv_id="190814_024148" sz_bot="1.53" sz_top="3.2" tfs="024143" tfs_zulu="2019-08-14T02:41:43.000Z" type="B" type_confidence="placeholder" vx0="0.5" vy0="-115.51" vz0="3.79" x="151.68" x0="-1.71" y="126.59" y0="50.0" z0="6.58" zone="placeholder"/><pitch ax="-0.81" ay="25.86" az="-12.91" break_angle="1.2" break_length="3.6" break_y="24.0" cc="" code="B" des="Ball" des_es="Ball" end_speed="87.1" event_num="36" id="36" mt="" nasty="" pfx_x="-0.42" pfx_z="10.09" pitch_type="FF" play_guid="57c71afd-bf74-4121-b1a7-7c5c3596e713" px="0.65" pz="1.6" spin_dir="placeholder" spin_rate="placeholder" start_speed="93.7" sv_id="190814_024213" sz_bot="1.49" sz_top="3.26" tfs="024208" tfs_zulu="2019-08-14T02:42:08.000Z" type="B" type_confidence="placeholder" vx0="5.62" vy0="-136.33" vz0="-9.95" x="92.21" x0="-1.37" y="195.71" y0="50.0" z0="6.15" zone="placeholder"/><pitch ax="5.09" ay="23.98" az="-44.33" break_angle="7.2" break_length="13.2" break_y="24.0" cc="" code="F" des="Foul" des_es="Ball" end_speed="75.2" event_num="36" id="36" mt="" nasty="" pfx_x="3.58" pfx_z="-8.56" pitch_type="KC" play_guid="2d85de91-6a82-4450-895e-9b4d35b44bcb" px="-0.14" pz="2.33" spin_dir="placeholder" spin_rate="placeholder" start_speed="81.6" sv_id="190814_024258" sz_bot="1.55" sz_top="3.29" tfs="024253" tfs_zulu="2019-08-14T02:42:53.000Z" type="S" type_confidence="placeholder" vx0="2.16" vy0="-118.76" vz0="-0.06" x="122.43" x0="-1.53" y="175.7" y0="50.0" z0="6.41" zone="placeholder"/><pitch ax="-0.79" ay="27.08" az="-13.29" break_angle="1.2" break_length="3.6" break_y="24.0" cc="" code="E" des="In play, run(s)" des_es="Foul" end_speed="87.2" event_num="36" id="36" mt="" nasty="" pfx_x="-0.41" pfx_z="9.78" pitch_type="FF" play_guid="19c1d038-a8b0-446b-b0a9-4850cd832976" px="-0.13" pz="3.28" spin_dir="placeholder" spin_rate="placeholder" start_speed="94.1" sv_id="190814_024339" sz_bot="1.55" sz_top="3.29" tfs="024324" tfs_zulu="2019-08-14T02:43:24.000Z" type="X" type_confidence="placeholder" vx0="3.3" vy0="-137.29" vz0="-5.87" x="121.96" x0="-1.29" y="150.23" y0="50.0" z0="6.33" zone="placeholder"/><runner end="3B" event="Double" event_num="37" id="656185" start="2B"/><runner earned="T" end="score" event="Double" event_num="38" id="656185" rbi="T" score="T" start="3B"/><runner end="2B" event="Double" event_num="39" id="596019" start=""/></atbat><action away_team_runs="6" b="0" des="Mound Visit." des_es="Mound Visit." event="Game Advisory" event_es="Game Advisory" event_num="40" home_team_runs="6" o="1" pitch="1" player="640458" s="0" tfs="024358" tfs_zulu="2019-08-14T02:43:58.000Z"/><action away_team_runs="6" b="1" des="Red Sox challenged (tag play), call on the field was overturned: Francisco Lindor caught stealing 3rd base, catcher Sandy Leon to third baseman Rafael Devers." des_es="Red Sox challenged (tag play), call on the field was overturned: Francisco Lindor caught stealing 3rd base, catcher Sandy Leon to third baseman Rafael Devers." event="Caught Stealing 3B" event_es="Caught Stealing 3B" event_num="41" home_team_runs="6" o="2" pitch="2" player="596019" s="1" tfs="024626" tfs_zulu="2019-08-14T02:46:26.000Z"/><atbat away_team_runs="6" b="2" b_height="6' 2" batter="640458" des="Oscar Mercado flies out to right fielder Mookie Betts." des_es="Oscar Mercado flies out to right fielder Mookie Betts." end_tfs_zulu="2019-08-14T02:48:56.000Z" event="Flyout" event_es="Flyout" event_num="42" home_team_runs="6" num="81" o="3" p_throws="R" pitcher="519443" play_guid="78e11bb9-46d2-43db-87b1-720077fc48ab" s="2" stand="R" start_tfs="024358" start_tfs_zulu="2019-08-14T02:43:58.000Z"><pitch ax="5.3" ay="21.98" az="-45.38" break_angle="7.2" break_length="13.2" break_y="24.0" cc="" code="F" des="Foul" des_es="Mound Visit." end_speed="73.3" event_num="42" id="42" mt="" nasty="" pfx_x="3.93" pfx_z="-9.78" pitch_type="KC" play_guid="2e28ff2f-84b2-4402-bf9e-34098a43fda4" px="-0.84" pz="2.81" spin_dir="placeholder" spin_rate="placeholder" start_speed="79.3" sv_id="190814_024522" sz_bot="1.67" sz_top="3.52" tfs="024516" tfs_zulu="2019-08-14T02:45:16.000Z" type="S" type_confidence="placeholder" vx0="0.84" vy0="-115.58" vz0="1.55" x="149.03" x0="-1.71" y="162.38" y0="50.0" z0="6.5" zone="placeholder"/><pitch ax="-2.37" ay="27.06" az="-14.37" break_angle="4.8" break_length="3.6" break_y="24.0" cc="" code="B" des="Ball" des_es="Foul" end_speed="85.3" event_num="42" id="42" mt="" nasty="" pfx_x="-1.27" pfx_z="9.57" pitch_type="FF" play_guid="84ed45f3-86d5-4695-bdea-f239d2898bf6" px="0.18" pz="4.68" spin_dir="placeholder" spin_rate="placeholder" start_speed="92.5" sv_id="190814_024617" sz_bot="1.55" sz_top="3.48" tfs="024610" tfs_zulu="2019-08-14T02:46:10.000Z" type="B" type_confidence="placeholder" vx0="5.19" vy0="-134.99" vz0="-1.84" x="110.12" x0="-1.59" y="112.38" y0="50.0" z0="6.38" zone="placeholder"/><pitch ax="3.34" ay="21.72" az="-44.78" break_angle="4.8" break_length="13.2" break_y="24.0" cc="" code="B" des="Ball" des_es="Red Sox challenged (tag play), call on the field was overturned: Francisco Lindor caught stealing 3rd base, catcher Sandy Leon to third baseman Rafael Devers." end_speed="72.1" event_num="42" id="42" mt="" nasty="" pfx_x="2.55" pfx_z="-9.63" pitch_type="KC" play_guid="d0071f9b-b203-4160-8220-f257f6fcf9a9" px="-1.32" pz="3.36" spin_dir="placeholder" spin_rate="placeholder" start_speed="78.2" sv_id="190814_024804" sz_bot="1.61" sz_top="3.46" tfs="024759" tfs_zulu="2019-08-14T02:47:59.000Z" type="B" type_confidence="placeholder" vx0="-0.25" vy0="-113.92" vz0="2.64" x="167.28" x0="-1.54" y="147.95" y0="50.0" z0="6.63" zone="placeholder"/><pitch ax="0.86" ay="21.62" az="-31.97" break_angle="2.4" break_length="8.4" break_y="24.0" cc="" code="F" des="Foul" des_es="Ball" end_speed="79.9" event_num="42" id="42" mt="" nasty="" pfx_x="0.53" pfx_z="0.13" pitch_type="FC" play_guid="101f471e-8445-4cc9-b042-6810a736ee51" px="-0.1" pz="3.46" spin_dir="placeholder" spin_rate="placeholder" start_speed="85.7" sv_id="190814_024823" sz_bot="1.67" sz_top="3.52" tfs="024818" tfs_zulu="2019-08-14T02:48:18.000Z" type="S" type_confidence="placeholder" vx0="4.53" vy0="-124.97" vz0="-0.77" x="120.71" x0="-1.99" y="145.2" y0="50.0" z0="6.36" zone="placeholder"/><pitch ax="0.22" ay="25.75" az="-12.25" break_angle="4.8" break_length="2.4" break_y="24.0" cc="" code="X" des="In play, out(s)" des_es="Foul" end_speed="85.9" event_num="42" id="42" mt="" nasty="" pfx_x="0.12" pfx_z="10.68" pitch_type="FF" play_guid="78e11bb9-46d2-43db-87b1-720077fc48ab" px="0.79" pz="2.49" spin_dir="placeholder" spin_rate="placeholder" start_speed="92.6" sv_id="190814_024855" sz_bot="1.67" sz_top="3.52" tfs="024842" tfs_zulu="2019-08-14T02:48:42.000Z" type="X" type_confidence="placeholder" vx0="5.74" vy0="-134.84" vz0="-8.04" x="86.72" x0="-1.37" y="171.45" y0="50.0" z0="6.35" zone="placeholder"/><runner end="" event="Caught Stealing 3B" event_num="43" id="596019" start="2B"/><runner end="" event="Field Out" event_num="44" id="640458" start=""/></atbat>"#;
    
    
    // let inning: Inning = serde_xml_rs::from_str(inning_xml).unwrap();
    // dbg!(inning);

    // dbg!(at_bat);

    // dbg!(game_download_parse(&games[0]));


}


// Tests to run:
// 1) Make sure we can parse empty values as None, as opposed to an integer error
//          Test with this url: http://gd2.mlb.com/components/game/rok/year_2008/month_06/day_10/gid_2008_06_10_dinrok_dacrok_1/players.xml