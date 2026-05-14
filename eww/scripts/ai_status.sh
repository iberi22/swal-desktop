#!/bin/bash

# Check if an agent is running or available
check_agent() {
    agent=$1
    if command -v "$agent" &> /dev/null; then
        # For this demo, we just check if it's in path. 
        # In a real scenario, we might check for a running process or a lock file.
        echo "Active"
    else
        echo "Inactive"
    fi
}

case $1 in
    hermes) check_agent "hermes" ;;
    gemini) check_agent "gemini" ;;
    codex) check_agent "codex" ;;
esac
