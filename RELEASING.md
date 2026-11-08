# Releasing

This project releases with one command, `just release`, and follows one rule:

> **The commit that is released is exactly the commit CI tested, and everything
> published is built from it once.**

The flow is the reason this repository exists as a template: copy it to your own
project and a release stops being a list of manual steps that can half-succeed. It is
also deliberately small. There is no script to maintain, no scratch branch and no
manifest: the whole release is one workflow file, [`release.yml`](.github/workflows/release.yml),
and a few small [local actions](.github/actions).

- [Why this flow](#why-this-flow)
- [Day to day](#day-to-day)
- [How a release works](#how-a-release-works)
- [When something fails](#when-something-fails)
- [Guarantees and limits](#guarantees-and-limits)
- [Security](#security)
- [Verifying a release](#verifying-a-release)
- [Settings](#settings)
- [Adopting this flow in another project](#adopting-this-flow-in-another-project)
- [Lessons learned](#lessons-learned)

## Why this flow

The usual release routine is: bump the version in `Cargo.toml` by hand, commit, tag,
push the tag, and let CI build and publish. It fails in ways that are hard to undo: the
tag is pushed before anyone knows whether the packages build, so a failed Windows
archive means deleting and re-pushing the tag, the crate may already be on crates.io
while the binaries are not, and `main` holds a version bump for a release that never
happened.

This repository's first answer to that was a long script (`just deploy`) that staged
the bump on a scratch `release` branch, waited for CI and a "candidate" workflow run,
then moved `develop`, `main` and a signed tag in one atomic push, after which the tag's
own run published the candidate's files from a manifest. It worked, but it was the
problem it set out to solve in a new form: more than a thousand lines of shell, a
second workflow mode, a manifest to keep in sync with the build jobs, and recovery
rules for each half-finished state.

The flow below keeps the guarantee and drops the machinery. The tag is created by the
release itself, after every build has succeeded, so there is never a tag to delete. The
version bump is an ordinary commit made by the operator, so CI tests it like any other
change. CI has already proven the commit, so the release does not test again; it builds,
packages and publishes.

## Day to day

Work, including dependency updates, lands on `sandbox`.

1. As the last commit of the work, the operator bumps the version (patch, minor or
   major) with a tool of their choice, such as `cargo set-version --bump patch` or
   `gbump`. The bump covers `Cargo.toml` and `Cargo.lock`. The reviewer checks that the
   bump is present and right.
2. When **Test & Build** is green on `sandbox`, fast-forward `develop` to that commit
   and push. The `CI OK` check is attached to the commit, so the run that passed on
   `sandbox` already counts for the release guard, and you can run `just release` at
   once; it waits for `develop`'s own run before it moves `main`.
3. Run `just release`. It starts the Release workflow on `develop` and follows the run.

Keep `develop` fast-forward only, so the history stays linear and `main` can always
move to the released commit. If you squash or rebase `sandbox` after its CI passed, the
commit changes and CI has to run again.

## How a release works

`just release` starts the workflow (`gh workflow run release.yml --ref develop`), follows
the run, and then moves `main`. Everything else is the workflow:

```
Release workflow (workflow_dispatch on develop)
 guard    develop only; the version is X.Y.Z, above the latest tag and untagged;
          main is an ancestor; the `CI OK` check passed on this exact commit
 build    one leg per target, each on a native runner: the archive, and for Linux the
          .deb and .rpm (checked for version, binary and architecture); the binary is
          run once (--version must report the crate's version)
 crate    cargo package and a publish dry run
 release  the exact file set is checked, SHA256SUMS, a build-provenance attestation,
          a draft GitHub release, then publish (this creates the tag at the tested
          commit), then a final check that the tag and the assets are right
 publish  crates.io (Trusted Publishing, no stored token)
```

Then `just release` waits for `develop`'s own Test & Build run on the commit and
fast-forwards `main` to it.

Nothing is published before every build job has succeeded, and the guard refuses
anything that is not the tested, bumped commit of `develop`. Notes for the GitHub
release are the commit subjects since the previous release (version bumps left out), so
no changelog is kept by hand.

## When something fails

Fix the cause and use **Re-run failed jobs** on the run. Every step is safe to repeat:
finished jobs are not run again, a draft release left by a failed run is replaced, an
already published release is kept (and re-checked), and a version crates.io already has
is skipped.

- **Before the `release` job succeeds** nothing is published and no tag exists. Re-run,
  or fix the code, merge to `develop` and run `just release` again.
- **After it** the tag and the release exist and are public. A transient failure of a
  later step (crates.io) is fixed by re-running. If `just release` stopped before moving
  `main`, push it yourself: `git push origin <released-sha>:main`. A failure that needs a
  code change is released as the next version: bump again and run `just release`.
- **The guard refuses** when the version is already tagged (you forgot to bump), is
  below the latest release, when `develop` does not contain `main`, or when `CI OK` has
  not passed on the commit. The message says which.
- **CI is flaky.** Re-run the failed jobs in Test & Build until `CI OK` is green on the
  commit. A re-run replaces the earlier result for that commit, so the guard sees the
  passing one.

## Guarantees and limits

- The released commit is the one `CI OK` passed on, and the release builds it once; the
  guard checks the check on the exact SHA.
- The tag is created only after every build has succeeded, so a release never needs a tag
  deleted or moved. The release job also checks that the tag points at the tested commit
  and that the published assets are exactly what was built.
- A version above the latest release is required, so a lower or repeated version cannot
  take the Latest flag.
- Limits: the tag is created by GitHub and is **not signed**. The release builds again
  rather than reusing CI's artifacts, so it relies on builds being repeatable. The file
  set the release checks is written out in `release.yml` and has to be kept in step with
  the build matrix; a missing or extra file stops the release, which is the safe failure.

## Security

- The workflow is read-only by default; each job asks for the permissions it needs, and
  no checkout keeps the token.
- Remote actions are pinned by full commit SHA with the tag in a comment, and Dependabot
  proposes the updates.
- crates.io publishing uses Trusted Publishing: a short-lived token minted by OIDC for
  this workflow, with no registry secret stored.
- Build provenance is attested for every release file, so downloads can be traced to the
  workflow run and commit that built them.

## Verifying a release

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify <file> --repo nbari/cron-when --signer-workflow nbari/cron-when/.github/workflows/release.yml
```

## Settings

One-time repository settings the flow relies on:

- **`main`**: branch protection that requires the `CI OK` check, signed commits and
  linear history, with no force pushes. `just release` fast-forwards it from the
  operator's machine (see [Lessons learned](#lessons-learned)), so the operator must be
  allowed to push to it.
- A **tag ruleset** that lets `X.Y.Z` tags be created but never moved or deleted.
- **crates.io Trusted Publishing** configured for the repository and the workflow file
  name `release.yml` (keep the file name stable; it is part of the identity).
- **Commit signing**: commits on `sandbox` and `develop` must be signed with a key GitHub
  knows as a signing key, because `main` requires verified signatures. Use a dedicated,
  registered signing key for unattended sessions.

Releasing needs `just`, `gh` (logged in) and a signing key.

## Adopting this flow in another project

1. Copy `.github/workflows/release.yml`, `.github/actions/` and the `just release` recipe.
2. Adjust the build matrix and keep the file-set check in `release.yml` in step with it.
   Remove the crate and publish jobs if you do not publish to crates.io, and add jobs
   for whatever else you publish (images, a docs site, a Helm dispatch) after the
   `release` job, each guarded so an older release never overwrites a newer one.
3. Add the packaging and crate checks to your CI workflow (`build.yml` here) so that a
   problem fails on `develop` before the release, and make sure the aggregate `CI OK`
   job exists, since the guard looks for it on the commit.
4. Set the repository settings above, and delete any old release script and its
   recipes.
5. Release a throwaway version first and read the run.

Projects already using this flow: this one, `zsmtp` (a crate with archives, packages and
crates.io), `permesi` (archives, Debian packages, three images, Cloudflare Pages, Helm
and docs) and `crono` (four-target binaries, packages, multi-platform images, Helm and
Pages).

## Lessons learned

These came from moving four projects off the scratch-branch flow:

- **Keep the release one file.** A script, a second "candidate" mode and a manifest each
  needed their own tests and recovery rules. One workflow with a guard is easier to
  review and to repair.
- **Check `CI OK` on the commit, not the workflow run.** A flaky non-gating job, such
  as coverage, should not block a release, and a re-run of the failed job replaces the
  earlier result for that commit. The check-runs API can answer `422 No commit found`
  for a moment right after a push; treat that as "not yet", not as a result.
- **Tags created by the release are lightweight.** Any helper that insists on annotated or
  signed tags (a "highest release" check, for example) will ignore them. Compare plain
  version order instead.
- **A pending `CI OK` blocks every update of a protected `main`.** Fast-forwarding `main`
  to a commit whose own Test & Build run (the one `develop`'s push starts) is still
  running is refused with `Required status check "CI OK" is expected`, for the
  workflow's token and for the operator alike, even though an earlier run already
  passed on that commit. The release workflow therefore does not touch `main`: `just
  release` pushes it after the run succeeded and `develop`'s own run has finished. Some
  projects also restrict `main` to named pushers, which the workflow's token is not.
- **Environments can restrict branches.** A Pages (or other) environment that allows only
  `main` and tags rejects a run on `develop`; add `develop` to its allowed branches.
- **Image scanners are part of CI, not the release.** A fixable vulnerability in a base
  image fails CI, and the fix is an `apk upgrade` or a newer base, not a release-time
  workaround.
- **Keep the workflow file name.** Signatures and attestations carry the workflow
  identity, so renaming it changes what consumers must verify.
- **Do not rely on one signing agent.** A forwarded agent disappears when the session
  ends; a dedicated, registered signing key keeps unattended work moving.
