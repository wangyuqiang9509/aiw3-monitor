#!/bin/bash
# AIW3 监控系统 - 告警管理工具
# 用于管理 Alertmanager 的告警、静默期等

set -e

# 配置
ALERTMANAGER_URL="${ALERTMANAGER_URL:-http://localhost:9093}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    local missing_deps=()
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "缺少依赖: ${missing_deps[*]}"
        print_info "请安装: sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi
}

# 检查 Alertmanager 状态
check_alertmanager() {
    print_info "检查 Alertmanager 状态..."
    
    if curl -s "${ALERTMANAGER_URL}/-/healthy" > /dev/null 2>&1; then
        print_success "Alertmanager 运行正常"
        return 0
    else
        print_error "无法连接到 Alertmanager: ${ALERTMANAGER_URL}"
        return 1
    fi
}

# 列出所有告警
list_alerts() {
    print_info "获取当前告警..."
    
    local response=$(curl -s "${ALERTMANAGER_URL}/api/v2/alerts")
    
    if [ -z "$response" ]; then
        print_warning "没有活动的告警"
        return
    fi
    
    echo "$response" | jq -r '.[] | "\(.labels.alertname) - \(.labels.severity) - \(.labels.node) - \(.status.state)"' | \
    while IFS= read -r line; do
        echo "  $line"
    done
}

# 列出所有静默期
list_silences() {
    print_info "获取当前静默期..."
    
    local response=$(curl -s "${ALERTMANAGER_URL}/api/v2/silences")
    
    if [ "$response" == "[]" ]; then
        print_warning "没有活动的静默期"
        return
    fi
    
    echo "$response" | jq -r '.[] | "\(.id) - \(.comment) - \(.status.state) - \(.endsAt)"' | \
    while IFS= read -r line; do
        echo "  $line"
    done
}

# 创建静默期
create_silence() {
    local alertname="$1"
    local node="$2"
    local duration="$3"
    local comment="$4"
    
    if [ -z "$alertname" ] || [ -z "$duration" ]; then
        print_error "用法: $0 silence <alertname> [node] <duration> <comment>"
        print_info "示例: $0 silence LowTPS validator1 2h 'Maintenance window'"
        return 1
    fi
    
    print_info "创建静默期..."
    
    # 计算结束时间
    local starts_at=$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")
    local ends_at=$(date -u -d "+${duration}" +"%Y-%m-%dT%H:%M:%S.000Z")
    
    # 构建 JSON
    local matchers='[{"name":"alertname","value":"'$alertname'","isRegex":false}'
    if [ -n "$node" ] && [ "$node" != "-" ]; then
        matchers+=',{"name":"node","value":"'$node'","isRegex":false}'
    fi
    matchers+=']'
    
    local json='{
        "matchers": '$matchers',
        "startsAt": "'$starts_at'",
        "endsAt": "'$ends_at'",
        "createdBy": "alert_manager.sh",
        "comment": "'${comment:-'Manual silence'}'"
    }'
    
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$json" \
        "${ALERTMANAGER_URL}/api/v2/silences")
    
    local silence_id=$(echo "$response" | jq -r '.silenceID')
    
    if [ -n "$silence_id" ] && [ "$silence_id" != "null" ]; then
        print_success "静默期创建成功: $silence_id"
        print_info "告警: $alertname"
        [ -n "$node" ] && print_info "节点: $node"
        print_info "持续时间: $duration"
        print_info "结束时间: $ends_at"
    else
        print_error "创建静默期失败"
        echo "$response" | jq '.'
    fi
}

# 删除静默期
delete_silence() {
    local silence_id="$1"
    
    if [ -z "$silence_id" ]; then
        print_error "用法: $0 delete-silence <silence_id>"
        return 1
    fi
    
    print_info "删除静默期: $silence_id"
    
    curl -s -X DELETE "${ALERTMANAGER_URL}/api/v2/silence/${silence_id}" > /dev/null
    
    print_success "静默期已删除"
}

# 查看告警统计
alert_stats() {
    print_info "告警统计..."
    
    local response=$(curl -s "${ALERTMANAGER_URL}/api/v2/alerts")
    
    if [ -z "$response" ] || [ "$response" == "[]" ]; then
        print_warning "没有活动的告警"
        return
    fi
    
    echo ""
    echo "按严重级别分组:"
    echo "$response" | jq -r 'group_by(.labels.severity) | .[] | "\(.[0].labels.severity): \(length)"'
    
    echo ""
    echo "按类别分组:"
    echo "$response" | jq -r 'group_by(.labels.category) | .[] | "\(.[0].labels.category // "unknown"): \(length)"'
    
    echo ""
    echo "按节点分组:"
    echo "$response" | jq -r 'group_by(.labels.node) | .[] | "\(.[0].labels.node // "unknown"): \(length)"'
}

# 测试告警
test_alert() {
    local alertname="${1:-TestAlert}"
    local severity="${2:-warning}"
    
    print_info "发送测试告警..."
    
    local json='[{
        "labels": {
            "alertname": "'$alertname'",
            "severity": "'$severity'",
            "node": "test-node",
            "category": "test"
        },
        "annotations": {
            "summary": "这是一个测试告警",
            "description": "测试告警系统是否正常工作"
        },
        "startsAt": "'$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")'"
    }]'
    
    curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$json" \
        "${ALERTMANAGER_URL}/api/v2/alerts" > /dev/null
    
    print_success "测试告警已发送"
    print_info "请检查 Alertmanager UI 和邮件"
}

# 重载配置
reload_config() {
    print_info "重载 Alertmanager 配置..."
    
    curl -s -X POST "${ALERTMANAGER_URL}/-/reload" > /dev/null
    
    print_success "配置已重载"
}

# 显示帮助
show_help() {
    cat << EOF
AIW3 监控系统 - 告警管理工具

用法: $0 <command> [options]

命令:
  status              检查 Alertmanager 状态
  list                列出所有活动告警
  silences            列出所有静默期
  silence <alertname> [node] <duration> <comment>
                      创建静默期
                      示例: $0 silence LowTPS validator1 2h "维护窗口"
  delete-silence <id> 删除静默期
  stats               显示告警统计
  test [name] [severity]
                      发送测试告警
  reload              重载 Alertmanager 配置
  help                显示此帮助信息

环境变量:
  ALERTMANAGER_URL    Alertmanager URL (默认: http://localhost:9093)

示例:
  # 检查状态
  $0 status
  
  # 列出告警
  $0 list
  
  # 创建 2 小时静默期
  $0 silence LowTPS validator1 2h "计划维护"
  
  # 查看统计
  $0 stats
  
  # 发送测试告警
  $0 test

EOF
}

# 主函数
main() {
    local command="${1:-help}"
    shift || true
    
    case "$command" in
        status)
            check_dependencies
            check_alertmanager
            ;;
        list)
            check_dependencies
            check_alertmanager || exit 1
            list_alerts
            ;;
        silences)
            check_dependencies
            check_alertmanager || exit 1
            list_silences
            ;;
        silence)
            check_dependencies
            check_alertmanager || exit 1
            create_silence "$@"
            ;;
        delete-silence)
            check_dependencies
            check_alertmanager || exit 1
            delete_silence "$@"
            ;;
        stats)
            check_dependencies
            check_alertmanager || exit 1
            alert_stats
            ;;
        test)
            check_dependencies
            check_alertmanager || exit 1
            test_alert "$@"
            ;;
        reload)
            check_dependencies
            check_alertmanager || exit 1
            reload_config
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "未知命令: $command"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"

