# bruq

Convert [Bruno](https://www.usebruno.com/) `.bru` files to curl commands.

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

## License

MIT
