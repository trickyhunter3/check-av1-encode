use std::{process::Command, fs::{File, self}, io::Write, env};
use serde_json::Value;

//constants
const WORKER_NUM: &str = "4";
const QUIN_NUM: &str = "255";

fn main() {
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

    let input_file = r"C:\Encode\720p_15s.mp4".to_string();
    let output_file = r"C:\Encode\out.mp4".to_string();
    let crf = "10".to_string();
    //let _a = encode_clip(input_file.clone(), output_file.clone(), crf, av1an_path);
    //let _b = ssim2_clip(input_file, output_file, arch_path, ssim2_path);
    let _c = extract_clips(input_file, 3, 2, ffmpeg_path, ffprobe_path).unwrap();
    print!("{:?}", _c);
}

fn encode_clip(clip_path: String, output_path: String, crf: String, av1an_path: String) -> Result<i32, String>{
    //
    //  start encoding a clip with crf given and additional settings
    //
    let av1an_settings: String = format!("%1 -i \"{}\" -y --verbose --keep --resume --split-method av-scenechange -m lsmash -c mkvmerge --photon-noise 2 --chroma-noise -e rav1e --force -v \"--speed {} --threads 2 --tiles 2 --quantizer {}\" --pix-format yuv420p10le -w {} -x 240 -o \"{}\""
    ,clip_path, crf, QUIN_NUM, WORKER_NUM, output_path);
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

fn ssim2_clip(original_clip_path: String, encoded_clip_path: String, arch_path: String, ssim2_path: String) -> Result<Vec<i32>, String>{
    //run ssmi2 with arch wsl
    //return 95th percentile and 5th percentile if succeeded
    let results_vec: Vec<i32> = Vec::new();

    //get current path to save files from arch wsl back to windows
    let current_dir = match env::current_dir(){
        Ok(ok) => ok,
        Err(err) => {
            let error_messege = "Cannot get current location\nError: ".to_string() + &err.to_string();
            return Err(error_messege);
        }
    };
    let save_file_name = "ssim2_output.txt".to_string();
    let output_save_location: String = current_dir.to_string_lossy().to_string() + "\\" + &save_file_name;

    let ssmi2_settings = format!("%1 runp {} video -f {} \"{}\" \"{}\" > {}",
        ssim2_path, WORKER_NUM, original_clip_path, encoded_clip_path, output_save_location);

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
    final_vec.push(av1an_path_value);
    final_vec.push(ssim2_path_value);
    final_vec.push(arch_path_value);
    final_vec.push(ffmpeg_path_value);
    final_vec.push(ffprobe_path_value);
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
    let file_name_ffprobe_output = "ffprobe_output.txt";
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
        let ffmpeg_settings = format!("%1 -ss {} -i \"{}\" -c copy -t {} \"{}\"",
            length_passed, full_video, clip_length, current_file_name);
        match create_file_encoding_settings(ffmpeg_settings.clone(), file_name_ffmpeg.clone()){
            Ok(ok) => ok,
            Err(_err) => {
                let error_messege = "Cannot Create File".to_string();
                return Err(error_messege);
            }
        };
        let ffmpeg_child_proccess = match Command::new("cmd").args(["/C", &file_name_ffmpeg, &ffmpeg_path]).output(){
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
        current_file_name_index += 1
    }
    return Ok(final_vec);
}
