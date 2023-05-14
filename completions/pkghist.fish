complete -c pkghist -s o -l output-format -d 'Select the output format' -r -f -a "{json	,plain	,compact	}"
complete -c pkghist -s l -l logfile -d 'Specify a logfile' -r
complete -c pkghist -s L -l limit -d 'How many versions to go back in report. [limit > 0]' -r
complete -c pkghist -l first -d 'Output the first \'n\' pacman events' -r
complete -c pkghist -l last -d 'Output the last \'n\' pacman events' -r
complete -c pkghist -s a -l after -d 'Only consider events that occurred after \'date\' [Format: "YYYY-MM-DD HH:MM"]' -r
complete -c pkghist -s r -l with-removed -d 'Include packages that are currently uninstalled'
complete -c pkghist -s R -l removed-only -d 'Only output packages that are currently uninstalled'
complete -c pkghist -l no-colors -d 'Disable colored output'
complete -c pkghist -l no-details -d 'Only output the package names'
complete -c pkghist -s x -l exclude -d 'If set, every filter result will be excluded.'
complete -c pkghist -s h -l help -d 'Print help'
complete -c pkghist -s V -l version -d 'Print version'
