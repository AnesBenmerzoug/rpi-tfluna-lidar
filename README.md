# Raspberry Pi TF-Luna Lidar

This repository contains code for using the TF-Luna Lidar sensor with a raspberry pi and visualizing the data.

## Package

- [tfluna](tfluna/README.md): Raspberry Pi with the TF-Luna LiDAR.

  Run the code with:

  ```shell
  cargo run --release --package tfluna
  ```

- [tfluna_pan_tilt](tfluna_pan_tilt/README.md): Raspberry Pi with the TF-Luna LiDAR mounted on a pan-tilt mechanism with two servos.

  Run the code with:

  ```shell
  cargo run --release --package tfluna_pan_tilt
  ```

## Hardware

- [Raspberry Pi 3 model B.](https://www.raspberrypi.com/products/raspberry-pi-3-model-b/)
- [TF-Luna LiDAR.](https://en.benewake.com/TFLuna/index.html)
- Computer to cross-compile the code and to run the rerun server and visualization.

## Getting Started

- Set up Raspberry Pi with SSH access
- [Install Rust](https://www.rust-lang.org/tools/install) 1.86
- [Install Rerun viewer](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer)

# License

This package is licensed under the [LGPL-2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html) license.
