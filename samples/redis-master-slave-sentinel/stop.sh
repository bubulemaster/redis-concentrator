#!/usr/bin/env bash

# From https://stackoverflow.com/questions/1058047/wait-for-a-process-to-finish
#
# $1 : pid
# $2 : max wait in second
wait_exit() {
    local pid
    local max_wait

    pid="$1"
    max_wait="$2"

    while s=$(ps -p ${pid} -o s=) && [[ "$s" && "$s" != 'Z' ]]; do
        echo -n "."
        sleep 1

        wait_time=$(($wait_time + 1))

        if [ "${max_wait}" -lt "${wait_time}" ]; then
            return 1
        fi
    done

    return 0
}

# $1 : path pattern
# $2 : pid file
kill_server() {
    local path_pattern
    local pid_file

    path_pattern="$1"
    pid_file="$2"

    for server in ${path_pattern}; do
    (
        cd "${server}"
        pid="$(cat ${pid_file})"
        
        echo -n "⛔ [Node] Stop node ${server}"

        kill "${pid}"

        wait_exit "${pid}" "10"

        echo ""

        if [ $? -ne 0 ]; then            
            echo "💀 [Node $i] Kill node ${server}"

            kill -s kill "${pid}"
        fi
    )
    done
}

kill_server "/tmp/redis-concentrator/sample/redis-master-slave-sentinel/sentinel-*" "redis-sentinel.pid"

kill_server "/tmp/redis-concentrator/sample/redis-master-slave-sentinel/node-*" "redis-server.pid"
