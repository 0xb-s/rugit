// src/git_utils.rs

use anyhow::{Context, Result};
use git2::{AnnotatedCommit, BranchType, Error, Repository, Signature};

/// Creates a new branch with the given name based on the current HEAD.
pub fn create_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    // Check if branch already exists
    if repo.find_branch(branch_name, BranchType::Local).is_ok() {
        anyhow::bail!("Branch '{}' already exists.", branch_name);
    }

    let head = repo
        .head()
        .context("Failed to get HEAD")?
        .peel_to_commit()
        .context("Failed to peel HEAD to commit")?;

    repo.branch(branch_name, &head, false)
        .with_context(|| format!("Failed to create branch '{}'", branch_name))?;

    Ok(())
}

/// Deletes the specified branch, ensuring it's not the current branch.
pub fn delete_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let head = repo
        .head()
        .context("Failed to get HEAD")?
        .shorthand()
        .unwrap_or("")
        .to_string();

    if head == branch_name {
        anyhow::bail!("Cannot delete the current active branch '{}'.", branch_name);
    }

    let mut branch = repo
        .find_branch(branch_name, BranchType::Local)
        .with_context(|| format!("Branch '{}' not found.", branch_name))?;

    branch
        .delete()
        .with_context(|| format!("Failed to delete branch '{}'", branch_name))?;

    Ok(())
}

/// Switches to the specified branch.
pub fn switch_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let annotated = repo
        .find_annotated_commit(
            repo.refname_to_id(&format!("refs/heads/{}", branch_name))
                .with_context(|| format!("Branch '{}' not found.", branch_name))?,
        )
        .with_context(|| {
            format!(
                "Failed to find annotated commit for branch '{}'",
                branch_name
            )
        })?;

    repo.set_head(&format!("refs/heads/{}", branch_name))
        .with_context(|| format!("Failed to set HEAD to '{}'", branch_name))?;

    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            .allow_conflicts(true)
            .force(),
    ))
    .context("Failed to checkout branch")?;

    Ok(())
}

/// Adds files to the staging area.
pub fn add_files(repo_path: &str, files: &[String]) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let mut index = repo.index().context("Failed to get repository index")?;

    for file in files {
        index
            .add_path(std::path::Path::new(file))
            .with_context(|| format!("Failed to add file '{}'", file))?;
    }

    index.write().context("Failed to write to index")?;

    Ok(())
}

/// Commits staged changes with the provided message.
pub fn commit_changes(repo_path: &str, message: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let mut index = repo.index().context("Failed to get repository index")?;

    if index.is_empty() {
        anyhow::bail!("No changes to commit.");
    }

    let tree_id = index.write_tree().context("Failed to write tree")?;
    let tree = repo
        .find_tree(tree_id)
        .context("Failed to find written tree")?;

    let signature = repo
        .signature()
        .context("Failed to get repository signature")?;

    let parent_commit = match repo.head() {
        Ok(head) => head
            .peel_to_commit()
            .context("Failed to peel HEAD to commit")?,
        Err(_) => {
            // No commits yet, initial commit
            repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
                .context("Failed to create initial commit")?;
            return Ok(());
        }
    };

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )
    .with_context(|| "Failed to create commit")?;

    Ok(())
}

/// Merges the specified branch into the current branch.
pub fn merge_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let current_branch = repo
        .head()
        .context("Failed to get HEAD")?
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("Invalid HEAD"))?
        .to_string();

    if current_branch == branch_name {
        anyhow::bail!("Cannot merge branch '{}' into itself.", branch_name);
    }

    let merge_branch = repo
        .find_branch(branch_name, BranchType::Local)
        .with_context(|| format!("Branch '{}' not found.", branch_name))?;

    let merge_commit = merge_branch
        .get()
        .peel_to_commit()
        .context("Failed to peel branch to commit")?;

    // Find AnnotatedCommit
    let annotated_merge_commit = repo
        .find_annotated_commit(merge_commit.id())
        .context("Failed to find annotated commit for merge")?;

    let analysis = repo
        .merge_analysis(&[&annotated_merge_commit])
        .context("Failed to perform merge analysis")?;

    if analysis.0.is_up_to_date() {
        anyhow::bail!("Branch '{}' is already up-to-date.", branch_name);
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo
            .find_reference(&refname)
            .context("Failed to find reference for fast-forward")?;
        reference
            .set_target(merge_commit.id(), "Fast-Forward Merge")
            .context("Failed to set target for fast-forward")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .context("Failed to checkout head after fast-forward")?;
    } else if analysis.0.is_normal() {
        repo.merge(&[&annotated_merge_commit], None, None)
            .context("Failed to merge branches")?;

        if repo.index()?.has_conflicts() {
            anyhow::bail!("Merge conflicts detected. Please resolve them manually.");
        }

        let signature = repo
            .signature()
            .context("Failed to get repository signature")?;

        let head_commit = repo
            .head()
            .context("Failed to get HEAD")?
            .peel_to_commit()
            .context("Failed to peel HEAD to commit")?;

        let merge_commit = repo
            .find_commit(merge_commit.id())
            .context("Failed to find merge commit")?;

        let tree_id = repo
            .index()?
            .write_tree()
            .context("Failed to write tree after merge")?;
        let tree = repo
            .find_tree(tree_id)
            .context("Failed to find tree after merge")?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Merge branch '{}'", branch_name),
            &tree,
            &[&head_commit, &merge_commit],
        )
        .context("Failed to create merge commit")?;
    } else {
        anyhow::bail!("Merge analysis returned unknown status.");
    }

    Ok(())
}

/// Adds a remote repository.
pub fn add_remote(repo_path: &str, remote_name: &str, remote_url: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    repo.remote(remote_name, remote_url).with_context(|| {
        format!(
            "Failed to add remote '{}' with URL '{}'",
            remote_name, remote_url
        )
    })?;

    Ok(())
}

/// Removes a remote repository.
pub fn remove_remote(repo_path: &str, remote_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    repo.remote_delete(remote_name)
        .with_context(|| format!("Failed to remove remote '{}'", remote_name))?;

    Ok(())
}

/// Pushes the current branch to the specified remote.
pub fn push_branch(repo_path: &str, remote_name: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let mut remote = repo
        .find_remote(remote_name)
        .with_context(|| format!("Remote '{}' not found.", remote_name))?;

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], None).with_context(|| {
        format!(
            "Failed to push branch '{}' to remote '{}'",
            branch_name, remote_name
        )
    })?;

    Ok(())
}

/// Pulls the latest changes from the specified remote and branch.
pub fn pull_branch(repo_path: &str, remote_name: &str, branch_name: &str) -> Result<()> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repository at '{}'", repo_path))?;

    let mut remote = repo
        .find_remote(remote_name)
        .with_context(|| format!("Remote '{}' not found.", remote_name))?;

    let annotated = repo
        .find_annotated_commit(
            repo.refname_to_id(&format!("refs/heads/{}", branch_name))
                .with_context(|| format!("Branch '{}' not found.", branch_name))?,
        )
        .with_context(|| {
            format!(
                "Failed to find annotated commit for branch '{}'",
                branch_name
            )
        })?;

    remote.fetch(&[branch_name], None, None).with_context(|| {
        format!(
            "Failed to fetch branch '{}' from remote '{}'",
            branch_name, remote_name
        )
    })?;

    let analysis = repo
        .merge_analysis(&[&annotated])
        .context("Failed to perform merge analysis")?;

    if analysis.0.is_up_to_date() {
        anyhow::bail!("Branch '{}' is already up-to-date.", branch_name);
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo
            .find_reference(&refname)
            .context("Failed to find reference for fast-forward")?;
        reference
            .set_target(annotated.id(), "Fast-Forward Merge")
            .context("Failed to set target for fast-forward")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .context("Failed to checkout head after fast-forward")?;
    } else if analysis.0.is_normal() {
        repo.merge(&[&annotated], None, None)
            .context("Failed to merge fetched changes")?;

        if repo.index()?.has_conflicts() {
            anyhow::bail!("Merge conflicts detected during pull. Please resolve them manually.");
        }

        let signature = repo
            .signature()
            .context("Failed to get repository signature")?;

        let head_commit = repo
            .head()
            .context("Failed to get HEAD")?
            .peel_to_commit()
            .context("Failed to peel HEAD to commit")?;

        let merge_commit = repo
            .find_commit(annotated.id())
            .context("Failed to find merge commit")?;

        let tree_id = repo
            .index()?
            .write_tree()
            .context("Failed to write tree after merge")?;
        let tree = repo
            .find_tree(tree_id)
            .context("Failed to find tree after merge")?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Pull from {}/{}", remote_name, branch_name),
            &tree,
            &[&head_commit, &merge_commit],
        )
        .context("Failed to create commit after pull")?;
    } else {
        anyhow::bail!("Merge analysis returned unknown status.");
    }

    Ok(())
}
