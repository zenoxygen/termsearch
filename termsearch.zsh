#!/bin/zsh

# Check if the script is being run by ZSH
if [[ ! "$ZSH_VERSION" ]]; then
    echo "Error: This script must be run by ZSH."
    return 1
fi

# Function to trigger the termsearch search functionality
termsearch-search() {
    # Create a temporary file for termsearch output
    local temp_file=$(mktemp -t termsearch.XXXXXX)

    # Run termsearch search, passing the current buffer and output file
    termsearch search -o "$temp_file" "$LBUFFER"

    # Read the command line from the temporary file
    local commandline
    while IFS=$'\t' read -r key val; do
        case "$key" in
            commandline) commandline="$val" ;;
        esac
    done < "$temp_file"

    # Clean up the temporary file
    command rm -f "$temp_file"

    # Update the buffer and cursor position if a command was selected
    if [[ -n "$commandline" ]]; then
        LBUFFER="$commandline"
        CURSOR=$#LBUFFER
        zle redisplay
    fi
}

# Create the ZSH widget
zle -N termsearch-search

# Bind Ctrl+r to the termsearch-search function
bindkey '^R' termsearch-search
