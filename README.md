# Raspberry Pi Lidar

This repository contains code for using the TF-Luna Lidar sensor with a raspberry pi and visualizing the data.

![Wiring diagram of TF-Luna to the Raspberry Pi 3 model B](images/wiring_diagram.svg)

## Hardware

- [Raspberry Pi 3 model B.](https://www.raspberrypi.com/products/raspberry-pi-3-model-b/)
- [TF-Luna LiDAR.](https://en.benewake.com/TFLuna/index.html)
- Computer to cross-compile the code and to run the rerun server and visualization.

## Getting Started

- Set up Raspberry Pi with SSH access
- [Install Rust](https://www.rust-lang.org/tools/install) 1.86
- [Install Rerun viewer](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer)
- Start Rerun viewer using:

  ```shell
  rerun
  ```

  This will open a new window for the viewer and a server that will start listening on 0.0.0.0:9876 (by default)

- Build, copy and run code using:

  ```shell
  cargo run
  ```

  This will build the code, copy the binary to the Raspberry Pi using scp,
  and run the binary inside the Raspberry Pi.

  Once the binary is running, it will start reading data from the TF-Luna sensor
  and stream the data to the rerun server.

# License

This package is licensed under the [LGPL-2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html) license.
