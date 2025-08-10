# Fish completion for rco

# Main commands
complete -c rco -n "__fish_use_subcommand" -s h -l help -d 'Show help information'
complete -c rco -n "__fish_use_subcommand" -s V -l version -d 'Show version information'
complete -c rco -n "__fish_use_subcommand" -s v -l verbose -d 'Enable verbose output'
complete -c rco -n "__fish_use_subcommand" -s y -l yes -d 'Skip confirmation prompts'
complete -c rco -n "__fish_use_subcommand" -l model -d 'AI model to use'
complete -c rco -n "__fish_use_subcommand" -l provider -d 'AI provider to use'

# Subcommands
complete -c rco -n "__fish_use_subcommand" -f -a "config" -d 'Manage configuration settings'
complete -c rco -n "__fish_use_subcommand" -f -a "auth" -d 'Manage authentication'
complete -c rco -n "__fish_use_subcommand" -f -a "status" -d 'Show status information'
complete -c rco -n "__fish_use_subcommand" -f -a "commit" -d 'Generate and create a commit'
complete -c rco -n "__fish_use_subcommand" -f -a "prepare" -d 'Prepare commit message'
complete -c rco -n "__fish_use_subcommand" -f -a "hook" -d 'Manage git hooks'
complete -c rco -n "__fish_use_subcommand" -f -a "commitlint" -d 'Lint commit messages'
complete -c rco -n "__fish_use_subcommand" -f -a "mcp" -d 'Model Context Protocol server'
complete -c rco -n "__fish_use_subcommand" -f -a "help" -d 'Show help information'

# Config subcommands
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "get" -d 'Get a configuration value'
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "set" -d 'Set a configuration value'
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "list" -d 'List all configuration values'
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "status" -d 'Show configuration status'
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "wizard" -d 'Run configuration wizard'
complete -c rco -n "__fish_seen_subcommand_from config" -f -a "generate-config" -d 'Generate default configuration'

# Auth subcommands
complete -c rco -n "__fish_seen_subcommand_from auth" -f -a "login" -d 'Authenticate with a provider'
complete -c rco -n "__fish_seen_subcommand_from auth" -f -a "logout" -d 'Remove authentication'
complete -c rco -n "__fish_seen_subcommand_from auth" -f -a "status" -d 'Show authentication status'

# Hook subcommands
complete -c rco -n "__fish_seen_subcommand_from hook" -f -a "install" -d 'Install git hooks'
complete -c rco -n "__fish_seen_subcommand_from hook" -f -a "uninstall" -d 'Remove git hooks'