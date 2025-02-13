mod htmlwriter;

use reqwest::{self, Body};
use reqwest::Error;
use tokio;
use std::fs::File;
use std::io::{ErrorKind, Error as stdError, Read};
use serde_json::{self, json};
use serde::Deserialize;
use std::{env, fs, time};
use std::cmp::{PartialEq, Reverse};
use std::collections::HashMap;
use std::io::prelude::*;
use std::thread;
use tokio::task;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use chrono::{Datelike, Local};


const HISCORES_URL_BASE: &str = "https://secure.runescape.com/m=hiscore_oldschool/index_lite.ws?player=";
const FILE_NAME: &str = "config/usernames.txt";
const OUTPUT_FILE: &str = "out/output.tex";

//Weights are subject to change; preferably configurable for each different entry, with custom milestones and custom point counts
//additionally it would be nice if milestones after level 99 were implemented, such as 25m xp. 
//by current weights a maxed player will have 3950 points from skills.

#[derive(Debug, Deserialize, Clone)]
struct HiScoreStructure(Vec<HiScoreCategory>);

#[derive(Debug, Deserialize, Clone)]
struct HiScoreCategory{
    name: String,
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Clone)]
struct Entry{
    name: String,
    milestones: Vec<Milestone>
}

#[derive(Debug, Deserialize, Clone)]
struct Milestone(isize,isize);

#[derive(Debug, Deserialize)]
pub struct EvaluatedHiscores {
    categories: Vec<EvaluatedCategory>,
    points: isize,
}

#[derive(Debug, Deserialize)]
pub struct EvaluatedCategory {
    name: String,
    evaluated_entries: Vec<EvaluatedEntry>,
    points: isize,
}

#[derive(Debug, Deserialize)]
pub struct EvaluatedEntry{
    name: String,
    score: isize,
    points: isize,
}

#[derive(Debug, Deserialize)]
struct apiScore{
    username: String,
    total: isize,
    skilling: isize,
    minigamesClues: isize,
    pvm: isize,
}

#[derive(Debug, Deserialize, Clone)]
struct PlayerList(Vec<Player>);

#[derive(Debug, Deserialize, Clone)]
struct Player{
    username: String,
}

#[derive(Debug, Deserialize)]
struct player_points_rank_tuple{
    username: String,
    total_points: isize,
    pvm_points: isize,
    skilling_points: isize,
    activities_points: isize,
    rank: Rank
}

#[derive(Debug, Deserialize, PartialEq)]
enum Rank {
    Unranked,
    RedTopaz,
    Sapphire,
    Emerald,
    Ruby,
    Diamond,
    Dragonstone,
    Onyx,
    Zenyte,
    Death,
    Blood,
    Soul,
    Wrath
}

impl Rank {
    fn from_name(name: &str) -> Self {
        match name {
            "Unranked" => Rank::Unranked,
            "RedTopaz" => Rank::RedTopaz,
            "Sapphire" => Rank::Sapphire,
            "Emerald" => Rank::Emerald,
            "Ruby" => Rank::Ruby,
            "Diamond" => Rank::Diamond,
            "Dragonstone" => Rank::Dragonstone,
            "Onyx" => Rank::Onyx,
            "Zenyte" => Rank::Zenyte,
            "Death" => Rank::Death,
            "Blood" => Rank::Blood,
            "Soul" => Rank::Soul,
            "Wrath" => Rank::Wrath,
            _ => Rank::Unranked, // Default to Unranked for unknown names
        }
    }
}



async fn get_hiscores(username: &str) -> Result<String, Error> {
    let hiscore_url = String::from(HISCORES_URL_BASE) + &username;

    let res = reqwest::get(hiscore_url).await?;
    let body = res.text().await?;

    Ok(body)
}

fn calc_points(score: isize, milestones: &Vec<Milestone>) -> isize {
    if score == 0 {
        return 0;
    }
    let mut points = 0;
    for milestone in milestones{
        if milestone.0 >= 0{
            //Milestones

            if score < milestone.0{
                return points;
            }
        } else {
            //Points per kill
            //this should really be refactored.
            points += ((score as f32 / (milestone.1) as f32).floor() as isize) * 2;
            return points
        }
        points += milestone.1;
    }   
    points
}

fn find_latest_ranks_file_path() -> Result<PathBuf, std::io::Error>{
    let dir_path = "out/ranks"; // Replace with your directory path

    let mut latest_file = None;
    let mut latest_time = None;

    for entry in fs::read_dir(dir_path).map_err(|_| ErrorKind::NotFound)? {
        let entry = entry.map_err(|_| ErrorKind::NotFound)?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified_time) = metadata.modified() {
                    if latest_time.is_none() || modified_time > latest_time.unwrap() {
                        latest_time = Some(modified_time);
                        latest_file = Some(path);
                    }
                }
            }
        }
    }

    match latest_file {
        Some(path) => Ok(path),
        None => Err(std::io::Error::from(ErrorKind::NotFound))
    }
}

fn read_config() -> Result<HiScoreStructure, Box<dyn std::error::Error>> {
    let file_path = "config/config.json";

    let mut file = File::open(file_path)?;
    let mut config_json = String::new();
    file.read_to_string(&mut config_json)?;

    // Deserialize the JSON into your configuration struct
    let config: HiScoreStructure = serde_json::from_str(&config_json)?;

    Ok(config)
}

async fn print_scores(username: &str, total_score: isize, activities_score: isize, skilling_score: isize, pvm_score: isize) {
    let borrowed_total_score = &total_score.to_string();
    let borrowed_skilling_score = &skilling_score.to_string();
    let borrowed_activities_score = &activities_score.to_string();
    let borrowed_pvm_score = &pvm_score.to_string();

    println!("{:?}, {:?}, {:?}, {:?}, {:?},", username, borrowed_total_score, borrowed_skilling_score, borrowed_activities_score, borrowed_pvm_score)
}

fn writefile(text: &str) -> std::io::Result<()> {
    let mut file = File::create("foo.txt")?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn trimmed_username(username: &str) -> String {
    username.replace(",", "").trim().trim_matches('"').to_string()
}

async fn process(config: HiScoreStructure, usernames: Vec<String>) -> Result<Vec<player_points_rank_tuple>, Error>{
    let mut results:Vec<player_points_rank_tuple> = Vec::new();
    for username in usernames{

        let trimmed_username = &trimmed_username(&username);
        let mut hiscoresstring = get_hiscores(trimmed_username).await?;
            if hiscoresstring.starts_with("<!DOCTYPE html><html><head><title>404"){
                println!("Hiscores not found for user: {:?}.", trimmed_username);
                break
            }
            while hiscoresstring.starts_with('<') {
                thread::sleep(time::Duration::from_secs(1));
                hiscoresstring = get_hiscores(&username).await?;
                if hiscoresstring.starts_with("<!DOCTYPE html><html><head><title>404") {
                    return Ok(results)
                }
            }
        let mut hiscoreslines = hiscoresstring.lines();

        let mut player_points = EvaluatedHiscores{categories: Vec::new(), points: 0};

        'outer: for hiscore_category in &config.0{
            let mut evaluated_category:EvaluatedCategory = EvaluatedCategory { name: hiscore_category.name.to_string(), evaluated_entries: Vec::new(), points: 0 };
            for entry in &hiscore_category.entries{
                let name = &entry.name;
                //For each entry we parse the appropriate hiscore information.
                let line = hiscoreslines.next().unwrap();

                //parse the hiscore info we have                
                let mut parts = line.split(',');
                let rank = parts.next().unwrap_or("");
                
                let lvltext = parts.next().unwrap_or("");
                let mut level:isize;

                match(lvltext.parse::<isize>()){
                    Ok(mut level) => {
                        level = if level < 0 {0} else {level};
                        let mut score = 0;
                        if (hiscore_category.name=="Skilling".to_string()){
                            score = parts.next().unwrap_or("").parse::<isize>().unwrap();
                            score = if score < 0 {0} else {score};
                        } else {
                            score = level;
                        } 

                        let points = calc_points(score, &entry.milestones);
                        let evaluated_entry = EvaluatedEntry{name: name.to_string(),score,points};
                        evaluated_category.points+=evaluated_entry.points;
                        evaluated_category.evaluated_entries.push(evaluated_entry);
                    }
                    Err(_) => break 'outer
                }
            }
            player_points.points += evaluated_category.points;
            player_points.categories.push(evaluated_category);
        }

        htmlwriter::save_hiscores_details_page(trimmed_username, &player_points);

        let total_points = player_points.points;
        let pvm_points  = player_points.categories.pop().unwrap().points;
        let activities_points = player_points.categories.pop().unwrap().points;
        let skilling_points = player_points.categories.pop().unwrap().points;
        
        if total_points > 1 {
            //print_scores(&username, total_points, activities_points, skilling_points, pvm_points).await;
            //println!("{:?}", &trimmed_username);
            let points = &player_points.points.clone();
            let tuple = player_points_rank_tuple {
                username: String::from(trimmed_username),
                total_points: total_points,
                pvm_points: pvm_points,
                activities_points: activities_points,
                skilling_points: skilling_points,
                rank: evaluate_rank(points),
            };
            results.push(tuple);
        }
    }
    Ok(results)
}

fn evaluate_rank(points: &isize) -> Rank {
    match points{
        n if n < &1 => Rank::Unranked,
        0..200 => Rank::RedTopaz,
        200 .. 500 => Rank::Sapphire,
        500 .. 750 => Rank :: Emerald,
        750 .. 1000 => Rank::Ruby,
        1000 .. 1500 => Rank::Diamond,
        1500 .. 2000 => Rank:: Dragonstone,
        2000 .. 3000 => Rank::Onyx,
        3000 .. 4000 => Rank::Zenyte,
        4000 .. 5000 => Rank::Death,
        5000 .. 6000 => Rank::Blood,
        6000 .. 8000 => Rank::Soul,
        _ => Rank::Wrath
    }
}

fn read_usernames_file() -> io::Result<Vec<String>>{
    let path = Path::new(FILE_NAME);
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines().collect::<Result<Vec<String>, io::Error>>()?;
    Ok(lines)

}

fn write_header(file: &mut File){
    writeln!(file, "\\documentclass{{article}}");
    writeln!(file, "\\begin{{document}}");
    writeln!(file, "\\begin{{table}}[htbp]");
    writeln!(file, "\\centering");
    writeln!(file, "\\pagenumbering{{gobble}}");
    writeln!(file, "\\begin{{tabular}}{{|l|r|r|r|r|l|}}");
    writeln!(file, "\\hline");
    writeln!(file, "\\textbf{{Username}} & \\textbf{{Total}} & \\textbf{{Skilling}} & \\textbf{{Clues and Activities}} & \\textbf{{PVM}} & \\textbf{{Fe Nixes Rank}} \\\\ \\hline");
}

fn write_footer(file: &mut File){
    writeln!(file, "\\end{{tabular}}");
    writeln!(file, "\\end{{table}}");
    writeln!(file, "\\end{{document}}");
}

fn process_results(results: &mut Vec<player_points_rank_tuple>){
    create_latex_output(results);
    check_for_promotions(results); //check for promos first, as the store will make a new text file.
    store_daily_ranks(results);
    generate_index_page(results);
}

fn create_latex_output(results: &mut Vec<player_points_rank_tuple>){
    let mut file = File::create(OUTPUT_FILE).unwrap();
    results.sort_by_key(|item| Reverse(item.total_points));

    write_header(&mut file);
    for result in results {
        writeln!(&file, "{:?} & {:?} & {:?} & {:?} & {:?} & {:?} \\\\ \\hline", trimmed_username(&result.username), result.total_points, result.skilling_points, result.activities_points, result.pvm_points, result.rank);
    }
    write_footer(&mut file);
    file.flush().unwrap()
}

fn store_daily_ranks(results: &mut Vec<player_points_rank_tuple>){
    let today = Local::now().date_naive();
    let year = today.year();
    let month = today.month();
    let day = today.day();

    let filename = format!("out/ranks/{}-{}-{}-RANKS.txt", year, month, day);

    let mut file = File::create(filename).unwrap();
    for result in results {
        writeln!(&file, "{:?}, {:?}", trimmed_username(&result.username), result.rank);
    }
    file.flush().unwrap()
}

fn check_for_promotions(results: &mut Vec<player_points_rank_tuple>){
    let latest_file = find_latest_ranks_file_path();
    match latest_file{
        Ok(value) => compare_results(results, value),
        Err(err) => println!("No previous results file found.")
    }
}

fn compare_results(results: &mut Vec<player_points_rank_tuple>, filepath: PathBuf){
    println!("Filepath selected: {:?}", filepath);
    let content = fs::read_to_string(filepath).unwrap();
    let previous_results = create_previous_results_map(content);

    for result in results {
        if let Some(previous_rank) = previous_results.get(&trimmed_username(&result.username)){
            if previous_rank != &result.rank {
                println!("New rank found: {:?} {:?} --> {:?}", &result.username, previous_rank, &result.rank);
            }
        }

    }
}

fn create_previous_results_map(content: String) -> HashMap<String, Rank> {
    let mut map = HashMap::new();

    for line in content.lines(){
        if let Some((username, rank)) = line.split_once(',') {
            map.insert(String::from(trimmed_username(username)), Rank::from_name(rank.trim()));
        }
    }


    map
}

fn generate_index_page(results: &mut Vec<player_points_rank_tuple>){
    htmlwriter::write_index(process_results_into_frontend_data(results)).unwrap();
}

fn process_results_into_frontend_data(results: &mut Vec<player_points_rank_tuple>) -> Vec<(String, u32, u32, u32, u32, String)>{
    results
        .iter()
        .map(|result|
        {
            (
                trimmed_username(&result.username).to_string(), // Username
                result.total_points as u32,                    // Total points
                result.skilling_points as u32,                 // Skilling points
                result.activities_points as u32,               // Activities points
                result.pvm_points as u32,                      // PvM points
                format!("{:?}", result.rank),                  // Rank (formatted as string)
            )
        }
    )
    .collect()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
        
    let config = read_config().unwrap();
    let mut results:Vec<player_points_rank_tuple> = Vec::new();

    let usernames = read_usernames_file().unwrap();

    let num_pieces = 10; //Number of threads.
    let mut pieces = Vec::new();
    let piece_size = (usernames.len() + num_pieces - 1) / num_pieces;

    for chunk in usernames.chunks(piece_size) {
        pieces.push(chunk.to_vec());
    }

    let mut handles = Vec::with_capacity(num_pieces);

    for piece in pieces {
        let copy_config = config.clone();
         let handle = task::spawn( async move {       
             let results_from_process = process(copy_config, piece.to_vec()).await;
             return results_from_process;
         });
        handles.push(handle);
    }

    for handle in handles {
        // Wait for the thread to finish and get its result
        let result = handle.await.unwrap().unwrap();
        results.extend(result);
    }

    process_results(&mut results);
    
    Ok(())
} 