NOT WORKING YET

please put paths.json inside the release folder

## Usage:
```
Usage: check_av1_encode.exe [OPTIONS] --input-file <INPUT_FILE> --output-file <OUTPUT_FILE> --speed <SPEED> --worker-num <WORKER_NUM>

Options:
  -i, --input-file <INPUT_FILE>        File to Encode
  -o, --output-file <OUTPUT_FILE>      Encoded File Destination
  -s, --speed <SPEED>                  Encoding Speed
  -w, --worker-num <WORKER_NUM>        Amount Of Workers
  -c, --crf <CRF>                      Starting Crf [default: 45]
  -l, --clip-length <CLIP_LENGTH>      Clip Length [default: 20]
  -n, --clip-interval <CLIP_INTERVAL>  Clip Interval [default: 360]
  -h, --help                           Print help
  -V, --version                        Print version
  ```

## Requirements:
arch wsl with: ssmi2 , ffmpeg, ffprobe

av1an
