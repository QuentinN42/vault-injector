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
    kubectl get "pods/$(hostname)" -o json | jq -rc '.metadata.annotations'
}


# # Main functions


function get_config() {
    annotations=$(get_current_pod_annotations)
    echo "${annotations}"

    version=$(echo "${annotations}" | jq -cr '.["vault-injector.io/version"]' || raise "Failed to get version")

    if [[ ! "${version}" = "dev" ]]; then
        raise "Invalid version '${version}'"
    fi

    config=$(echo "${annotations}" | jq -cr '.["vault-injector.io/config"]' || raise "Failed to get config")

    # Parse env
    for env in $(echo "${config}" | jq -rc '.env[] | .');
    do
        name=$(echo "${env}" | jq -rc '.name')
        field=$(echo "${env}" | jq -rc '.field')
        engine=$(echo "${env}" | jq -rc '.engine')
        secret=$(echo "${env}" | jq -rc '.secret')
        if [[ -z "${name}" ]] || [[ -z "${field}" ]];
        then
            if [[ -z "${name}" ]] && [[ -z "${field}" ]];
            then
                raise "Invalid env config, need name or field"
            fi
            if [[ -z "${name}" ]];
            then
                name="${field}"
            else
                field="${name}"
            fi
        fi
        if [[ -z "${engine}" ]] || [[ -z "${secret}" ]];
        then
            raise "Invalid env config, need engine and secret"
        fi
        val=$(read_vault "${engine}" "${secret}" "${field}")
        echo "${name} = ${val}"
    done
}


function main() {
    check_env VAULT_ADDR VAULT_PASS VAULT_USER

    login_vault || raise "Failed to login to vault"
}

# main
