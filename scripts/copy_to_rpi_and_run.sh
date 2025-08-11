#!/bin/bash

# Default values
DEFAULT_HOST="raspberrypi"
DEFAULT_DESTINATION="~/Projects/main"

# Function to display usage information
usage() {
    echo "Usage: $0 FILE [options]"
    echo "Positional arguments:"
    echo "  FILE                  Source file to copy and run (required)"
    echo "Options:"
    echo "  -h, --host HOST       Specify the target host (default: $DEFAULT_HOST)"
    echo "  -d, --dest DEST       Specify the destination path (default: $DEFAULT_DESTINATION)"
    echo "  --help                Display this help message"
    exit 1
}

# Check for help flag
if [[ "$1" == "--help" ]]; then
    usage
fi

# Check if file argument is provided
if [[ $# -eq 0 ]]; then
    echo "Error: Source file argument is required."
    usage
fi

# First argument is the file
FILE="$1"
shift

# Parse remaining command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -h|--host) HOST="$2"; shift ;;
        -d|--dest) DESTINATION="$2"; shift ;;
        *) echo "Unknown parameter passed: $1"; usage ;;
    esac
    shift
done

# Set default values if not provided
HOST="${HOST:-$DEFAULT_HOST}"
DESTINATION="${DESTINATION:-$DEFAULT_DESTINATION}"

# Validate source file exists
if [ ! -f "$FILE" ]; then
    echo "Error: Source file '$FILE' does not exist or is not a regular file."
    exit 1
fi

# Validate host is not empty
if [ -z "$HOST" ]; then
    echo "Error: Target host cannot be empty."
    exit 1
fi

# Validate destination is not empty
if [ -z "$DESTINATION" ]; then
    echo "Error: Destination path cannot be empty."
    exit 1
fi

# Execute SCP command
echo "Copying $FILE to $HOST:$DESTINATION"
scp "$FILE" "$HOST:$DESTINATION"

# Check if scp command was successful
if [ $? -eq 0 ]; then
    echo "File copied successfully."
else
    echo "Error: File copy failed."
    exit 1
fi

# Run copied binary file
echo "Running $FILE on $HOST:$DESTINATION"
ssh $HOST $DESTINATION
