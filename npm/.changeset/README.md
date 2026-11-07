# Package changesets

Use Changesets for independent npm package versions and coordinated changelogs.
Add a changeset when a change affects a package's consumer-visible behavior or
published contents:

```sh
npm run changeset
```

The root npm workspace is private and exists only for development, CI, version
planning, and tarball staging. It is not an npm package. Staging must remain
non-publishing: do not add npm credentials, OIDC publish permissions, or
`npm publish` to CI. Public publication remains deferred to issue #293.

The workspace currently includes the existing Node binding, its four platform
packages, browser WASM binding, and initial TypeScript facade. Future core,
React, and Vue packages join through the workspace globs when their
implementation issues are completed. Preserve established package names unless
a separately reviewed migration is approved.
