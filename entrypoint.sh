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


function raise(){
    error="${1:-"Unknown error"}"
    echo "Error: ${error}" >&2
    exit 1
}

function read_vault() {
    vault read -format=json "${1}" | jq -re ".data"
}

function login_vault() {
    echo "Logging in to vault..."
    vault login -no-print -non-interactive -method=userpass username="${VAULT_USER}" password="${VAULT_PASS}" || exit 1
}

