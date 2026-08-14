# TypeScript SDK fixture

This small app demonstrates the preferred typed DSL. `defineSpec("share-links")`
returns one spec-scoped authoring context. Its fluent `Source`, `Requirement`,
and `Rule` calls return new immutable declarations; `.name(...)` and
`.description(...)` are canonical metadata, not comments.

`provenance.spec.ts` directly exports the `expiry` Rule declaration. The test
imports that Rule, so removing or renaming the export is an ordinary TypeScript
module error. The Rule's `.implementedBy(createShareLink)` binding points to the
ordinary exported function in `share-links.ts`; removing that production export
is likewise an ordinary TypeScript error. Production code never imports
Provenance, and this example needs only this one honest Rule.

Construction and `build()` are synchronous, deterministic, and in-memory.
Importing the spec does not invoke the engine or write state. `apply.ts` is the
deliberate persistence entry point; `plan(spec)` can preview the same document
without writing. The test applies first, then uses the exported Rule to record a
verification while exercising ordinary production code.

## Run from a source checkout

These commands intentionally use local development overrides. From the
repository root:

```sh
cargo build -p provenance-cli
npm ci --prefix packages/provenance
npm run build --prefix packages/provenance
npm ci --prefix examples/typescript-sdk
target/debug/provenance init --path examples/typescript-sdk --scope default --path-prefix .
PROVENANCE_BIN="$PWD/target/debug/provenance" \
PROVENANCE_REPO="$PWD/examples/typescript-sdk" \
PROVENANCE_SPEC_OWNER="spec://typescript/share-links" \
PROVENANCE_VERIFICATION_OWNER="dev://typescript-example" \
npm test --prefix examples/typescript-sdk
target/debug/provenance rules list --repo examples/typescript-sdk --scope default --format json
target/debug/provenance sdk verification-runs --repo examples/typescript-sdk --scope default --format json
target/debug/provenance wiki build --repo examples/typescript-sdk --scope default --format json
```

The example's `file:../../packages/provenance` dependency links the SDK source
checkout, so it relies on the separately built local package and explicitly
sets `PROVENANCE_BIN` to the separately built Rust binary. This is development
plumbing, not the installed package contract. No package is published and no
global CLI is required.

By contrast, `npm run test:packed --prefix packages/provenance` packs the SDK and
a matching platform engine into tarballs, installs them in an isolated consumer,
clears `PROVENANCE_BIN` and `PATH`, and proves that the packed installation is
self-contained. A normal registry installation has that same package-supplied
engine contract; it does not depend on this source checkout or a global CLI.
