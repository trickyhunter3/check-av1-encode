use std::{process::Command, fs::File, io::Write, env};

//constants
const AV1AN_LOCATION: &str = r"E:\Encoding\av1an.exe";
const AV1AN_ADDITIONAL_SETTINGS: &str = r"";
const SSIMULACRA2_LOCATION_INSIDE_ARCH: &str = r"/home/denisplay/ssimulacra2_bin/target/release/ssimulacra2_rs";
const ARCH_WSL_LOCATION: &str = r"C:\arch\Arch.exe";
const WORKER_NUM: &str = "2";
const QUIN_NUM: &str = "255";

fn main() {
    let input_file = r"E:\a_Projects\AN\Ever\Season 1\720p_15s.mp4".to_string();
    let output_file = r"E:\a_Projects\AN\Ever\Season 1\out.mkv".to_string();
    let crf = "10".to_string();
    //let _a = encode_clip(input_file, output_file, crf);
    let _b = ssim2_clip(input_file, output_file);
}

fn encode_clip(clip_file: String, output_name: String, crf: String) -> Result<i32, String>{
    //start encoding a clip with crf given and additional settings

    let av1an_settings: String = format!("%1 -i \"{}\" -y --verbose --keep --resume --split-method av-scenechange -m lsmash -c mkvmerge --photon-noise 2 --chroma-noise -e rav1e --force -v \"--speed {} --threads 2 --tiles 2 --quantizer {}\" --pix-format yuv420p10le -w {} -x 240 -o \"{}\""
    ,clip_file, crf, QUIN_NUM, WORKER_NUM, output_name);
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
    let av1an_child_proccess = match Command::new("cmd").args(["/C", &file_name, AV1AN_LOCATION]).spawn(){
        Ok(out) => out,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "Cannot start encoding file: ".to_string() + &clip_file + "\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    //waiting for procces to finish
    let av1an_output = match av1an_child_proccess.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "While waiting for file: ".to_string() + &clip_file + "To encode it errored\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    if av1an_output.status.success(){
        println!("Encoded successfully clip: {}", clip_file);
    }
    return Ok(0);
}

fn ssim2_clip(original_clip_file: String, encoded_clip_file: String) -> Result<Vec<i32>, String>{
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
        SSIMULACRA2_LOCATION_INSIDE_ARCH, WORKER_NUM, original_clip_file, encoded_clip_file, output_save_location);

    let file_name = "ssmi2_encode_settings.bat".to_string();
    match create_file_encoding_settings(ssmi2_settings, file_name.clone()){
        Ok(ok) => ok,
        Err(_err) => {
            let error_messege = "Cannot Create File".to_string();
            return Err(error_messege);
        }
    };

    //try start ssim2 and 
    let ssim2_child_proccess = match Command::new("cmd").args(["/C", &file_name, ARCH_WSL_LOCATION]).spawn(){
        Ok(out) => out,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "Cannot start ssmi2 file: ".to_string() + &encoded_clip_file + "\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    //waiting for procces to finish
    let ssim2_output = match ssim2_child_proccess.wait_with_output() {
        Ok(ok) => ok,
        Err(err) => {
            //send clip file that errored and the error, as Err
            let error_messege = "While waiting for file: ".to_string() + &encoded_clip_file + "To ssim2 it errored\nError: " + &err.to_string();
            return Err(error_messege);
        }
    };

    if ssim2_output.status.success(){
        println!("ssmi2 successfully clip: {}", encoded_clip_file);
    }
    return Ok(results_vec);
}

fn create_file_encoding_settings(settings: String, file_name: String) -> Result<String, i32>{
    //writes a file to encode with
    let mut file = match File::create(file_name) {
        Ok(ok) => ok,
        Err(_err) => return Err(3)
    };
    match file.write_all(settings.as_bytes()){
        Ok(_ok) => return Ok("Success".to_string()),
        Err(_err) => return Err(3)
    };
    
}

