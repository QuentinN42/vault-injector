#!/bin/bash
#
#  GNU AFFERO GENERAL PUBLIC LICENSE
#     Version 3, 19 November 2007
#
# # Needed env vars
# VAULT_ADDR
# VAULT_PASS
# VAULT_USER

# Set bash in strict mode
# Any errors will cause the script to exit
set -e
set -o pipefail


# # Utils functions


function raise(){
    error="${1:-"Unknown error"}"
    echo "Error: ${error}" >&2
    exit 1
}


function check_env() {
    for var in "$@"; do
        if [[ -z "${!var}" ]]; then
            raise "${var} is not set"
        fi
    done
}


# # Vault functions


function read_vault() {
    engine="${1}"
    secret="${2}"
    field="${3}"
    if [[ -z "${engine}" ]] || [[ -z "${secret}" ]]; then
        raise "Invalid arguments, need engine and secret"
    fi
    if [[ -z "${field}" ]]; then
        vault read -format=json "${engine}/${secret}" | jq -r '.data' || return 1
    else
        vault read -format=json -field="${field}" "${engine}/${secret}" | jq -r || return 1
    fi
}


function login_vault() {
    echo "Logging in to vault..."
    vault login -no-print -non-interactive -method=userpass username="${VAULT_USER}" password="${VAULT_PASS}" || return 1
    echo "Done"
}


# # K8s functions


function get_current_pod_annotations() {
    kubectl get "pods/$(hostname)" -o json | jq '.metadata.annotations'
}


# # Main functions


function get_config() {
    get_current_pod_annotations | jq -r '.["vault-injector.io/config"]' | jq
}


function main() {
    check_env VAULT_ADDR VAULT_PASS VAULT_USER

    login_vault || raise "Failed to login to vault"
}

# main
