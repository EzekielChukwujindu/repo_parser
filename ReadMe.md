# Code Segmenter

## Overview

The Code Segmenter is a Rust-based tool designed to parse and analyze code repositories. It generates a simplified code structure to help users understand the architecture of the codebase. The tool can handle both local directories and remote Git repositories.

## Features

- **Cloning Git Repositories**: Automatically clones a Git repository into a temporary directory for analysis.
- **Code Simplification**: Processes and simplifies code files based on language-specific segmenters.
- **Error Logging**: Logs errors encountered during file processing to an `temp/_arch_/error.txt` file.
- **Directory Handling**: Saves simplified code into a parallel `_arch_` directory structure outside the original directory being processed.

## Requirements

- Rust (with `tokio` and `git2` crates)
- Compatible code segmenters for various languages (e.g., Python, JavaScript)

## Installation

To use this project, clone the repository and ensure that you have Rust installed on your system. Run the following commands to build and run the application:

```bash
git clone <repository-url>
cd <repository-directory>
cargo build
cargo run -- <git-repo-url|directory-path>
```

Replace <repository-url> with the URL of the Git repository or <directory-path> with the local directory path.

## Usage

The application requires a command-line argument specifying either a Git repository URL or a local directory path. Here's how to run the application:

```
cargo run -- <git-repo-url|directory-path>
```

<git-repo-url>: The URL of the Git repository you want to clone and analyze.
<directory-path>: The path to the local directory you want to analyze.

The application will:

1. Clone the Git repository if a URL is provided.
2. Normalize the input path.
3. Process and simplify code files based on language-specific segmenters.
4. Save the simplified code and log errors into the temp/_arch_ directory.