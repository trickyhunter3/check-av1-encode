use std::{process::Command, fs::{File, self}, io::Write, env};
use serde_json::Value;

//constants
const WORKER_NUM: &str = "4";

fn main() {
    check_and_create_folders_helpers();
    let input_file = r"C:\Encode\720p_15s.mp4".to_string();
    let output_file = r"C:\Encode\speed4-Q60.mp4".to_string();

    let json_paths = match get_json(){
        Ok(ok) => ok,
        Err(err) =>{
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


    let starting_crf = 60;
    let speed = "4".to_string();
    let worker_num = WORKER_NUM.to_string();
    //get clips
    let clip_names = extract_clips(input_file.clone(), 3, 2, ffmpeg_path, ffprobe_path).unwrap();
    let mut ssmi2_check_valid = false;
    
    //for each clip get make the settings do ssim2 90 on
    let mut current_crf = starting_crf;
    let mut crf_delta_down = 10;
    let mut crf_delta_up = 5;
    let mut was_above_90 = false;
    let mut was_below_90 = false;
    for i in 0..clip_names.len(){
        let current_clip_name = format!("output_helper/clips/{}", clip_names[i]);
        let current_clip_encoded_name = format!("output_helper/clips_encoded/{}",clip_names[i]);
        while !ssmi2_check_valid {
            let current_crf_str: String = current_crf.to_string();
            let av1an_settings = format_encoding_settings(av1an_setings_unformatted.clone(), current_clip_name.clone(), speed.clone(), current_crf_str.clone(), worker_num.clone(), current_clip_encoded_name.clone());
            encode_clip(current_clip_name.clone(), av1an_path.clone(), av1an_settings).unwrap();
            let ssim2_results = ssim2_clip(current_clip_name.clone(), current_clip_encoded_name.clone(), arch_path.clone(), ssim2_path.clone()).unwrap();
            println!("ssim2: \"{}\"", ssim2_results[0]);
            let result_95: i32 = ssim2_results[0].parse().unwrap();
            println!("current_clip: {}, current_crf: {}, current_ssim2: {}", current_clip_name.clone(), current_crf, ssim2_results[0]);
            if result_95 == 90{
                ssmi2_check_valid = true;
                break;//found the crf wanted, checking this crf with the other clips
            }
            if result_95 < 90{
                if was_above_90{
                    was_below_90 = true;
                    current_crf -= 2;
                }
                else{
                    current_crf -= crf_delta_down;
                }
            }
            if result_95 > 90{
                was_above_90 = true;
                if was_below_90{//it should be "was below 90 while it was above 90 just now"
                    current_crf += 1;
                }
                else{
                    current_crf += crf_delta_up;
                }
            }
            fs::remove_file(current_clip_encoded_name.clone()).unwrap();//delete encoded file to encode again
        }
    }
}

fn encode_clip(clip_path: String, av1an_path: String, av1an_settings: String) -> Result<i32, String>{
    //
    //  start encoding a clip with crf given and additional settings
    //
    let file_name = "av1an_encode_settings.bat".to_string();
    //try to create a file to encode with
    match create_file_encoding_settings(av1an_settings.clone(), file_name.clone()){
        Ok(ok) => ok,
        Err(_err) => {
            let error_messege = "Cannot Create File".to_string();
            return Err(error_messege);
        }
    };
    //try start encoding
    let av1an_child_proccess = match Command::new("cmd").args(["/C", &file_name, &av1an_path]).spawn(){
        Ok(out) => out,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "Cannot start encoding file: ".to_string() + &clip_path + "\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    //waiting for procces to finish
    let av1an_output = match av1an_child_proccess.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "While waiting for file: ".to_string() + &clip_path + "To encode it errored\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    if av1an_output.status.success(){
        println!("successfully Encoded clip: {}", clip_path);
    }
    return Ok(0);
}

fn ssim2_clip(original_clip_path: String, encoded_clip_path: String, arch_path: String, ssim2_path: String) -> Result<Vec<String>, String>{
    //run ssmi2 with arch wsl
    //return 95th percentile and 5th percentile if succeeded
    let mut results_vec: Vec<String> = Vec::new();

    let save_file_name = "output_helper/ssim2/ssim2_output.txt".to_string();

    let ssmi2_settings = format!("%1 runp {} video -f {} \"{}\" \"{}\" > {}",
        ssim2_path, WORKER_NUM, original_clip_path, encoded_clip_path, save_file_name);

    let file_name = "ssmi2_encode_settings.bat".to_string();
    match create_file_encoding_settings(ssmi2_settings, file_name.clone()){
        Ok(ok) => ok,
        Err(_err) => {
            let error_messege = "Cannot Create File".to_string();
            return Err(error_messege);
        }
    };

    //try start ssim2 and 
    let ssim2_child_proccess = match Command::new("cmd").args(["/C", &file_name, &arch_path]).spawn(){
        Ok(out) => out,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "Cannot start ssmi2 file: ".to_string() + &encoded_clip_path + "\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    //waiting for procces to finish
    let ssim2_output = match ssim2_child_proccess.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "While waiting for file: ".to_string() + &encoded_clip_path + "To ssim2 it errored\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    if ssim2_output.status.success(){
        println!("ssmi2 successfully clip: {}", encoded_clip_path);
    }

    let output_file_content = fs::read_to_string(save_file_name).expect("Should have been able to read ssim2_output.txt");
    let lines: Vec<&str> = output_file_content.split("\n").collect();
    let pre_last_line = lines[lines.len() - 2];//last line is empty
    let first_colon_index = pre_last_line.find(":").unwrap();
    let first_dot_index = pre_last_line.find(".").unwrap();
    let ninty_fifth_percent_in_str = pre_last_line.get((first_colon_index+2)..first_dot_index).unwrap();
    //let ninty_fifth_percent: i32 = ninty_fifth_percent_in_str.parse().unwrap();

    results_vec.push(ninty_fifth_percent_in_str.to_string());
    return Ok(results_vec);
}

fn create_file_encoding_settings(settings: String, file_name: String) -> Result<String, i32>{
    //
    //  writes a batch file to encode with later
    //  this is becuase procces in rust use string leterals or something :(
    // 
    let mut file = match File::create(file_name) {
        Ok(ok) => ok,
        Err(_err) => return Err(3)
    };
    match file.write_all(settings.as_bytes()){
        Ok(_ok) => return Ok("Success".to_string()),
        Err(_err) => return Err(3)
    };
    
}

fn get_json() -> Result<Vec<String>, String>{
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
            let error_messege = "\"encoding_settings\" was not found inside \"paths.json\"".to_string();
            return Err(error_messege);
        }
    };
    final_vec.push(av1an_path_value);
    final_vec.push(ssim2_path_value);
    final_vec.push(arch_path_value);
    final_vec.push(ffmpeg_path_value);
    final_vec.push(ffprobe_path_value);
    final_vec.push(av1an_settings_path_value);
    return Ok(final_vec);
}

fn extract_clips(full_video: String, clip_length: i32, interval: i32, ffmpeg_path: String, ffprobe_path: String) -> Result<Vec<String>, String>{
    //
    //  first get the video length using ffprobe
    //  then in a for loop extract each clip using the clip_length and the interval
    //  last return all the clip names in a vec
    //
    let mut final_vec: Vec<String> = Vec::new();

    let file_name_ffprobe = "ffprobe_settings.bat".to_string();
    let file_name_ffmpeg = "ffmpeg_settings.bat".to_string();
    let file_name_ffprobe_output = "output_helper/ffprobe/ffprobe_output.txt";
    //ffprobe -v error -select_streams v:0 -show_entries stream=duration -of default=noprint_wrappers=1:nokey=1 "/mnt/c/Encode/720p_15s.mp4"
    let ffprobe_settings = format!("%1 -v error -select_streams v:0 -show_entries stream=duration -of default=noprint_wrappers=1:nokey=1 \"{}\" > {}",
        full_video, file_name_ffprobe_output);
    
    //try to create a file to encode with
    match create_file_encoding_settings(ffprobe_settings.clone(), file_name_ffprobe.clone()){
        Ok(ok) => ok,
        Err(_err) => {
            let error_messege = "Cannot Create File".to_string();
            return Err(error_messege);
        }
    };

    let _ffprobe_child_proccess = match Command::new("cmd").args(["/C", &file_name_ffprobe, &ffprobe_path]).output(){
        Ok(out) => out,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "Cannot probe file: ".to_string() + &full_video + "\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };
    println!("Probed");
    //read the result that was saved to a file
    let output_file_content = fs::read_to_string(file_name_ffprobe_output).expect("Should have been able to read ffprobe_output.txt");
    let first_dot_index = output_file_content.find(".").unwrap();
    let video_length_in_str = output_file_content.get(0..first_dot_index).unwrap();
    let video_length: i32 = video_length_in_str.parse().unwrap();
    if video_length < clip_length{
        //dont make clip just tell the av1an to encode the whole video
        //as it is really small, smaller then the clip that the user wanted
        final_vec.push(full_video);
        return Ok(final_vec);
    }
    let mut length_passed = 0;
    let mut current_file_name_index = 0;
    while length_passed < video_length {
        let current_file_name = length_passed.to_string() + &"-".to_string() + &(length_passed + clip_length).to_string() + &"-".to_string() + &current_file_name_index.to_string() + ".mkv";
        let ffmpeg_settings = format!("%1 -ss {} -i \"{}\" -c copy -t {} \"output_helper/clips/{}\"",
            length_passed, full_video, clip_length, current_file_name);
        match create_file_encoding_settings(ffmpeg_settings.clone(), file_name_ffmpeg.clone()){
            Ok(ok) => ok,
            Err(_err) => {
                let error_messege = "Cannot Create File".to_string();
                return Err(error_messege);
            }
        };
        let _ffmpeg_child_proccess = match Command::new("cmd").args(["/C", &file_name_ffmpeg, &ffmpeg_path]).output(){
            Ok(out) => out,
            Err(err) => {
                //send clip file that errored and the error, as Err
                let error_messege = "Cannot clip file: ".to_string() + &full_video + "\nError: " + &err.to_string();
                return Err(error_messege);
            }
        };
        println!("Created Clip {}", current_file_name);
        final_vec.push(current_file_name);

        length_passed += clip_length + interval;
        current_file_name_index += 1;
    }
    return Ok(final_vec);
}

fn format_encoding_settings(settings: String, input_file: String, speed: String, crf: String, worker_num: String, output_file: String) -> String{
    let mut final_string = settings.clone();
    final_string = final_string.replace("INPUT", &("\"".to_string() + &input_file + &"\"".to_string()));//SPEED
    final_string = final_string.replace("SPEED", &speed);//SPEED
    final_string = final_string.replace("QUANTIZER", &crf);//CRF/QUANTIZER
    final_string = final_string.replace("WORKER_NUM", &worker_num);//WORKER_NUM
    final_string = final_string.replace("OUTPUT", &("\"".to_string() + &output_file + &"\"".to_string()));//WORKER_NUM
    println!("{}", final_string);
    return final_string;
}

fn check_and_create_folders_helpers(){
    fs::create_dir_all("output_helper/ssim2").unwrap();
    fs::create_dir_all("output_helper/clips").unwrap();
    fs::create_dir_all("output_helper/clips_encoded").unwrap();
    fs::create_dir_all("output_helper/ffprobe").unwrap();
}