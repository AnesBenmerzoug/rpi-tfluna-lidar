#!/bin/bash

set -e

HOST="raspberrypi"
EXECUTABLE="~/Projects/main"
RERUN_DATA_FILE="pan_tilt.rrd"
SERVO_MOTOR_DELAYS=("50" "100")
ANGLE_STEPS=("5.0" "10.0")

# Start rerun server
echo "Starting rerun server and streaming received data to file '$RERUN_DATA_FILE'"
rerun --save $RERUN_DATA_FILE &
RERUN_SERVER_PID=$!

cleanup_rerun_server() {
    # Stop rerun server
    kill $RERUN_SERVER_PID
}
trap cleanup_rerun_server EXIT;

# Wait for rerun server to start
sleep 1;

for SERVO_MOTOR_DELAY in "${SERVO_MOTOR_DELAYS[@]}"
do
    for ANGLE_STEP in "${ANGLE_STEPS[@]}"
    do
        echo "Running executable '$EXECUTABLE' on host '$HOST' with servo motor delay $SERVO_MOTOR_DELAY ms and angle step $ANGLE_STEP deg";
        ssh $HOST "$EXECUTABLE --servo-motor-delay=$SERVO_MOTOR_DELAY --angle-step=$ANGLE_STEP";
    done
done

# Wait for rerun server to stop
echo "Successfully ran all combinations. Stopping rerun server"
sleep 1;
