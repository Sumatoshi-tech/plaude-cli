//! `plaude template` command group — manage summarization templates.
//!
//! Provides list, show, add, edit, and delete operations for
//! summarization prompt templates.

use std::path::{Path, PathBuf};

use clap::Subcommand;

use crate::DispatchError;

/// Default plaude config directory when none is specified.
fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join("plaude"))
        .unwrap_or_else(|| PathBuf::from(".config/plaude"))
}

/// Template management commands.
#[derive(Debug, Subcommand)]
pub(crate) enum TemplateCommand {
    /// List all available templates (built-in + user).
    List,
    /// Show the content of a template.
    Show(ShowArgs),
    /// Create a new user template (optionally from a built-in).
    Add(AddArgs),
    /// Open a user template in $EDITOR.
    Edit(EditArgs),
    /// Delete a user template.
    #[command(alias = "rm")]
    Delete(DeleteArgs),
}

/// Arguments for `plaude template show`.
#[derive(Debug, clap::Args)]
pub(crate) struct ShowArgs {
    /// Template name.
    #[arg(value_name = "NAME")]
    name: String,
}

/// Arguments for `plaude template add`.
#[derive(Debug, clap::Args)]
pub(crate) struct AddArgs {
    /// Name for the new template.
    #[arg(value_name = "NAME")]
    name: String,

    /// Copy content from a built-in template instead of starting blank.
    #[arg(long, value_name = "BUILTIN", conflicts_with = "body")]
    from: Option<String>,

    /// Read template body from a file (use `-` for stdin).
    #[arg(long, value_name = "FILE", conflicts_with = "from")]
    body: Option<PathBuf>,
}

/// Arguments for `plaude template edit`.
#[derive(Debug, clap::Args)]
pub(crate) struct EditArgs {
    /// Template name.
    #[arg(value_name = "NAME")]
    name: String,
}

/// Arguments for `plaude template delete`.
#[derive(Debug, clap::Args)]
pub(crate) struct DeleteArgs {
    /// Template name.
    #[arg(value_name = "NAME")]
    name: String,
}

/// Run the `plaude template` subcommand.
pub(crate) fn run(cmd: TemplateCommand, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let dir = config_dir.map(PathBuf::from).unwrap_or_else(default_config_dir);

    match cmd {
        TemplateCommand::List => list_templates(&dir),
        TemplateCommand::Show(args) => show_template(&dir, &args.name),
        TemplateCommand::Add(args) => add_template(&dir, &args.name, args.from.as_deref(), args.body.as_deref()),
        TemplateCommand::Edit(args) => edit_template(&dir, &args.name),
        TemplateCommand::Delete(args) => delete_template(&dir, &args.name),
    }
}

/// `plaude template list`
fn list_templates(config_dir: &Path) -> Result<(), DispatchError> {
    use plaud_llm::template::TemplateRegistry;

    let registry = TemplateRegistry::load(config_dir);
    let infos = registry.list();

    let h_name = "NAME";
    let h_source = "SOURCE";
    let h_preview = "PREVIEW";
    println!("{h_name:<20} {h_source:<10} {h_preview}");
    let s_name = "----";
    let s_source = "------";
    let s_preview = "-------";
    println!("{s_name:<20} {s_source:<10} {s_preview}");

    for info in &infos {
        println!("{:<20} {:<10} {}", info.name, info.source, info.preview);
    }

    println!();
    println!("{} template(s) available", infos.len());
    Ok(())
}

/// `plaude template show <name>`
fn show_template(config_dir: &Path, name: &str) -> Result<(), DispatchError> {
    use plaud_llm::template::TemplateRegistry;

    let registry = TemplateRegistry::load(config_dir);
    let template = registry.get(name).ok_or_else(|| {
        DispatchError::Runtime(format!(
            "template '{name}' not found. Run `plaude template list` to see available templates"
        ))
    })?;

    println!("{}", template.body);
    Ok(())
}

/// `plaude template add <name> [--from <builtin>] [--body <file>]`
fn add_template(config_dir: &Path, name: &str, from: Option<&str>, body_path: Option<&Path>) -> Result<(), DispatchError> {
    use plaud_llm::template::{TemplateRegistry, templates_dir, user_template_path};

    let path = user_template_path(config_dir, name);
    if path.exists() {
        return Err(DispatchError::Runtime(format!(
            "template '{name}' already exists at {}. Use `plaude template edit {name}` to modify it",
            path.display()
        )));
    }

    let body = if let Some(file) = body_path {
        if file == Path::new("-") {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| DispatchError::Runtime(format!("failed to read stdin: {e}")))?;
            buf
        } else {
            std::fs::read_to_string(file).map_err(|e| DispatchError::Runtime(format!("failed to read {}: {e}", file.display())))?
        }
    } else if let Some(builtin_name) = from {
        TemplateRegistry::builtin_body(builtin_name)
            .ok_or_else(|| {
                let names: Vec<&str> = TemplateRegistry::builtin_names().iter().map(|(n, _)| *n).collect();
                DispatchError::Runtime(format!(
                    "unknown built-in template '{builtin_name}'. Available: {}",
                    names.join(", ")
                ))
            })?
            .to_owned()
    } else {
        "You are a helpful assistant. Given the transcript of a voice recording, \
produce a clear summary.\n\n## Rules\n\n- Be concise.\n- Preserve key information.\n"
            .to_owned()
    };

    // Ensure templates directory exists.
    let tpl_dir = templates_dir(config_dir);
    std::fs::create_dir_all(&tpl_dir).map_err(|e| DispatchError::Runtime(format!("failed to create templates directory: {e}")))?;

    std::fs::write(&path, &body).map_err(|e| DispatchError::Runtime(format!("failed to write template: {e}")))?;

    eprintln!("Created template '{name}' at {}", path.display());
    eprintln!("Edit with: plaude template edit {name}");
    Ok(())
}

/// `plaude template edit <name>`
fn edit_template(config_dir: &Path, name: &str) -> Result<(), DispatchError> {
    use plaud_llm::template::user_template_path;

    let path = user_template_path(config_dir, name);
    if !path.exists() {
        return Err(DispatchError::Runtime(format!(
            "template '{name}' not found. Create it first with `plaude template add {name}`"
        )));
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_owned());

    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| DispatchError::Runtime(format!("failed to launch editor '{editor}': {e}")))?;

    if status.success() {
        eprintln!("Template '{name}' saved");
    } else {
        eprintln!("Editor exited with {status}");
    }
    Ok(())
}

/// `plaude template delete <name>`
fn delete_template(config_dir: &Path, name: &str) -> Result<(), DispatchError> {
    use plaud_llm::template::{TemplateRegistry, user_template_path};

    // Prevent deleting built-ins that have no user override.
    let path = user_template_path(config_dir, name);
    if !path.exists() {
        if TemplateRegistry::builtin_body(name).is_some() {
            return Err(DispatchError::Runtime(format!(
                "'{name}' is a built-in template and cannot be deleted"
            )));
        }
        return Err(DispatchError::Runtime(format!("template '{name}' not found")));
    }

    std::fs::remove_file(&path).map_err(|e| DispatchError::Runtime(format!("failed to delete template: {e}")))?;

    eprintln!("Deleted template '{name}'");

    if TemplateRegistry::builtin_body(name).is_some() {
        eprintln!("Note: built-in '{name}' is now restored as the default");
    }
    Ok(())
}
