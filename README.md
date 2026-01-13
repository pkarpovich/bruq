# bruq

Convert [Bruno](https://www.usebruno.com/) `.bru` files to curl commands.

Perfect for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) and other AI coding assistants - keep your API requests in Bruno as the single source of truth, and let your AI execute them via curl.

## Installation

### Homebrew (macOS)

```bash
brew install pkarpovich/apps/bruq
```

### From source

```bash
cargo install --git https://github.com/pkarpovich/bruq
```

## Usage

```bash
# Generate curl command
bruq path/to/request.bru

# With environment variables
bruq path/to/request.bru -e Local

# With curl flags
bruq path/to/request.bru -e Local -v    # verbose
bruq path/to/request.bru -e Local -s    # silent
```

### Execute the request

```bash
# bash/zsh
eval "$(bruq request.bru -e Local)"

# fish
eval (bruq request.bru -e Local)
```

## Example

Given a Bruno request file:

```
post {
  url: {{API_URL}}/users
  body: json
}

body:json {
  {
    "name": "John"
  }
}
```

And environment file `environments/Local.bru`:

```
vars {
  API_URL: https://api.example.com
}
```

Running:

```bash
bruq request.bru -e Local
```

Outputs:

```bash
curl -X POST 'https://api.example.com/users' -H 'Content-Type: application/json' -d '{"name": "John"}'
```

## Use with Claude Code

Instead of manually crafting curl commands or having Claude guess API structures, point it to your Bruno collection:

```
Run the "Create User" request from my Bruno collection at ./api/users.bru with Local environment
```

Claude Code can then:
```bash
eval "$(bruq ./api/users.bru -e Local)"
```

This keeps Bruno as your single source of truth for API requests while letting AI assistants execute them reliably.

## License

MIT
