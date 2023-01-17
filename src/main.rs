use std::{process::Command, fs::File, io::Write};

//constants
const AV1AN_LOCATION: &str = r"E:\Encoding\av1an.exe";
const AV1AN_ADDITIONAL_SETTINGS: &str = r"";
const SSIMULACRA2_LOCATION: &str = r"";
const WORKER_NUM: &str = "2";
const QUIN_NUM: &str = "255";
fn main() {
    let input_file = r"E:\a_Projects\AN\Ever\Season 1\720p_15s.mp4".to_string();
    let output_file = r"E:\a_Projects\AN\Ever\Season 1\out.mkv".to_string();
    let crf = "10".to_string();
    let _a = encode_clip(input_file, output_file, crf);
}


fn encode_clip(clip_file: String, output_name: String, crf: String) -> Result<i32, String>{
    //start encoding a clip with crf given and additional settings

    let av1an_settings: String = format!("%1 -i \"{}\" -y --verbose --keep --resume --split-method av-scenechange -m lsmash -c mkvmerge --photon-noise 2 --chroma-noise -e rav1e --force -v \"--speed {} --threads 2 --tiles 2 --quantizer {}\" --pix-format yuv420p10le -w {} -x 240 -o \"{}\""
    ,clip_file, crf, QUIN_NUM, WORKER_NUM, output_name);
    //try to start encoding
    println!("{}", av1an_settings);
    println!();
    //try to create a file to encode with
    let encoding_file_name = match create_file_encoding_settings(av1an_settings.clone()){
        Ok(ok) => ok,
        Err(_err) => {
            let error_messege = "Cannot Create File".to_string();
            return Err(error_messege);
        }
    };
    //try start encoding
    let av1an_child_proccess = match Command::new("cmd").args(["/C", &encoding_file_name, AV1AN_LOCATION]).spawn(){
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

fn create_file_encoding_settings(settings: String) -> Result<String, i32>{
    //writes a file to encode with
    let file_name = "foo.bat";
    let mut file = match File::create(file_name) {
        Ok(ok) => ok,
        Err(_err) => return Err(3)
    };
    match file.write_all(settings.as_bytes()){
        Ok(_ok) => return Ok(file_name.to_string()),
        Err(_err) => return Err(3)
    };
    
}