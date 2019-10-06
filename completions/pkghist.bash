_pkghist() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            pkghist)
                cmd="pkghist"
                ;;
            
            *)
                ;;
        esac
    done

    case "${cmd}" in
        pkghist)
            opts=" -r -R -x -h -V -o -l -L -a  --with-removed --removed-only --no-colors --no-details --exclude --help --version --output-format --logfile --limit --first --last --after  <filter>... "
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                
                --output-format)
                    COMPREPLY=($(compgen -W "json plain compact" -- "${cur}"))
                    return 0
                    ;;
                    -o)
                    COMPREPLY=($(compgen -W "json plain compact" -- "${cur}"))
                    return 0
                    ;;
                --logfile)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -L)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --first)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --last)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --after)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                    -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        
    esac
}

complete -F _pkghist -o bashdefault -o default pkghist
