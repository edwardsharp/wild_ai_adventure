# Wordlist Configuration for Invite Codes

This directory contains the wordlist used for generating memorable, word-based invite codes instead of random character strings.

## Quick Start

Generate a wordlist:
```bash
cargo run --bin cli wordlist generate
```

Generate word-based invite codes (now the default):
```bash
cargo run --bin cli users generate-invite
# Generates codes like: tiger-rubber-ferret
```

Generate traditional random codes:
```bash
cargo run --bin cli users generate-invite --random
# Generates codes like: ABC123XYZ456
```

## Files

- `wordlist.txt` - The active wordlist (generated, required)
- `wordlist.example.txt` - Example wordlist with curated fun words

## Word-Based Invite Codes

Word-based codes are:
- **More memorable**: `bacon-llama-disco` vs `XK7N2P9Q`
- **Easier to share**: Can be spoken over phone/video calls
- **Fun and engaging**: Uses silly, entertaining words
- **Still secure**: 100³ = 1M combinations for 3-word codes

## Wordlist Management

### Generate a new wordlist
```bash
# Generate with 100 mixed words (default)
cargo run --bin cli wordlist generate

# Generate with specific categories
cargo run --bin cli wordlist generate --silly --count 50
cargo run --bin cli wordlist generate --animals --food --count 80

# Custom output location
cargo run --bin cli wordlist generate -o custom/path/words.txt
```

### Validate wordlist
```bash
cargo run --bin cli wordlist validate
```

### Show statistics
```bash
cargo run --bin cli wordlist stats
```

## Word Categories

The generator includes three categories of fun words:

- **Silly/Fun**: `bacon`, `burp`, `giggle`, `kazoo`, `tickle`, `zoom`
- **Animals**: `llama`, `penguin`, `hamster`, `octopus`, `walrus`
- **Food**: `taco`, `waffle`, `mango`, `pickle`, `hummus`

## Customization

You can create your own wordlist by:

1. Copy `wordlist.example.txt` to `wordlist.txt`
2. Edit the file with your preferred words
3. Validate: `cargo run --bin cli wordlist validate`

### Wordlist Requirements

- Minimum 50 words (recommended 100+)
- Words must be 3-12 characters
- Only alphabetic characters (a-z)
- No duplicates
- One word per line
- Comments start with `#`

## Security Considerations

- **3-word codes**: ~1M combinations (comparable to 6-digit PIN)
- **4-word codes**: ~100M combinations (very secure)
- **Word count vs entropy**: More words = exponentially more secure
- **Dictionary attacks**: Choose obscure/silly words over common ones

## Example Configurations

### High Security (4 words)
```bash
cargo run --bin cli users generate-invite --words 4
# Produces: meerkat-bacon-sloth-zebra
```

### Compact but Fun (2 words)
```bash
cargo run --bin cli users generate-invite --words 2
# Produces: pizza-walrus
```

### Account Linking
```bash
cargo run --bin cli users generate-account-link username --length 16
# Uses random chars for account links (more secure)
```

## Troubleshooting

### "Wordlist not initialized"
Run: `cargo run --bin cli wordlist generate`

### "Wordlist validation failed"
Check your wordlist file meets the requirements above.

### Server startup warnings
The server will warn if wordlist is missing but continue running. Word-based codes won't be available until wordlist is generated.

## Integration

The wordlist system:
- ✅ Automatically loads at server startup
- ✅ Falls back gracefully if missing (warns but continues)
- ✅ CLI commands auto-initialize when needed
- ✅ Validates format and content
- ✅ Thread-safe global caching
