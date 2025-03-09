#!/bin/sh

#----------------------------------------------------------------------------
# DESCRIPTION		
# DATE				2025
# AUTHOR			ss401533@gmail.com                                           
#----------------------------------------------------------------------------
# template found at ~/.vim/sh_header.temp

set -o errexit
echo "$0" >> ~/db.git/command_history.txt | ts >> ~/db.git/command_history_timestamped.txt

cat <<EOF | batcat --style=plain --paging=never --language sh --theme TwoDark
cargo new my_cli_app && cd my_cli_app
cargo build
./target/debug/my_cli_app --help
./target/debug/my_cli_app -v -i input.txt
EOF

