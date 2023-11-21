_myapp() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="myapp"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        myapp)
            opts="-c -i -T -t -a -w -d -f -L -l -p -h -V --config-path --generate-autocomplete --export-config --export-default-config --active-color --init-widget --window-title --todo-path --archive-path --priority-colors --wrap-preview --list-active-color --pending-active-color --done-active-color --autosave-duration --log-file --log-format --log-level --file-watcher --list-refresh-rate --list-shift --pending-sort --done-sort --preview-format --layout --category-style --projects-style --contexts-style --hashtags-style --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --generate-autocomplete)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export-config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export-default-config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --init-widget)
                    COMPREPLY=($(compgen -W "list done project context hashtag preview" -- "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -W "list done project context hashtag preview" -- "${cur}"))
                    return 0
                    ;;
                --window-title)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --todo-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --archive-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --priority-colors)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --wrap-preview)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -w)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --list-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pending-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --done-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --autosave-duration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --log-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --log-format)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --log-level)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --file-watcher)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --list-refresh-rate)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -L)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-shift)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pending-sort)
                    COMPREPLY=($(compgen -W "none reverse priority alphanumeric alphanumeric-reverse" -- "${cur}"))
                    return 0
                    ;;
                --done-sort)
                    COMPREPLY=($(compgen -W "none reverse priority alphanumeric alphanumeric-reverse" -- "${cur}"))
                    return 0
                    ;;
                --preview-format)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --layout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --projects-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --contexts-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hashtags-style)
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

complete -F _myapp -o nosort -o bashdefault -o default myapp
