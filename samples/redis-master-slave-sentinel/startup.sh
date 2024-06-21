#!/usr/bin/env bash

if [ $# -eq 1 ]; then
  REDIS_COUNT=$1
else
  REDIS_COUNT=3
fi

# Remove previous config file
rm -rf /tmp/redis-concentrator/

REDIS_PORT=6000
REDIS_FILENAME="redis-server.conf"
REDIS_SENTINEL_FILENAME="redis-sentinel.conf"

export REDIS_PORT
export REDIS_SENTINEL_PORT

for i in $(seq 1 "${REDIS_COUNT}"); do
    echo "⚙  [Node $i] Create redis configuration"

    REDIS_NODE_DIR="/tmp/redis-concentrator/sample/redis-master-slave-sentinel/node-${REDIS_PORT}"
    
    mkdir -p "${REDIS_NODE_DIR}"
    cat "redis/${REDIS_FILENAME}" | envsubst > "${REDIS_NODE_DIR}/${REDIS_FILENAME}"
    
    REDIS_SENTINEL_PORT="2${REDIS_PORT}"

    REDIS_SENTINEL_DIR="/tmp/redis-concentrator/sample/redis-master-slave-sentinel/sentinel-${REDIS_SENTINEL_PORT}"

    mkdir -p "${REDIS_SENTINEL_DIR}"
    cat "redis/${REDIS_SENTINEL_FILENAME}" | envsubst > "${REDIS_SENTINEL_DIR}/${REDIS_SENTINEL_FILENAME}"

    echo "🚀 [Node $i] Start node     - port ${REDIS_PORT}"

    redis-server "${REDIS_NODE_DIR}/${REDIS_FILENAME}" > /dev/null &

    echo "🔍 [Node $i] Start sentinel - port ${REDIS_SENTINEL_PORT}"
    
    redis-server "${REDIS_SENTINEL_DIR}/${REDIS_SENTINEL_FILENAME}" --sentinel > /dev/null &

    echo ""

    REDIS_PORT=$(($REDIS_PORT + 1))
done

