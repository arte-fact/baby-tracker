# baby-tracker

A CLI tool to track baby feeding activity. Built with Rust.

## Features

- Track feedings: breast (left/right), bottle, and solid food
- Record amount (ml), duration (minutes), and notes
- List recent feedings in a formatted table
- View feeding summaries and statistics
- SQLite-backed persistent storage

## Usage

```sh
# Add a bottle feeding
baby-tracker add --name "Emma" --type bottle --amount 120

# Add a breastfeeding session
baby-tracker add --name "Emma" --type breast-left --duration 15

# Add solid food
baby-tracker add --name "Emma" --type solid --notes "Banana puree"

# Add with a specific time
baby-tracker add --name "Emma" --type bottle --amount 90 --time "2026-02-15 08:00"

# List recent feedings
baby-tracker list
baby-tracker list --name "Emma" --limit 20

# View today's summary
baby-tracker summary --name "Emma"
baby-tracker summary --days 7

# Delete a feeding
baby-tracker delete 3
```

## Feeding Types

| Type | Short | Description |
|------|-------|-------------|
| `breast-left` | `bl` | Left breast |
| `breast-right` | `br` | Right breast |
| `bottle` | `b` | Bottle feeding |
| `solid` | `s` | Solid food |

## Building

```sh
cargo build --release
```
