//! Interactive prompts and confirmations

use std::io::{self, Write};

use crate::error::Result;

/// Prompt for yes/no confirmation
pub fn confirm(prompt: &str, default_yes: bool) -> Result<bool> {
    let default_str = if default_yes { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        return Ok(default_yes);
    }

    Ok(input == "y" || input == "yes")
}

/// Prompt for text input
pub fn prompt(message: &str) -> Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt for password with confirmation
pub fn prompt_password_with_confirm(prompt: &str) -> Result<String> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?;
        let pass1 = rpassword::read_password()?;

        print!("Confirm password: ");
        io::stdout().flush()?;
        let pass2 = rpassword::read_password()?;

        if pass1 == pass2 {
            if pass1.is_empty() {
                println!("⚠️  Password cannot be empty. Please try again.");
                continue;
            }
            return Ok(pass1);
        } else {
            println!("⚠️  Passwords do not match. Please try again.");
        }
    }
}

/// Prompt for password
pub fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    let password = rpassword::read_password()?;
    Ok(password)
}
