# Raspberry Pi Lidar

This repository contains code for using the TF-Luna Lidar sensor with a raspberry pi and visualizing the data.

## Hardware

- Raspberry Pi 3 B
- TF-Luna Lidar

## Getting Started

- Set up Raspberry Pi with SSH access
- [Install Rust](https://www.rust-lang.org/tools/install) 1.86
- [Install Rerun viewer](https://rerun.io/docs/getting-started/installing-viewer#installing-the-viewer)
- Start Rerun viewer using:

  ```shell
  rerun
  ```

  This will open a new window and will start listening on 0.0.0.0:9876 (by default)

- Build, copy and run code using:

  ```shell
  cargo run
  ```

  This will build the code, copy the binary to the Raspberry Pi using scp,
  and run the binary inside the Raspberry Pi.

  Once the the
