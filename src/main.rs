#![allow(unused)]
/// baseball 
///     
/// **TODO** Add Documention to the root here

use isahc::prelude::*;
use rayon::prelude::*;
use serde::{Serialize, Deserialize, Deserializer};
use std::{error, fmt, num};
use std::collections::{HashMap};
use std::sync::{Arc, RwLock};
use std::time;
use futures::{executor, join, stream::*};

mod draft;

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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Player {
    id: u32,
    #[serde(rename="first")]
    name_first: String,
    #[serde(rename="last")]
    name_last: String,
    game_position: Option<String>,
    bat_order: Option<u8>,
    position: String,
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
    #[serde(rename="home_inning_runs", deserialize_with = "empty_string_is_none")]
    //needs to be an Option, since home team doesn't always bat.
    home_runs: Option<u32>,
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
    pitcher_throws: Option<char>,
    #[serde(rename="des")]
    plate_app_des: String,
    #[serde(rename="o")]
    outs_end: u8,
    #[serde(rename="event")]
    plate_app_result: String,
    #[serde(rename="$value")]
    pitch_or_runner: Vec<PitchOrRunner>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Inning {
    top: HalfInning,
    bottom: Option<HalfInning>,
}

#[derive(Deserialize, Serialize, Debug)]
struct HalfInning {
    #[serde(rename="$value")]
    at_bat_action: Vec<AtBatOrAction>,
}

#[derive(Deserialize, Serialize, Debug)]
enum AtBatOrAction {
    #[serde(rename="action")]
    Action (Action),
    #[serde(rename="atbat")]
    PlateAppearance (PlateAppearance),
}

#[derive(Deserialize, Serialize, Debug)]
enum PitchOrRunner {
    #[serde(rename="pitch")]
    Pitch (Pitch),
    #[serde(rename="runner")]
    Runner (Runner),
    #[serde(rename="po")]
    PickOff (PickOff),
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
    pitch: u16,
    event: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PickOff {
    des: String,
}

// The fields *could* be integers, but generally speaking, we need them to load into urls, so better to just
// leave them as strings
#[derive(Deserialize, Serialize, Debug)]
struct GameDateLevel {
    year: String,
    month: String,
    day: String,
    level: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct PlateAppearanceState {
    balls_start: u8,
    balls_end: u8,
    strikes_start: u8,
    strikes_end: u8,
    outs_start: u8,
    outs_end: u8,
    runs_scored: u8,
    base_state_start: u8,
    base_state_end: u8,
    re_24_start: f32,
    re_24_end: f32,
    re_288_start: f32,
    re_288_end: f32,
    batter_responsible: bool,
}

impl Default for PlateAppearanceState {
    fn default() -> Self {
        PlateAppearanceState {
            balls_start: 0, balls_end: 0,
            strikes_start: 0, strikes_end: 0,
            outs_start: 0, outs_end: 0,
            runs_scored: 0,
            base_state_start: 0, base_state_end: 0,
            re_24_start: 0.0, re_24_end: 0.0,
            re_288_start: 0.0, re_288_end: 0.0,
            batter_responsible: true,
        }
    }
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
    vx0: Option<f32>,
    vy0: Option<f32>,
    vz0: Option<f32>,
    x0: Option<f32>,
    y0: Option<f32>,
    z0: Option<f32>,

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
    pitch_type: Option<String>,

    #[serde(skip_deserializing)]
    plate_appearance_state: PlateAppearanceState,
    #[serde(skip_deserializing)]
    plate_appearance_pitch_num: usize,
    #[serde(skip_deserializing)]
    plate_appearance_num: u16,
    #[serde(skip_deserializing)]
    batter: u32,
    #[serde(skip_deserializing)]
    pitcher: u32,
    #[serde(skip_deserializing)]
    batter_stands: char,
    #[serde(skip_deserializing)]
    pitcher_throws: Option<char>,
    #[serde(skip_deserializing)]
    plate_appearance_des: String,
    #[serde(skip_deserializing)]
    plate_appearance_result: String,

    #[serde(skip_deserializing)]
    batter_bat_order: Option<u8>,
    #[serde(skip_deserializing)]
    batter_game_position: Option<String>,
    #[serde(skip_deserializing)]
    pitcher_sp_rp: SPRP,

    #[serde(skip_deserializing)]
    swing: u8,

    #[serde(skip_deserializing)]
    in_play_pixels_x: Option<f32>,
    #[serde(skip_deserializing)]
    in_play_pixels_y: Option<f32>,
    #[serde(skip_deserializing)]
    in_play_result: Option<String>,
    #[serde(skip_deserializing)]
    in_play_trajectory: Option<String>,
    
}

#[derive(Deserialize, Serialize, Debug)]
enum SPRP {
    SP,
    RP,
}

impl Default for SPRP {
    fn default() -> Self {
        SPRP::RP
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Runner {
    code: Option<char>,
    id: u32,
    start: String,
    end: String,
    event: String,
    score: Option<char>,
}


#[derive(Deserialize, Serialize, Debug)]
struct GameData {
    linescore_data: LineScoreData,
    boxscore_data: BoxScoreData,
    game_umps: GameUmpires,
    pitches: Vec<Pitch>,
}

impl GameData {
    fn new(
        boxscore_data: BoxScoreData, 
        linescore_data: LineScoreData, 
        game_umps: GameUmpires, 
        pitches: Vec<Pitch>,
        ) -> Self {
        GameData {
            boxscore_data,
            linescore_data,
            game_umps,
            pitches,
        }
    }
}

struct PlayerWeightCache {
    id: u32,
    year: String,
    level: String,
    weight: u16,
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
    QuickXMLParse(quick_xml::de::DeError),
    JSONParse(serde_json::Error),
    ParseIntError(num::ParseIntError),
    Weather(WeatherMissingError),
    GameDayLinks(GameDayMissingLinksError),        
}

impl fmt::Display for GameDayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameDayError::Request(ref err) => write!(f, "Network Error: {}", err),
            GameDayError::XMLParse(ref err) => write!(f, "XML Parse Error: {}", err),
            GameDayError::QuickXMLParse(ref err) => write!(f, "Quick XML Parse Error: {}", err),
            GameDayError::JSONParse(ref err) => write!(f, "JSON Parse Error: {}", err),
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
            GameDayError::QuickXMLParse(ref err) => err.description(),
            GameDayError::JSONParse(ref err) => err.description(),
            GameDayError::Weather(ref err) => err.description(),
            GameDayError::ParseIntError(ref err) => err.description(),
            GameDayError::GameDayLinks(ref err) => err.description(),
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            GameDayError::Request(ref err) => Some(err),
            GameDayError::XMLParse(ref err) => Some(err),
            GameDayError::QuickXMLParse(ref err) => Some(err),
            GameDayError::JSONParse(ref err) => Some(err),
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

impl From<quick_xml::de::DeError> for GameDayError {
    fn from(err: quick_xml::de::DeError) -> GameDayError {
        GameDayError::QuickXMLParse(err)
    }
}

impl From<serde_json::Error> for GameDayError {
    fn from(err: serde_json::Error) -> GameDayError {
        GameDayError::JSONParse(err)
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

/// Converts the Umpires struct into the GameUmpires struct
/// We need to pivot the umpires into defined fields, to flatten it out for our game metadata
/// The From impl automatically provides an Into, which allows for a very readable .into()
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

    let base_url = "http://gd2.mlb.com/components/game/";

    [base_url, "mlb", "/year_", year, "/month_", month, "/day_", day].concat()       
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
        .collect::<Vec<String>>()
}


fn inning_parse (inning_data: Vec<Response<Body>>) -> Result<Vec<Inning>, GameDayError> {

    inning_data
        .into_iter()
        .map(|mut resp| Ok(quick_xml::de::from_str(&resp.text()?)?))
        .collect()
}


fn inning_xml_parse (http_client: &HttpClient, inning_links: Vec<String>) -> Result<Vec<Inning>, GameDayError> {

    inning_links.iter()
        .map(|link| 
            {
                let inning_xml = http_client.get(link)?.text()?;
                Ok(quick_xml::de::from_str(&inning_xml)?)
            })
        .collect()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PlayerBio {
    id: u32,
    birth_date: String,
    birth_city: String,
    birth_country: String,
    birth_state_province: Option<String>,
    height: String,
    #[serde(skip_deserializing)]
    height_in: u32,
    weight: u16,
    weight_v2: Option<u16>,
    full_name: String,
    draft_year: Option<u16>,
    mlb_debut_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerXml {
    weight: u16,
}

// We have game specific data as well as unchanging bio info
#[derive(Serialize, Debug)]
struct PlayerBioGame {
    bio: PlayerBio,
    game: Player,
}

//Decides wether to look in the batter or pitcher folder for the player xml


fn get_player_data
    (http_client: &HttpClient,
     url: &str,
     player_bio_cache: &Arc<RwLock<HashMap<u32, PlayerBio>>>,
     player_weight_cache: &Arc<RwLock<HashMap<(u32, String, String), PlayerWeightCache>>>,
     player: Player,
    ) 
    -> Result<PlayerBioGame, GameDayError> {


    // We first check to see if we have any bio data for the player. This will effectively cache the
    // player's bio once we've grabbed it once. TODO - replace the .unwrap() with proper error handling that
    // will capture if the RwLock was poisoned. We may also want to keep the .unwrap() and just panic.

    let mut player_bio = match player_bio_cache.read().unwrap().get(&player.id) {
        Some(player_bio_cache) => player_bio_cache.clone(),
        None => {
            let url_json =  String::from("http://statsapi.mlb.com/api/v1/people/") + &player.id.to_string();

            // We pull the primary data from the mlb API as it is more reliable. Eventually, we'll want to avoid doing this once
            // per game and store a local version that will be checked first. For the first iteration, this simplifies the
            // implementation considerably.
            
            let raw_json_data = &http_client.get(&url_json)?.text()?;

            // This is an ugly way to just get at the "people" field, but it has less indirection so doing it this way for now        
            let json_data = raw_json_data
                        .split (r#""people" : [ "#)
                        .nth(1).unwrap_or("")
                        .trim_end_matches("}")
                        .trim()
                        .trim_end_matches("]")
                        .trim()
                        ;

            let mut player_bio: PlayerBio = serde_json::from_str(&json_data)?;

            // We want to have our height measurable as an integral number suitable for input into models
            // First, we split by the ' giving us a small vector of length 1 or 2. We enumerate this vector
            // and then multiply the first element by 12^1 and the second element by 12^0. If we have a third
            // element for some reason, this should be an error, but we're ignoring that case for now.
            player_bio.height_in = player_bio.height
                            .split('\'')
                            .filter_map(|h| h.trim().parse().ok())
                            .enumerate()
                            .map(|(n, h): (usize, u32)| h * 12u32.pow(1-n as u32))
                            .sum();
                            
            player_bio
        }
    };

    

    // This is a really ugly way of getting the year, month. TODO - fix this up, ideally by passing this infrom the initial
    // function that starts the scraping

    let (year, level) = {
        let year_month_level: Vec<&str> = url.split("/").collect();
        // dbg!(&year_month_level);
        (   
            year_month_level[6].split("_").nth(1).unwrap().to_string(),
            year_month_level[5].to_string(),
        )
    };

    let player_weight_v2 = match player_weight_cache.read().unwrap().get(&(player.id, year, level)) {

        Some (player_weight_cache) => Some(player_weight_cache.weight),
        None => {

            // The GameDay xml has a weight field which holds some potentially interesting historical data. 
            // The above API only shows the current player's weight, not how much he weighed 10 years ago.
            // Seeing a player's weight gain curve could be interesting. These data are sometimes missing, so at
            // some point we'll need to iterate through all those records and 
            // 
            // We don't want to propogate an error here, since we don't want this to fail when the file isn't available.player_json
            // We'll just return none if there's an issue.

            let url_xml = 
                if player.position == "P" {
                    String::from(url) + "pitchers/" + &player.id.to_string() + ".xml"
                }
                else {
                    String::from(url) + "batters/" + &player.id.to_string() + ".xml"
                };

            let player_xml_text = match http_client.get(&url_xml) {
                Ok(mut resp) => resp.text().unwrap(),
                _ => "".to_string(),
            };

            let player_xml: Result<PlayerXml, quick_xml::de::DeError> = quick_xml::de::from_str(&player_xml_text);

            match (player_xml) {
                Ok(player) => Some(player.weight),
                Err(e) => None,
            }
        }
    };

    player_bio.weight_v2 = player_weight_v2;

    Ok(PlayerBioGame {
        bio: player_bio,
        game: player,
    })


}

fn process_base_state (base_state_start: u8, runner: Runner) -> (u8, u8, u8) {

    // Base state converts the binary representation of the base runners into a number between 0-7.
    // A runner on first is worth 2^0, second base, 2^1 and third base 2^2. We get each runner separately, so update the base state at each step and add up the number of runs scored.


    let base_state_end = {
        base_state_start 
        - {
            if runner.start == "1B".to_string() {1}
            else if runner.start == "2B".to_string() {2}
            else if runner.start == "3B".to_string() {4}
            else {0}
        }
        + {
            if runner.end == "1B".to_string() {1}
            else if runner.end == "2B".to_string() {2}
            else if runner.end == "3B".to_string() {4}
            else {0}
        }
    };
    
    let runs = if runner.score == Some('T') {1} else {0};
    let outs = if runner.end == "".to_string() && runner.score != Some('T') {1} else {0};

    (base_state_end, runs, outs)
}

fn process_plate_appearance (plate_appearance: PlateAppearance, base_state: u8, outs: u8) 
-> (Vec<Pitch>, u8) {
    let mut pitches: Vec<Pitch> = Vec::with_capacity(plate_appearance.pitch_or_runner.len());
    
    let mut pitch_num = 0;
    let mut balls: u8 = 0;
    let mut strikes: u8 = 0;

    let mut base_state = base_state;
    let mut outs = outs;
    // let mut plate_appearance_state = PlateAppearanceState::default();

    // We loop through all the pitch or runner events. If we find a pitch, we push it to our vector.
    // If we find a runner field, we update our base state
    for pitch_or_runner in plate_appearance.pitch_or_runner {

        match pitch_or_runner {
            PitchOrRunner::PickOff(pickoff) => {

            },
            PitchOrRunner::Runner(runner) => {
                // The Runner data will update values relative to the previous pitch
                let event = runner.event.to_lowercase();

                
                let (end, runs, outs) = process_base_state(base_state, runner);
                if pitch_num > 0 {
                    pitches[pitch_num-1].plate_appearance_state.base_state_end = end;
                    pitches[pitch_num-1].plate_appearance_state.outs_end = end;
                    pitches[pitch_num-1].plate_appearance_state.runs_scored += runs;
                    // If there is a stolen base attempt, or pick off attempt we want to make sure we don't credit/debit the batter for the changed base/out state
                    if event.contains("stolen") 
                        || event.contains("stealing") 
                        || event.contains("pickoff") 
                        || event.contains("picked off")
                        {pitches[pitch_num-1].plate_appearance_state.batter_responsible = false}
                    else {pitches[pitch_num-1].plate_appearance_state.batter_responsible = true};
                }

                base_state = end;        

                //TODO Add a check here to make sure our runner state is consistent


            },
            PitchOrRunner::Pitch(pitch) => {
                let mut pitch = pitch;
                pitch.plate_appearance_state.base_state_start = base_state;
                pitch.plate_appearance_state.base_state_end = base_state;
                pitch.plate_appearance_state.balls_start = balls;
                pitch.plate_appearance_state.strikes_start = strikes;
                pitch.plate_appearance_state.outs_start = outs;

                match pitch.pitch_result {
                    'B' => {balls += 1}
                    'S' => {
                        if pitch.des != "Foul" &&
                            pitch.des != "Foul (Runner Going)" &&
                            pitch.des != "Foul Pitchout" {
                                strikes +=1;
                            }
                        }
                    _ => {}
                }
                pitch.plate_appearance_state.balls_end = balls;
                pitch.plate_appearance_state.strikes_end = strikes;
                pitch.batter = plate_appearance.batter;
                pitch.batter_stands = plate_appearance.batter_stands;
                pitch.pitcher = plate_appearance.pitcher;
                pitch.pitcher_throws = plate_appearance.pitcher_throws;
                pitch.plate_appearance_num = plate_appearance.at_bat_num;
                pitch.plate_appearance_des = plate_appearance.plate_app_des.clone();
                pitch.plate_appearance_result = plate_appearance.plate_app_result.clone();


                //Determine if there was a swing or take

                if pitch.des.starts_with("In play")
                    || pitch.des.starts_with("Foul")
                    || pitch.des.starts_with("Swinging")
                    || pitch.des.contains("Bunt")
                    {pitch.swing = 1}
                else {pitch.swing = 0};

                pitch_num +=1;
                pitch.plate_appearance_pitch_num = pitch_num;
                pitches.push(pitch);



            }
        }

    }

    if pitch_num > 0 {
        // This should be authomatically calculated by the runners, however in the chance that the data are corrupt 
        // we force the last pitch of at_bat to reflect the plate appearance outs_end value
        pitches[pitch_num-1].plate_appearance_state.outs_end = plate_appearance.outs_end;

        // There is a bug in the data where if there is a pickoff attempt early in count, a base hit runner event
        // could be logged in as a "pickoff attempt". If the last pitch of the atbat was put in play, we'll override the
        // responsibility here
        if pitches[pitch_num-1].pitch_result == 'X' {
            pitches[pitch_num-1].plate_appearance_state.batter_responsible = true
        }
    }

    (pitches, base_state)

}


//for the first iteration, we'll ignore all actions
fn process_half_inning (half_inning: HalfInning) -> Vec<Pitch> {
    //We're going to pre-size each half inning vec for 30 pitches, which should be good for most innings and minimize re-sizing
    let mut pitches: Vec<Pitch> = Vec::with_capacity(30);
    let mut base_state: u8 = 0;
    let mut outs: u8 = 0;
    
    for at_bat_or_action in half_inning.at_bat_action {

        match at_bat_or_action {
            AtBatOrAction::Action (action) => {},
            AtBatOrAction::PlateAppearance (plate_appearance) => {
                let outs_end = plate_appearance.outs_end;
                let (pitches_from_pa, base_state_update) = process_plate_appearance(plate_appearance, base_state, outs);
                pitches.extend(pitches_from_pa);
                base_state = base_state_update;
                outs = outs_end;
            },
        };
    }

    pitches
}


//TODO - Refactor this code to capture potential errors in the data and switch this to a Result type
fn process_inning_data (inning_data: Vec<Inning>, players: HashMap<u32, PlayerBioGame>) -> Vec<Pitch> {

    let mut pitches: Vec<Pitch> = Vec::new();


    //loop through each inning
    for inning in inning_data {

        let top = inning.top;
        let bottom = inning.bottom;

        pitches.extend(process_half_inning(top));
        if let Some(bottom) = bottom {pitches.extend(process_half_inning(bottom))};

    }


    pitches

}


async fn inning_xml_download (http_client: &HttpClient, inning_links: Vec<String>)
-> Result<Vec<Response<Body>>, isahc::Error> {
        futures::future::try_join_all(
            inning_links.into_iter()
            .map(|link| http_client.get_async(&link))
        ).await
}

type IsahcResponse = Result<Response<Body>, isahc::Error>;
async fn get_async(linescore: &str, boxscore: &str, players: &str) 
-> (IsahcResponse, IsahcResponse, IsahcResponse) {

    join!(   
        isahc::get_async(linescore),
        isahc::get_async(boxscore),
        isahc::get_async(players),
    )
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
/// 

fn game_download_parse 
    (player_bio_cache: &Arc<RwLock<HashMap<u32, PlayerBio>>>,
     player_weight_cache: &Arc<RwLock<HashMap<(u32, String, String), PlayerWeightCache>>>,
     url: &str) 
    -> Result <GameData, GameDayError> {

    let http_client = HttpClient::new().unwrap();

    let linescore_url = format!("{}linescore.xml", url);
    let boxscore_url = format!("{}boxscore.xml", url);
    let players_url = format!("{}players.xml", url);

       
    // dbg!(&linescore_url);
    
    // let linescore_xml = http_client.get(&linescore_url)?.text()?.replace('&', "&amp;");
    // let boxscore_xml = http_client.get(&boxscore_url)?.text()?;
    // let players_xml = http_client.get(&players_url)?.text()?;

    let (l_xml, b_xml, p_xml) = executor::block_on(get_async(&linescore_url, &boxscore_url, &players_url));

    let linescore_xml = l_xml?.text()?.replace('&', "&amp;");
    let boxscore_xml =  b_xml?.text()?;
    let players_xml =  p_xml?.text()?;   

    let linescore_data: LineScoreData =  quick_xml::de::from_str(&linescore_xml)?;

    // let linescore_data_qml: LineScoreData = quick_xml::de::from_str(&linescore_xml).unwrap();
    

    // dbg!(&linescore_data);

    let inning_links = create_inning_links(url, &linescore_data.innings);
    let inning_hit_link = url.to_string() + "inning/inning_hit.xml";

    let inning_download = executor::block_on(inning_xml_download(&http_client, inning_links.clone()))?;
    let inning_data = inning_parse(inning_download)?;
    
    // let inning_data = inning_xml_parse(&http_client, inning_links)?;
    

    // dbg!(&inning_data[0]);

    
    let items = split_boxscore_xml(&boxscore_xml)?;

    let (weather_temp, weather_condition) = parse_weather(items[0])?;
    let (wind_speed, wind_direction) = parse_weather(items[1])?;
    let attendance: Option<u32> = if items.len() > 2 {parse_attendance(items[2])?} else {None};

    // let players_v2: Result<Game, quick_xml::de::DeError> = quick_xml::de::from_str(&players_xml);
    // dbg!(players_v2);

    let player_data: Game = quick_xml::de::from_str(&players_xml)?;


    let players: HashMap <u32, PlayerBioGame> = player_data.teams.par_iter()
                        .map(|team| team.players.clone())
                        .flatten()
                        .filter_map(|player| get_player_data(&http_client, url, player_bio_cache, player_weight_cache, player).ok())
                        .map(|player| (player.game.id, player))
                        .collect();

    let (year, level) = {
        let year_month_level: Vec<&str> = url.split("/").collect();
        (   
            year_month_level[6].split("_").nth(1).unwrap().to_string(),
            year_month_level[5].to_string(),
        )
    };  

    for (id, player) in &players {
        if !player_bio_cache.read().unwrap().contains_key(&id) {
            player_bio_cache.write().unwrap().insert(*id, player.bio.clone());
        };
        let key = (*id, year.clone(), level.clone());
        if !player_weight_cache.read().unwrap().contains_key(&key) && player.bio.weight_v2.is_some() {
            player_weight_cache.write().unwrap().insert(key, PlayerWeightCache{
                id: *id,
                year: year.clone(),
                level: level.clone(),
                weight: player.bio.weight_v2.unwrap(),
            });
        };
    };                        
   
    let game_umps: GameUmpires = player_data.umpires.into();
    
    let boxscore_data = BoxScoreData {weather_temp, weather_condition, wind_speed, wind_direction, attendance};

    let pitches = process_inning_data(inning_data, players);

    let game_data = GameData::new(boxscore_data, linescore_data, game_umps, pitches);

    Ok(game_data)
}

fn main () {

    let days = ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10",
                "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
                "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
                "31",];

    let months = ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12"];

    let years = ["2008", "2009", "2010", "2011", "2012", "2013", "2014", "2015", "2016", "2017", "2018", "2019", "2020"];

    let mut game_days: Vec<String> = Vec::with_capacity(4_000);
        
    println! ("Collecting Game Links...");

    for month in &months {
        for day in &days {
            let url = game_day_url("mlb", "2008", month, day);
            game_days.push(url);
            // let url = game_day_url("mlb", "2009", month, day);
            // game_days.push(url);
        }
    };

    let games: Vec<String> = game_days.par_iter()
                .map(|url| game_day_links(&url).unwrap_or(Vec::new()))
                // .skip(100)
                // .take(1)
                .flatten()
                .collect();
    
    let start_time = time::Instant::now();
   
    println! ("Starting game processing...");

    // dbg!(&games[0]);

    // Create the main player bio cache structure. We'll send it to each thread through an Arc<RwLock<>>
    // which will induce a small cost. We'll have two separate RwLocks to use, but since we're loading both
    // in the same function, we ensure that the locks will always be acquired in the same order, which will
    // prevent deadlocks (based on my limited understanding of deadlocks).
    let player_bio_cache: Arc<RwLock<HashMap<u32, PlayerBio>>> = Arc::new(RwLock::new (HashMap::new()));

    // We want to store one player entry per Month, Year and Level of Play. Once per player-year is probably sufficent, but
    // we're doing a little over-kill here. This likely will entail some measure of performance cost.
    let player_weight_cache: Arc<RwLock<HashMap<(u32, String, String), PlayerWeightCache>>> = Arc::new(RwLock::new (HashMap::new()));

    let game_data: Vec<_> = games.par_iter()
                // .skip(1200)
                // .take(1)
                .map(|game| game_download_parse(&player_bio_cache, &player_weight_cache, game))
                .collect(); 

    // dbg!(&game_data[0]);

    // for pitch in &game_data[0].as_ref().unwrap().pitches {
    //     if pitch.az == Some(-11.462) || pitch.az == Some(-40.059) {
    //         dbg!(pitch);
    //     }
    // }

    let time_elapsed = start_time.elapsed().as_millis() as f32;
    
    println! ("Processed {} games in {} seconds", game_data.len(), time_elapsed / 1000.0);

}


// Tests to run:
// 1) Make sure we can parse empty values as None, as opposed to an integer error
//          Test with this url: http://gd2.mlb.com/components/game/rok/year_2008/month_06/day_10/gid_2008_06_10_dinrok_dacrok_1/players.xml