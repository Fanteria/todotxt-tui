_todotxt-tui() {
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
                cmd="todotxt__tui"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        todotxt__tui)
            opts="-i -t -W -R -S -l -d -f -T -C -s -L -p -w -A -P -D -c -h -V --init-widget --window-title --window-keybinds --list-refresh-rate --save-state-path --layout --use-done --pending-sort --done-sort --delete-final-date --set-final-date --todo-path --archive-path --autosave-duration --file-watcher --save-policy --tasks-keybind --category-keybind --border-type --list-shift --list-keybind --preview-format --wrap-preview --list-active-color --pending-active-color --done-active-color --category-active-color --projects-active-color --contexts-active-color --tags-active-color --active-color --priority-style --projects-style --contexts-style --hashtags-style --category-style --category-select-style --category-remove-style --custom-category-style --highlight --pre-new-task --post-new-task --pre-remove-task --post-remove-task --pre-move-task --post-move-task --pre-update-task --post-update-task --export-autocomplete --export-config --export-default-config --config-path --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
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
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --window-keybinds)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -W)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-refresh-rate)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --save-state-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -S)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --layout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --use-done)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
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
                --delete-final-date)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --set-final-date)
                    COMPREPLY=($(compgen -W "override only-missing never" -- "${cur}"))
                    return 0
                    ;;
                --todo-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --archive-path)
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
                --file-watcher)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --save-policy)
                    COMPREPLY=($(compgen -W "manual-only auto-save on-change" -- "${cur}"))
                    return 0
                    ;;
                --tasks-keybind)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -T)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category-keybind)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -C)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --border-type)
                    COMPREPLY=($(compgen -W "plain rounded double thick" -- "${cur}"))
                    return 0
                    ;;
                --list-shift)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-keybind)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -L)
                    COMPREPLY=($(compgen -f "${cur}"))
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
                -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pending-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --done-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -D)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --projects-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --contexts-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tags-active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --active-color)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --priority-style)
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
                --category-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category-select-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category-remove-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --custom-category-style)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --highlight)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pre-new-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-new-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pre-remove-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-remove-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pre-move-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-move-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pre-update-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-update-task)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export-autocomplete)
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
                --config-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
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

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _todotxt-tui -o nosort -o bashdefault -o default todotxt-tui
else
    complete -F _todotxt-tui -o bashdefault -o default todotxt-tui
fi
