#!/bin/bash

LOG=$1

if [ -z "$LOG" ]; then
    echo "Usage: $0 time_log_file"
    exit 1
fi

# Real time: convert h:mm:ss to minutes
REAL_RAW=$(grep "Elapsed (wall clock) time" "$LOG" | awk '{print $8}')
IFS=: read -r h m s <<< "$REAL_RAW"
if [ -z "$s" ]; then
    # format m:ss
    s=$m
    m=$h
    h=0
fi
REAL_MIN=$(echo "$h*60 + $m + $s/60" | bc -l | awk '{printf "%.2f", $0}')

# CPU time: User + System, convert seconds to minutes
USER=$(grep "User time" "$LOG" | awk '{print $4}')
SYS=$(grep "System time" "$LOG" | awk '{print $4}')
CPU_MIN=$(echo "($USER + $SYS)/60" | bc -l | awk '{printf "%.2f", $0}')

# Peak memory: KB -> GB
MEM=$(grep "Maximum resident set size" "$LOG" | awk '{printf "%.4f", $6/1024/1024}')

# Output
echo -e "Real_time_min\tCPU_time_min\tPeak_memory_GB"
echo -e "${REAL_MIN}\t${CPU_MIN}\t${MEM}"
