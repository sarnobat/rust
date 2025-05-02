#!/bin/sh

#----------------------------------------------------------------------------
# DESCRIPTION		
# DATE				2025
# AUTHOR			ss401533@gmail.com                                           
#----------------------------------------------------------------------------
# template found at ~/.vim/sh_header.temp

set -o errexit
echo `date +%s::`"$0" >> ~/db.git/command_history.txt >> ~/db.git/command_history_timestamped.txt

cat <<EOF | batcat --style=plain --paging=never --language sh --theme TwoDark
TODO: return true if current day of year modulo 13 (or whatever frequency) is zero.

This will help with cron jobs, because all the following fire on day 1 too which causes a flurry of career page launches:

* * */13 * * 
* * */15 * *
* * */16 * *

Even if they didn't, it leaves a lot of gaps

See also
--------
/Volumes/git/github/python/05_datemod/datemod.py
EOF

