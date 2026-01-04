#!/usr/bin/env nu
# Pre-commit hook: clippy auto-fix, lint scan, treefmt

def main [] {
    if (which treefmt | is-empty) {
        print "treefmt not found. Run 'nix develop' first."
        exit 1
    }
    
    let staged_files = (
        git diff --cached --name-only --diff-filter=ACM
        | complete | get stdout | lines
        | where { $in | str trim | is-not-empty }
    )
    
    if ($staged_files | is-empty) { return }
    
    if (which cargo | is-not-empty) {
        print "Running clippy auto-fix..."
        cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged | complete | ignore
    }
    
    if (which lint | is-not-empty) {
        print "Running lint scan..."
        let result = (lint | complete)
        if $result.exit_code != 0 {
            print "Lint issues found:"
            print $result.stdout
            exit 1
        }
    }
    
    print "Running treefmt..."
    $staged_files | each {|file| ^treefmt --no-cache $file } | ignore
    
    $staged_files | where { $in | path exists } | each {|file| git add $file } | ignore
    
    print "Done."
}
