vsswrap
=======

A simple wrapper around vshadow64.exe that makes it easier to script.

## Usage

```
vsswrap create C
vsswrap delete
```

## Example

I use this in a restic backup script I use in windows. Here is the script (backup.bash):

```
export RESTIC_REPOSITORY="rest:http://192.168.0.3:8000/"
export RESTIC_PASSWORD="pass"
DRIVES[0]="C"
DRIVES[1]="D"
DRIVES[2]="G"

declare -A mapping=(); 

function vss {  
  while read -r a b; do 
    mapping["$a"]="$b"; 
  done < <(./vsswrap create ${DRIVES[@]})
}

vss

restic init
restic unlock
restic backup "${mapping[D]}:/Programming" "${mapping[C]}:/Users/" "${mapping[G]}:/"
restic forget --keep-daily 7 --keep-weekly 4 --keep-monthly 6
restic prune
#restic check --read-data
rundll32.exe powrprof.dll,SetSuspendState 1,0,0

./vsswrap delete
```

I then run this in windows using PortableGit:

```
PortableGit\bin\bash.exe --login backup.bash
```