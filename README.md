# udco2s

Command line tool for UD-CO2S.

## Usage

```sh
# Get serial port
$ ls /dev/tty*
# Ubuntu
/dev/ttyACM0
# macOS
/dev/tty.usbmodem12345678
```

Reading from a port:

```sh
$ udco2s --port /dev/ttyACM0
CO2=641,HUM=56.6,TMP=29.8
CO2=642,HUM=56.5,TMP=29.8
CO2=642,HUM=56.5,TMP=29.8
...
# Stop
# CTRL+D
```

Reading from a port only once:

```sh
$ udco2s --port /dev/ttyACM0 --once
CO2=642,HUM=56.5,TMP=29.8
```

JSON Output:

```sh
$ udco2s --port /dev/ttyACM0 --format json
{"CO2":641,"HUM":56.6,"TMP":29.8}
{"CO2":642,"HUM":56.6,"TMP":29.8}
{"CO2":642,"HUM":56.6,"TMP":29.8}
...
```

```sh
$ udco2s --port /dev/ttyACM0 --format json --once
{"CO2":641,"HUM":56.6,"TMP":29.8}

$ udco2s --port /dev/ttyACM0 --format json --once | jq
{
  "CO2": 641,
  "HUM": 56.6,
  "TMP": 29.8
}
```

### Options

```sh
$ udco2s -h
Usage: udco2s [OPTIONS] --port <PORT>

Options:
      --port <PORT>      Device path to a serial port
      --format <FORMAT>  Output format [default: kv] [possible values: json, kv]
      --once             Process the output from a serial port only once and then exit
  -h, --help             Print help
  -V, --version          Print version
```

## Development

## Build

```sh
$ cargo run -- --port /dev/ttyACM0
$ cargo build

# cross build for Raspberry Pi 4 Model B
$ cross build --release --target armv7-unknown-linux-musleabihf
```
