please put paths.json inside the release folder


the "ssim2" path inside paths.json is the path inside arch wsl to ssimulacra2_rs

## Usage:
```
Usage: check_av1_encode.exe [OPTIONS] --input-file <INPUT_FILE> --output-file <OUTPUT_FILE> --speed <SPEED> --worker-num <WORKER_NUM>

Options:
  -i, --input-file <INPUT_FILE>        File to Encode
  -o, --output-file <OUTPUT_FILE>      Encoded File Destination
  -s, --speed <SPEED>                  Encoding Speed
  -w, --worker-num <WORKER_NUM>        Amount Of Workers
  -c, --crf <CRF>                      Starting Crf [default: 45]
  -l, --clip-length <CLIP_LENGTH>      Clip Length in seconds [default: 20]
  -n, --clip-interval <CLIP_INTERVAL>  Clip Interval in seconds [default: 360]
  -u, --crf-option <CRF_OPTION>        select what crf to use on output video (average/smallest) [default: smallest]
  -h, --help                           Print help
  -V, --version                        Print version
  ```

# changing encoding settings:
paste your encoding settings inside json and place

INPUT

SPEED

CRF

OUTPUT

in the appropriate places



## Requirements:
arch wsl with: ssmi2

av1an, ffmpeg, ffprobe


## TODO:

multithreading
