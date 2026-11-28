# Releasing

Prepare the version on `sandbox`, promote the same commit to `develop`, and
choose when to publish with `just release`. GitHub creates the version tag when
the verified draft is published.

## Prepare and publish

Install Bash, Git, Just, jq, Cargo Edit (`cargo set-version`), and the GitHub CLI.
Authenticate `gh` for the release repository and configure a Git signing key
that GitHub recognizes. The `origin` fetch and push URLs must identify that same
GitHub.com repository.

1. Make the bump the last commit on `sandbox`.

   ```sh
   git switch sandbox
   just bump                 # patch
   # just bump-minor         # minor
   # just bump-major         # major
   candidate=$(git rev-parse HEAD)
   git push origin "$candidate:refs/heads/sandbox"
   ```

   The bump runs `just test` (Clippy, formatting checks, and unit tests), changes
   the version and refreshes the lockfile with `cargo update`, then runs the same
   checks on the candidate. It creates a signed local commit.

2. Wait for Test & Build's **CI OK** on that `sandbox` SHA. Fast-forward
   `develop` to it.

   ```sh
   git push origin "$candidate:refs/heads/develop"
   ```

   Use the same full SHA if you resume in another shell. The regular push
   rejects a non-fast-forward update. A rebase or squash creates a new SHA
   that needs its own CI.

3. Wait for **CI OK** on the `develop` push. When you are ready to publish, run
   `just release`. It captures the remote `develop` SHA, checks the gate, and
   follows the workflow run returned by GitHub. Optional coverage can finish
   independently. After the Release run succeeds, the recipe checks the gate
   again and fast-forwards `main` to the released SHA.

The [Release workflow](.github/workflows/release.yml) builds the archives, Linux
packages, and crate using one resolved Rust version. It checks the file set,
writes `SHA256SUMS`, and attests the build. Before publishing the draft it
checks the uploaded asset digests, the latest stable tag, and the crates.io
checksum if the version already exists there. It verifies the public tag and
downloaded files, then publishes or verifies the crate on crates.io.

After the first stable tag, each new release must be the next patch, minor, or
major version above the highest stable tag. For example, after `0.22.4` the
allowed versions are `0.22.5`, `0.23.0`, and `1.0.0`. Releasing `0.22.6`
requires `0.22.5` to be tagged first. Both the bump recipe and the workflow
enforce this sequence.

## When something fails

- **CI or guard refused the release.** Check the workflow link and the SHA.
  Rerun failed CI jobs on the same push run for a transient failure. A source
  fix goes through `sandbox` and a fresh CI run.
- **Release failed before tagging.** Fix the cause and choose **Re-run failed
  jobs** on the original run. A run can replace its own incomplete draft.
  Inspect a draft owned by another run before removing it. If a newer version
  has already shipped, prepare the next version from the highest stable tag.
  For a crates.io checksum conflict, reconcile the package or choose another
  allowed bump from the highest stable tag.
- **Release failed after tagging.** The GitHub release is public. For a
  transient failure, rerun the failed jobs on the original run. A crates.io
  retry verifies any existing package against the built checksum. A source
  fix goes into the next version. Keep existing release tags in place.
- **The CLI was interrupted or the push to `main` failed.** Find the original
  run and confirm that its SHA matches the published tag, that the Release run
  succeeded, and that `develop`'s **CI OK** passed for that SHA. Then run
  `git push origin <released-sha>:refs/heads/main`. If the dispatch response
  was lost, check Actions for the run before retrying.

## Verify a download

Download the asset and `SHA256SUMS` from the release, then run the following.

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify <asset> --repo nbari/cron-when \
  --signer-workflow nbari/cron-when/.github/workflows/release.yml
```

The release notes record the source SHA and the resolved Rust version.

## Adopting this flow in another project

1. For the bump commands, copy [`.justfiles/release.just`](.justfiles/release.just)
   and add `import '.justfiles/release.just'` to the project's Justfile. Provide
   a check-only `test` recipe and commit `Cargo.lock`. Run the import from the
   Cargo project, such as the Justfile under `backend/` in a monorepo. The recipe
   supports a single crate or a workspace with a shared stable version.
2. For the full release, copy the [Release workflow](.github/workflows/release.yml),
   its [local actions](.github/actions), the release helpers under
   [`.github/scripts`](.github/scripts), and the `release` entry in
   [`.justfile`](.justfile). Keep the build matrix and its expected file list
   aligned. The CI helper expects the `CI OK` job in a `build.yml` push to
   `develop`. Use the package and publish jobs that match your destinations.
   Adapt the artifact list and packaging metadata using the
   [workflow template](.github/TEMPLATE.md).
3. Apply the [repository settings](#settings) and put `release.yml` on the
   default branch before the first dispatch. For a repository adopting the
   workflow, bootstrap it by fast-forwarding `main` to the first green
   `develop` SHA that contains it. Then follow the normal flow for a fresh release
   version. Check the first release's tag, files, and registry publication.

The bump accepts named working branches and reserves `main` and `develop` by
default. To deliberately bump on `develop` in another project, export
`RELEASE_BUMP_BRANCH := "develop"` from its Justfile. `main` is always reserved.

For generated version metadata, export `RELEASE_BUMP_FILES` as a newline-separated
list of tracked files relative to the Cargo workspace. Define a
`_release-bump-metadata version` recipe that updates those files. A Permesi
hook can run its existing generator.

```just
export RELEASE_BUMP_FILES := "docs/openapi/permesi.json\ndocs/openapi/genesis.json"

_release-bump-metadata version:
    @just openapi
```

- [s3m](https://github.com/s3m/s3m/blob/develop/.github/RELEASING.md) shares a
  version across two crates, updates the internal dependency requirement,
  and publishes the dependency first.
- [Permesi](https://github.com/permesi/permesi/blob/develop/CONTRIBUTING.md#releasing)
  regenerates OpenAPI specs from the workspace and qualifies image digests
  against the Helm charts before production delivery.
- [pg_exporter](https://github.com/nbari/pg_exporter/blob/develop/RELEASING.md)
  includes its Grafana dashboard version in the bump and verifies the
  crates.io checksum.
- [Textarea](https://github.com/wurzellos/textarea/blob/develop/RELEASING.md)
  bumps the backend on `develop` and promotes frontend-only changes without
  a Rust bump.

## Settings

- **Branches.** Make `main` the default branch. Require `CI OK` from GitHub
  Actions on `main`. Require signed commits and linear history on `main` and
  `develop`, and disallow force pushes and deletions. Permit the operator to
  fast-forward both branches directly. `develop` receives the candidate and
  then runs CI.
- **Tags and releases.** Protect stable version tags from updates and
  deletion and permit the release workflow to create them. Enable GitHub
  immutable releases where available.
- **Actions.** Use read-only defaults and grant write permissions to the jobs
  that need them. Environments used by the release must allow `develop`.
  Check your GitHub plan's support for branch protection and attestations
  when adopting the template in a private repository.
- **crates.io.** Configure Trusted Publishing for this repository and
  `release.yml` on the existing crate. Keep that workflow filename stable.
  For a new crate, follow the registry's [first-publish instructions](https://crates.io/docs/trusted-publishing)
  before configuring the trusted publisher.
