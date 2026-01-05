# Raspberry Pi TF-Luna Lidar

This repository contains code for using the TF-Luna Lidar sensor with a raspberry pi and visualizing the data.

## Hardware

- [Raspberry Pi 3 model B.](https://www.raspberrypi.com/products/raspberry-pi-3-model-b/)
- [TF-Luna LiDAR.](https://en.benewake.com/TFLuna/index.html)
- Computer to cross-compile the code and to run the rerun server and visualization.

## Getting Started

- Set up Raspberry Pi with SSH access
- [Install Rust](https://www.rust-lang.org/tools/install) 1.86
- [Install Rerun viewer](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer)

Each of the packages assumes that you have a running Rerun server that can be started with:

```shell
rerun --save pan_tilt.rrd
```

This will listen for incoming gRPC connections from the logging SDK and stream the results to the `pan_tilt.rrd` file.

Once the sending is done, you can visualize the data using:

```shell
rerun pan_tilt
```

This will open a new window for the viewer and stream the data to it and allow you to visualize.

## Packages

### [tfluna](tfluna/README.md)

Raspberry Pi with the TF-Luna LiDAR.

Run the code with:

```shell
cargo run --release --package tfluna
```

### [tfluna_pan_tilt](tfluna_pan_tilt)

Raspberry Pi with the TF-Luna LiDAR mounted on a pan-tilt mechanism with two servos.

Run the main code with:

```shell
cargo run --release --package tfluna_pan_tilt
```

Once that's done, use this to run different combinations of parameters and save the data:

```shell
bash scripts/run_pan_tilt_combinations.sh
```

### [tfluna_data_analysis](tfluna_data_analysis)

Finally, run the data analysis on that data with:

```shell
cargo run --release --package tfluna_data_analysis --target x86_64-unknown-linux-gnu
```

This will load the data from the rrd file `data/pan_tilt_combinations.rrd`, analyze it, print and plot the results, save the plots under `data/` 

> If you're not running linux or simply have a different target architecture, use this command to find the target:
>
> ```shell
> rustc --version --verbose
> ```
> 
> It will correspond to the `host` key.



# License

This package is licensed under the [LGPL-2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html) license.
