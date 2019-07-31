#![allow(unused)]
use reqwest;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use csv::Writer;
use regex::Regex;

/// Step 1: 
///     For any given day and level, scan the xml file for that day and return all gameday links.
///     We can then loop through all possible days and years to get all the gameday files
///     To do this, we'll need to create a constant that allows us to loop through a range of levels

// pub mod baseball;

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

///Testing a small change

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
    id: String,
}

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

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
struct GameUmpires {
    ump_HP_id: u32,
    ump_1B_id: u32,
    ump_2B_id: u32,
    ump_3B_id: u32,
    ump_LF_id: Option<u32>,
    ump_RF_id: Option<u32>,
    ump_HP_name: String,
    ump_1B_name: String,
    ump_2B_name: String,
    ump_3B_name: String,
    ump_LF_name: Option<String>,
    ump_RF_name: Option<String>,
}


#[derive(Deserialize, Serialize, Debug)]
struct LineScoreMetaData {
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
}

#[derive(Deserialize, Serialize, Debug)]
struct BoxScoreData {
    weather_temp: u8,
    weather_condition: String,
    wind_speed: u8,
    wind_direction: String,
    attendance: Option <u32>,
}

fn game_day_url (level: &str, year: &str, month: &str, day: &str) -> String {

    String::from("http://gd2.mlb.com/components/game/") + level 
                    + "/year_" + year 
                    + "/month_" + month 
                    + "/day_" + day
}

fn game_day_links (url: &str) -> Vec<String> {

    let resp = reqwest::get(url);

    if resp.is_ok() {
        let links = resp.unwrap().text().unwrap_or("".to_string());
        links.split("<li>")
            .filter(|line| line.contains("gid_"))
            .map(|line| url.to_string().clone() + "/" 
                + line.split(|c| c == '>'|| c == '<').nth(2).unwrap().trim()
            )
        .collect::<Vec<String>>()
    }
    else {
        vec![]
    }
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


fn linescore_parse (url: &str) -> Option<LineScoreMetaData> {
    
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


// #[derive(Debug, Deserialize)]
// enum ParseError {
//     #[serde(skip_deserializing)]
//     ReqwestError(reqwest::Error),
//     #[serde(skip_deserializing)]
//     SerdeError(serde_xml_rs::Error),
// }

// impl From<reqwest::Error> for ParseError {
//     fn from(err: reqwest::Error) -> ParseError {
//         ParseError::ReqwestError(err)
//     }
// }

// impl From<serde_xml_rs::Error> for ParseError {
//     fn from(err: serde_xml_rs::Error) -> ParseError {
//         ParseError::SerdeError(err)
//     }
// }

// fn get_xml (url: &str) -> Result<String, reqwest::Error> {
//     reqwest::get(url)?.text()?
    
// }


// fn linescore_parse_2 (url: &str) -> Result <LineScoreMetaData, serde_xml_rs::Error> {

//     match reqwest::get(url).unwrap().text().unwrap().replace('&', "&amp;") {
//         Ok(xml) => serde_xml_rs::from_str(&xml),
//         Err(_)
//     }
//     // dbg!(&xml);
//     // let tmp:LineScoreMetaData = serde_xml_rs::from_str(&xml).unwrap();
//     // dbg!(tmp);
    
// }


fn main () {

    let url = game_day_url("mlb", "2008", "06", "10");
    let games = game_day_links(&url);

    let players = games.par_iter()
                    .map(|game| game.to_string() + "players.xml")
                    .filter_map(|url| players_parse(&url))
                    .collect::<Vec<_>>();

    // let linescores = games.par_iter()
    //                 .map(|game| game.to_string() + "linescore.xml")
    //                 .filter_map(|url| linescore_parse(&url))
    //                 .collect::<Vec<_>>();

    // dbg!(linescores);

    // let boxscores = games.par_iter()
    //                 .map(|game| game.to_string() + "boxscore.xml")
    //                 .filter_map(|url| boxscore_parse(&url))
    //                 .collect::<Vec<_>>();


    // let file_name = "boxscore.csv";
    // let mut csv_writer = csv::Writer::from_path(file_name).unwrap();

    // for record in boxscores.iter() {
    //     csv_writer.serialize(record).unwrap();
    // }

    // dbg!(boxscores);


    // dbg!(&games);
    // for game in games {
    //     let game_url = game + "linescore.xml";
    //     let linescore = linescore_parse(&game_url);
    //     dbg!(linescore);
    //     break;
    // }

    // let test_string = r#"<player id="134181" first="Adrian" last="Beltre" num="29" boxname="Beltre" rl="R" bats="R" position="3B" current_position="DH" status="A" team_abbrev="TEX" team_id="140" parent_team_abbrev="TEX" parent_team_id="140" bat_order="4" game_position="DH" avg=".288" hr="2" rbi="15"/>"#;
    
    // let test_string_2 = r#"
    //                     <umpires><umpire position="home" name="Alfonso Marquez" id="427315" first="Alfonso" last="Marquez"/><umpire position="first" name="D.J. Reyburn" id="482666" first="D.J." last="Reyburn"/><umpire position="second" name="Ryan Blakney" id="503502" first="Ryan" last="Blakney"/><umpire position="third" name="Sam Holbrook" id="427235" first="Sam" last="Holbrook"/></umpires>
    //                     "#;

    

    // let players_xml = r#"<game venue="Yankee Stadium" date="October 9, 2018"><team type="away" id="BOS" name="Boston Red Sox"><player id="435079" first="Ian" last="Kinsler" num="5" boxname="Kinsler" rl="R" bats="R" position="2B" current_position="2B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="6" game_position="2B" avg=".000" hr="0" rbi="0"/><player id="456030" first="Dustin" last="Pedroia" num="15" boxname="Pedroia" rl="R" bats="R" position="2B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="456034" first="David" last="Price" num="24" boxname="Price" rl="L" bats="L" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="456488" first="Eduardo" last="Nunez" num="36" boxname="Nunez, E" rl="R" bats="R" position="2B" current_position="3B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="7" game_position="3B" avg=".000" hr="0" rbi="0"/><player id="456665" first="Steve" last="Pearce" num="25" boxname="Pearce" rl="R" bats="R" position="1B" current_position="1B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="3" game_position="1B" avg=".000" hr="0" rbi="0"/><player id="502110" first="J.D." last="Martinez" num="28" boxname="Martinez, J" rl="R" bats="R" position="LF" current_position="DH" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="4" game_position="DH" avg=".000" hr="0" rbi="0"/><player id="506702" first="Sandy" last="Leon" num="3" boxname="Leon" rl="R" bats="S" position="C" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="518489" first="Ryan" last="Brasier" num="70" boxname="Brasier" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="518886" first="Craig" last="Kimbrel" num="46" boxname="Kimbrel" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="519048" first="Mitch" last="Moreland" num="18" boxname="Moreland" rl="L" bats="L" position="1B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="519144" first="Rick" last="Porcello" num="22" boxname="Porcello" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="0" game_position="P" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="519242" first="Chris" last="Sale" num="41" boxname="Sale" rl="L" bats="L" position="P" current_position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="519443" first="Brandon" last="Workman" num="44" boxname="Workman" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="523260" first="Joe" last="Kelly" num="56" boxname="Kelly" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="543135" first="Nathan" last="Eovaldi" num="17" boxname="Eovaldi" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="543877" first="Christian" last="Vazquez" num="7" boxname="Vazquez" rl="R" bats="R" position="C" current_position="C" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="9" game_position="C" avg=".000" hr="0" rbi="0"/><player id="545348" first="Austin" last="Maddox" num="62" boxname="Maddox" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="571788" first="Brock" last="Holt" num="12" boxname="Holt" rl="R" bats="L" position="2B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="592390" first="Heath" last="Hembree" num="37" boxname="Hembree" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="593428" first="Xander" last="Bogaerts" num="2" boxname="Bogaerts" rl="R" bats="R" position="SS" current_position="SS" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="5" game_position="SS" avg=".000" hr="0" rbi="0"/><player id="593523" first="Marco" last="Hernandez" num="40" boxname="Hernandez, M" rl="R" bats="L" position="3B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="593958" first="Eduardo" last="Rodriguez" num="57" boxname="Rodriguez, E" rl="L" bats="L" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="596119" first="Blake" last="Swihart" num="23" boxname="Swihart" rl="R" bats="S" position="C" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><player id="598264" first="Matt" last="Barnes" num="32" boxname="Barnes, M" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="598265" first="Jackie" last="Bradley" num="19" boxname="Bradley Jr." rl="R" bats="L" position="CF" current_position="CF" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="8" game_position="CF" avg=".000" hr="0" rbi="0"/><player id="605141" first="Mookie" last="Betts" num="50" boxname="Betts" rl="R" bats="R" position="RF" current_position="RF" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="1" game_position="RF" avg=".000" hr="0" rbi="0"/><player id="605476" first="Carson" last="Smith" num="39" boxname="Smith, C" rl="R" bats="R" position="P" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="643217" first="Andrew" last="Benintendi" num="16" boxname="Benintendi" rl="L" bats="L" position="LF" current_position="LF" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" bat_order="2" game_position="LF" avg=".000" hr="0" rbi="0"/><player id="646240" first="Rafael" last="Devers" num="11" boxname="Devers" rl="R" bats="L" position="3B" status="A" team_abbrev="BOS" team_id="111" parent_team_abbrev="BOS" parent_team_id="111" avg=".000" hr="0" rbi="0"/><coach position="manager" first="Alex" last="Cora" id="133321" num="20"/><coach position="hitting_coach" first="Tim" last="Hyers" id="116377" num="51"/><coach position="assistant_hitting_coach" first="Andy" last="Barkett" id="406723" num="58"/><coach position="pitching_coach" first="Dana" last="LeVangie" id="427298" num="60"/><coach position="assistant_pitching_coach" first="Brian" last="Bannister" id="446454" num="86"/><coach position="first_base_coach" first="Tom" last="Goodwin" id="114961" num="82"/><coach position="third_base_coach" first="Carlos" last="Febles" id="136866" num="52"/><coach position="bench_coach" first="Ron" last="Roenicke" id="121373" num="10"/><coach position="bullpen_coach" first="Craig" last="Bjornson" id="459643" num="53"/><coach position="bullpen_catcher" first="Mani" last="Martinez" id="463799" num="88"/><coach position="coach" first="Ramon" last="Vazquez" id="407496" num="84"/></team><team type="home" id="NYY" name="New York Yankees"><player id="282332" first="CC" last="Sabathia" num="52" boxname="Sabathia" rl="L" bats="L" position="P" current_position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="0" game_position="P" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="435522" first="Neil" last="Walker" num="14" boxname="Walker" rl="R" bats="S" position="1B" current_position="3B" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="6" game_position="3B" avg=".000" hr="0" rbi="0"/><player id="453056" first="Jacoby" last="Ellsbury" num="22" boxname="Ellsbury" rl="L" bats="L" position="CF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="457705" first="Andrew" last="McCutchen" num="26" boxname="McCutchen" rl="R" bats="R" position="RF" current_position="LF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="457918" first="J.A." last="Happ" num="34" boxname="Happ, J" rl="L" bats="L" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="458681" first="Lance" last="Lynn" num="36" boxname="Lynn" rl="R" bats="S" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="458731" first="Brett" last="Gardner" num="11" boxname="Gardner" rl="L" bats="L" position="LF" current_position="LF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="9" game_position="LF" avg=".000" hr="0" rbi="0"/><player id="476454" first="Dellin" last="Betances" num="68" boxname="Betances" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="502085" first="David" last="Robertson" num="30" boxname="Robertson, D" rl="R" bats="R" position="P" current_position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="502154" first="Zach" last="Britton" num="53" boxname="Britton" rl="L" bats="L" position="P" current_position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="519222" first="Austin" last="Romine" num="28" boxname="Romine, Au" rl="R" bats="R" position="C" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="519317" first="Giancarlo" last="Stanton" num="27" boxname="Stanton" rl="R" bats="R" position="LF" current_position="DH" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="4" game_position="DH" avg=".000" hr="0" rbi="0"/><player id="543305" first="Aaron" last="Hicks" num="31" boxname="Hicks, A" rl="R" bats="S" position="CF" current_position="CF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="1" game_position="CF" avg=".000" hr="0" rbi="0"/><player id="544369" first="Didi" last="Gregorius" num="18" boxname="Gregorius" rl="R" bats="L" position="SS" current_position="SS" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="3" game_position="SS" avg=".000" hr="0" rbi="0"/><player id="547888" first="Masahiro" last="Tanaka" num="19" boxname="Tanaka" rl="R" bats="R" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="547973" first="Aroldis" last="Chapman" num="54" boxname="Chapman, A" rl="L" bats="L" position="P" current_position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="572228" first="Luke" last="Voit" num="45" boxname="Voit" rl="R" bats="R" position="1B" current_position="1B" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="5" game_position="1B" avg=".000" hr="0" rbi="0"/><player id="588751" first="Adeiny" last="Hechavarria" num="29" boxname="Hechavarria" rl="R" bats="R" position="SS" current_position="PR" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="592450" first="Aaron" last="Judge" num="99" boxname="Judge" rl="R" bats="R" position="RF" current_position="RF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="2" game_position="RF" avg=".000" hr="0" rbi="0"/><player id="596142" first="Gary" last="Sanchez" num="24" boxname="Sanchez, G" rl="R" bats="R" position="C" current_position="C" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="7" game_position="C" avg=".000" hr="0" rbi="0"/><player id="605501" first="Stephen" last="Tarpley" num="71" boxname="Tarpley" rl="L" bats="R" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="609280" first="Miguel" last="Andujar" num="41" boxname="Andujar" rl="R" bats="R" position="3B" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="621294" first="Ben" last="Heller" num="" boxname="Heller" rl="R" bats="R" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="622663" first="Luis" last="Severino" num="40" boxname="Severino" rl="R" bats="R" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="640449" first="Clint" last="Frazier" num="77" boxname="Frazier, C" rl="R" bats="R" position="LF" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0"/><player id="643338" first="Chad" last="Green" num="57" boxname="Green" rl="R" bats="L" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="650402" first="Gleyber" last="Torres" num="25" boxname="Torres" rl="R" bats="R" position="2B" current_position="2B" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" bat_order="8" game_position="2B" avg=".000" hr="0" rbi="0"/><player id="656547" first="Jonathan" last="Holder" num="56" boxname="Holder" rl="R" bats="R" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><player id="656756" first="Jordan" last="Montgomery" num="47" boxname="Montgomery, J" rl="L" bats="L" position="P" status="A" team_abbrev="NYY" team_id="147" parent_team_abbrev="NYY" parent_team_id="147" avg=".000" hr="0" rbi="0" wins="0" losses="0" era="-"/><coach position="manager" first="Aaron" last="Boone" id="111213" num="17"/><coach position="hitting_coach" first="Marcus" last="Thames" id="407801" num="62"/><coach position="assistant_hitting_coach" first="P.J." last="Pilittere" id="452063" num="63"/><coach position="pitching_coach" first="Larry" last="Rothschild" id="121495" num="58"/><coach position="first_base_coach" first="Reggie" last="Willits" id="435065" num="50"/><coach position="third_base_coach" first="Phil" last="Nevin" id="119732" num="35"/><coach position="bench_coach" first="Josh" last="Bard" id="408036" num="59"/><coach position="bullpen_coach" first="Mike" last="Harkey" id="115476" num="60"/><coach position="major_league_coach" first="Carlos" last="Mendoza" id="425825" num="64"/><coach position="bullpen_catcher" first="Radley" last="Haddad" id="645080" num=""/><coach position="catching_coach" first="Jason" last="Brown" id="440785" num=""/><coach position="assistant_coach" first="Brett" last="Weber" id="437906" num=""/></team><umpires><umpire position="home" name="Angel Hernandez" id="427220" first="Angel" last="Hernandez"/><umpire position="first" name="Fieldin Culbreth" id="427090" first="Fieldin" last="Culbreth"/><umpire position="second" name="D.J. Reyburn" id="482666" first="D.J." last="Reyburn"/><umpire position="third" name="Cory Blaser" id="484183" first="Cory" last="Blaser"/><umpire position="left" name="Dan Bellino" id="483564" first="Dan" last="Bellino"/><umpire position="right" name="Mike Winters" id="427552" first="Mike" last="Winters"/></umpires></game>"#;

     // let player: Player = serde_xml_rs::from_str(&test_string).unwrap();
    // let umpires: Umpires = serde_xml_rs::from_str(&test_string_2).unwrap();
    // dbg!(&umpires);
   
    // let game: Game = serde_xml_rs::from_str(&players_xml).unwrap();
    // dbg!(&game.teams[0].home_away);
    // dbg!(&game.teams[1].home_away);
    // // dbg!(&game.umpires);
   
    // let umps = game.umpires.umpires;

    // #[allow(non_snake_case)]
    // let (ump_HP_id, ump_HP_name) = (umps[0].id.parse().unwrap_or(0), umps[0].name.clone());
    // #[allow(non_snake_case)]
    // let (ump_1B_id, ump_1B_name) = (umps[1].id.parse().unwrap_or(0), umps[1].name.clone());
    // #[allow(non_snake_case)]
    // let (ump_2B_id, ump_2B_name) = (umps[2].id.parse().unwrap_or(0), umps[2].name.clone());
    // #[allow(non_snake_case)]
    // let (ump_3B_id, ump_3B_name) = (umps[3].id.parse().unwrap_or(0), umps[3].name.clone());
    // #[allow(non_snake_case)]
    // let (ump_LF_id, ump_LF_name) = if umps.len() > 4 {
    //         (Some(umps[4].id.parse().unwrap_or(0)), Some(umps[4].name.clone()))
    //     }
    //     else {
    //         (None, None)
    //     };
    // #[allow(non_snake_case)]    
    // let (ump_RF_id, ump_RF_name) = if umps.len() > 5 {
    //         (Some(umps[5].id.parse().unwrap_or(0)), Some(umps[5].name.clone()))
    //     }
    //     else {
    //         (None, None)
    //     };    
    
    
    // let game_umps = GameUmpires {
    //     ump_HP_id, ump_HP_name,
    //     ump_1B_id, ump_1B_name,
    //     ump_2B_id, ump_2B_name,
    //     ump_3B_id, ump_3B_name,
    //     ump_LF_id, ump_LF_name,
    //     ump_RF_id, ump_RF_name,
    // };

    // dbg!(game_umps);


    // let umpire_hp_name = &game_umpires.filter(|u| u.position=="home").nth(0).unwrap().name.clone();
    // let umpire_hp_id = &game_umpires.filter(|u| u.position=="home").nth(0).unwrap().id.clone();

    // dbg!(umpire_hp_name);
    
    let regex = r#"(<umpires>.*</umpires>)"#;
    let player_regex = Regex::new(regex).unwrap();

    // let umpires = player_regex.captures(&players_xml).unwrap();
    // dbg!(&umpires);

    // dbg!(home_team);
    // dbg!(away_team);

    
    // dbg!(games);

   
    // dbg!(linescore);

}