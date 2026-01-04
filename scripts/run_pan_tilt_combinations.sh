#!/bin/bash

HOST="raspberrypi"
EXECUTABLE="~/Projects/main"
RERUN_DATA_FILE="data/pan_tilt.rrd"
N_REPEAT=3
SERVO_MOTOR_DELAYS=("500" "200" "100" "50" "10")
ANGLE_STEPS=("1" "5" "15.0")
RERUN_SERVER_IP=${RERUN_SERVER_IP:-"192.168.178.21"}

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

for SERVO_MOTOR_DELAY in "${SERVO_MOTOR_DELAYS[@]}"; do
    for ANGLE_STEP in "${ANGLE_STEPS[@]}"; do
        echo "=================="
        for i in $(seq $N_REPEAT); do
            echo "Iteration $i: Running executable '$EXECUTABLE' on host '$HOST' with servo motor delay $SERVO_MOTOR_DELAY ms and angle step $ANGLE_STEP deg";
            ssh $HOST "$EXECUTABLE --servo-motor-delay=$SERVO_MOTOR_DELAY --angle-step=$ANGLE_STEP --min-angle-top=0.0";
            sleep 1;
        done
    done
done

# Wait for rerun server to stop
echo "Successfully ran all combinations. Stopping rerun server"
sleep 1;
