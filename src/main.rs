use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use std::fs;
use std::process::Stdio;
use std::process::Command;

#[derive(Parser)]
#[command(name = "Check-AV1-Encode")]
#[command(author = "Dennis S.")]
#[command(version = "0.9")]
#[command(about = "Finds the crf needed to make a video ssim2 score 90", long_about = None)]
struct Args {
    /// File to Encode
    #[arg(short = 'i', long)]
    input_file: String,

    /// Encoded File Destination
    #[arg(short = 'o', long)]
    output_file: String,

    /// Encoding Speed
    #[arg(short = 's', long)]
    speed: String,

    /// Amount Of Workers
    #[arg(short = 'w', long)]
    worker_num: String,

    /// Starting Crf
    #[arg(short = 'c', long, default_value_t = 45)]
    crf: i32,

    /// Clip Length in seconds
    #[arg(short = 'l', long, default_value_t = 20)]
    clip_length: i32,

    /// Clip Interval in seconds
    #[arg(short = 'n', long, default_value_t = 360)] //every 6 min
    clip_interval: i32,

    /// select what crf to use on output video (average/smallest)
    #[arg(short = 'u', long, default_value_t = String::from("smallest"))]
    crf_option: String,

    /// if run inside arch wsl enable this
    #[arg(short = 'a', long, default_value_t = false)]
    inside_arch_wsl: bool,
}

fn main() {
    let args = Args::parse();
    let input_file = args.input_file;
    let output_file = args.output_file;
    let speed = args.speed;
    let worker_num = args.worker_num;
    let current_crf = args.crf;
    let clip_length = args.clip_length;
    let clip_interval = args.clip_interval;
    let crf_used = args.crf_option;
    let inside_arch_wsl = args.inside_arch_wsl;

    check_and_create_folders_helpers();

    let json_paths = match get_json() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Err: {}", err);
            let mut line = "".to_string();
            std::io::stdin().read_line(&mut line).unwrap();
            return;
        }
    };
    let av1an_path = json_paths[0].to_string();
    let ssim2_path = json_paths[1].to_string();
    let arch_path = json_paths[2].to_string();
    let ffmpeg_path = json_paths[3].to_string();
    let ffprobe_path = json_paths[4].to_string();
    let av1an_setings_unformatted = json_paths[5].to_string();

    //get clips
    let clip_names = extract_clips(
        &input_file,
        clip_length,
        clip_interval,
        &ffmpeg_path,
        &ffprobe_path,
    )
    .unwrap();

    if clip_names[0] == input_file {
        println!("Clip_Length is bigger then the whole video, please check the settings");
        let mut line = "".to_string();
        std::io::stdin().read_line(&mut line).unwrap();
        return;
    }

    let workers: usize = worker_num
        .parse()
        .expect("Failed parsing number of workers");

    let num_of_clips = clip_names.len();
    let threads_to_use = (workers).min(num_of_clips);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads_to_use)
        .build_global()
        .unwrap(); // Sets the threads used by rayon's internal ThreadPool


    let worker_for_each_thread = (workers / num_of_clips).max(1);
    //for each clip find crf
    let crf_values: Vec<i32> = clip_names
        .par_iter()
        .map_init(
            || 0u32,
            |index, clip_name| {
                let crf_90 = find_crf_for_90_ssim2(
                    current_crf,
                    clip_name,
                    &av1an_setings_unformatted,
                    &speed,
                    &worker_for_each_thread.to_string(),
                    &av1an_path,
                    &arch_path,
                    &ssim2_path,
                    inside_arch_wsl,
                );
                *index += 1;
                crf_90
            },
        )
        .collect();
    // Par iter creates an parallel iterator using the global threads defined previously
    // Then map returns the value of the function find_crf_for_90_ssim2
    // Finally we use collect(), which automatically parses the collection generated by the iterator and stores it as an i32
    // ty Valenciano

    let min_crf = find_lowest_crf(crf_values.clone());
    let average_crf = find_average_crf(crf_values.clone());
    let crf_used_use: i32 = if crf_used == "smallest"{
        min_crf
    }
    else{
        average_crf
    };
    println!("crf_values: {:?}", crf_values);
    println!("min_crf: {}", min_crf);
    println!("average_crf: {}", average_crf);
    let av1an_settings = format_encoding_settings(
        &av1an_setings_unformatted,
        &input_file,
        &speed,
        &crf_used_use.to_string(),
        &worker_num,
        &output_file,
    );
    encode_clip(&av1an_settings, &av1an_path).unwrap();
    println!("Finished Encoding: {}", input_file);
}

fn encode_clip(av1an_settings: &String, av1an_path: &String) -> Result<i32, String> {
    //
    //  start encoding a clip with crf given and additional settings
    //
    let av1an_settings_formated = format_for_process(av1an_settings);
    let av1an_settings_formated_ref: Vec<&str> = av1an_settings_formated.iter().map(|s| s.as_str()).collect();
    match spawn_a_process(av1an_path, av1an_settings_formated_ref){
        Ok(_out) => println!("Clip encoded"),
        Err(err) => panic!("couldnt encode clip Err: {}", err),
    };
    Ok(0)
}

fn ssim2_clip(original_clip_path: &String,encoded_clip_path: &String,arch_path: &String,ssim2_path: &String,worker_num: &String, inside_arch_wsl: bool) -> Result<Vec<String>, String> {
    //run ssmi2 with arch wsl
    //return 95th percentile and 5th percentile if succeeded
    let mut results_vec: Vec<String> = Vec::new();

    let ssmi2_settings = if inside_arch_wsl {
        format!(
            "{} video -f {} \"{}\" \"{}\"",
            ssim2_path, worker_num, original_clip_path, encoded_clip_path
        )
    }
    else{
        format!(
            "runp {} video -f {} \"{}\" \"{}\"",
            ssim2_path, worker_num, original_clip_path, encoded_clip_path
        )
    };
    let ssmi2_settings_formated = format_for_process(&ssmi2_settings);
    let ssmi2_settings_formated_ref: Vec<&str> = ssmi2_settings_formated.iter().map(|s| s.as_str()).collect();

    let ssim2_score = match spawn_a_process(arch_path, ssmi2_settings_formated_ref){
        Ok(out) => {
            println!("ssim2 successfully");
            out
        },
        Err(err) => panic!("Couldnt ssim2 a video Err: {}", err),
    };
    let lines: Vec<&str> = ssim2_score.split('\n').collect();
    let pre_last_line = lines[lines.len() - 2]; //last line is empty
    let first_colon_index = pre_last_line.find(':').unwrap();
    let first_dot_index = pre_last_line.find('.').unwrap();
    let ninty_fifth_percent_in_str = pre_last_line
        .get((first_colon_index + 2)..first_dot_index)
        .unwrap();
    //let ninty_fifth_percent: i32 = ninty_fifth_percent_in_str.parse().unwrap();

    results_vec.push(ninty_fifth_percent_in_str.to_string());
    Ok(results_vec)
}

fn get_json() -> Result<Vec<String>, String> {
    //
    //  get paths for programs with json
    //
    let mut final_vec: Vec<String> = Vec::new();
    let json_file_string = match fs::read_to_string("paths.json") {
        Ok(string) => string,
        Err(_err) => {
            let error_messege = "Cannot Open/find json file \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    //whole json
    let json_values: Value = match serde_json::from_str(&json_file_string) {
        Ok(value) => value,
        Err(_err) => {
            let error_messege = "\"paths.json\" fromatted incorectly".to_string();
            return Err(error_messege);
        }
    };

    //paths inside json
    let av1an_path_value = match json_values["av1an"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege = "\"av1an\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    let ssim2_path_value = match json_values["ssim2"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege = "\"ssim2\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    let arch_path_value = match json_values["arch"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege = "\"arch\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    let ffmpeg_path_value = match json_values["ffmpeg"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege = "\"ffmpeg\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    let ffprobe_path_value = match json_values["ffprobe"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege = "\"ffprobe\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    let av1an_settings_path_value = match json_values["encoding_settings"].as_str() {
        Some(str) => str.to_string(),
        None => {
            let error_messege =
                "\"encoding_settings\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    final_vec.push(av1an_path_value);
    final_vec.push(ssim2_path_value);
    final_vec.push(arch_path_value);
    final_vec.push(ffmpeg_path_value);
    final_vec.push(ffprobe_path_value);
    final_vec.push(av1an_settings_path_value);
    Ok(final_vec)
}

fn extract_clips(full_video: &String, clip_length: i32, interval: i32, ffmpeg_path: &String, ffprobe_path: &String) -> Result<Vec<String>, String> {
    //
    //  first get the video length using ffprobe
    //  then in a for loop extract each clip using the clip_length and the interval
    //  last return all the clip names in a vec
    //
    let mut final_vec: Vec<String> = Vec::new();

    let ffprobe_settings = format!("-v error -select_streams v:0 -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 \"{}\"", full_video);

    //try to create a file to encode with
    let ffprobe_settings_formated = format_for_process(&ffprobe_settings);
    let ffprobe_settings_formated_ref: Vec<&str> = ffprobe_settings_formated.iter().map(|s| s.as_str()).collect();
    let probe_in_string = match spawn_a_process(ffprobe_path, ffprobe_settings_formated_ref){
        Ok(out) => {
            println!("ffprobe successfully");
            out},
        Err(err) => panic!("Couldnt ffprobe the file err: {}", err),
    };

    //read the result that was saved to a file
    let first_dot_index = probe_in_string.find('.').unwrap();
    let video_length_in_str = probe_in_string.get(0..first_dot_index).unwrap();
    let video_length: i32 = video_length_in_str.parse().unwrap();
    if video_length < clip_length {
        //dont make clip just tell the av1an to encode the whole video
        //as it is really small, smaller then the clip that the user wanted
        final_vec.push(full_video.to_string());
        return Ok(final_vec);
    }
    let mut length_passed = 0;
    let mut current_file_name_index = 0;
    while length_passed < video_length {
        let current_file_name = length_passed.to_string()
            + "-"
            + &(length_passed + clip_length).to_string()
            + "-"
            + &current_file_name_index.to_string()
            + ".mkv";

        let ffmpeg_settings = format!(
            "-ss {} -i \"{}\" -c copy -t {} \"output_helper/clips/{}\"",
            length_passed, full_video, clip_length, current_file_name
        );
        let ffmpeg_settings_formated = format_for_process(&ffmpeg_settings);
        let ffmpeg_settings_formated_ref: Vec<&str> = ffmpeg_settings_formated.iter().map(|s| s.as_str()).collect();

        match spawn_a_process(ffmpeg_path, ffmpeg_settings_formated_ref){
            Ok(_out) => println!("ffmpeg successfully (created clip)"),
            Err(err) => panic!("Couldnt ffmpeg the file err: {}", err),
        };
        final_vec.push(current_file_name);

        length_passed += clip_length + interval;
        current_file_name_index += 1;
    }
    println!("Created all the Clips");
    Ok(final_vec)
}

fn format_encoding_settings(settings: &str, input_file: &String, speed: &str, crf: &str, worker_num: &str, output_file: &String) -> String {
    let mut final_string = settings.to_owned();
    final_string = final_string.replace(
        "INPUT",
        &("\"".to_string() + input_file + "\""),
    ); //INPUT
    final_string = final_string.replace("SPEED", speed); //SPEED
    final_string = final_string.replace("CRF", crf); //CRF/QUANTIZER
    final_string = final_string.replace("WORKER_NUM", worker_num); //WORKER_NUM
    final_string = final_string.replace(
        "OUTPUT",
        &("\"".to_string() + output_file + "\""),
    ); //OUTPUT
    final_string
}

fn check_and_create_folders_helpers() {
    //delete latest encode
    fs::create_dir_all("output_helper").unwrap();
    fs::remove_dir_all("output_helper/").unwrap();
    fs::create_dir_all("output_helper/clips").unwrap();
    fs::create_dir_all("output_helper/clips_encoded").unwrap();
}

fn spawn_a_process(app_name: &String, args: Vec<&str>) -> Result<String, String>{
    //using spawn to show the user the program running
    let process = match Command::new(app_name).args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(out) => out,
        Err(err) => {
            let temp = "ERR: ".to_string() + &err.to_string();
            return Err(temp);
        },
    };

    let output = match process.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => {
            let temp = "ERR: ".to_string() + &err.to_string();
            return Err(temp);
        },
    };

    if !output.status.success() {
        println!("status: {}", output.status);
        println!("stderr: {:?}", &output.stderr);
        println!("stdout: {:?}", &output.stdout);
        Err(String::from_utf8(output.stderr).unwrap())
    }
    else{
        Ok(String::from_utf8(output.stdout).unwrap())
    }
}

fn find_crf_for_90_ssim2(starting_crf: i32, clip_name: &String, av1an_setings_unformatted: &str, speed: &str, worker_num: &String, av1an_path: &String, arch_path: &String, ssim2_path: &String, inside_arch_wsl: bool) -> i32 {
    let mut current_crf = starting_crf;
    let current_clip_name = format!("output_helper/clips/{}", clip_name);
    let current_clip_encoded_name = format!("output_helper/clips_encoded/{}", clip_name);
    let mut was_above_90 = false;
    let mut was_below_90 = false;
    while current_crf > 15 {//crf should not be less them 15
        let current_crf_str: String = current_crf.to_string();
        let av1an_settings = format_encoding_settings(
            av1an_setings_unformatted,
            &current_clip_name,
            speed,
            &current_crf_str,
            worker_num,
            &current_clip_encoded_name,
        );
        println!("Trying to encode: {}", &current_clip_name);
        encode_clip(&av1an_settings, av1an_path).unwrap();
        println!("Encoded Succesfully: {}", &current_clip_name);
        println!("Trying to ssim2: {}", &current_clip_name);
        let ssim2_results = ssim2_clip(
            &current_clip_name,
            &current_clip_encoded_name,
            arch_path,
            ssim2_path,
            &worker_num.to_string(),
            inside_arch_wsl,
        )
        .unwrap();
        println!("ssim2 Succesfully: {}", &current_clip_name);
        let result_95: i32 = ssim2_results[0].parse().unwrap();
        println!(
            "\n\n\n\ncurrent_clip: {}, current_crf: {}, current_ssim2: {}",
            current_clip_name, current_crf, ssim2_results[0]
        );
        if result_95 == 90 {
            return current_crf;
            //found the crf wanted
        }
        if result_95 < 90 {
            if was_above_90 {
                was_below_90 = true;
                current_crf -= 1;
            } else {
                current_crf -= 5;
            }
        }
        if result_95 > 90 {
            was_above_90 = true;
            if was_below_90 {
                current_crf += 1;
            } else {
                current_crf += 5;
            }
        }
        fs::remove_file(&current_clip_encoded_name).unwrap(); //delete encoded file to encode again
        fs::remove_file(current_clip_encoded_name.to_string() + ".lwi").unwrap();
        //delete encoded file iwi for ssim2
    }

    current_crf
}

fn find_lowest_crf(crf_list: Vec<i32>) -> i32 {
    crf_list
        .iter()
        .min()
        .expect("Failed getting the minimum crf")
        .to_owned()
}

fn find_average_crf(crf_list: Vec<i32>) -> i32 {
    let list_len = crf_list.len();
    let sum: i32 = crf_list.iter().sum();

    sum / (list_len as i32)
}

fn format_for_process(settings: &String) -> Vec<String>{
    //need to take this and change into 1 argument at a time
    let settings_final = settings.to_string();
    let mut final_vec: Vec<String> = Vec::new();
    let everything_splitted: Vec<String> = settings_final.split(' ').map(|e| e.to_string()).collect();
    let mut skip_until = 0;
    for i in 0..everything_splitted.len(){
        if skip_until == 0{
            if everything_splitted[i].starts_with('"'){
                //find the end of the qoute
                let mut qoute_found_on = 0;
                for (j, current_split) in everything_splitted.iter().enumerate().skip(i){
                    if current_split.ends_with('"'){
                        qoute_found_on = j;
                        break;
                    }
                }
                //take all the parameters inside the double qoute and make them one argument
                let mut strings_together: String;
                if i != qoute_found_on{
                    strings_together = everything_splitted[i][1..].to_string();
                    strings_together.push(' ');
                    for j in everything_splitted.iter().take(qoute_found_on).skip(i+1){
                        strings_together.push_str(j);
                        strings_together.push(' ');
                    }
                    //add the last part but without the double qoute
                    let temp_string = everything_splitted[qoute_found_on].clone();
                    strings_together.push_str(&temp_string[..temp_string.len()-1]);
                }
                else{
                    let strings_length = everything_splitted[i].len();
                    strings_together = everything_splitted[i][1..strings_length-1].to_string();
                }
                final_vec.push(strings_together);
                skip_until = qoute_found_on - i;
            }
            else{
                final_vec.push(everything_splitted[i].clone());
            }
        }
        else {
            skip_until -= 1;
        }
    }

    final_vec
}
