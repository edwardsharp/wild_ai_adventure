//! Wordlist management module
//!
//! This module handles generation and validation of wordlists used for creating
//! memorable invite codes. It provides commands to generate wordlists from various
//! sources and validate existing wordlists.

use clap::Subcommand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Subcommand, Clone)]
pub enum WordlistCommands {
    /// Generate a new wordlist file
    Generate {
        /// Output file path (default: assets/config/wordlist.txt)
        #[arg(short, long, default_value = "assets/config/wordlist.txt")]
        output: String,
        /// Number of words to include
        #[arg(short, long, default_value = "100")]
        count: usize,
        /// Use built-in silly/fun words
        #[arg(long)]
        silly: bool,
        /// Use built-in animals
        #[arg(long)]
        animals: bool,
        /// Use built-in food words
        #[arg(long)]
        food: bool,
        /// Mix all categories (default)
        #[arg(long)]
        mixed: bool,
    },
    /// Validate an existing wordlist
    Validate {
        /// Wordlist file path (default: assets/config/wordlist.txt)
        #[arg(short, long, default_value = "assets/config/wordlist.txt")]
        file: String,
    },
    /// Show statistics about the wordlist
    Stats {
        /// Wordlist file path (default: assets/config/wordlist.txt)
        #[arg(short, long, default_value = "assets/config/wordlist.txt")]
        file: String,
    },
}

impl WordlistCommands {
    pub async fn handle(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            WordlistCommands::Generate {
                output,
                count,
                silly,
                animals,
                food,
                mixed,
            } => Self::generate_wordlist(output, *count, *silly, *animals, *food, *mixed).await,
            WordlistCommands::Validate { file } => Self::validate_wordlist(file).await,
            WordlistCommands::Stats { file } => Self::show_stats(file).await,
        }
    }

    async fn generate_wordlist(
        output: &str,
        count: usize,
        silly: bool,
        animals: bool,
        food: bool,
        mixed: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎲 Generating wordlist with {} words...", count);

        // Determine which categories to use
        let use_all = !silly && !animals && !food || mixed;
        let mut word_pool = Vec::new();

        if silly || use_all {
            word_pool.extend_from_slice(&SILLY_WORDS);
            println!("  📝 Added silly/fun words");
        }

        if animals || use_all {
            word_pool.extend_from_slice(&ANIMAL_WORDS);
            println!("  🐾 Added animal words");
        }

        if food || use_all {
            word_pool.extend_from_slice(&FOOD_WORDS);
            println!("  🍕 Added food words");
        }

        // Remove duplicates and shuffle
        let mut unique_words: Vec<_> = word_pool
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        unique_words.shuffle(&mut thread_rng());

        // Take the requested number of words
        let selected_words: Vec<_> = unique_words.into_iter().take(count).collect();

        if selected_words.len() < count {
            println!(
                "⚠️  Warning: Only {} unique words available, requested {}",
                selected_words.len(),
                count
            );
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = Path::new(output).parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate the wordlist file
        let mut content = String::new();
        content.push_str("# Generated Wordlist for Invite Code Generation\n");
        content.push_str("# This file contains silly, fun, and memorable words\n");
        content.push_str("# for generating human-readable invite codes\n");
        content.push_str("# One word per line, automatically generated\n\n");

        for word in &selected_words {
            content.push_str(word);
            content.push('\n');
        }

        fs::write(output, content)?;

        println!("✅ Generated wordlist with {} words", selected_words.len());
        println!("   📁 Saved to: {}", output);
        println!(
            "   🎯 Entropy: ~{:.1} bits per word",
            (selected_words.len() as f64).log2()
        );
        println!(
            "   🔐 3-word codes: ~{:.0} combinations",
            (selected_words.len() as f64).powi(3)
        );

        Ok(())
    }

    async fn validate_wordlist(file: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Validating wordlist: {}", file);

        if !Path::new(file).exists() {
            return Err(format!("Wordlist file not found: {}", file).into());
        }

        let content = fs::read_to_string(file)?;
        let words = Self::parse_wordlist(&content)?;

        // Validation checks
        let mut issues = Vec::new();

        if words.len() < 50 {
            issues.push(format!(
                "Too few words: {} (recommended: at least 50)",
                words.len()
            ));
        }

        // Check for duplicates
        let unique_words: HashSet<_> = words.iter().collect();
        if unique_words.len() != words.len() {
            issues.push(format!(
                "Duplicate words found: {} total, {} unique",
                words.len(),
                unique_words.len()
            ));
        }

        // Check word lengths
        let too_short: Vec<_> = words.iter().filter(|w| w.len() < 3).collect();
        let too_long: Vec<_> = words.iter().filter(|w| w.len() > 12).collect();

        if !too_short.is_empty() {
            issues.push(format!("Words too short (< 3 chars): {:?}", too_short));
        }

        if !too_long.is_empty() {
            issues.push(format!("Words too long (> 12 chars): {:?}", too_long));
        }

        // Check for invalid characters
        let invalid_chars: Vec<_> = words
            .iter()
            .filter(|w| !w.chars().all(|c| c.is_ascii_alphabetic()))
            .collect();

        if !invalid_chars.is_empty() {
            issues.push(format!(
                "Words with invalid characters: {:?}",
                invalid_chars
            ));
        }

        if issues.is_empty() {
            println!("✅ Wordlist validation passed!");
            println!("   📊 {} words, {} unique", words.len(), unique_words.len());
            println!(
                "   🎯 Entropy: ~{:.1} bits per word",
                (words.len() as f64).log2()
            );
        } else {
            println!("❌ Wordlist validation failed:");
            for issue in issues {
                println!("   • {}", issue);
            }
            return Err("Wordlist validation failed".into());
        }

        Ok(())
    }

    async fn show_stats(file: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Wordlist Statistics: {}", file);

        if !Path::new(file).exists() {
            return Err(format!("Wordlist file not found: {}", file).into());
        }

        let content = fs::read_to_string(file)?;
        let words = Self::parse_wordlist(&content)?;

        let unique_words: HashSet<_> = words.iter().collect();
        let avg_length = words.iter().map(|w| w.len()).sum::<usize>() as f64 / words.len() as f64;
        let min_length = words.iter().map(|w| w.len()).min().unwrap_or(0);
        let max_length = words.iter().map(|w| w.len()).max().unwrap_or(0);

        println!("  📝 Total words: {}", words.len());
        println!("  🎯 Unique words: {}", unique_words.len());
        println!("  📏 Average length: {:.1} characters", avg_length);
        println!(
            "  📐 Length range: {} - {} characters",
            min_length, max_length
        );
        println!(
            "  🔐 Entropy per word: ~{:.1} bits",
            (words.len() as f64).log2()
        );
        println!();
        println!("  Combination possibilities:");
        println!("    2 words: ~{:.0}", (words.len() as f64).powi(2));
        println!("    3 words: ~{:.0}", (words.len() as f64).powi(3));
        println!("    4 words: ~{:.0}", (words.len() as f64).powi(4));

        Ok(())
    }

    /// Parse wordlist from file content, filtering comments and empty lines
    pub fn parse_wordlist(content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let words: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect();

        if words.is_empty() {
            return Err("No valid words found in wordlist".into());
        }

        Ok(words)
    }
}

// Silly/Fun Words - entertaining and memorable
const SILLY_WORDS: &[&str] = &[
    "bacon", "banana", "burp", "cheese", "clown", "disco", "fart", "funky", "giggle", "jiggly",
    "kazoo", "noodle", "pickle", "rubber", "silly", "tickle", "wiggle", "yodel", "zoom", "boing",
    "splat", "whoosh", "bonk", "plop", "fizz", "buzz", "zap", "ping", "blob", "goofy", "quirky",
    "wacky", "zany", "nutty", "loopy", "dizzy", "fuzzy", "bubbly",
];

// Animal Words - cute and memorable creatures
const ANIMAL_WORDS: &[&str] = &[
    "ant", "bat", "bee", "cat", "cow", "dog", "eel", "elk", "fox", "goat", "hen", "pig", "rat",
    "yak", "bear", "deer", "duck", "frog", "goose", "horse", "llama", "moose", "mouse", "otter",
    "panda", "sheep", "sloth", "snail", "snake", "tiger", "whale", "zebra", "bunny", "puppy",
    "kitten", "hamster", "ferret", "gecko", "iguana", "koala", "lemur", "meerkat", "octopus",
    "penguin", "quail", "rabbit", "turkey", "walrus", "wombat",
];

// Food Words - tasty and fun
const FOOD_WORDS: &[&str] = &[
    "apple",
    "bagel",
    "bread",
    "cake",
    "candy",
    "chip",
    "cookie",
    "cream",
    "donut",
    "egg",
    "fries",
    "grape",
    "honey",
    "jam",
    "kale",
    "lemon",
    "mango",
    "nuts",
    "olive",
    "pasta",
    "rice",
    "soup",
    "taco",
    "waffle",
    "pizza",
    "burger",
    "muffin",
    "pretzel",
    "salad",
    "sauce",
    "spice",
    "toast",
    "vanilla",
    "yogurt",
    "zucchini",
    "avocado",
    "broccoli",
    "carrot",
    "dumpling",
    "enchilada",
    "falafel",
    "gumbo",
    "hummus",
    "jerky",
    "kiwi",
    "lobster",
];
