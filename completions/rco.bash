# Bash completion for rco
_rco() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Main commands
    opts="config auth status commit prepare hook commitlint mcp help"
    
    case "${prev}" in
        rco)
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
        config)
            COMPREPLY=( $(compgen -W "get set list status wizard generate-config" -- ${cur}) )
            return 0
            ;;
        auth)
            COMPREPLY=( $(compgen -W "login logout status" -- ${cur}) )
            return 0
            ;;
        hook)
            COMPREPLY=( $(compgen -W "install uninstall" -- ${cur}) )
            return 0
            ;;
        *)
            # Handle flags
            if [[ ${cur} == -* ]]; then
                COMPREPLY=( $(compgen -W "--help --version --verbose --yes --model --provider" -- ${cur}) )
                return 0
            fi
            ;;
    esac
}

complete -F _rco rco