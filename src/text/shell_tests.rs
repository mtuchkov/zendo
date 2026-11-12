use super::*;

fn cmds(src: &str) -> Vec<String> {
    blocks(src).into_iter().map(|b| b.command).collect()
}

// TERM-7: fenced blocks of a shell language, and blocks where each line with text has a `$ ` prompt.
#[test]
fn shell_blocks_are_shell_fences_and_prompt_blocks() {
    let src = "Run:\n\n```sh\ncargo test\nls -la\n```\n\n```rust\nfn main() {}\n```\n\n```\n$ echo hi\n```\n\n```\nplain text\n```\n";
    assert_eq!(cmds(src), vec!["cargo test\nls -la", "echo hi"]);
    let b = &blocks(src)[0];
    assert_eq!((b.fence, b.close), (2, 5), "the source lines of the fences");
    for lang in ["bash", "zsh", "fish", "shell", "console", "Bash"] {
        assert_eq!(cmds(&format!("```{lang}\nmake\n```\n")), vec!["make"], "{lang}");
    }
}

// TERM-9: from a block with prompts only the text after each `$ ` goes, the output stays.
#[test]
fn a_prompt_block_gives_only_the_commands() {
    let src = "```console\n$ echo hi\nhi\n$ ls\na.md\n```\n";
    assert_eq!(cmds(src), vec!["echo hi\nls"]);
    let untagged = "```\n$ echo hi\nhi\n```\n";
    assert!(cmds(untagged).is_empty(), "an untagged block with output is not a shell block");
}

#[test]
fn an_indented_fence_and_an_open_fence_at_the_end() {
    let src = "- step:\n\n  ```bash\n  cd docs\n  ```\n\n```sh\necho tail";
    assert_eq!(cmds(src), vec!["cd docs", "echo tail"]);
}

// TERM-1: without bracketed paste, the lines of a block become one line that runs only on enter.
#[test]
fn without_bracketed_paste_the_lines_join() {
    assert_eq!(for_shell("a\nb", true), "a\nb");
    assert_eq!(for_shell("a\nb", false), "a && b");
    assert_eq!(for_shell("cargo test \\\n  --all\nls", false), "cargo test --all && ls");
}
