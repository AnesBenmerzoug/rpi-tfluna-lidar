# Raspberry Pi TF-Luna Lidar

This workspace contains code for using the TF-Luna Lidar sensor with a raspberry pi and visualizing the data.

![Wiring diagram of TF-Luna to the Raspberry Pi 3 model B](wiring_diagram.svg)

## Getting Started

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
