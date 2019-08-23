    #![allow(unused)]
/// baseball 
///     
/// **TODO** Add Documention to the root here

use reqwest;
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
    Request(reqwest::Error),
    XMLParse(serde_xml_rs::Error),
    ParseIntError(num::ParseIntError),
    Weather(WeatherMissingError),
    GameDayLinks(GameDayMissingLinksError),        
}

impl fmt::Display for GameDayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameDayError::Request(ref err) => write!(f, "Reqwest Error: {}", err),
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


impl From<reqwest::Error> for GameDayError {
    fn from(err: reqwest::Error) -> GameDayError {
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

    let response = reqwest::get(url)?.text()?;

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


fn players_parse (url: &str) -> (Option<Game>) {
    
    let resp = reqwest::get(url);

    if resp.is_ok() {
        match resp.unwrap().text() {
            Ok (xml) => Some(serde_xml_rs::from_str(&xml.replace('&', "&amp;")).unwrap()),
            Err (_) => None,
        }
    }
    else {
        None
    }
    
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

    let linescore_xml = reqwest::get(&linescore_url)?.text()?.replace('&', "&amp;");
    let linescore_data: LineScoreData = serde_xml_rs::from_str(&linescore_xml)?;

    let inning_links = create_inning_links(url, &linescore_data.innings);
    // dbg!(inning_links);

    let boxscore_xml = reqwest::get(&boxscore_url)?.text()?;
    
    let items = split_boxscore_xml(&boxscore_xml)?;

    let (weather_temp, weather_condition) = parse_weather(items[0])?;
    let (wind_speed, wind_direction) = parse_weather(items[1])?;
    let attendance: Option<u32> = if items.len() > 2 {parse_attendance(items[2])?} else {None};

    let players_xml = reqwest::get(&players_url)?.text()?;
    let player_data: Game = serde_xml_rs::from_str(&players_xml)?;

    let game_umps: GameUmpires = player_data.umpires.into();
    
    let boxscore_data = BoxScoreData {weather_temp, weather_condition, wind_speed, wind_direction, attendance};

    let game_data = GameData::new(boxscore_data, linescore_data, game_umps);

    Ok(game_data)
}

fn linescore_parse (url: &str) -> Option<LineScoreData> {
    
    let resp = reqwest::get(url);

    if resp.is_ok() {
        match resp.unwrap().text() {
            Ok (xml) => Some(serde_xml_rs::from_str(&xml.replace('&', "&amp;")).unwrap()),
            Err (_) => None,
        }
    }
    else {
        None
    }
}

fn boxscore_parse (url: &str) -> Option<BoxScoreData> {

    let resp = reqwest::get(url);

    if resp.is_ok() {
        let xml_data = resp.unwrap().text().unwrap_or("".to_string());
        let items: Vec<&str> = xml_data
                .split("<b>")
                .filter(|item|item.starts_with("Weather") || item.starts_with("Wind") || item.starts_with("Att") )
                .map(|item| item.split(":").nth(1).unwrap().trim())
                .collect();
               
        let weather_temp: u8 = items[0]
                .split(" ").nth(0).unwrap()
                .parse().unwrap();
        let weather_condition = items[0]
                .split(",").nth(1).unwrap()
                .split("<").nth(0).unwrap()
                .trim_end_matches(".").trim().to_string();
        
        let wind_speed:u8 = items[1]
                .split(" ").nth(0).unwrap()
                .parse().unwrap();
        let wind_direction = items[1]
                .split(",").nth(1).unwrap()
                .split("<").nth(0).unwrap()
                .trim_end_matches(".").trim().to_string();

        let attendance: Option <u32> =
            if items.len() > 2 {
                let att_temp = items[2]
                    .replace(":", "")
                    .replace(",", "")
                    .replace(".", "")
                    .split("<").nth(0).unwrap()
                    .trim().to_string();
                Some (att_temp.parse().unwrap())
            }
            else {
                None
            };

        Some (
            BoxScoreData {
                weather_temp,
                weather_condition,
                wind_speed,
                wind_direction,
                attendance,
            }
        )
    }
    else {
        None
    }
}

fn main () {

    let url = game_day_url("mlb", "2008", "06", "10");
    let games = game_day_links(&url).unwrap();

    // game_download_parse(&games[0]);

    let at_bat_xml = r#"<atbat num="77" b="0" s="0" o="1" batter="544881" stand="R" b_height="6-0" pitcher="501640" p_throws="R" des="Donell Linares doubles (4) on a line drive to left fielder Guillermo Pimentel.    Erick Epifano to 3rd.  " event="Double"><pitch des="In play, no out" id="446" type="X" x="97.85" y="120.88" on_1b="542573"/><runner id="542573" start="1B" end="3B" event="Double"/><runner id="544881" start="" end="2B" event="Double"/></atbat>"#;

    let at_bat: PlateAppearance = serde_xml_rs::from_str(at_bat_xml).unwrap();

    dbg!(at_bat);

    // dbg!(game_download_parse(&games[0]));


}


// Tests to run:
// 1) Make sure we can parse empty values as None, as opposed to an integer error
//          Test with this url: http://gd2.mlb.com/components/game/rok/year_2008/month_06/day_10/gid_2008_06_10_dinrok_dacrok_1/players.xml